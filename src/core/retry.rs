// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Políticas de resiliencia, reintentos automáticos y backoff exponencial para el cliente USPS.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use reqwest::StatusCode;

static JITTER_SEED: AtomicU64 = AtomicU64::new(0);

fn next_random_u64() -> u64 {
    let mut state = JITTER_SEED.load(Ordering::Relaxed);
    if state == 0 {
        state = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
    }
    // Algoritmo SplitMix64
    state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    JITTER_SEED.store(state, Ordering::Relaxed);
    let mut z = state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Configuración de política de reintentos para mitigar errores transitorios de red o saturación de cuota.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Cantidad máxima de reintentos permitidos (por defecto 3).
    pub max_retries: u32,
    /// Demora inicial antes del primer reintento (por defecto 200 milisegundos).
    pub initial_delay: Duration,
    /// Tiempo de espera máximo entre reintentos consecutivos (por defecto 3 segundos).
    pub max_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(3),
        }
    }
}

impl RetryPolicy {
    /// Inicia una política de reintentos sin reintentos (deshabilitada).
    #[must_use]
    pub fn none() -> Self {
        Self {
            max_retries: 0,
            initial_delay: Duration::ZERO,
            max_delay: Duration::ZERO,
        }
    }

    /// Construye una política personalizada con un número máximo de reintentos.
    #[must_use]
    pub fn new(max_retries: u32, initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            max_retries,
            initial_delay,
            max_delay,
        }
    }

    /// Determina si un código de estado HTTP corresponde a un error transitorio reintentable.
    #[must_use]
    pub fn is_retryable_status(status: StatusCode) -> bool {
        matches!(
            status,
            StatusCode::TOO_MANY_REQUESTS
                | StatusCode::INTERNAL_SERVER_ERROR
                | StatusCode::BAD_GATEWAY
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
        )
    }

    /// Calcula la duración de espera para un intento determinado usando backoff exponencial con Jitter
    /// para mitigar problemas de sincronización de reintentos concurrentes (*thundering herd*).
    #[must_use]
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        if attempt == 0 || self.max_retries == 0 {
            return Duration::ZERO;
        }

        let factor = 2u64.saturating_pow(attempt.saturating_sub(1));
        let max_calculated = self
            .initial_delay
            .saturating_mul(factor as u32)
            .min(self.max_delay);
        let max_millis = max_calculated.as_millis() as u64;

        if max_millis == 0 {
            return Duration::ZERO;
        }

        // Decorrelated Equal Jitter: rango uniforme en [max_millis / 2, max_millis]
        let half = max_millis / 2;
        let random_part = next_random_u64() % (half.max(1) + 1);
        Duration::from_millis(half + random_part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_status_codes() {
        assert!(RetryPolicy::is_retryable_status(
            StatusCode::TOO_MANY_REQUESTS
        ));
        assert!(RetryPolicy::is_retryable_status(StatusCode::BAD_GATEWAY));
        assert!(RetryPolicy::is_retryable_status(
            StatusCode::SERVICE_UNAVAILABLE
        ));
        assert!(!RetryPolicy::is_retryable_status(StatusCode::UNAUTHORIZED));
        assert!(!RetryPolicy::is_retryable_status(StatusCode::BAD_REQUEST));
        assert!(!RetryPolicy::is_retryable_status(StatusCode::NOT_FOUND));
    }

    #[test]
    fn backoff_calculation_progression() {
        let policy = RetryPolicy::default();
        let delay_1 = policy.calculate_backoff(1);
        let delay_2 = policy.calculate_backoff(2);
        let delay_3 = policy.calculate_backoff(3);

        // Intento 1: base = 200ms -> rango con jitter [100ms, 200ms]
        assert!(delay_1 >= Duration::from_millis(100) && delay_1 <= Duration::from_millis(200));
        // Intento 2: base = 400ms -> rango con jitter [200ms, 400ms]
        assert!(delay_2 >= Duration::from_millis(200) && delay_2 <= Duration::from_millis(400));
        // Intento 3: base = 800ms -> rango con jitter [400ms, 800ms]
        assert!(delay_3 >= Duration::from_millis(400) && delay_3 <= Duration::from_millis(800));

        let capped = policy.calculate_backoff(10);
        // Capped a max_delay (3s) -> rango con jitter [1500ms, 3000ms]
        assert!(capped >= Duration::from_millis(1500) && capped <= Duration::from_secs(3));
    }
}
