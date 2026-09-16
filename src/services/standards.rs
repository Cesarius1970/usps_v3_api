// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y modelos para la API REST v3 de Estándares de Servicio y Tiempos de Tránsito (`Service Standards v3`).
//!
//! Permite calcular compromisos de entrega postal, fechas estimadas de entrega (EDD - Expected Delivery Date)
//! y días de tránsito garantizados o proyectados entre códigos postales de origen y destino.

use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::prices::MailClass;
use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Solicitud de consulta de estándares de servicio y tiempos de entrega (`GET /service-standards/v3/estimates`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStandardRequest {
    /// Código postal de 5 dígitos de origen donde se deposita el envío.
    #[serde(rename = "originZIPCode")]
    pub origin_zip_code: String,
    /// Código postal de 5 dígitos de destino.
    #[serde(rename = "destinationZIPCode")]
    pub destination_zip_code: String,
    /// Fecha de imposición o depósito del envío en formato YYYY-MM-DD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acceptance_date: Option<String>,
    /// Filtro opcional por clase postal específica.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mail_class: Option<MailClass>,
}

impl ServiceStandardRequest {
    /// Inicia una consulta de estándares de servicio indicando códigos postales de origen y destino.
    #[must_use]
    pub fn new(
        origin_zip_code: impl Into<String>,
        destination_zip_code: impl Into<String>,
    ) -> Self {
        Self {
            origin_zip_code: origin_zip_code.into(),
            destination_zip_code: destination_zip_code.into(),
            acceptance_date: None,
            mail_class: None,
        }
    }

    /// Asigna la fecha estimada de aceptación o depósito postal (YYYY-MM-DD).
    #[must_use]
    pub fn acceptance_date(mut self, date: impl Into<String>) -> Self {
        self.acceptance_date = Some(date.into());
        self
    }

    /// Filtra el cálculo por una clase de envío específica (ej. Priority Mail).
    #[must_use]
    pub fn mail_class(mut self, mail_class: MailClass) -> Self {
        self.mail_class = Some(mail_class);
        self
    }
}

/// Estimación o estándar de servicio para una clase de correo específica.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStandardEstimate {
    /// Clase postal evaluada (ej. "PRIORITY_MAIL", "USPS_GROUND_ADVANTAGE").
    pub mail_class: String,
    /// Días o compromiso estándar de servicio (ej. "1 Day", "2 Days", "3 Days").
    pub service_standard: String,
    /// Mensaje descriptivo con el compromiso de entrega.
    #[serde(default)]
    pub service_standard_message: Option<String>,
    /// Fecha estimada de entrega calculada (YYYY-MM-DD).
    #[serde(default)]
    pub scheduled_delivery_date: Option<String>,
    /// Hora límite de corte de depósito para cumplir el plazo (ej. "17:00:00").
    #[serde(default)]
    pub cutoff_time: Option<String>,
}

/// Respuesta devuelta por la API de Estándares de Servicio de USPS.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStandardResponse {
    /// Código postal de origen.
    #[serde(rename = "originZIPCode")]
    pub origin_zip_code: String,
    /// Código postal de destino.
    #[serde(rename = "destinationZIPCode")]
    pub destination_zip_code: String,
    /// Fecha de depósito postal considerada.
    #[serde(default)]
    pub acceptance_date: Option<String>,
    /// Lista de compromisos y estándares de servicio calculados por clase postal.
    #[serde(default)]
    pub service_standards: Vec<ServiceStandardEstimate>,
}

/// Servicio de la API v3 de Estándares de Servicio y Tiempos de Tránsito de USPS.
#[derive(Debug, Clone)]
pub struct ServiceStandardsService {
    client: UspsClient,
}

impl ServiceStandardsService {
    /// Crea un nuevo servicio vinculado a [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Consulta los estándares de servicio y fechas estimadas de entrega (`GET /service-standards/v3/estimates`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si los códigos postales de origen o destino no tienen 5 dígitos numéricos.
    #[instrument(skip(self), name = "get_service_standard_estimates")]
    pub async fn get_estimates(
        &self,
        req: &ServiceStandardRequest,
    ) -> Result<ServiceStandardResponse> {
        let origin = req.origin_zip_code.trim();
        if origin.len() != 5 || !origin.chars().all(|c| c.is_ascii_digit()) {
            return Err(UspsError::InvalidInput(
                "El código postal de origen (originZIPCode) debe contener 5 dígitos numéricos"
                    .to_string(),
            ));
        }

        let destination = req.destination_zip_code.trim();
        if destination.len() != 5 || !destination.chars().all(|c| c.is_ascii_digit()) {
            return Err(UspsError::InvalidInput(
                "El código postal de destino (destinationZIPCode) debe contener 5 dígitos numéricos"
                    .to_string(),
            ));
        }

        let endpoint = "/service-standards/v3/estimates";
        self.client.get_with_query(endpoint, req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_standard_request_builder() {
        let req = ServiceStandardRequest::new("90210", "10001")
            .acceptance_date("2026-09-22")
            .mail_class(MailClass::PriorityMail);

        assert_eq!(req.origin_zip_code, "90210");
        assert_eq!(req.destination_zip_code, "10001");
        assert_eq!(req.acceptance_date.as_deref(), Some("2026-09-22"));
        assert_eq!(req.mail_class, Some(MailClass::PriorityMail));
    }

    #[test]
    fn service_standard_response_deserialization() {
        let json = r#"{
            "originZIPCode": "90210",
            "destinationZIPCode": "10001",
            "acceptanceDate": "2026-09-22",
            "serviceStandards": [
                {
                    "mailClass": "PRIORITY_MAIL",
                    "serviceStandard": "2 Days",
                    "serviceStandardMessage": "Expected delivery in 2 business days",
                    "scheduledDeliveryDate": "2026-09-24",
                    "cutoffTime": "17:00:00"
                },
                {
                    "mailClass": "USPS_GROUND_ADVANTAGE",
                    "serviceStandard": "4 Days",
                    "serviceStandardMessage": "Expected delivery in 4 business days",
                    "scheduledDeliveryDate": "2026-09-26",
                    "cutoffTime": "17:00:00"
                }
            ]
        }"#;

        let res: ServiceStandardResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.origin_zip_code, "90210");
        assert_eq!(res.destination_zip_code, "10001");
        assert_eq!(res.service_standards.len(), 2);
        assert_eq!(res.service_standards[0].mail_class, "PRIORITY_MAIL");
        assert_eq!(res.service_standards[0].service_standard, "2 Days");
        assert_eq!(
            res.service_standards[0].scheduled_delivery_date.as_deref(),
            Some("2026-09-24")
        );
    }
}
