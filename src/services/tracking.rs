// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y modelos para la API REST v3 de Seguimiento de Envíos de USPS (`Tracking v3`).
//!
//! Proporciona consulta de estado en tiempo real, eventos de tránsito, fechas estimadas
//! de entrega y detalle histórico de paquetes procesados por la red de USPS.

use std::fmt;

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Nivel de detalle solicitado para la consulta de seguimiento.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackingExpand {
    /// Devuelve todos los eventos históricos detallados del paquete.
    #[default]
    Detail,
    /// Devuelve únicamente el resumen del estado más reciente.
    Summary,
}

impl fmt::Display for TrackingExpand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Detail => write!(f, "detail"),
            Self::Summary => write!(f, "summary"),
        }
    }
}

/// Evento individual en la línea de tiempo del paquete reportado por USPS.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackingEvent {
    /// Tipo abreviado de evento postal (ej. "OF", "DE", "01").
    #[serde(default)]
    pub event_type: Option<String>,
    /// Fecha y hora del evento.
    #[serde(default)]
    pub event_timestamp: Option<String>,
    /// Ciudad donde se registró el evento de escaneo.
    #[serde(default)]
    pub event_city: Option<String>,
    /// Abreviatura del estado del evento.
    #[serde(default)]
    pub event_state: Option<String>,
    /// Código postal donde ocurrió el escaneo.
    #[serde(rename = "eventZIP", default)]
    pub event_zip: Option<String>,
    /// País donde se realizó el evento postal.
    #[serde(default)]
    pub event_country: Option<String>,
    /// Nombre o descripción legible del evento (ej. "Delivered, Front Door", "Arrived at USPS Facility").
    #[serde(default)]
    pub name: Option<String>,
    /// Código numérico normalizado de evento.
    #[serde(default)]
    pub event_code: Option<String>,
}

/// Respuesta estructurada con la información de rastreo de un paquete en USPS v3.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackingResponse {
    /// Número de seguimiento USPS del paquete.
    pub tracking_number: String,
    /// Fecha estimada de entrega (Estimated Time of Arrival) si está disponible.
    #[serde(default)]
    pub delivery_eta: Option<String>,
    /// Fecha garantizada de entrega (si aplica al servicio, ej. Priority Mail Express).
    #[serde(default)]
    pub guaranteed_delivery_date: Option<String>,
    /// Clase de correo o servicio utilizado (ej. "Priority Mail", "Ground Advantage").
    #[serde(default)]
    pub mail_class: Option<String>,
    /// Ciudad de origen del envío.
    #[serde(default)]
    pub origin_city: Option<String>,
    /// Estado de origen.
    #[serde(default)]
    pub origin_state: Option<String>,
    /// Código postal de origen.
    #[serde(rename = "originZIP", default)]
    pub origin_zip: Option<String>,
    /// Ciudad de destino.
    #[serde(default)]
    pub destination_city: Option<String>,
    /// Estado de destino.
    #[serde(default)]
    pub destination_state: Option<String>,
    /// Código postal de destino.
    #[serde(rename = "destinationZIP", default)]
    pub destination_zip: Option<String>,
    /// Estado actual del paquete (ej. "Delivered", "In Transit", "Pre-Shipment").
    #[serde(default)]
    pub status: Option<String>,
    /// Categoría general del estado.
    #[serde(default)]
    pub status_category: Option<String>,
    /// Resumen legible redactado por USPS del estado actual.
    #[serde(default)]
    pub status_summary: Option<String>,
    /// Historial de eventos de escaneo ordenados cronológicamente.
    #[serde(default)]
    pub tracking_events: Vec<TrackingEvent>,
}

/// Servicio de consulta y rastreo de envíos para la API v3 de USPS.
#[derive(Debug, Clone)]
pub struct TrackingService {
    client: UspsClient,
}

impl TrackingService {
    /// Crea una nueva instancia del servicio de rastreo asociado a [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Consulta el estado completo y los eventos detallados de un número de seguimiento.
    ///
    /// Equivale a consultar con [`TrackingExpand::Detail`].
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si el número de tracking está vacío o contiene caracteres inválidos.
    #[instrument(skip(self), name = "track_package")]
    pub async fn track(&self, tracking_number: &str) -> Result<TrackingResponse> {
        self.track_with_expand(tracking_number, TrackingExpand::Detail)
            .await
    }

    /// Consulta el estado de un paquete especificando si se requiere el desglose detallado o solo resumen.
    ///
    /// Realiza una solicitud `GET /tracking/v3/tracking/{trackingNumber}?expand={expand}`.
    #[instrument(skip(self), name = "track_package_with_expand")]
    pub async fn track_with_expand(
        &self,
        tracking_number: &str,
        expand: TrackingExpand,
    ) -> Result<TrackingResponse> {
        let clean_number = tracking_number.trim();
        if clean_number.is_empty() {
            return Err(UspsError::InvalidInput(
                "El número de seguimiento no puede estar vacío".to_string(),
            ));
        }

        if clean_number.contains(['/', '?', '&', '#', ' ']) {
            return Err(UspsError::InvalidInput(
                "El número de seguimiento contiene caracteres inválidos".to_string(),
            ));
        }

        let endpoint = format!("/tracking/v3/tracking/{clean_number}");
        let query = [("expand", expand.to_string())];

        self.client.get_with_query(&endpoint, &query).await
    }

    /// Consulta el estado de múltiples números de seguimiento simultáneamente (hasta 35 paquetes por lote).
    ///
    /// Realiza una solicitud `GET /tracking/v3/tracking?trackingNumbers={nums}&expand={expand}`.
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si la lista está vacía, excede 35 números o contiene identificadores inválidos.
    #[instrument(skip(self, tracking_numbers), name = "track_batch")]
    pub async fn track_batch(
        &self,
        tracking_numbers: &[impl AsRef<str>],
        expand: TrackingExpand,
    ) -> Result<Vec<TrackingResponse>> {
        if tracking_numbers.is_empty() {
            return Err(UspsError::InvalidInput(
                "Debe indicarse al menos un número de seguimiento para la consulta por lotes"
                    .to_string(),
            ));
        }

        if tracking_numbers.len() > 35 {
            return Err(UspsError::InvalidInput(
                "USPS admite un máximo de 35 números de seguimiento por consulta por lotes"
                    .to_string(),
            ));
        }

        let cleaned: Vec<String> = tracking_numbers
            .iter()
            .map(|n| n.as_ref().trim().to_string())
            .collect();

        for n in &cleaned {
            if n.is_empty() || n.contains(['/', '?', '&', '#', ' ']) {
                return Err(UspsError::InvalidInput(format!(
                    "Número de seguimiento inválido: '{n}'"
                )));
            }
        }

        let joined = cleaned.join(",");
        let endpoint = "/tracking/v3/tracking";
        let query = [("trackingNumbers", joined), ("expand", expand.to_string())];

        self.client.get_with_query(endpoint, &query).await
    }

    /// Solicita el envío por correo electrónico de la Prueba Electrónica de Entrega (ePOD - Electronic Proof of Delivery).
    ///
    /// Realiza una solicitud `POST /tracking/v3/proof-of-delivery`.
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si el número de seguimiento, correo o nombres son inválidos o están vacíos.
    #[instrument(skip(self), name = "request_proof_of_delivery")]
    pub async fn request_proof_of_delivery(
        &self,
        req: &ProofOfDeliveryRequest,
    ) -> Result<ProofOfDeliveryResponse> {
        let clean_number = req.tracking_number.trim();
        if clean_number.is_empty() || clean_number.contains(['/', '?', '&', '#', ' ']) {
            return Err(UspsError::InvalidInput(
                "El número de seguimiento no puede estar vacío ni contener caracteres especiales"
                    .to_string(),
            ));
        }

        let clean_email = req.email.trim();
        if clean_email.is_empty() || !clean_email.contains('@') {
            return Err(UspsError::InvalidInput(
                "Debe proporcionar una dirección de correo electrónico válida para recibir el comprobante".to_string(),
            ));
        }

        if req.first_name.trim().is_empty() || req.last_name.trim().is_empty() {
            return Err(UspsError::InvalidInput(
                "El nombre y apellido del solicitante son obligatorios".to_string(),
            ));
        }

        let endpoint = "/tracking/v3/proof-of-delivery";
        self.client.post_json(endpoint, req).await
    }
}

/// Formato solicitado para la Prueba Electrónica de Entrega (ePOD).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProofOfDeliveryFormat {
    /// Formato carta oficial de USPS en PDF.
    Letter,
    /// Datos y hoja de firma del receptor.
    Signature,
}

impl fmt::Display for ProofOfDeliveryFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Letter => write!(f, "LETTER"),
            Self::Signature => write!(f, "SIGNATURE"),
        }
    }
}

/// Solicitud de Prueba Electrónica de Entrega (`POST /tracking/v3/proof-of-delivery`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofOfDeliveryRequest {
    /// Número de seguimiento del envío USPS.
    pub tracking_number: String,
    /// Correo electrónico donde se remitirá la prueba de entrega.
    pub email: String,
    /// Nombre de pila del solicitante.
    pub first_name: String,
    /// Apellidos del solicitante.
    pub last_name: String,
    /// Formato deseado del comprobante (carta o firma).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ProofOfDeliveryFormat>,
}

impl ProofOfDeliveryRequest {
    /// Crea una nueva solicitud de prueba de entrega electrónica.
    #[must_use]
    pub fn new(
        tracking_number: impl Into<String>,
        email: impl Into<String>,
        first_name: impl Into<String>,
        last_name: impl Into<String>,
    ) -> Self {
        Self {
            tracking_number: tracking_number.into(),
            email: email.into(),
            first_name: first_name.into(),
            last_name: last_name.into(),
            format: None,
        }
    }

    /// Asigna el formato de entrega preferido (carta o firma).
    #[must_use]
    pub fn format(mut self, format: ProofOfDeliveryFormat) -> Self {
        self.format = Some(format);
        self
    }
}

/// Respuesta tras la solicitud de Prueba Electrónica de Entrega.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofOfDeliveryResponse {
    /// Número de seguimiento asociado a la solicitud.
    pub tracking_number: String,
    /// Identificador único de solicitud generado por USPS.
    #[serde(default)]
    pub request_id: Option<String>,
    /// Estado de tramitación de la solicitud (ej. "Request Processed", "Request Submitted").
    #[serde(default)]
    pub status: Option<String>,
    /// Correo electrónico confirmado al que se remitió el comprobante.
    #[serde(default)]
    pub email: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracking_response_deserialization() {
        let json = r#"{
            "trackingNumber": "9400100000000000000000",
            "status": "In Transit",
            "statusCategory": "In Transit",
            "statusSummary": "Your package is moving within the USPS network.",
            "originCity": "DALLAS",
            "originState": "TX",
            "originZIP": "75201",
            "destinationCity": "SEATTLE",
            "destinationState": "WA",
            "destinationZIP": "98101",
            "trackingEvents": [
                {
                    "eventType": "10",
                    "eventTimestamp": "2026-09-15T08:30:00",
                    "eventCity": "DALLAS",
                    "eventState": "TX",
                    "eventZIP": "75201",
                    "name": "Accepted at USPS Origin Facility"
                }
            ]
        }"#;

        let res: TrackingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.tracking_number, "9400100000000000000000");
        assert_eq!(res.status.as_deref(), Some("In Transit"));
        assert_eq!(res.tracking_events.len(), 1);
        assert_eq!(
            res.tracking_events[0].name.as_deref(),
            Some("Accepted at USPS Origin Facility")
        );
    }

    #[test]
    fn tracking_expand_display() {
        assert_eq!(TrackingExpand::Detail.to_string(), "detail");
        assert_eq!(TrackingExpand::Summary.to_string(), "summary");
    }

    #[tokio::test]
    async fn track_batch_validation_should_fail_when_empty() {
        let client = UspsClient::builder()
            .credentials("test", "secret")
            .build()
            .unwrap();

        let empty: [&str; 0] = [];
        let err = client
            .tracking()
            .track_batch(&empty, TrackingExpand::Summary)
            .await
            .unwrap_err();

        assert!(matches!(err, UspsError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn track_batch_validation_should_fail_when_over_35_items() {
        let client = UspsClient::builder()
            .credentials("test", "secret")
            .build()
            .unwrap();

        let mut nums = Vec::new();
        for i in 0..36 {
            nums.push(format!("94001000000000000000{i:02}"));
        }

        let err = client
            .tracking()
            .track_batch(&nums, TrackingExpand::Summary)
            .await
            .unwrap_err();

        assert!(matches!(err, UspsError::InvalidInput(_)));
    }

    #[test]
    fn proof_of_delivery_request_builder() {
        let req = ProofOfDeliveryRequest::new(
            "9400100000000000000000",
            "shipper@example.com",
            "Jane",
            "Doe",
        )
        .format(ProofOfDeliveryFormat::Letter);

        assert_eq!(req.tracking_number, "9400100000000000000000");
        assert_eq!(req.email, "shipper@example.com");
        assert_eq!(req.first_name, "Jane");
        assert_eq!(req.last_name, "Doe");
        assert_eq!(req.format, Some(ProofOfDeliveryFormat::Letter));
        assert_eq!(ProofOfDeliveryFormat::Letter.to_string(), "LETTER");
        assert_eq!(ProofOfDeliveryFormat::Signature.to_string(), "SIGNATURE");
    }

    #[test]
    fn proof_of_delivery_response_deserialization() {
        let json = r#"{
            "trackingNumber": "9400100000000000000000",
            "requestId": "POD-998877",
            "status": "Request Processed",
            "email": "shipper@example.com"
        }"#;

        let res: ProofOfDeliveryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.tracking_number, "9400100000000000000000");
        assert_eq!(res.request_id.as_deref(), Some("POD-998877"));
        assert_eq!(res.status.as_deref(), Some("Request Processed"));
        assert_eq!(res.email.as_deref(), Some("shipper@example.com"));
    }

    #[tokio::test]
    async fn proof_of_delivery_validation_should_fail_on_bad_inputs() {
        let client = UspsClient::builder()
            .credentials("test", "secret")
            .build()
            .unwrap();

        // Empty tracking
        let req1 = ProofOfDeliveryRequest::new("", "user@example.com", "Jane", "Doe");
        let err1 = client
            .tracking()
            .request_proof_of_delivery(&req1)
            .await
            .unwrap_err();
        assert!(matches!(err1, UspsError::InvalidInput(_)));

        // Invalid email
        let req2 =
            ProofOfDeliveryRequest::new("9400100000000000000000", "invalid-email", "Jane", "Doe");
        let err2 = client
            .tracking()
            .request_proof_of_delivery(&req2)
            .await
            .unwrap_err();
        assert!(matches!(err2, UspsError::InvalidInput(_)));

        // Empty name
        let req3 =
            ProofOfDeliveryRequest::new("9400100000000000000000", "user@example.com", "", "Doe");
        let err3 = client
            .tracking()
            .request_proof_of_delivery(&req3)
            .await
            .unwrap_err();
        assert!(matches!(err3, UspsError::InvalidInput(_)));
    }
}
