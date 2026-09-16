// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y modelos para la API REST v3 de Precios y Tarifas de USPS (`Prices v3`).
//!
//! Permite calcular cotizaciones y tarifas de franqueo para envíos nacionales (Domestic Base Rates),
//! considerando clase postal, peso, dimensiones de empaque y servicios complementarios.

use std::fmt;

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::client::UspsClient;
use crate::error::{Result, UspsError};

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

    /// Asigna las dimensiones volumétricas del paquete en pulgadas.
    #[must_use]
    pub fn dimensions(mut self, length: f64, width: f64, height: f64) -> Self {
        self.length = Some(length);
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Filtra la cotización para una clase de correo específica.
    #[must_use]
    pub fn mail_class(mut self, mail_class: MailClass) -> Self {
        self.mail_class = Some(mail_class);
        self
    }

    /// Asigna la categoría de procesamiento.
    #[must_use]
    pub fn processing_category(mut self, category: ProcessingCategory) -> Self {
        self.processing_category = Some(category);
        self
    }

    /// Define la fecha estimada de despacho postal (YYYY-MM-DD).
    #[must_use]
    pub fn mailing_date(mut self, date: impl Into<String>) -> Self {
        self.mailing_date = Some(date.into());
        self
    }
}

/// Tarifa cotizada para una clase o servicio complementario.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RateItem {
    /// Clase de correo o servicio tasado.
    #[serde(default)]
    pub mail_class: Option<String>,
    /// Precio o tarifa base calculado en USD.
    pub price: f64,
    /// Descripción comercial de la tarifa.
    #[serde(default)]
    pub description: Option<String>,
    /// Código SKU o identificador de catálogo de la tarifa.
    #[serde(default)]
    pub sku: Option<String>,
    /// Zona postal calculada entre el origen y el destino (ej. "Zone 4").
    #[serde(default)]
    pub zone: Option<String>,
    /// Cargo o recargo adicional por dimensiones no estándar si aplica.
    #[serde(default)]
    pub nonstandard_fee: Option<f64>,
}

/// Respuesta devuelta por el servicio de tarifas de USPS.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomesticRateResponse {
    /// Precio total o menor base calculado.
    #[serde(default)]
    pub total_base_price: Option<f64>,
    /// Lista de opciones de tarifas cotizadas.
    #[serde(default)]
    pub rates: Vec<RateItem>,
    /// Advertencias o notas arancelarias devueltas por USPS.
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Servicio de la API v3 de Precios y Tarifas de USPS.
#[derive(Debug, Clone)]
pub struct PricesService {
    client: UspsClient,
}

impl PricesService {
    /// Crea un nuevo servicio vinculado al cliente central [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Calcula las tarifas postales nacionales (`POST /prices/v3/base-rates/search`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si los códigos postales o el peso son inválidos,
    /// o errores de red/autenticación si la llamada remota falla.
    #[instrument(skip(self), name = "calculate_domestic_rates")]
    pub async fn calculate_domestic_rates(
        &self,
        req: &DomesticRateRequest,
    ) -> Result<DomesticRateResponse> {
        let origin = req.origin_zip_code.trim();
        let dest = req.destination_zip_code.trim();

        if origin.len() != 5 || !origin.chars().all(|c| c.is_ascii_digit()) {
            return Err(UspsError::InvalidInput(
                "El código postal de origen debe tener exactamente 5 dígitos".to_string(),
            ));
        }

        if dest.len() != 5 || !dest.chars().all(|c| c.is_ascii_digit()) {
            return Err(UspsError::InvalidInput(
                "El código postal de destino debe tener exactamente 5 dígitos".to_string(),
            ));
        }

        if req.weight <= 0.0 {
            return Err(UspsError::InvalidInput(
                "El peso del paquete debe ser estrictamente mayor a 0 libras".to_string(),
            ));
        }

        let endpoint = "/prices/v3/base-rates/search";
        self.client.post_json(endpoint, req).await
    }

    /// Calcula las tarifas postales internacionales (`POST /prices/v3/international-base-rates/search`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si el origen, el código de país o el peso son inválidos.
    #[instrument(skip(self), name = "calculate_international_rates")]
    pub async fn calculate_international_rates(
        &self,
        req: &InternationalRateRequest,
    ) -> Result<InternationalRateResponse> {
        let origin = req.origin_zip_code.trim();
        let country = req.destination_country_code.trim();

        if origin.len() != 5 || !origin.chars().all(|c| c.is_ascii_digit()) {
            return Err(UspsError::InvalidInput(
                "El código postal de origen debe tener exactamente 5 dígitos".to_string(),
            ));
        }

        if country.len() != 2 {
            return Err(UspsError::InvalidInput(
                "El código del país de destino debe ser un código ISO de 2 caracteres (ej. 'CA', 'GB', 'MX')".to_string(),
            ));
        }

        if req.weight <= 0.0 {
            return Err(UspsError::InvalidInput(
                "El peso del paquete debe ser estrictamente mayor a 0 libras".to_string(),
            ));
        }

        let endpoint = "/prices/v3/international-base-rates/search";
        self.client.post_json(endpoint, req).await
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domestic_rate_request_builder() {
        let req = DomesticRateRequest::new("90210", "10001", 3.25)
            .dimensions(12.0, 8.0, 4.5)
            .mail_class(MailClass::PriorityMail)
            .processing_category(ProcessingCategory::Machinable);

        assert_eq!(req.origin_zip_code, "90210");
        assert_eq!(req.destination_zip_code, "10001");
        assert_eq!(req.weight, 3.25);
        assert_eq!(req.length, Some(12.0));
        assert_eq!(req.mail_class, Some(MailClass::PriorityMail));
        assert_eq!(
            req.processing_category,
            Some(ProcessingCategory::Machinable)
        );
    }

    #[test]
    fn domestic_rate_response_deserialization() {
        let json = r#"{
            "totalBasePrice": 9.65,
            "rates": [
                {
                    "mailClass": "PRIORITY_MAIL",
                    "price": 9.65,
                    "description": "Priority Mail 2-Day",
                    "zone": "Zone 8",
                    "sku": "PM-D-08"
                },
                {
                    "mailClass": "USPS_GROUND_ADVANTAGE",
                    "price": 7.15,
                    "description": "USPS Ground Advantage",
                    "zone": "Zone 8"
                }
            ],
            "warnings": []
        }"#;

        let res: DomesticRateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.total_base_price, Some(9.65));
        assert_eq!(res.rates.len(), 2);
        assert_eq!(res.rates[0].mail_class.as_deref(), Some("PRIORITY_MAIL"));
        assert_eq!(res.rates[0].price, 9.65);
        assert_eq!(res.rates[0].zone.as_deref(), Some("Zone 8"));
        assert_eq!(res.rates[1].price, 7.15);
    }

    #[test]
    fn mail_class_display() {
        assert_eq!(MailClass::PriorityMail.to_string(), "PRIORITY_MAIL");
        assert_eq!(
            MailClass::UspsGroundAdvantage.to_string(),
            "USPS_GROUND_ADVANTAGE"
        );
        assert_eq!(
            InternationalMailClass::PriorityMailInternational.to_string(),
            "PRIORITY_MAIL_INTERNATIONAL"
        );
    }

    #[test]
    fn international_rate_request_builder() {
        let req = InternationalRateRequest::new("90210", "CA", 4.5)
            .foreign_postal_code("M5V 2T6")
            .dimensions(10.0, 8.0, 6.0)
            .mail_class(InternationalMailClass::PriorityMailInternational);

        assert_eq!(req.origin_zip_code, "90210");
        assert_eq!(req.destination_country_code, "CA");
        assert_eq!(req.foreign_postal_code.as_deref(), Some("M5V 2T6"));
        assert_eq!(req.weight, 4.5);
        assert_eq!(
            req.mail_class,
            Some(InternationalMailClass::PriorityMailInternational)
        );
    }

    #[test]
    fn international_rate_response_deserialization() {
        let json = r#"{
            "totalBasePrice": 34.50,
            "rates": [
                {
                    "mailClass": "PRIORITY_MAIL_INTERNATIONAL",
                    "price": 34.50,
                    "description": "Priority Mail International 6-10 Days",
                    "sku": "PMI-CA-01"
                }
            ],
            "warnings": []
        }"#;

        let res: InternationalRateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.total_base_price, Some(34.50));
        assert_eq!(res.rates.len(), 1);
        assert_eq!(
            res.rates[0].mail_class.as_deref(),
            Some("PRIORITY_MAIL_INTERNATIONAL")
        );
        assert_eq!(res.rates[0].price, 34.50);
    }
}
