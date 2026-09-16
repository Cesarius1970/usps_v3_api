// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Configuración y entornos para el cliente de la API v3 de USPS.

use std::fmt;
use std::time::Duration;

use crate::error::{Result, UspsError};

/// URL base oficial del entorno de pruebas CAT (Customer Acceptance Testing / Sandbox) de USPS.
pub const USPS_CAT_BASE_URL: &str = "https://api-cat.usps.com";

/// URL base oficial del entorno de producción de USPS.
pub const USPS_PROD_BASE_URL: &str = "https://api.usps.com";

/// Entornos de ejecución disponibles para la API REST v3 de USPS.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UspsEnvironment {
    /// Entorno de pruebas / sandbox (CAT - Customer Acceptance Testing).
    #[default]
    Sandbox,
    /// Entorno de producción en vivo.
    Production,
    /// Entorno personalizado (ej. para proxies corporativos o servidores de mock en pruebas).
    Custom(String),
}

impl UspsEnvironment {
    /// Retorna la URL base correspondiente al entorno.
    #[must_use]
    pub fn base_url(&self) -> &str {
        match self {
            Self::Sandbox => USPS_CAT_BASE_URL,
            Self::Production => USPS_PROD_BASE_URL,
            Self::Custom(custom_url) => custom_url.trim_end_matches('/'),
        }
    }
}

/// Configuración de credenciales y parámetros de conexión para el cliente [`crate::client::UspsClient`].
#[derive(Clone)]
pub struct UspsConfig {
    /// Client ID de la aplicación registrada en el portal de desarrolladores de USPS.
    pub client_id: String,
    /// Client Secret de la aplicación (protegido contra filtración en logs de debug).
    pub client_secret: String,
    /// Entorno de destino (Sandbox por defecto).
    pub environment: UspsEnvironment,
    /// Tiempo de espera (timeout) para las solicitudes HTTP.
    pub timeout: Duration,
}

impl fmt::Debug for UspsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UspsConfig")
            .field("client_id", &self.client_id)
            .field("client_secret", &"[REDACTED]")
            .field("environment", &self.environment)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl UspsConfig {
    /// Crea una nueva instancia de configuración validando que las credenciales no estén vacías.
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si `client_id` o `client_secret` están vacíos.
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        environment: UspsEnvironment,
    ) -> Result<Self> {
        let client_id = client_id.into().trim().to_string();
        let client_secret = client_secret.into().trim().to_string();

        if client_id.is_empty() {
            return Err(UspsError::InvalidInput(
                "El client_id no puede estar vacío".to_string(),
            ));
        }

        if client_secret.is_empty() {
            return Err(UspsError::InvalidInput(
                "El client_secret no puede estar vacío".to_string(),
            ));
        }

        Ok(Self {
            client_id,
            client_secret,
            environment,
            timeout: Duration::from_secs(30),
        })
    }

    /// Define un tiempo de espera personalizado para las solicitudes HTTP.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_should_sanitize_debug_logs() {
        let config =
            UspsConfig::new("my-client-id", "super-secret-key", UspsEnvironment::Sandbox).unwrap();

        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("my-client-id"));
        assert!(!debug_str.contains("super-secret-key"));
        assert!(debug_str.contains("[REDACTED]"));
    }

    #[test]
    fn config_should_fail_on_empty_credentials() {
        let err = UspsConfig::new("", "secret", UspsEnvironment::Sandbox).unwrap_err();
        assert!(matches!(err, UspsError::InvalidInput(_)));

        let err = UspsConfig::new("id", "", UspsEnvironment::Sandbox).unwrap_err();
        assert!(matches!(err, UspsError::InvalidInput(_)));
    }

    #[test]
    fn environment_should_return_correct_base_url() {
        assert_eq!(UspsEnvironment::Sandbox.base_url(), USPS_CAT_BASE_URL);
        assert_eq!(UspsEnvironment::Production.base_url(), USPS_PROD_BASE_URL);
        assert_eq!(
            UspsEnvironment::Custom("https://my-proxy.com/api/".to_string()).base_url(),
            "https://my-proxy.com/api"
        );
    }
}
