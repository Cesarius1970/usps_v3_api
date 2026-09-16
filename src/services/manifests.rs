// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y modelos para la API REST v3 de Manifiestos y Formularios SCAN Form (`Manifests v3`).
//!
//! Permite consolidar múltiples envíos y paquetes etiquetados en una sola hoja de manifiesto oficial
//! de aceptación postal (**USPS SCAN Form - PS Form 5630**) con un código de barras maestro único.

use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::labels::{LabelImageType, LabelPartyAddress};
use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Solicitud de generación de manifiesto postal / SCAN Form (`POST /manifests/v3/manifest`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateManifestRequest {
    /// Dirección del remitente u origen del despacho postal.
    pub from_address: LabelPartyAddress,
    /// Código postal de 5 dígitos de la oficina o instalación donde se ingresarán los bultos.
    #[serde(rename = "entryFacilityZIPCode")]
    pub entry_facility_zip_code: String,
    /// Formato gráfico deseado para el documento del manifiesto (ej. PDF).
    pub image_type: LabelImageType,
    /// Fecha estimada de imposición postal en formato YYYY-MM-DD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mailing_date: Option<String>,
    /// Lista de identificadores de etiquetas (`label_id`) que se consolidarán en el manifiesto.
    pub label_ids: Vec<String>,
}

impl CreateManifestRequest {
    /// Inicia una solicitud de manifiesto postal requiriendo la dirección de origen y el código postal de ingreso.
    #[must_use]
    pub fn new(
        from_address: LabelPartyAddress,
        entry_facility_zip_code: impl Into<String>,
        label_ids: Vec<String>,
    ) -> Self {
        Self {
            from_address,
            entry_facility_zip_code: entry_facility_zip_code.into(),
            image_type: LabelImageType::Pdf,
            mailing_date: None,
            label_ids,
        }
    }

    /// Define el formato de imagen del manifiesto (por defecto PDF).
    #[must_use]
    pub fn image_type(mut self, image_type: LabelImageType) -> Self {
        self.image_type = image_type;
        self
    }

    /// Asigna la fecha de despacho postal (YYYY-MM-DD).
    #[must_use]
    pub fn mailing_date(mut self, date: impl Into<String>) -> Self {
        self.mailing_date = Some(date.into());
        self
    }
}

/// Respuesta devuelta tras emitir un formulario de manifiesto postal SCAN Form.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateManifestResponse {
    /// Identificador único del manifiesto generado por USPS.
    pub manifest_id: String,
    /// Número maestro de seguimiento / código de barras impreso en el Formulario PS 5630.
    pub manifest_tracking_number: String,
    /// Documento del manifiesto codificado en Base64 (PDF/PNG).
    #[serde(default)]
    pub manifest_image: Option<String>,
    /// URL temporal directa para descargar o imprimir el documento del manifiesto (si aplica).
    #[serde(default)]
    pub manifest_url: Option<String>,
    /// Cantidad total de paquetes o envíos consolidados en este manifiesto.
    pub total_packages: u32,
    /// Advertencias o notas operativas devueltas por USPS.
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Servicio de la API v3 de Manifiestos y Formularios SCAN Form de USPS.
#[derive(Debug, Clone)]
pub struct ManifestsService {
    client: UspsClient,
}

impl ManifestsService {
    /// Crea una nueva instancia del servicio de manifiestos asociada a [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Genera y emite un nuevo formulario de manifiesto postal SCAN Form (`POST /manifests/v3/manifest`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si no se incluye al menos un `label_id`, o si el código postal de ingreso es inválido.
    #[instrument(skip(self), name = "create_manifest")]
    pub async fn create_manifest(
        &self,
        req: &CreateManifestRequest,
    ) -> Result<CreateManifestResponse> {
        if req.label_ids.is_empty() {
            return Err(UspsError::InvalidInput(
                "Debe especificarse al menos una etiqueta (label_id) para generar el manifiesto"
                    .to_string(),
            ));
        }

        let zip = req.entry_facility_zip_code.trim();
        if zip.len() != 5 || !zip.chars().all(|c| c.is_ascii_digit()) {
            return Err(UspsError::InvalidInput(
                "El código postal de la instalación de ingreso (entryFacilityZIPCode) debe contener 5 dígitos"
                    .to_string(),
            ));
        }

        let endpoint = "/manifests/v3/manifest";
        self.client.post_json(endpoint, req).await
    }

    /// Consulta y recupera un manifiesto emitido previamente (`GET /manifests/v3/manifest/{manifestId}`).
    #[instrument(skip(self), name = "get_manifest")]
    pub async fn get_manifest(&self, manifest_id: &str) -> Result<CreateManifestResponse> {
        let clean_id = manifest_id.trim();
        if clean_id.is_empty() {
            return Err(UspsError::InvalidInput(
                "El identificador del manifiesto (manifest_id) no puede estar vacío".to_string(),
            ));
        }

        let endpoint = format!("/manifests/v3/manifest/{clean_id}");
        let empty_query: [(&str, &str); 0] = [];
        self.client.get_with_query(&endpoint, &empty_query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_manifest_request_builder() {
        let from = LabelPartyAddress::new("100 Warehouse Way", "Orlando", "FL", "32801");
        let labels = vec!["LBL-1001".to_string(), "LBL-1002".to_string()];

        let req = CreateManifestRequest::new(from, "32801", labels)
            .image_type(LabelImageType::Pdf)
            .mailing_date("2026-09-21");

        assert_eq!(req.entry_facility_zip_code, "32801");
        assert_eq!(req.label_ids.len(), 2);
        assert_eq!(req.image_type, LabelImageType::Pdf);
        assert_eq!(req.mailing_date.as_deref(), Some("2026-09-21"));
    }

    #[test]
    fn create_manifest_response_deserialization() {
        let json = r#"{
            "manifestId": "MNF-998877",
            "manifestTrackingNumber": "9275099999999999999999",
            "manifestImage": "JVBERi0xLjQKJ...",
            "totalPackages": 2,
            "warnings": []
        }"#;

        let res: CreateManifestResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.manifest_id, "MNF-998877");
        assert_eq!(res.manifest_tracking_number, "9275099999999999999999");
        assert_eq!(res.total_packages, 2);
        assert!(res.manifest_image.is_some());
    }
}
