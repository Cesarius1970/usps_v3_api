// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Gestor de autenticación OAuth 2.0 con refresco automático y concurrencia segura para USPS v3.

use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::Client as HttpClient;
use serde::Deserialize;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

use crate::config::UspsConfig;
use crate::error::{Result, UspsError};

/// Margen de seguridad de 60 segundos para refrescar el token antes de que expire oficialmente.
const EXPIRATION_BUFFER_SECS: u64 = 60;

/// Respuesta del endpoint OAuth 2.0 de USPS (`POST /oauth2/v3/token`).
#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    /// Token de acceso Bearer emitido por USPS.
    pub access_token: String,
    /// Tipo de token (típicamente "Bearer").
    pub token_type: Option<String>,
    /// Tiempo de vida del token en segundos (ej. 28799 ~ 8 horas).
    pub expires_in: Option<i64>,
    /// Estado de la emisión (ej. "approved").
    pub status: Option<String>,
}

/// Información interna de vigencia de un token en memoria.
#[derive(Debug, Clone)]
struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

impl CachedToken {
    /// Verifica si el token aún es válido considerando el margen de anticipación.
    fn is_valid(&self) -> bool {
        Instant::now() < self.expires_at
    }
}

/// Gestor de tokens OAuth 2.0 thread-safe y asíncrono para USPS v3.
#[derive(Debug, Clone)]
pub struct TokenManager {
    config: UspsConfig,
    http_client: HttpClient,
    cached_token: Arc<RwLock<Option<CachedToken>>>,
}

impl TokenManager {
    /// Crea un nuevo [`TokenManager`].
    #[must_use]
    pub fn new(config: UspsConfig, http_client: HttpClient) -> Self {
        Self {
            config,
            http_client,
            cached_token: Arc::new(RwLock::new(None)),
        }
    }

    /// Obtiene un token Bearer válido, reutilizando el token en caché o renovándolo de forma transparente.
    #[instrument(skip(self), name = "usps_get_token")]
    pub async fn get_token(&self) -> Result<String> {
        // 1. Intento de lectura concurrente rápida con ReadLock
        {
            let read_guard = self.cached_token.read().await;
            if let Some(ref cached) = *read_guard {
                if cached.is_valid() {
                    debug!("Reutilizando token OAuth 2.0 en caché");
                    return Ok(cached.access_token.clone());
                }
            }
        }

        // 2. Adquisición de WriteLock con patrón double-check
        let mut write_guard = self.cached_token.write().await;
        if let Some(ref cached) = *write_guard {
            if cached.is_valid() {
                debug!("Token ya renovado concurrentemente por otra tarea");
                return Ok(cached.access_token.clone());
            }
        }

        // 3. Solicitud de nuevo token al servidor OAuth de USPS
        info!("Renovando token OAuth 2.0 con USPS");
        let token_resp = self.fetch_new_token().await?;
        let expires_in = token_resp.expires_in.unwrap_or(3600).max(0) as u64;

        let usable_duration = if expires_in > EXPIRATION_BUFFER_SECS {
            Duration::from_secs(expires_in - EXPIRATION_BUFFER_SECS)
        } else {
            Duration::from_secs(expires_in)
        };

        let cached = CachedToken {
            access_token: token_resp.access_token.clone(),
            expires_at: Instant::now() + usable_duration,
        };

        *write_guard = Some(cached);
        Ok(token_resp.access_token)
    }

    /// Realiza la llamada HTTP directa `POST /oauth2/v3/token` para obtener un nuevo Bearer token.
    async fn fetch_new_token(&self) -> Result<OAuthTokenResponse> {
        let token_url = format!("{}/oauth2/v3/token", self.config.environment.base_url());

        let form_params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self
            .http_client
            .post(&token_url)
            .form(&form_params)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(UspsError::from_response(status, &body));
        }

        let token_response: OAuthTokenResponse = serde_json::from_str(&body).map_err(|err| {
            UspsError::Auth(format!("Error al deserializar respuesta OAuth: {err}"))
        })?;

        Ok(token_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_token_validity_check() {
        let token = CachedToken {
            access_token: "test_token".to_string(),
            expires_at: Instant::now() + Duration::from_secs(10),
        };
        assert!(token.is_valid());

        let expired = CachedToken {
            access_token: "test_token".to_string(),
            expires_at: Instant::now() - Duration::from_secs(1),
        };
        assert!(!expired.is_valid());
    }
}
