// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Definición jerárquica y fuertemente tipada de errores del SDK `usps_v3_api`.

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Tipo alias para resultados que retornan [`UspsError`].
pub type Result<T> = std::result::Result<T, UspsError>;

/// Catálogo tipado de códigos de error oficiales y frecuentes de las APIs REST v3 de USPS.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum UspsErrorCode {
    // --- Errores de Direcciones y Códigos Postales ---
    /// Dirección no encontrada en la base de datos oficial de USPS.
    AddressNotFound,
    /// Código postal inválido o malformado.
    InvalidZipCode,
    /// Múltiples direcciones encontradas que coinciden con los criterios de búsqueda.
    MultipleAddressesFound,

    // --- Errores de Autenticación y Autorización ---
    /// Credenciales de cliente (Client ID / Client Secret) inválidas o no autorizadas.
    InvalidCredentials,
    /// Token Bearer OAuth 2.0 expirado o revocado.
    TokenExpired,
    /// Acceso no autorizado o permisos insuficientes para el endpoint solicitado.
    Unauthorized,

    // --- Errores de Límites, Cuotas y Servidor ---
    /// Exceso en la tasa de peticiones permitidas (Rate Limit / HTTP 429).
    RateLimitExceeded,
    /// Cuota mensual o de suscripción por volumen excedida.
    QuotaExceeded,
    /// Servicio de USPS no disponible temporalmente (HTTP 503).
    ServiceUnavailable,

    // --- Errores de Seguimiento / Tracking ---
    /// Número de seguimiento no encontrado en el sistema postal.
    TrackingNumberNotFound,
    /// Formato del número de seguimiento inválido.
    InvalidTrackingNumberFormat,

    // --- Errores de Etiquetas, Pagos y Manifiestos ---
    /// Etiqueta postal previamente cancelada.
    LabelAlreadyCancelled,
    /// Etiqueta postal expirada o fuera de vigencia.
    LabelExpired,
    /// Fondos insuficientes en la cuenta EPS de pago.
    InsufficientFunds,
    /// Manifiesto SCAN Form ya consolidado o duplicado.
    DuplicateManifest,

    // --- Errores de Recolección (Carrier Pickup) ---
    /// Recolección no disponible para el código postal o fecha indicada.
    PickupNotAvailable,

    // --- Variante de Extensión ---
    /// Otro código de error reportado por USPS no categorizado previamente.
    Other(String),
}

impl std::fmt::Display for UspsErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AddressNotFound => write!(f, "AddressNotFound"),
            Self::InvalidZipCode => write!(f, "InvalidZipCode"),
            Self::MultipleAddressesFound => write!(f, "MultipleAddressesFound"),
            Self::InvalidCredentials => write!(f, "InvalidCredentials"),
            Self::TokenExpired => write!(f, "TokenExpired"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::RateLimitExceeded => write!(f, "RateLimitExceeded"),
            Self::QuotaExceeded => write!(f, "QuotaExceeded"),
            Self::ServiceUnavailable => write!(f, "ServiceUnavailable"),
            Self::TrackingNumberNotFound => write!(f, "TrackingNumberNotFound"),
            Self::InvalidTrackingNumberFormat => write!(f, "InvalidTrackingNumberFormat"),
            Self::LabelAlreadyCancelled => write!(f, "LabelAlreadyCancelled"),
            Self::LabelExpired => write!(f, "LabelExpired"),
            Self::InsufficientFunds => write!(f, "InsufficientFunds"),
            Self::DuplicateManifest => write!(f, "DuplicateManifest"),
            Self::PickupNotAvailable => write!(f, "PickupNotAvailable"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}

impl UspsErrorCode {
    /// Resuelve un código alfanumérico o mensaje textual devuelto por USPS al enum tipado [`UspsErrorCode`].
    #[must_use]
    pub fn parse(code: &str) -> Self {
        let normalized = code.trim().to_uppercase();
        match normalized.as_str() {
            "ADDRESS_NOT_FOUND" | "ADDRESS NOT FOUND" => Self::AddressNotFound,
            "INVALID_ZIP" | "INVALID_ZIP_CODE" | "INVALID ZIP CODE" => Self::InvalidZipCode,
            "MULTIPLE_ADDRESSES_FOUND" | "MULTIPLE ADDRESSES" => Self::MultipleAddressesFound,
            "INVALID_CLIENT" | "INVALID_CREDENTIALS" | "UNAUTHORIZED_CLIENT" => {
                Self::InvalidCredentials
            }
            "TOKEN_EXPIRED" | "INVALID_TOKEN" => Self::TokenExpired,
            "UNAUTHORIZED" | "FORBIDDEN" | "ACCESS_DENIED" => Self::Unauthorized,
            "RATE_LIMIT_EXCEEDED" | "TOO_MANY_REQUESTS" | "THROTTLED" => Self::RateLimitExceeded,
            "QUOTA_EXCEEDED" => Self::QuotaExceeded,
            "SERVICE_UNAVAILABLE" => Self::ServiceUnavailable,
            "TRACKING_NOT_FOUND" | "TRACKING_NUMBER_NOT_FOUND" | "PACKAGE_NOT_FOUND" => {
                Self::TrackingNumberNotFound
            }
            "INVALID_TRACKING_NUMBER" | "INVALID_TRACKING_FORMAT" => {
                Self::InvalidTrackingNumberFormat
            }
            "LABEL_ALREADY_CANCELLED" | "LABEL_CANCELLED" => Self::LabelAlreadyCancelled,
            "LABEL_EXPIRED" => Self::LabelExpired,
            "INSUFFICIENT_FUNDS" => Self::InsufficientFunds,
            "DUPLICATE_MANIFEST" => Self::DuplicateManifest,
            "PICKUP_NOT_AVAILABLE" => Self::PickupNotAvailable,
            _ => Self::Other(code.to_string()),
        }
    }
}

/// Detalle individual de error devuelto por los servicios REST de USPS v3.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ApiErrorDetail {
    /// Código alfanumérico o identificador del error según la especificación USPS.
    #[serde(default)]
    pub code: Option<String>,
    /// Mensaje descriptivo o diagnóstico del error.
    #[serde(default)]
    pub message: Option<String>,
    /// Origen o campo causante del error (si está disponible).
    #[serde(default)]
    pub source: Option<String>,
}

/// Estructura estándar de respuesta de error devuelta por la API v3 de USPS.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct UspsApiErrorResponse {
    /// Identificador o código principal del error.
    #[serde(default)]
    pub error: Option<String>,
    /// Descripción textual del error devuelto por la pasarela de USPS.
    #[serde(default)]
    pub error_description: Option<String>,
    /// Lista de errores detallados o de validación de campos.
    #[serde(default)]
    pub errors: Vec<ApiErrorDetail>,
    /// Mensaje genérico de error de alto nivel.
    #[serde(default)]
    pub message: Option<String>,
}

/// Enumeración exhaustiva de los errores producidos por el cliente SDK de USPS v3.
#[derive(Debug, Error)]
pub enum UspsError {
    /// Fallo en la comunicación de red o transporte HTTP subyacente.
    #[error("Error de transporte HTTP: {0}")]
    Http(#[from] reqwest::Error),

    /// Fallo en la serialización o deserialización de datos JSON.
    #[error("Error de serialización/deserialización JSON: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Error de autenticación OAuth 2.0 (ej. credenciales inválidas o token expirado).
    #[error("Fallo de autenticación OAuth 2.0: {0}")]
    Auth(String),

    /// Error reportado por los endpoints de USPS con código de estado HTTP y cuerpo estructurado.
    #[error("Error de API USPS (HTTP {status}): {message}")]
    Api {
        /// Código de estado HTTP retornado por USPS.
        status: StatusCode,
        /// Mensaje descriptivo resumido del error.
        message: String,
        /// Estructura deserializada de la respuesta de error de USPS (si estuvo disponible).
        response: Option<UspsApiErrorResponse>,
    },

    /// Entrada inválida suministrada por el usuario a un método del SDK.
    #[error("Parámetro de entrada inválido: {0}")]
    InvalidInput(String),

    /// El cliente o recurso solicitado no fue encontrado en el servidor.
    #[error("Recurso no encontrado: {0}")]
    NotFound(String),

    /// Error general o interno del SDK.
    #[error("Error interno del SDK: {0}")]
    Internal(String),
}

impl UspsError {
    /// Construye un error de API a partir de un código de estado HTTP y texto de respuesta bruto.
    #[must_use]
    pub fn from_response(status: StatusCode, body: &str) -> Self {
        if let Ok(parsed) = serde_json::from_str::<UspsApiErrorResponse>(body) {
            let message = parsed
                .error_description
                .clone()
                .or_else(|| parsed.message.clone())
                .or_else(|| parsed.error.clone())
                .unwrap_or_else(|| format!("HTTP status {status}"));

            Self::Api {
                status,
                message,
                response: Some(parsed),
            }
        } else {
            Self::Api {
                status,
                message: if body.is_empty() {
                    format!("HTTP status {status}")
                } else {
                    body.to_string()
                },
                response: None,
            }
        }
    }

    /// Intenta extraer o inferir el código de error tipado de USPS ([`UspsErrorCode`]).
    #[must_use]
    pub fn error_code(&self) -> Option<UspsErrorCode> {
        match self {
            Self::Auth(_) => Some(UspsErrorCode::InvalidCredentials),
            Self::NotFound(_) => Some(UspsErrorCode::AddressNotFound),
            Self::Api {
                status,
                message,
                response,
            } => {
                // 1. Revisar detalles específicos en response.errors
                if let Some(resp) = response {
                    for detail in &resp.errors {
                        if let Some(ref c) = detail.code {
                            return Some(UspsErrorCode::parse(c));
                        }
                    }
                    if let Some(ref c) = resp.error {
                        return Some(UspsErrorCode::parse(c));
                    }
                }
                // 2. Revisar por código de estado HTTP
                match *status {
                    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                        Some(UspsErrorCode::Unauthorized)
                    }
                    StatusCode::TOO_MANY_REQUESTS => Some(UspsErrorCode::RateLimitExceeded),
                    StatusCode::SERVICE_UNAVAILABLE => Some(UspsErrorCode::ServiceUnavailable),
                    StatusCode::NOT_FOUND => {
                        let lower = message.to_lowercase();
                        if lower.contains("track") {
                            Some(UspsErrorCode::TrackingNumberNotFound)
                        } else {
                            Some(UspsErrorCode::AddressNotFound)
                        }
                    }
                    _ => {
                        let lower = message.to_lowercase();
                        if lower.contains("rate limit") || lower.contains("too many requests") {
                            Some(UspsErrorCode::RateLimitExceeded)
                        } else if lower.contains("invalid zip") {
                            Some(UspsErrorCode::InvalidZipCode)
                        } else if lower.contains("not found") {
                            Some(UspsErrorCode::AddressNotFound)
                        } else if !message.is_empty() {
                            Some(UspsErrorCode::Other(message.clone()))
                        } else {
                            None
                        }
                    }
                }
            }
            _ => None,
        }
    }

    /// Retorna `true` si el error corresponde a un recurso no encontrado (HTTP 404, NotFound o Tracking/Address Not Found).
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        match self {
            Self::NotFound(_) => true,
            Self::Api { status, .. } if *status == StatusCode::NOT_FOUND => true,
            _ => matches!(
                self.error_code(),
                Some(UspsErrorCode::AddressNotFound | UspsErrorCode::TrackingNumberNotFound)
            ),
        }
    }

    /// Retorna `true` si el error fue originado por exceso de tasa de peticiones (HTTP 429 o RateLimitExceeded).
    #[must_use]
    pub fn is_rate_limited(&self) -> bool {
        match self {
            Self::Api { status, .. } if *status == StatusCode::TOO_MANY_REQUESTS => true,
            _ => matches!(self.error_code(), Some(UspsErrorCode::RateLimitExceeded)),
        }
    }

    /// Retorna `true` si el error es atribuible a fallos de autenticación o credenciales (HTTP 401, 403 o error Auth).
    #[must_use]
    pub fn is_auth_error(&self) -> bool {
        match self {
            Self::Auth(_) => true,
            Self::Api { status, .. }
                if *status == StatusCode::UNAUTHORIZED || *status == StatusCode::FORBIDDEN =>
            {
                true
            }
            _ => matches!(
                self.error_code(),
                Some(
                    UspsErrorCode::InvalidCredentials
                        | UspsErrorCode::TokenExpired
                        | UspsErrorCode::Unauthorized
                )
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_response_should_parse_structured_json() {
        let raw = r#"{
            "error": "invalid_client",
            "error_description": "Invalid client credentials provided"
        }"#;

        let err = UspsError::from_response(StatusCode::UNAUTHORIZED, raw);
        match err {
            UspsError::Api {
                status,
                message,
                response,
            } => {
                assert_eq!(status, StatusCode::UNAUTHORIZED);
                assert_eq!(message, "Invalid client credentials provided");
                assert!(response.is_some());
            }
            _ => panic!("Expected UspsError::Api variant"),
        }
    }

    #[test]
    fn from_response_should_fallback_when_body_is_not_json() {
        let raw = "Bad Gateway";
        let err = UspsError::from_response(StatusCode::BAD_GATEWAY, raw);
        match err {
            UspsError::Api {
                status,
                message,
                response,
            } => {
                assert_eq!(status, StatusCode::BAD_GATEWAY);
                assert_eq!(message, "Bad Gateway");
                assert!(response.is_none());
            }
            _ => panic!("Expected UspsError::Api variant"),
        }
    }

    #[test]
    fn error_code_parsing_and_mapping() {
        assert_eq!(
            UspsErrorCode::parse("ADDRESS_NOT_FOUND"),
            UspsErrorCode::AddressNotFound
        );
        assert_eq!(
            UspsErrorCode::parse("invalid_zip_code"),
            UspsErrorCode::InvalidZipCode
        );
        assert_eq!(
            UspsErrorCode::parse("INVALID_CLIENT"),
            UspsErrorCode::InvalidCredentials
        );
        assert_eq!(
            UspsErrorCode::parse("TOKEN_EXPIRED"),
            UspsErrorCode::TokenExpired
        );
        assert_eq!(
            UspsErrorCode::parse("RATE_LIMIT_EXCEEDED"),
            UspsErrorCode::RateLimitExceeded
        );
        assert_eq!(
            UspsErrorCode::parse("TRACKING_NOT_FOUND"),
            UspsErrorCode::TrackingNumberNotFound
        );
        assert_eq!(
            UspsErrorCode::parse("LABEL_ALREADY_CANCELLED"),
            UspsErrorCode::LabelAlreadyCancelled
        );
        assert_eq!(
            UspsErrorCode::parse("INSUFFICIENT_FUNDS"),
            UspsErrorCode::InsufficientFunds
        );
        assert_eq!(
            UspsErrorCode::parse("DUPLICATE_MANIFEST"),
            UspsErrorCode::DuplicateManifest
        );
        assert_eq!(
            UspsErrorCode::parse("PICKUP_NOT_AVAILABLE"),
            UspsErrorCode::PickupNotAvailable
        );
        assert_eq!(
            UspsErrorCode::parse("CUSTOM_UNKNOWN_CODE"),
            UspsErrorCode::Other("CUSTOM_UNKNOWN_CODE".to_string())
        );
    }

    #[test]
    fn error_code_display_and_serde() {
        let code = UspsErrorCode::RateLimitExceeded;
        assert_eq!(code.to_string(), "RateLimitExceeded");

        let json = serde_json::to_string(&code).expect("serialize");
        assert_eq!(json, "\"RateLimitExceeded\"");

        let deserialized: UspsErrorCode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, UspsErrorCode::RateLimitExceeded);
    }

    #[test]
    fn error_inspection_helpers_should_work_correctly() {
        let auth_err = UspsError::Auth("credenciales no válidas".into());
        assert!(auth_err.is_auth_error());
        assert_eq!(
            auth_err.error_code(),
            Some(UspsErrorCode::InvalidCredentials)
        );
        assert!(!auth_err.is_not_found());
        assert!(!auth_err.is_rate_limited());

        let not_found_err = UspsError::NotFound("recurso inexistente".into());
        assert!(not_found_err.is_not_found());
        assert_eq!(
            not_found_err.error_code(),
            Some(UspsErrorCode::AddressNotFound)
        );

        let rate_limited = UspsError::from_response(StatusCode::TOO_MANY_REQUESTS, "");
        assert!(rate_limited.is_rate_limited());
        assert_eq!(
            rate_limited.error_code(),
            Some(UspsErrorCode::RateLimitExceeded)
        );

        let structured_detail = r#"{
            "errors": [{
                "code": "TRACKING_NOT_FOUND",
                "message": "Tracking number not in system"
            }]
        }"#;
        let tracking_err = UspsError::from_response(StatusCode::NOT_FOUND, structured_detail);
        assert!(tracking_err.is_not_found());
        assert_eq!(
            tracking_err.error_code(),
            Some(UspsErrorCode::TrackingNumberNotFound)
        );

        let invalid_input = UspsError::InvalidInput("valor faltante".into());
        assert!(!invalid_input.is_auth_error());
        assert!(!invalid_input.is_not_found());
        assert!(!invalid_input.is_rate_limited());
        assert_eq!(invalid_input.error_code(), None);
    }
}
