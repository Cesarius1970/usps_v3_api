// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y modelos para la API REST v3 de Direcciones de USPS (`Addresses v3`).
//!
//! Permite estandarización de direcciones postales de EE. UU., búsqueda de códigos postales
//! (ZIP Code Lookup) y resolución de ciudad/estado a partir de un código postal.

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Solicitud de estandarización y validación de dirección postal.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressStandardizationRequest {
    /// Línea principal de dirección física (ej. "475 L'Enfant Plaza SW").
    pub street_address: String,
    /// Línea secundaria (ej. "Apt 2B", "Suite 100").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_address: Option<String>,
    /// Nombre de la ciudad.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// Abreviatura oficial de dos letras del estado o territorio (ej. "DC", "CA", "PR").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Código postal de 5 dígitos.
    #[serde(rename = "ZIPCode", skip_serializing_if = "Option::is_none")]
    pub zip_code: Option<String>,
    /// Código de extensión postal ZIP+4 de 4 dígitos.
    #[serde(rename = "ZIPPlus4", skip_serializing_if = "Option::is_none")]
    pub zip_plus4: Option<String>,
    /// Urbanización (aplica principalmente para áreas residenciales en Puerto Rico).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urbanization: Option<String>,
}

impl AddressStandardizationRequest {
    /// Inicia una solicitud de estandarización con la línea de calle requerida.
    #[must_use]
    pub fn new(street_address: impl Into<String>) -> Self {
        Self {
            street_address: street_address.into(),
            ..Default::default()
        }
    }

    /// Asigna la línea secundaria (apartamento, suite, oficina).
    #[must_use]
    pub fn secondary_address(mut self, secondary: impl Into<String>) -> Self {
        self.secondary_address = Some(secondary.into());
        self
    }

    /// Asigna la ciudad y estado.
    #[must_use]
    pub fn city_state(mut self, city: impl Into<String>, state: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self.state = Some(state.into());
        self
    }

    /// Asigna el código postal de 5 dígitos.
    #[must_use]
    pub fn zip_code(mut self, zip: impl Into<String>) -> Self {
        self.zip_code = Some(zip.into());
        self
    }

    /// Asigna el código postal extendido (ZIP + 4).
    #[must_use]
    pub fn zip_plus4(mut self, plus4: impl Into<String>) -> Self {
        self.zip_plus4 = Some(plus4.into());
        self
    }
}

/// Dirección estandarizada devuelta por el servicio de validación de USPS.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardizedAddress {
    /// Línea principal estandarizada de la calle.
    #[serde(default)]
    pub street_address: Option<String>,
    /// Abreviatura oficial de la calle estandarizada por USPS.
    #[serde(default)]
    pub street_address_abbreviation: Option<String>,
    /// Línea secundaria estandarizada.
    #[serde(default)]
    pub secondary_address: Option<String>,
    /// Nombre de la ciudad estandarizado.
    #[serde(default)]
    pub city: Option<String>,
    /// Abreviatura oficial de la ciudad (si aplica).
    #[serde(default)]
    pub city_abbreviation: Option<String>,
    /// Abreviatura de 2 caracteres del estado.
    #[serde(default)]
    pub state: Option<String>,
    /// Código postal de 5 dígitos.
    #[serde(rename = "ZIPCode", default)]
    pub zip_code: Option<String>,
    /// Extensión de 4 dígitos (ZIP+4).
    #[serde(rename = "ZIPPlus4", default)]
    pub zip_plus4: Option<String>,
    /// Urbanización de Puerto Rico.
    #[serde(default)]
    pub urbanization: Option<String>,
    /// Código de punto de entrega (Delivery Point Code).
    #[serde(default)]
    pub delivery_point: Option<String>,
    /// Código de ruta del cartero (Carrier Route).
    #[serde(default)]
    pub carrier_route: Option<String>,
    /// Confirmación de validación de punto de entrega (DPV Confirmation: "Y", "N", "D", "S").
    #[serde(rename = "DPVConfirmation", default)]
    pub dpv_confirmation: Option<String>,
    /// Indicador de entrega comercial o buzón privado (CMRA).
    #[serde(rename = "DPVCMRA", default)]
    pub dpv_cmra: Option<String>,
    /// Indicador si la dirección corresponde a una empresa/comercio.
    #[serde(default)]
    pub business: Option<String>,
    /// Indicador si el buzón o dirección se encuentra vacante.
    #[serde(default)]
    pub vacant: Option<String>,
}

/// Respuesta devuelta por el endpoint `/addresses/v3/address`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AddressResponse {
    /// Objeto con la dirección postal estandarizada.
    #[serde(default)]
    pub address: Option<StandardizedAddress>,
    /// Advertencias o notas informativas generadas durante la estandarización.
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Mensaje de corrección o ajuste aplicado por la base de datos de USPS.
    #[serde(default)]
    pub correction: Option<String>,
}

/// Solicitud de búsqueda de código postal (ZIP Code Lookup).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZipCodeLookupRequest {
    /// Línea de calle.
    pub street_address: String,
    /// Línea secundaria (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_address: Option<String>,
    /// Ciudad.
    pub city: String,
    /// Estado (código de 2 letras).
    pub state: String,
    /// Urbanización (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urbanization: Option<String>,
}

impl ZipCodeLookupRequest {
    /// Construye una nueva solicitud de búsqueda de código postal.
    #[must_use]
    pub fn new(
        street_address: impl Into<String>,
        city: impl Into<String>,
        state: impl Into<String>,
    ) -> Self {
        Self {
            street_address: street_address.into(),
            secondary_address: None,
            city: city.into(),
            state: state.into(),
            urbanization: None,
        }
    }
}

/// Respuesta de búsqueda de ciudad y estado a partir de un código postal (`/addresses/v3/city-state`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CityStateResponse {
    /// Código postal consultado.
    #[serde(rename = "ZIPCode")]
    pub zip_code: String,
    /// Ciudad principal asociada al código postal.
    pub city: String,
    /// Abreviatura del estado (2 letras).
    pub state: String,
}

/// Servicio de la API v3 de Direcciones de USPS.
#[derive(Debug, Clone)]
pub struct AddressesService {
    client: UspsClient,
}

impl AddressesService {
    /// Crea un nuevo servicio vinculado al cliente central [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Estandariza y valida una dirección postal en los Estados Unidos.
    ///
    /// Realiza una solicitud `GET /addresses/v3/address`.
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si la dirección está vacía, o errores de red/API.
    #[instrument(skip(self), name = "standardize_address")]
    pub async fn standardize(
        &self,
        req: &AddressStandardizationRequest,
    ) -> Result<AddressResponse> {
        if req.street_address.trim().is_empty() {
            return Err(UspsError::InvalidInput(
                "La dirección principal 'street_address' es obligatoria".to_string(),
            ));
        }

        let endpoint = "/addresses/v3/address";
        self.client.get_with_query(endpoint, req).await
    }

    /// Consulta el código postal correspondiente a una dirección de calle, ciudad y estado.
    ///
    /// Realiza una llamada `GET /addresses/v3/zipcode`.
    #[instrument(skip(self), name = "lookup_zip_code")]
    pub async fn lookup_zip_code(&self, req: &ZipCodeLookupRequest) -> Result<AddressResponse> {
        if req.street_address.trim().is_empty() {
            return Err(UspsError::InvalidInput(
                "La dirección principal 'street_address' es requerida".to_string(),
            ));
        }
        if req.city.trim().is_empty() || req.state.trim().is_empty() {
            return Err(UspsError::InvalidInput(
                "La ciudad y el estado son requeridos para consultar el código postal".to_string(),
            ));
        }

        let endpoint = "/addresses/v3/zipcode";
        self.client.get_with_query(endpoint, req).await
    }

    /// Obtiene la ciudad y estado correspondientes a un código postal de 5 dígitos.
    ///
    /// Realiza una llamada `GET /addresses/v3/city-state?ZIPCode={zip_code}`.
    #[instrument(skip(self), name = "lookup_city_state")]
    pub async fn lookup_city_state(&self, zip_code: &str) -> Result<CityStateResponse> {
        let trimmed = zip_code.trim();
        if trimmed.len() != 5 || !trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Err(UspsError::InvalidInput(
                "El código postal debe contener exactamente 5 dígitos numéricos".to_string(),
            ));
        }

        let endpoint = "/addresses/v3/city-state";
        let query = [("ZIPCode", trimmed)];
        self.client.get_with_query(endpoint, &query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_standardization_request_builder() {
        let req = AddressStandardizationRequest::new("475 L'Enfant Plaza SW")
            .secondary_address("Room 100")
            .city_state("Washington", "DC")
            .zip_code("20260");

        assert_eq!(req.street_address, "475 L'Enfant Plaza SW");
        assert_eq!(req.secondary_address.as_deref(), Some("Room 100"));
        assert_eq!(req.city.as_deref(), Some("Washington"));
        assert_eq!(req.state.as_deref(), Some("DC"));
        assert_eq!(req.zip_code.as_deref(), Some("20260"));
    }

    #[test]
    fn address_response_deserialization() {
        let json = r#"{
            "address": {
                "streetAddress": "475 LENFANT PLZ SW",
                "city": "WASHINGTON",
                "state": "DC",
                "ZIPCode": "20260",
                "ZIPPlus4": "0004",
                "DPVConfirmation": "Y"
            },
            "warnings": []
        }"#;

        let res: AddressResponse = serde_json::from_str(json).unwrap();
        let addr = res.address.unwrap();
        assert_eq!(addr.street_address.as_deref(), Some("475 LENFANT PLZ SW"));
        assert_eq!(addr.city.as_deref(), Some("WASHINGTON"));
        assert_eq!(addr.state.as_deref(), Some("DC"));
        assert_eq!(addr.zip_code.as_deref(), Some("20260"));
        assert_eq!(addr.zip_plus4.as_deref(), Some("0004"));
        assert_eq!(addr.dpv_confirmation.as_deref(), Some("Y"));
    }

    #[test]
    fn city_state_response_deserialization() {
        let json = r#"{
            "ZIPCode": "90210",
            "city": "BEVERLY HILLS",
            "state": "CA"
        }"#;

        let res: CityStateResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.zip_code, "90210");
        assert_eq!(res.city, "BEVERLY HILLS");
        assert_eq!(res.state, "CA");
    }
}
