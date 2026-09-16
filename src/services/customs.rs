// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Modelos y validaciones para Declaraciones de Aduana y Datos Electrónicos Internacionales (`Customs v3`).
//!
//! Permite estructurar declaraciones aduaneras para envíos internacionales de USPS (Formularios CN22 y CP72,
//! PS Form 2976 / 2976-A), detallando artículos, códigos arancelarios HTS, país de origen y exenciones AES/ITN.

use serde::{Deserialize, Serialize};

/// Naturaleza o tipo de contenido aduanero del envío internacional.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomsContentType {
    /// Bienes comerciales o ventas minoristas (Merchandise).
    Merchandise,
    /// Regalos de uso personal o donaciones privadas (Gift).
    Gift,
    /// Documentos comerciales o correspondencia oficial sin valor comercial (Documents).
    Documents,
    /// Muestras comerciales sin valor comercial para evaluación (Sample).
    Sample,
    /// Mercancía devuelta al remitente original (Returned Goods).
    ReturnedGoods,
    /// Donaciones o suministros humanitarios (Humanitarian Donation).
    HumanitarianDonation,
    /// Otro tipo de contenido específico.
    Other,
}

impl std::fmt::Display for CustomsContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Merchandise => write!(f, "MERCHANDISE"),
            Self::Gift => write!(f, "GIFT"),
            Self::Documents => write!(f, "DOCUMENTS"),
            Self::Sample => write!(f, "SAMPLE"),
            Self::ReturnedGoods => write!(f, "RETURNED_GOODS"),
            Self::HumanitarianDonation => write!(f, "HUMANITARIAN_DONATION"),
            Self::Other => write!(f, "OTHER"),
        }
    }
}

/// Instrucción ante imposibilidad de entrega en el país de destino.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NonDeliveryOption {
    /// Devolver el paquete al remitente (puede generar costos de reexpedición).
    Return,
    /// Abandonar o destruir el paquete en aduana de destino sin costo de retorno.
    Abandon,
}

/// Detalle individual de un artículo o producto dentro de la declaración aduanera.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomsItem {
    /// Descripción clara y específica del artículo en inglés (ej. "Cotton T-Shirt").
    pub description: String,
    /// Cantidad de unidades idénticas de este artículo.
    pub quantity: u32,
    /// Valor monetario unitario declarado en USD.
    pub value: f64,
    /// Peso neto unitario o total del artículo en libras.
    pub weight_lbs: f64,
    /// Código arancelario del Sistema Armonizado (HTS - Harmonized Tariff Schedule) de 6 a 10 dígitos.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hs_tariff_number: Option<String>,
    /// Código de 2 caracteres ISO 3166-1 alpha-2 del país de origen de fabricación (ej. "US", "MX").
    pub country_of_origin: String,
}

impl CustomsItem {
    /// Construye una nueva línea de artículo aduanero.
    #[must_use]
    pub fn new(
        description: impl Into<String>,
        quantity: u32,
        value: f64,
        weight_lbs: f64,
        country_of_origin: impl Into<String>,
    ) -> Self {
        Self {
            description: description.into(),
            quantity,
            value,
            weight_lbs,
            hs_tariff_number: None,
            country_of_origin: country_of_origin.into(),
        }
    }

    /// Asigna el código arancelario del Sistema Armonizado (HTS).
    #[must_use]
    pub fn hs_tariff_number(mut self, hs: impl Into<String>) -> Self {
        self.hs_tariff_number = Some(hs.into());
        self
    }
}

/// Declaración aduanera consolidada para envíos postales internacionales de USPS.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomsDeclaration {
    /// Tipo de contenido aduanero.
    pub content_type: CustomsContentType,
    /// Instrucción en caso de imposibilidad de entrega en destino.
    pub non_delivery_option: NonDeliveryOption,
    /// Código de exención AES (ej. "NOEEI 30.37(a)") o número ITN si el valor supera $2,500 USD.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aes_itn: Option<String>,
    /// Identificador fiscal o tributario del destinatario (ej. IVA, RFC, EORI, número IOSS para la UE).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub importer_tax_id: Option<String>,
    /// Lista de artículos individuales declarados en el envío.
    pub items: Vec<CustomsItem>,
}

impl CustomsDeclaration {
    /// Inicia una nueva declaración aduanera requiriendo tipo de contenido y opción de no entrega.
    #[must_use]
    pub fn new(
        content_type: CustomsContentType,
        non_delivery_option: NonDeliveryOption,
        items: Vec<CustomsItem>,
    ) -> Self {
        Self {
            content_type,
            non_delivery_option,
            aes_itn: None,
            importer_tax_id: None,
            items,
        }
    }

    /// Asigna la exención AES o número de seguimiento ITN para control aduanero de exportación.
    #[must_use]
    pub fn aes_itn(mut self, itn: impl Into<String>) -> Self {
        self.aes_itn = Some(itn.into());
        self
    }

    /// Asigna el identificador fiscal o de importación del destinatario (EORI, VAT, IOSS, etc.).
    #[must_use]
    pub fn importer_tax_id(mut self, tax_id: impl Into<String>) -> Self {
        self.importer_tax_id = Some(tax_id.into());
        self
    }

    /// Calcula el valor declarado total en USD sumando cada artículo por su cantidad.
    #[must_use]
    pub fn total_declared_value(&self) -> f64 {
        self.items
            .iter()
            .map(|item| f64::from(item.quantity) * item.value)
            .sum()
    }

    /// Calcula el peso total de los artículos declarados en libras.
    #[must_use]
    pub fn total_weight_lbs(&self) -> f64 {
        self.items.iter().map(|item| item.weight_lbs).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn customs_declaration_builder_and_calculations() {
        let item1 = CustomsItem::new("Men's Leather Shoes", 1, 120.0, 2.5, "US")
            .hs_tariff_number("6403.59");
        let item2 =
            CustomsItem::new("Cotton Socks", 3, 10.0, 0.6, "US").hs_tariff_number("6115.95");

        let decl = CustomsDeclaration::new(
            CustomsContentType::Merchandise,
            NonDeliveryOption::Return,
            vec![item1, item2],
        )
        .aes_itn("NOEEI 30.37(a)")
        .importer_tax_id("EU123456789");

        assert_eq!(decl.content_type, CustomsContentType::Merchandise);
        assert_eq!(decl.non_delivery_option, NonDeliveryOption::Return);
        assert_eq!(decl.aes_itn.as_deref(), Some("NOEEI 30.37(a)"));
        assert_eq!(decl.items.len(), 2);

        // Valor total: (1 * 120) + (3 * 10) = 150.0 USD
        assert_eq!(decl.total_declared_value(), 150.0);
        // Peso total: 2.5 + 0.6 = 3.1 lbs
        assert!((decl.total_weight_lbs() - 3.1).abs() < 1e-6);
    }

    #[test]
    fn customs_content_type_display() {
        assert_eq!(CustomsContentType::Merchandise.to_string(), "MERCHANDISE");
        assert_eq!(CustomsContentType::Gift.to_string(), "GIFT");
        assert_eq!(CustomsContentType::Documents.to_string(), "DOCUMENTS");
        assert_eq!(CustomsContentType::Sample.to_string(), "SAMPLE");
        assert_eq!(
            CustomsContentType::ReturnedGoods.to_string(),
            "RETURNED_GOODS"
        );
        assert_eq!(
            CustomsContentType::HumanitarianDonation.to_string(),
            "HUMANITARIAN_DONATION"
        );
        assert_eq!(CustomsContentType::Other.to_string(), "OTHER");
    }
}
