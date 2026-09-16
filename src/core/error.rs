// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Definición jerárquica y fuertemente tipada de errores del SDK `usps_v3_api`.

use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

/// Tipo alias para resultados que retornan [`UspsError`].
pub type Result<T> = std::result::Result<T, UspsError>;

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
}
