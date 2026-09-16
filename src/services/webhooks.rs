// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y modelos para la API REST v3 de Suscripciones y Webhooks (`Subscriptions v3`).
//!
//! Permite registrar callbacks HTTP / webhooks para recibir notificaciones automáticas en tiempo real
//! cuando se registren eventos de rastreo o cambios de estado en paquetes de USPS.

use std::fmt;

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Tipos de eventos postales a los cuales un webhook puede suscribirse.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionEventType {
    /// Cualquier evento o actualización general de escaneo de rastreo.
    TrackingEvents,
    /// Notificación específica cuando un paquete es entregado al destinatario.
    PackageDelivered,
    /// Notificación de intento de entrega fallido o excepciones de transporte.
    DeliveryException,
    /// Paquete devuelto al remitente (Return to Sender).
    ReturnToSender,
}

impl fmt::Display for SubscriptionEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TrackingEvents => write!(f, "TRACKING_EVENTS"),
            Self::PackageDelivered => write!(f, "PACKAGE_DELIVERED"),
            Self::DeliveryException => write!(f, "DELIVERY_EXCEPTION"),
            Self::ReturnToSender => write!(f, "RETURN_TO_SENDER"),
        }
    }
}

/// Solicitud de registro de una nueva suscripción de webhook (`POST /subscriptions/v3/subscription`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSubscriptionRequest {
    /// URL del endpoint HTTP/HTTPS receptor que recibirá los payloads de notificación.
    pub callback_url: String,
    /// Lista de eventos específicos a los que se suscribe.
    pub event_types: Vec<SubscriptionEventType>,
    /// Clave secreta o token para verificar la firma de autenticidad en cada webhook (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_token: Option<String>,
    /// Descripción o alias descriptivo de la suscripción (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CreateSubscriptionRequest {
    /// Inicia una solicitud de webhook con la URL receptora y los eventos suscritos.
    #[must_use]
    pub fn new(callback_url: impl Into<String>, event_types: Vec<SubscriptionEventType>) -> Self {
        Self {
            callback_url: callback_url.into(),
            event_types,
            secret_token: None,
            description: None,
        }
    }

    /// Asigna una clave secreta compartida para validar firmas HMAC de los webhooks entrantes.
    #[must_use]
    pub fn secret_token(mut self, token: impl Into<String>) -> Self {
        self.secret_token = Some(token.into());
        self
    }

    /// Asigna una descripción identificadora al webhook.
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Respuesta tras registrar o consultar una suscripción de webhook en USPS.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionResponse {
    /// Identificador único de la suscripción emitido por USPS.
    pub subscription_id: String,
    /// URL del callback configurada.
    pub callback_url: String,
    /// Estado de la suscripción (ej. "ACTIVE", "SUSPENDED").
    pub status: String,
    /// Eventos suscritos.
    #[serde(default)]
    pub event_types: Vec<String>,
    /// Fecha y hora de creación de la suscripción (si está disponible).
    #[serde(default)]
    pub created_at: Option<String>,
    /// Descripción asociada.
    #[serde(default)]
    pub description: Option<String>,
}

/// Respuesta de cancelación o eliminación de una suscripción de webhook.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSubscriptionResponse {
    /// Estado de la eliminación.
    pub status: String,
    /// Mensaje descriptivo retornado por USPS.
    #[serde(default)]
    pub message: Option<String>,
}

/// Servicio de la API v3 de Suscripciones y Notificaciones Webhook (`Subscriptions v3`).
#[derive(Debug, Clone)]
pub struct WebhooksService {
    client: UspsClient,
}

impl WebhooksService {
    /// Crea una nueva instancia del servicio de webhooks asociada a [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Registra una nueva suscripción de webhook (`POST /subscriptions/v3/subscription`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si la URL de callback está vacía o no es HTTPS, o si no se especifican eventos.
    #[instrument(skip(self), name = "create_subscription")]
    pub async fn subscribe(&self, req: &CreateSubscriptionRequest) -> Result<SubscriptionResponse> {
        let url = req.callback_url.trim();
        if url.is_empty() || (!url.starts_with("https://") && !url.starts_with("http://")) {
            return Err(UspsError::InvalidInput(
                "La URL del callback (callback_url) debe ser una URL HTTP/HTTPS válida".to_string(),
            ));
        }

        if req.event_types.is_empty() {
            return Err(UspsError::InvalidInput(
                "Debe seleccionarse al menos un tipo de evento (event_types) para la suscripción"
                    .to_string(),
            ));
        }

        let endpoint = "/subscriptions/v3/subscription";
        self.client.post_json(endpoint, req).await
    }

    /// Consulta una suscripción activa por su identificador (`GET /subscriptions/v3/subscription/{subscriptionId}`).
    #[instrument(skip(self), name = "get_subscription")]
    pub async fn get_subscription(&self, subscription_id: &str) -> Result<SubscriptionResponse> {
        let clean_id = subscription_id.trim();
        if clean_id.is_empty() {
            return Err(UspsError::InvalidInput(
                "El identificador subscription_id no puede estar vacío".to_string(),
            ));
        }

        let endpoint = format!("/subscriptions/v3/subscription/{clean_id}");
        let empty_query: [(&str, &str); 0] = [];
        self.client.get_with_query(&endpoint, &empty_query).await
    }

    /// Elimina o desuscribe un webhook (`DELETE /subscriptions/v3/subscription/{subscriptionId}`).
    #[instrument(skip(self), name = "delete_subscription")]
    pub async fn delete_subscription(
        &self,
        subscription_id: &str,
    ) -> Result<DeleteSubscriptionResponse> {
        let clean_id = subscription_id.trim();
        if clean_id.is_empty() {
            return Err(UspsError::InvalidInput(
                "El identificador subscription_id no puede estar vacío".to_string(),
            ));
        }

        let endpoint = format!("/subscriptions/v3/subscription/{clean_id}");
        let body = self.client.delete(&endpoint).await?;

        if body.trim().is_empty() {
            Ok(DeleteSubscriptionResponse {
                status: "DELETED".to_string(),
                message: Some(format!(
                    "Suscripción {clean_id} eliminada satisfactoriamente"
                )),
            })
        } else {
            serde_json::from_str::<DeleteSubscriptionResponse>(&body)
                .map_err(UspsError::Serialization)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_subscription_request_builder() {
        let req = CreateSubscriptionRequest::new(
            "https://api.myapp.com/webhooks/usps",
            vec![
                SubscriptionEventType::PackageDelivered,
                SubscriptionEventType::DeliveryException,
            ],
        )
        .secret_token("super_secret_hmac_key")
        .description("Producción e-commerce");

        assert_eq!(req.callback_url, "https://api.myapp.com/webhooks/usps");
        assert_eq!(req.event_types.len(), 2);
        assert_eq!(req.secret_token.as_deref(), Some("super_secret_hmac_key"));
        assert_eq!(req.description.as_deref(), Some("Producción e-commerce"));
    }

    #[test]
    fn subscription_response_deserialization() {
        let json = r#"{
            "subscriptionId": "SUB-12345",
            "callbackUrl": "https://api.myapp.com/webhooks/usps",
            "status": "ACTIVE",
            "eventTypes": ["PACKAGE_DELIVERED"],
            "createdAt": "2026-09-15T20:00:00Z"
        }"#;

        let res: SubscriptionResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.subscription_id, "SUB-12345");
        assert_eq!(res.callback_url, "https://api.myapp.com/webhooks/usps");
        assert_eq!(res.status, "ACTIVE");
        assert_eq!(res.event_types, vec!["PACKAGE_DELIVERED"]);
    }
}
