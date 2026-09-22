// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tipos de datos, modelos de solicitud y respuesta para la API de Etiquetas Postales (`Labels v3`).

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::services::prices::MailClass;

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

/// Solicitud de generación de identificador y código QR de Label Broker (`POST /labels/v3/label-broker`).
///
/// Permite que un cliente imprima la etiqueta directamente en una oficina o quiosco de USPS
/// presentando el código QR desde su dispositivo móvil sin necesidad de contar con impresora.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelBrokerRequest {
    /// Dirección del remitente u origen del envío.
    pub from_address: LabelPartyAddress,
    /// Dirección del destinatario de entrega.
    pub to_address: LabelPartyAddress,
    /// Especificaciones del bulto y clase de envío.
    pub package_description: PackageDescription,
    /// Referencia externa o ID de pedido en el sistema del remitente.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_reference_id: Option<String>,
}

impl LabelBrokerRequest {
    /// Construye una nueva solicitud de Label Broker para entrega postal.
    #[must_use]
    pub fn new(
        from_address: LabelPartyAddress,
        to_address: LabelPartyAddress,
        package_description: PackageDescription,
    ) -> Self {
        Self {
            from_address,
            to_address,
            package_description,
            customer_reference_id: None,
        }
    }

    /// Asigna una referencia de cliente externa (ej. número de orden).
    #[must_use]
    pub fn customer_reference_id(mut self, ref_id: impl Into<String>) -> Self {
        self.customer_reference_id = Some(ref_id.into());
        self
    }
}

/// Respuesta tras registrar una solicitud de Label Broker en USPS.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelBrokerResponse {
    /// Identificador único Label Broker ID para impresión en quiosco o mostrador.
    pub label_broker_id: String,
    /// Número de seguimiento USPS asignado al envío.
    pub tracking_number: String,
    /// Imagen del código QR codificada en Base64 para escaneo en ventanilla o quiosco.
    #[serde(default)]
    pub qr_code: Option<String>,
    /// Estado del registro (ej. "ACTIVE", "PENDING").
    pub status: String,
    /// Fecha de vencimiento o expiración del código Label Broker (si aplica).
    #[serde(default)]
    pub expiration_date: Option<String>,
}
