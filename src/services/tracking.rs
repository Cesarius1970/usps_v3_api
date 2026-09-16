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
}
