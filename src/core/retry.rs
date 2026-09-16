// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Políticas de resiliencia, reintentos automáticos y backoff exponencial para el cliente USPS.

use std::time::Duration;

use reqwest::StatusCode;

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

    /// Calcula la duración de espera para un intento determinado usando backoff exponencial acotado.
    #[must_use]
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        if attempt == 0 || self.max_retries == 0 {
            return Duration::ZERO;
        }

        let factor = 2u64.saturating_pow(attempt.saturating_sub(1));
        let calculated = self.initial_delay.saturating_mul(factor as u32);

        calculated.min(self.max_delay)
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

        assert_eq!(delay_1, Duration::from_millis(200));
        assert_eq!(delay_2, Duration::from_millis(400));
        assert_eq!(delay_3, Duration::from_millis(800));

        let capped = policy.calculate_backoff(10);
        assert_eq!(capped, Duration::from_secs(3));
    }
}
