// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y servicio para la API REST v3 de Precios y Tarifas de USPS (`Prices v3`).
//!
//! Permite calcular cotizaciones y tarifas de franqueo para envíos nacionales,
//! internacionales y servicios complementarios de USPS.

pub mod types;

pub use types::*;

use tracing::instrument;

use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Servicio de cotización de tarifas y cálculo de precios postales de USPS v3.
#[derive(Debug, Clone)]
pub struct PricesService {
    client: UspsClient,
}

impl PricesService {
    /// Crea una nueva instancia del servicio de tarifas asociado a [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Calcula las tarifas postales nacionales (`POST /prices/v3/base-rates/search`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si los códigos postales no tienen 5 dígitos o el peso es menor o igual a cero.
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

    /// Calcula las tarifas para servicios adicionales o especiales (`POST /prices/v3/extra-services`).
    ///
    /// Permite cotizar opciones complementarias como firma de entrega, seguro postal, acuse de recibo o entrega restringida.
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si el precio base o el peso son menores a cero, o si el valor declarado es negativo.
    #[instrument(skip(self), name = "calculate_extra_services")]
    pub async fn calculate_extra_services(
        &self,
        req: &ExtraServicesRateRequest,
    ) -> Result<ExtraServicesRateResponse> {
        if req.price < 0.0 {
            return Err(UspsError::InvalidInput(
                "El precio de franqueo base no puede ser negativo".to_string(),
            ));
        }

        if req.weight <= 0.0 {
            return Err(UspsError::InvalidInput(
                "El peso del paquete debe ser estrictamente mayor a 0 libras".to_string(),
            ));
        }

        if let Some(val) = req.declared_value {
            if val < 0.0 {
                return Err(UspsError::InvalidInput(
                    "El valor declarado no puede ser negativo".to_string(),
                ));
            }
        }

        let endpoint = "/prices/v3/extra-services";
        self.client.post_json(endpoint, req).await
    }
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

    #[test]
    fn extra_services_rate_request_builder() {
        let req = ExtraServicesRateRequest::new(MailClass::PriorityMail, 10.50, 2.0)
            .declared_value(250.0)
            .add_extra_service(ExtraServiceType::Insurance)
            .add_extra_service(ExtraServiceType::SignatureConfirmation);

        assert_eq!(req.mail_class, MailClass::PriorityMail);
        assert_eq!(req.price, 10.50);
        assert_eq!(req.weight, 2.0);
        assert_eq!(req.declared_value, Some(250.0));
        assert_eq!(req.extra_services.len(), 2);
        assert_eq!(req.extra_services[0], ExtraServiceType::Insurance);
        assert_eq!(
            req.extra_services[1],
            ExtraServiceType::SignatureConfirmation
        );
        assert_eq!(ExtraServiceType::Insurance.to_string(), "INSURANCE");
        assert_eq!(
            ExtraServiceType::CertifiedMail.to_string(),
            "CERTIFIED_MAIL"
        );
    }

    #[test]
    fn extra_services_rate_response_deserialization() {
        let json = r#"{
            "extraServices": [
                {
                    "name": "Insurance",
                    "serviceId": "100",
                    "price": 4.20,
                    "description": "Coverage up to $300.00"
                },
                {
                    "name": "Signature Confirmation",
                    "serviceId": "108",
                    "price": 3.65,
                    "description": "Recipient signature required"
                }
            ],
            "warnings": []
        }"#;

        let res: ExtraServicesRateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.extra_services.len(), 2);
        assert_eq!(res.extra_services[0].name, "Insurance");
        assert_eq!(res.extra_services[0].price, 4.20);
        assert_eq!(res.extra_services[0].service_id.as_deref(), Some("100"));
        assert_eq!(res.extra_services[1].name, "Signature Confirmation");
        assert_eq!(res.extra_services[1].price, 3.65);
    }
}
