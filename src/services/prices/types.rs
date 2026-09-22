// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Tipos de datos, modelos de solicitud y respuesta para la API de Precios y Tarifas (`Prices v3`).

use std::fmt;

use serde::{Deserialize, Serialize};

/// Clases de correspondencia y servicios de paquetería de USPS.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MailClass {
    /// USPS Ground Advantage (servicio terrestre económico estándar).
    UspsGroundAdvantage,
    /// Priority Mail (entrega rápida de 1 a 3 días hábiles).
    PriorityMail,
    /// Priority Mail Express (entrega nocturna o garantizada de 1 a 2 días).
    PriorityMailExpress,
    /// Media Mail (libros y medios educativos calificados).
    MediaMail,
    /// Library Mail (materiales de préstamo bibliotecario calificados).
    LibraryMail,
    /// First-Class Mail (cartas y postales ligeras).
    FirstClassMail,
}

impl fmt::Display for MailClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UspsGroundAdvantage => write!(f, "USPS_GROUND_ADVANTAGE"),
            Self::PriorityMail => write!(f, "PRIORITY_MAIL"),
            Self::PriorityMailExpress => write!(f, "PRIORITY_MAIL_EXPRESS"),
            Self::MediaMail => write!(f, "MEDIA_MAIL"),
            Self::LibraryMail => write!(f, "LIBRARY_MAIL"),
            Self::FirstClassMail => write!(f, "FIRST_CLASS_MAIL"),
        }
    }
}

/// Categoría de procesamiento postal según la maquinabilidad y morfología del bulto.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProcessingCategory {
    /// Cartas y sobres estándar.
    Letters,
    /// Sobres grandes y planos (Flats).
    Flats,
    /// Paquetes procesables en maquinaria automática.
    Machinable,
    /// Paquetes con dimensiones o formas irregulares no maquinables.
    NonMachinable,
}

/// Solicitud de cotización de tarifas nacionales (`POST /prices/v3/base-rates/search`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DomesticRateRequest {
    /// Código postal de origen (5 dígitos).
    #[serde(rename = "originZIPCode")]
    pub origin_zip_code: String,
    /// Código postal de destino (5 dígitos).
    #[serde(rename = "destinationZIPCode")]
    pub destination_zip_code: String,
    /// Peso del paquete en libras (ej. 2.5).
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
    /// Clase de correo específica a cotizar (si se omite, USPS cotiza todas las clases elegibles).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mail_class: Option<MailClass>,
    /// Categoría de procesamiento del paquete.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub processing_category: Option<ProcessingCategory>,
    /// Fecha proyectada de imposición postal en formato YYYY-MM-DD (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mailing_date: Option<String>,
}

impl DomesticRateRequest {
    /// Inicia una solicitud de cotización con origen, destino y peso requeridos.
    #[must_use]
    pub fn new(
        origin_zip_code: impl Into<String>,
        destination_zip_code: impl Into<String>,
        weight_lbs: f64,
    ) -> Self {
        Self {
            origin_zip_code: origin_zip_code.into(),
            destination_zip_code: destination_zip_code.into(),
            weight: weight_lbs,
            length: None,
            width: None,
            height: None,
            mail_class: None,
            processing_category: None,
            mailing_date: None,
        }
    }

    /// Asigna las dimensiones físicas del paquete en pulgadas.
    #[must_use]
    pub fn dimensions(mut self, length: f64, width: f64, height: f64) -> Self {
        self.length = Some(length);
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Filtra por una clase de correo específica.
    #[must_use]
    pub fn mail_class(mut self, mail_class: MailClass) -> Self {
        self.mail_class = Some(mail_class);
        self
    }

    /// Especifica la categoría morfológica del envío.
    #[must_use]
    pub fn processing_category(mut self, category: ProcessingCategory) -> Self {
        self.processing_category = Some(category);
        self
    }

    /// Asigna la fecha proyectada de imposición postal (YYYY-MM-DD).
    #[must_use]
    pub fn mailing_date(mut self, date: impl Into<String>) -> Self {
        self.mailing_date = Some(date.into());
        self
    }
}

/// Detalle de una opción de tarifa cotizada devuelta por USPS.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateItem {
    /// Nombre de la clase de correo correspondiente.
    #[serde(default)]
    pub mail_class: Option<String>,
    /// Precio de franqueo base en dólares (USD).
    pub price: f64,
    /// Descripción legible del servicio.
    #[serde(default)]
    pub description: Option<String>,
    /// Zona tarifaria asignada según la distancia origen-destino (ej. "Zone 4").
    #[serde(default)]
    pub zone: Option<String>,
    /// Código SKU o identificador del producto tarifario.
    #[serde(default)]
    pub sku: Option<String>,
}

/// Respuesta tras cotizar tarifas de franqueo nacionales.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomesticRateResponse {
    /// Precio total o menor base calculado en USD.
    #[serde(default)]
    pub total_base_price: Option<f64>,
    /// Lista de opciones tarifarias calculadas.
    #[serde(default)]
    pub rates: Vec<RateItem>,
    /// Advertencias devueltas por el motor de tarifas de USPS.
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Clases de servicio postal internacional de USPS.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InternationalMailClass {
    /// Global Express Guaranteed (GXG - entrega internacional más rápida garantizada).
    GlobalExpressGuaranteed,
    /// Priority Mail Express International (entrega rápida de 3 a 5 días hábiles).
    PriorityMailExpressInternational,
    /// Priority Mail International (entrega económica confiable de 6 a 10 días hábiles).
    PriorityMailInternational,
    /// First-Class Package International Service (para paquetes pequeños y ligeros de hasta 4 lbs).
    FirstClassPackageInternationalService,
    /// First-Class Mail International (cartas y postales internacionales).
    FirstClassMailInternational,
}

impl fmt::Display for InternationalMailClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GlobalExpressGuaranteed => write!(f, "GLOBAL_EXPRESS_GUARANTEED"),
            Self::PriorityMailExpressInternational => {
                write!(f, "PRIORITY_MAIL_EXPRESS_INTERNATIONAL")
            }
            Self::PriorityMailInternational => write!(f, "PRIORITY_MAIL_INTERNATIONAL"),
            Self::FirstClassPackageInternationalService => {
                write!(f, "FIRST_CLASS_PACKAGE_INTERNATIONAL_SERVICE")
            }
            Self::FirstClassMailInternational => write!(f, "FIRST_CLASS_MAIL_INTERNATIONAL"),
        }
    }
}

/// Solicitud de cotización de tarifas internacionales (`POST /prices/v3/international-base-rates/search`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternationalRateRequest {
    /// Código postal de origen en EE. UU. (5 dígitos).
    #[serde(rename = "originZIPCode")]
    pub origin_zip_code: String,
    /// Código del país de destino (código de 2 letras ISO 3166-1 alpha-2, ej. "CA", "GB", "MX", "DE", "ES").
    pub destination_country_code: String,
    /// Código postal del país de destino si aplica (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreign_postal_code: Option<String>,
    /// Peso del paquete en libras (ej. 1.75).
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
    /// Clase internacional específica a cotizar (si se omite, USPS cotiza todas las clases elegibles).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mail_class: Option<InternationalMailClass>,
    /// Fecha proyectada de despacho postal (YYYY-MM-DD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mailing_date: Option<String>,
}

impl InternationalRateRequest {
    /// Inicia una solicitud de cotización internacional con origen, país de destino y peso.
    #[must_use]
    pub fn new(
        origin_zip_code: impl Into<String>,
        destination_country_code: impl Into<String>,
        weight_lbs: f64,
    ) -> Self {
        Self {
            origin_zip_code: origin_zip_code.into(),
            destination_country_code: destination_country_code.into(),
            foreign_postal_code: None,
            weight: weight_lbs,
            length: None,
            width: None,
            height: None,
            mail_class: None,
            mailing_date: None,
        }
    }

    /// Asigna las dimensiones físicas del paquete en pulgadas.
    #[must_use]
    pub fn dimensions(mut self, length: f64, width: f64, height: f64) -> Self {
        self.length = Some(length);
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Asigna el código postal extranjero de destino.
    #[must_use]
    pub fn foreign_postal_code(mut self, postal_code: impl Into<String>) -> Self {
        self.foreign_postal_code = Some(postal_code.into());
        self
    }

    /// Filtra la cotización para una clase internacional específica.
    #[must_use]
    pub fn mail_class(mut self, mail_class: InternationalMailClass) -> Self {
        self.mail_class = Some(mail_class);
        self
    }

    /// Asigna la fecha proyectada de despacho postal (YYYY-MM-DD).
    #[must_use]
    pub fn mailing_date(mut self, date: impl Into<String>) -> Self {
        self.mailing_date = Some(date.into());
        self
    }
}

/// Respuesta de cotización de tarifas internacionales.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InternationalRateResponse {
    /// Precio total o menor base calculado en USD.
    #[serde(default)]
    pub total_base_price: Option<f64>,
    /// Lista de tarifas internacionales cotizadas.
    #[serde(default)]
    pub rates: Vec<RateItem>,
    /// Advertencias o notas arancelarias devueltas por USPS.
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Servicios especiales y complementarios de USPS (Extra Services).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExtraServiceType {
    /// Certificación de envío con acuse y rastreo especial.
    CertifiedMail,
    /// Seguro postal por pérdida o daño de mercancías.
    Insurance,
    /// Confirmación de firma del destinatario al momento de entrega.
    SignatureConfirmation,
    /// Firma obligatoria de un adulto (21+ años).
    AdultSignatureRequired,
    /// Firma obligatoria de adulto con entrega restringida únicamente al destinatario.
    AdultSignatureRestrictedDelivery,
    /// Acuse de recibo postal verde o electrónico (Return Receipt).
    ReturnReceipt,
    /// Correo registrado (Registered Mail - máxima seguridad con custodia estricta).
    RegisteredMail,
    /// Cobro contra entrega (Collect on Delivery - COD).
    CollectOnDelivery,
    /// Manejo especial para artículos frágiles o delicados.
    SpecialHandling,
    /// Entrega restringida personalmente al destinatario.
    RestrictedDelivery,
}

impl fmt::Display for ExtraServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CertifiedMail => write!(f, "CERTIFIED_MAIL"),
            Self::Insurance => write!(f, "INSURANCE"),
            Self::SignatureConfirmation => write!(f, "SIGNATURE_CONFIRMATION"),
            Self::AdultSignatureRequired => write!(f, "ADULT_SIGNATURE_REQUIRED"),
            Self::AdultSignatureRestrictedDelivery => {
                write!(f, "ADULT_SIGNATURE_RESTRICTED_DELIVERY")
            }
            Self::ReturnReceipt => write!(f, "RETURN_RECEIPT"),
            Self::RegisteredMail => write!(f, "REGISTERED_MAIL"),
            Self::CollectOnDelivery => write!(f, "COLLECT_ON_DELIVERY"),
            Self::SpecialHandling => write!(f, "SPECIAL_HANDLING"),
            Self::RestrictedDelivery => write!(f, "RESTRICTED_DELIVERY"),
        }
    }
}

/// Solicitud de cotización de servicios adicionales (`POST /prices/v3/extra-services`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraServicesRateRequest {
    /// Clase de correo postal para la que se cotizan los servicios adicionales.
    pub mail_class: MailClass,
    /// Precio de franqueo base del paquete en USD.
    pub price: f64,
    /// Peso del paquete en libras (ej. 2.5).
    pub weight: f64,
    /// Valor monetario declarado del paquete en USD (requerido para cotizar seguro).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_value: Option<f64>,
    /// Lista de tipos de servicios específicos a cotizar (si está vacía, se devuelven todos los compatibles).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub extra_services: Vec<ExtraServiceType>,
}

impl ExtraServicesRateRequest {
    /// Crea una nueva solicitud de cotización de servicios adicionales con clase, precio base y peso.
    #[must_use]
    pub fn new(mail_class: MailClass, base_price: f64, weight_lbs: f64) -> Self {
        Self {
            mail_class,
            price: base_price,
            weight: weight_lbs,
            declared_value: None,
            extra_services: Vec::new(),
        }
    }

    /// Asigna el valor declarado del paquete para el cálculo del costo de seguro.
    #[must_use]
    pub fn declared_value(mut self, value_usd: f64) -> Self {
        self.declared_value = Some(value_usd);
        self
    }

    /// Agrega un tipo de servicio adicional específico a la consulta.
    #[must_use]
    pub fn add_extra_service(mut self, service: ExtraServiceType) -> Self {
        self.extra_services.push(service);
        self
    }
}

/// Detalle de tarifa para un servicio adicional específico cotizado.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraServiceRateItem {
    /// Nombre oficial del servicio adicional según USPS.
    pub name: String,
    /// Identificador numérico o alfanumérico del servicio si está disponible.
    #[serde(default)]
    pub service_id: Option<String>,
    /// Tarifa adicional calculada en USD.
    pub price: f64,
    /// Descripción detallada del servicio adicional.
    #[serde(default)]
    pub description: Option<String>,
    /// Advertencias o restricciones para este servicio.
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Respuesta de cotización de servicios adicionales (`POST /prices/v3/extra-services`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraServicesRateResponse {
    /// Desglose de servicios adicionales cotizados y sus costos.
    #[serde(default)]
    pub extra_services: Vec<ExtraServiceRateItem>,
    /// Advertencias generales devueltas por USPS.
    #[serde(default)]
    pub warnings: Vec<String>,
}
