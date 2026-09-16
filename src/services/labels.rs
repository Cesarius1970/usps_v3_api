// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y modelos para la API REST v3 de Generación de Etiquetas Postales (`Labels v3`).
//!
//! Permite la emisión de etiquetas oficiales con código de barras USPS, cálculo de franqueo,
//! generación de archivos de imagen (PDF, PNG, TIFF) y codificación Base64 para impresión.

use std::fmt;

use serde::{Deserialize, Serialize};
use tracing::instrument;

use super::prices::MailClass;
use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Formato de salida y representación gráfica de la etiqueta postal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LabelImageType {
    /// Documento PDF estándar (ideal para impresoras láser o térmicas).
    #[default]
    Pdf,
    /// Imagen en formato PNG de alta resolución.
    Png,
    /// Formato TIFF para sistemas de impresión industrial.
    Tiff,
    /// Gráficos vectoriales SVG.
    Svg,
}

impl fmt::Display for LabelImageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pdf => write!(f, "PDF"),
            Self::Png => write!(f, "PNG"),
            Self::Tiff => write!(f, "TIFF"),
            Self::Svg => write!(f, "SVG"),
        }
    }
}

/// Dimensiones físicas y tipo de etiqueta a imprimir.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LabelSize {
    /// Formato térmico estándar de 4x6 pulgadas.
    #[default]
    #[serde(rename = "4X6")]
    Label4x6,
    /// Formato compacto de 4x4 pulgadas.
    #[serde(rename = "4X4")]
    Label4x4,
}

/// Metadatos sobre el formato visual de la etiqueta solicitada.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    /// Formato de archivo (PDF, PNG, etc.).
    pub image_type: LabelImageType,
    /// Dimensión física de la etiqueta.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label_type: Option<LabelSize>,
}

/// Dirección postal para remitente (from) o destinatario (to) en una etiqueta.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelPartyAddress {
    /// Nombre de la persona o contacto.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// Apellido de la persona o contacto.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// Razón social, empresa o nombre de fantasía.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firm_name: Option<String>,
    /// Línea principal de calle.
    pub street_address: String,
    /// Línea secundaria (ej. apto, suite).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_address: Option<String>,
    /// Ciudad.
    pub city: String,
    /// Estado o territorio (2 caracteres).
    pub state: String,
    /// Código postal (5 dígitos).
    #[serde(rename = "ZIPCode")]
    pub zip_code: String,
    /// Extensión postal ZIP+4 (opcional).
    #[serde(rename = "ZIPPlus4", skip_serializing_if = "Option::is_none")]
    pub zip_plus4: Option<String>,
    /// Teléfono de contacto (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// Correo electrónico (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

impl LabelPartyAddress {
    /// Crea una nueva dirección postal requerida para la etiqueta.
    #[must_use]
    pub fn new(
        street_address: impl Into<String>,
        city: impl Into<String>,
        state: impl Into<String>,
        zip_code: impl Into<String>,
    ) -> Self {
        Self {
            first_name: None,
            last_name: None,
            firm_name: None,
            street_address: street_address.into(),
            secondary_address: None,
            city: city.into(),
            state: state.into(),
            zip_code: zip_code.into(),
            zip_plus4: None,
            phone: None,
            email: None,
        }
    }

    /// Asigna el nombre de la persona o destinatario.
    #[must_use]
    pub fn person(mut self, first_name: impl Into<String>, last_name: impl Into<String>) -> Self {
        self.first_name = Some(first_name.into());
        self.last_name = Some(last_name.into());
        self
    }

    /// Asigna la empresa o razón social.
    #[must_use]
    pub fn firm_name(mut self, firm: impl Into<String>) -> Self {
        self.firm_name = Some(firm.into());
        self
    }

    /// Asigna la línea secundaria (suite, piso, oficina).
    #[must_use]
    pub fn secondary_address(mut self, sec: impl Into<String>) -> Self {
        self.secondary_address = Some(sec.into());
        self
    }
}

/// Características físicas y operativas del paquete para la emisión de la etiqueta.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageDescription {
    /// Clase de correo USPS (ej. Priority Mail, Ground Advantage).
    pub mail_class: MailClass,
    /// Peso en libras.
    pub weight: f64,
    /// Longitud en pulgadas (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<f64>,
    /// Ancho en pulgadas (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    /// Altura en pulgadas (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// Fecha estimada de despacho postal (YYYY-MM-DD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mailing_date: Option<String>,
}

impl PackageDescription {
    /// Inicializa la descripción física del paquete con su clase y peso en libras.
    #[must_use]
    pub fn new(mail_class: MailClass, weight_lbs: f64) -> Self {
        Self {
            mail_class,
            weight: weight_lbs,
            length: None,
            width: None,
            height: None,
            mailing_date: None,
        }
    }

    /// Añade dimensiones físicas al paquete.
    #[must_use]
    pub fn dimensions(mut self, length: f64, width: f64, height: f64) -> Self {
        self.length = Some(length);
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

/// Solicitud completa de creación y emisión de etiqueta postal (`POST /labels/v3/label`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLabelRequest {
    /// Metadatos sobre la imagen solicitada (formato, dimensiones).
    pub image_info: ImageInfo,
    /// Dirección física del remitente.
    pub from_address: LabelPartyAddress,
    /// Dirección física del destinatario.
    pub to_address: LabelPartyAddress,
    /// Especificaciones del bulto y clase de envío.
    pub package_description: PackageDescription,
}

impl CreateLabelRequest {
    /// Construye una nueva solicitud de etiqueta postal.
    #[must_use]
    pub fn new(
        from_address: LabelPartyAddress,
        to_address: LabelPartyAddress,
        package_description: PackageDescription,
    ) -> Self {
        Self {
            image_info: ImageInfo {
                image_type: LabelImageType::Pdf,
                label_type: Some(LabelSize::Label4x6),
            },
            from_address,
            to_address,
            package_description,
        }
    }

    /// Asigna el formato de imagen deseado (PDF, PNG, etc.).
    #[must_use]
    pub fn image_type(mut self, image_type: LabelImageType) -> Self {
        self.image_info.image_type = image_type;
        self
    }
}

/// Respuesta devuelta por USPS tras generar satisfactoriamente una etiqueta postal.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLabelResponse {
    /// Identificador único de la etiqueta en el sistema USPS.
    pub label_id: String,
    /// Número de seguimiento USPS generado para el paquete.
    pub tracking_number: String,
    /// Código Label Broker ID para impresión en quioscos postales (si aplica).
    #[serde(default)]
    pub label_broker_id: Option<String>,
    /// Contenido binario de la imagen o documento de la etiqueta codificado en Base64.
    #[serde(default)]
    pub label_image: Option<String>,
    /// URL directa temporal provista por USPS para la descarga de la etiqueta (si aplica).
    #[serde(default)]
    pub label_url: Option<String>,
    /// Costo total de franqueo devengado por la emisión de la etiqueta.
    #[serde(default)]
    pub total_price: Option<f64>,
    /// Advertencias o notas operativas generadas por USPS.
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Respuesta de cancelación o anulación de etiqueta postal.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelLabelResponse {
    /// Estado o mensaje de confirmación de la cancelación.
    pub status: String,
    /// Mensaje descriptivo retornado por USPS.
    #[serde(default)]
    pub message: Option<String>,
}

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

    /// Genera y emite una nueva etiqueta postal (`POST /labels/v3/label`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si faltan datos requeridos en las direcciones o el peso es inválido,
    /// o fallas de red/API devueltas por USPS.
    #[instrument(skip(self), name = "create_label")]
    pub async fn create_label(&self, req: &CreateLabelRequest) -> Result<CreateLabelResponse> {
        if req.from_address.street_address.trim().is_empty()
            || req.from_address.city.trim().is_empty()
            || req.from_address.state.trim().is_empty()
        {
            return Err(UspsError::InvalidInput(
                "La dirección del remitente (from_address) está incompleta".to_string(),
            ));
        }

        if req.to_address.street_address.trim().is_empty()
            || req.to_address.city.trim().is_empty()
            || req.to_address.state.trim().is_empty()
        {
            return Err(UspsError::InvalidInput(
                "La dirección del destinatario (to_address) está incompleta".to_string(),
            ));
        }

        if req.package_description.weight <= 0.0 {
            return Err(UspsError::InvalidInput(
                "El peso del paquete debe ser estrictamente mayor a 0 libras".to_string(),
            ));
        }

        let endpoint = "/labels/v3/label";
        self.client.post_json(endpoint, req).await
    }

    /// Anula o cancela una etiqueta emitida previamente (`DELETE /labels/v3/label/{labelId}`).
    #[instrument(skip(self), name = "cancel_label")]
    pub async fn cancel_label(&self, label_id: &str) -> Result<CancelLabelResponse> {
        let clean_id = label_id.trim();
        if clean_id.is_empty() {
            return Err(UspsError::InvalidInput(
                "El label_id no puede estar vacío".to_string(),
            ));
        }

        let endpoint = format!("/labels/v3/label/{clean_id}");
        let body = self.client.delete(&endpoint).await?;

        if body.trim().is_empty() {
            Ok(CancelLabelResponse {
                status: "CANCELLED".to_string(),
                message: Some(format!("Etiqueta {clean_id} anulada satisfactoriamente")),
            })
        } else {
            serde_json::from_str::<CancelLabelResponse>(&body).map_err(UspsError::Serialization)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_label_request_builder() {
        let from = LabelPartyAddress::new("123 Sender Way", "Austin", "TX", "78701")
            .person("Alice", "Smith");
        let to = LabelPartyAddress::new("456 Receiver Ave", "New York", "NY", "10001")
            .person("Bob", "Jones");
        let pkg = PackageDescription::new(MailClass::PriorityMail, 2.0).dimensions(10.0, 5.0, 4.0);

        let req = CreateLabelRequest::new(from, to, pkg).image_type(LabelImageType::Pdf);

        assert_eq!(req.image_info.image_type, LabelImageType::Pdf);
        assert_eq!(req.from_address.city, "Austin");
        assert_eq!(req.to_address.city, "New York");
        assert_eq!(req.package_description.mail_class, MailClass::PriorityMail);
        assert_eq!(req.package_description.weight, 2.0);
    }

    #[test]
    fn create_label_response_deserialization() {
        let json = r#"{
            "labelId": "LBL-1234567890",
            "trackingNumber": "9405500000000000000000",
            "labelBrokerId": "LB-999",
            "labelImage": "JVBERi0xLjQKJ...",
            "totalPrice": 10.50,
            "warnings": []
        }"#;

        let res: CreateLabelResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.label_id, "LBL-1234567890");
        assert_eq!(res.tracking_number, "9405500000000000000000");
        assert_eq!(res.label_broker_id.as_deref(), Some("LB-999"));
        assert_eq!(res.total_price, Some(10.50));
        assert!(res.label_image.is_some());
    }

    #[test]
    fn label_image_type_display() {
        assert_eq!(LabelImageType::Pdf.to_string(), "PDF");
        assert_eq!(LabelImageType::Png.to_string(), "PNG");
        assert_eq!(LabelImageType::Tiff.to_string(), "TIFF");
        assert_eq!(LabelImageType::Svg.to_string(), "SVG");
    }
}
