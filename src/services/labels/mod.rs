// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y servicio para la API REST v3 de Generación de Etiquetas Postales (`Labels v3`).
//!
//! Permite la emisión de etiquetas oficiales con código de barras USPS, cancelación,
//! generación de códigos QR con Label Broker y recuperación de imágenes.

pub mod types;

pub use types::*;

use tracing::instrument;

use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Servicio de la API v3 de Generación y Gestión de Etiquetas Postales (`Labels v3`).
#[derive(Debug, Clone)]
pub struct LabelsService {
    client: UspsClient,
}

impl LabelsService {
    /// Crea un nuevo servicio vinculado al cliente central [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Emite una nueva etiqueta postal oficial con franqueo y código de barras (`POST /labels/v3/label`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si las direcciones o los datos de peso del paquete son inválidos.
    #[instrument(skip(self), name = "create_label")]
    pub async fn create_label(&self, req: &CreateLabelRequest) -> Result<CreateLabelResponse> {
        let from = &req.from_address;
        let to = &req.to_address;

        if from.street_address.trim().is_empty() || to.street_address.trim().is_empty() {
            return Err(UspsError::InvalidInput(
                "Las direcciones de origen y destino deben contener una calle válida".to_string(),
            ));
        }

        if from.zip_code.trim().len() != 5 || to.zip_code.trim().len() != 5 {
            return Err(UspsError::InvalidInput(
                "Los códigos postales origen y destino deben tener exactamente 5 dígitos"
                    .to_string(),
            ));
        }

        if req.package_description.weight <= 0.0 {
            return Err(UspsError::InvalidInput(
                "El peso del paquete a etiquetar debe ser estrictamente mayor a 0 libras"
                    .to_string(),
            ));
        }

        let endpoint = "/labels/v3/label";
        self.client.post_json(endpoint, req).await
    }

    /// Cancela o anula una etiqueta postal previamente emitida (`DELETE /labels/v3/label/{labelId}`).
    ///
    /// Permite reversar cargos o tramitar reembolsos de franqueo no utilizado en la pasarela de USPS.
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si el identificador de la etiqueta está vacío o contiene caracteres inválidos.
    #[instrument(skip(self), name = "cancel_label")]
    pub async fn cancel_label(&self, label_id: &str) -> Result<CancelLabelResponse> {
        let clean_id = label_id.trim();
        if clean_id.is_empty() || clean_id.contains(['/', '?', '&', '#', ' ']) {
            return Err(UspsError::InvalidInput(
                "El identificador de etiqueta para anulación no es válido".to_string(),
            ));
        }

        let endpoint = format!("/labels/v3/label/{clean_id}");
        let raw_body = self.client.delete(&endpoint).await?;

        if raw_body.trim().is_empty() {
            Ok(CancelLabelResponse {
                status: "CANCELLED".to_string(),
                message: Some("La etiqueta fue anulada con éxito".to_string()),
            })
        } else {
            serde_json::from_str::<CancelLabelResponse>(&raw_body).map_err(UspsError::Serialization)
        }
    }

    /// Genera un identificador y código QR de USPS Label Broker (`POST /labels/v3/label-broker`).
    ///
    /// El código QR devuelto permite al remitente acudir a cualquier mostrador postal o quiosco de USPS
    /// para que el personal imprima físicamente la etiqueta de franqueo.
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si los datos del remitente, destinatario o peso son inválidos.
    #[instrument(skip(self), name = "create_label_broker")]
    pub async fn create_label_broker(
        &self,
        req: &LabelBrokerRequest,
    ) -> Result<LabelBrokerResponse> {
        let from = &req.from_address;
        let to = &req.to_address;

        if from.street_address.trim().is_empty() || to.street_address.trim().is_empty() {
            return Err(UspsError::InvalidInput(
                "Las direcciones de origen y destino deben contener una calle válida".to_string(),
            ));
        }

        if from.zip_code.trim().len() != 5 || to.zip_code.trim().len() != 5 {
            return Err(UspsError::InvalidInput(
                "Los códigos postales origen y destino deben tener exactamente 5 dígitos"
                    .to_string(),
            ));
        }

        if req.package_description.weight <= 0.0 {
            return Err(UspsError::InvalidInput(
                "El peso del paquete para Label Broker debe ser mayor a 0 libras".to_string(),
            ));
        }

        let endpoint = "/labels/v3/label-broker";
        self.client.post_json(endpoint, req).await
    }

    /// Consulta y recupera los datos y metadatos de una etiqueta previamente emitida (`GET /labels/v3/label/{labelId}`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si el `label_id` está vacío o contiene caracteres inválidos.
    #[instrument(skip(self), name = "get_label_data")]
    pub async fn get_label_data(&self, label_id: &str) -> Result<CreateLabelResponse> {
        let clean_id = label_id.trim();
        if clean_id.is_empty() || clean_id.contains(['/', '?', '&', '#', ' ']) {
            return Err(UspsError::InvalidInput(
                "El identificador de etiqueta proporcionado no es válido".to_string(),
            ));
        }

        let endpoint = format!("/labels/v3/label/{clean_id}");
        let empty_query: [(&str, &str); 0] = [];
        self.client.get_with_query(&endpoint, &empty_query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::prices::MailClass;

    #[test]
    fn create_label_request_builder() {
        let from = LabelPartyAddress::new("123 Main St", "Orlando", "FL", "32801")
            .person("John", "Doe")
            .firm_name("Acme Corp")
            .secondary_address("Suite 100");

        let to =
            LabelPartyAddress::new("456 Market St", "Miami", "FL", "33101").person("Jane", "Smith");

        let pkg = PackageDescription::new(MailClass::PriorityMail, 2.5).dimensions(10.0, 6.0, 4.0);

        let req = CreateLabelRequest::new(from, to, pkg).image_type(LabelImageType::Pdf);

        assert_eq!(req.from_address.street_address, "123 Main St");
        assert_eq!(req.from_address.city, "Orlando");
        assert_eq!(req.from_address.state, "FL");
        assert_eq!(req.from_address.zip_code, "32801");
        assert_eq!(req.from_address.first_name.as_deref(), Some("John"));
        assert_eq!(
            req.from_address.secondary_address.as_deref(),
            Some("Suite 100")
        );

        assert_eq!(req.to_address.city, "Miami");
        assert_eq!(req.package_description.weight, 2.5);
        assert_eq!(req.package_description.length, Some(10.0));
        assert_eq!(req.image_info.image_type, LabelImageType::Pdf);
    }

    #[test]
    fn create_label_response_deserialization() {
        let json = r#"{
            "labelId": "LBL-123456789",
            "trackingNumber": "9400100000000000000000",
            "labelBrokerId": "LB-9988",
            "labelImage": "JVBERi0xLjQKJ...",
            "labelUrl": "https://api.usps.com/labels/v3/label/LBL-123456789",
            "totalPrice": 10.45,
            "warnings": []
        }"#;

        let res: CreateLabelResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.label_id, "LBL-123456789");
        assert_eq!(res.tracking_number, "9400100000000000000000");
        assert_eq!(res.label_broker_id.as_deref(), Some("LB-9988"));
        assert_eq!(res.total_price, Some(10.45));
        assert_eq!(res.warnings.len(), 0);
    }

    #[test]
    fn label_image_type_display() {
        assert_eq!(LabelImageType::Pdf.to_string(), "PDF");
        assert_eq!(LabelImageType::Png.to_string(), "PNG");
        assert_eq!(LabelImageType::Tiff.to_string(), "TIFF");
        assert_eq!(LabelImageType::Svg.to_string(), "SVG");
    }

    #[test]
    fn label_broker_request_builder() {
        let from = LabelPartyAddress::new("123 Main St", "Orlando", "FL", "32801");
        let to = LabelPartyAddress::new("456 Market St", "Miami", "FL", "33101");
        let pkg = PackageDescription::new(MailClass::PriorityMail, 1.5);

        let req = LabelBrokerRequest::new(from, to, pkg).customer_reference_id("ORD-999");

        assert_eq!(req.customer_reference_id.as_deref(), Some("ORD-999"));
        assert_eq!(req.package_description.weight, 1.5);
    }

    #[test]
    fn label_broker_response_deserialization() {
        let json = r#"{
            "labelBrokerId": "LB-123456",
            "trackingNumber": "9400100000000000000000",
            "qrCode": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...",
            "status": "ACTIVE",
            "expirationDate": "2026-10-15"
        }"#;

        let res: LabelBrokerResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.label_broker_id, "LB-123456");
        assert_eq!(res.tracking_number, "9400100000000000000000");
        assert_eq!(res.status, "ACTIVE");
        assert!(res.qr_code.is_some());
        assert_eq!(res.expiration_date.as_deref(), Some("2026-10-15"));
    }
}
