// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y modelos para la API REST v3 de Ubicaciones e Instalaciones Postales (`Locations v3`).
//!
//! Permite buscar oficinas de correos de USPS, buzones públicos de recolección (Collection Boxes),
//! centros de procesamiento, quioscos de autoservicio y consultar horarios de atención y servicios disponibles.

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Servicios específicos ofrecidos en una oficina o instalación de USPS.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocationServiceType {
    /// Trámite de pasaportes estadounidenses (fotos y citas de solicitud).
    PassportAppointments,
    /// Casilleros y apartados postales (PO Boxes).
    PoBoxes,
    /// Ventanilla de atención comercial y venta de franqueo al público.
    RetailServices,
    /// Buzón público exterior de depósito de cartas y paquetes.
    CollectionBox,
    /// Quiosco de autoservicio automatizado (APC - Automated Postal Center).
    SelfServiceKiosks,
    /// Aceptación de envíos masivos o comerciales (Bulk Mail).
    BulkMailAcceptance,
    /// Servicios de empaque y venta de suministros postales.
    GreetingCardsSupplies,
    /// Venta de estampillas filatélicas (Duck Stamps).
    DuckStamps,
}

/// Horario de atención para un día específico de la semana.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyHours {
    /// Día de la semana (ej. "Monday", "Saturday").
    pub day_of_the_week: String,
    /// Hora de apertura (ej. "09:00").
    #[serde(default)]
    pub open_time: Option<String>,
    /// Hora de cierre (ej. "17:00").
    #[serde(default)]
    pub close_time: Option<String>,
    /// Indica si la oficina permanece cerrada durante este día.
    #[serde(default)]
    pub is_closed: bool,
}

/// Información detallada de una instalación física u oficina postal de USPS.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationFacility {
    /// Identificador único de la instalación en la base de datos de USPS.
    pub location_id: String,
    /// Nombre oficial de la oficina o instalación (ej. "WASHINGTON MAIN POST OFFICE").
    pub name: String,
    /// Tipo de instalación (ej. "POST_OFFICE", "COLLECTION_BOX", "CONTRACT_POSTAL_UNIT").
    #[serde(default)]
    pub location_type: Option<String>,
    /// Dirección física de la instalación.
    pub street_address: String,
    /// Ciudad.
    pub city: String,
    /// Estado o territorio (2 caracteres).
    pub state: String,
    /// Código postal (5 dígitos).
    #[serde(rename = "ZIPCode")]
    pub zip_code: String,
    /// Extensión de 4 dígitos (ZIP+4) si está disponible.
    #[serde(rename = "ZIPPlus4", default)]
    pub zip_plus4: Option<String>,
    /// Teléfono de contacto directo de la oficina.
    #[serde(default)]
    pub phone: Option<String>,
    /// Latitud geográfica en grados decimales.
    #[serde(default)]
    pub latitude: Option<f64>,
    /// Longitud geográfica en grados decimales.
    #[serde(default)]
    pub longitude: Option<f64>,
    /// Distancia en millas calculada desde el punto de origen de la búsqueda.
    #[serde(default)]
    pub distance: Option<f64>,
    /// Lista de horarios semanales de atención al público.
    #[serde(default)]
    pub operating_hours: Vec<DailyHours>,
    /// Catálogo de servicios disponibles en esta sede.
    #[serde(default)]
    pub services: Vec<String>,
}

/// Parámetros de búsqueda de instalaciones postales (`GET /locations/v3/location`).
#[derive(Debug, Clone, PartialEq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationSearchRequest {
    /// Código postal de 5 dígitos como centro de búsqueda.
    #[serde(rename = "ZIPCode", skip_serializing_if = "Option::is_none")]
    pub zip_code: Option<String>,
    /// Latitud geográfica de búsqueda (opcional si se usa código postal).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// Longitud geográfica de búsqueda (opcional si se usa código postal).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    /// Radio máximo de búsqueda en millas (por defecto 10 millas, máximo 100).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<u32>,
    /// Cantidad máxima de resultados a retornar.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_results: Option<u32>,
    /// Filtro de servicio específico requerido en la oficina (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_type: Option<LocationServiceType>,
}

impl LocationSearchRequest {
    /// Inicia una búsqueda geográfica basada en un código postal de 5 dígitos.
    #[must_use]
    pub fn from_zip_code(zip_code: impl Into<String>) -> Self {
        Self {
            zip_code: Some(zip_code.into()),
            radius: Some(10),
            max_results: Some(20),
            ..Default::default()
        }
    }

    /// Inicia una búsqueda basada en coordenadas geográficas (latitud y longitud).
    #[must_use]
    pub fn from_coordinates(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude: Some(latitude),
            longitude: Some(longitude),
            radius: Some(10),
            max_results: Some(20),
            ..Default::default()
        }
    }

    /// Define el radio de búsqueda en millas.
    #[must_use]
    pub fn radius(mut self, miles: u32) -> Self {
        self.radius = Some(miles);
        self
    }

    /// Define el límite máximo de resultados.
    #[must_use]
    pub fn max_results(mut self, limit: u32) -> Self {
        self.max_results = Some(limit);
        self
    }

    /// Filtra instalaciones que ofrezcan un servicio determinado.
    #[must_use]
    pub fn with_service(mut self, service: LocationServiceType) -> Self {
        self.service_type = Some(service);
        self
    }
}

/// Respuesta devuelta por el servicio de consulta de ubicaciones de USPS.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationSearchResponse {
    /// Lista de instalaciones y oficinas postales que coinciden con los criterios.
    #[serde(default)]
    pub locations: Vec<LocationFacility>,
    /// Conteo total de resultados disponibles.
    #[serde(default)]
    pub total_locations: Option<usize>,
}

/// Servicio de la API v3 de Ubicaciones e Instalaciones Postales de USPS.
#[derive(Debug, Clone)]
pub struct LocationsService {
    client: UspsClient,
}

impl LocationsService {
    /// Crea un nuevo servicio vinculado al cliente central [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Busca instalaciones postales según código postal o coordenadas geográficas.
    ///
    /// Realiza una solicitud `GET /locations/v3/location`.
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si no se especifica ni código postal ni coordenadas válidas,
    /// o errores de red/API si la consulta remota falla.
    #[instrument(skip(self), name = "search_locations")]
    pub async fn search(&self, req: &LocationSearchRequest) -> Result<LocationSearchResponse> {
        if req.zip_code.is_none() && (req.latitude.is_none() || req.longitude.is_none()) {
            return Err(UspsError::InvalidInput(
                "Debe proporcionarse un código postal o coordenadas (latitud/longitud) para la búsqueda".to_string(),
            ));
        }

        if let Some(ref zip) = req.zip_code {
            let trimmed = zip.trim();
            if trimmed.len() != 5 || !trimmed.chars().all(|c| c.is_ascii_digit()) {
                return Err(UspsError::InvalidInput(
                    "El código postal debe contener exactamente 5 dígitos numéricos".to_string(),
                ));
            }
        }

        let endpoint = "/locations/v3/location";
        self.client.get_with_query(endpoint, req).await
    }

    /// Consulta los detalles completos y horarios de una instalación específica por su identificador.
    ///
    /// Realiza una solicitud `GET /locations/v3/location/{locationId}`.
    #[instrument(skip(self), name = "get_location_details")]
    pub async fn get_details(&self, location_id: &str) -> Result<LocationFacility> {
        let clean_id = location_id.trim();
        if clean_id.is_empty() {
            return Err(UspsError::InvalidInput(
                "El identificador location_id no puede estar vacío".to_string(),
            ));
        }

        let endpoint = format!("/locations/v3/location/{clean_id}");
        let empty_query: [(&str, &str); 0] = [];
        self.client.get_with_query(&endpoint, &empty_query).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn location_search_request_from_zip() {
        let req = LocationSearchRequest::from_zip_code("20260")
            .radius(15)
            .max_results(5)
            .with_service(LocationServiceType::PassportAppointments);

        assert_eq!(req.zip_code.as_deref(), Some("20260"));
        assert_eq!(req.radius, Some(15));
        assert_eq!(req.max_results, Some(5));
        assert_eq!(
            req.service_type,
            Some(LocationServiceType::PassportAppointments)
        );
    }

    #[test]
    fn location_search_response_deserialization() {
        let json = r#"{
            "locations": [
                {
                    "locationId": "1380568",
                    "name": "CURLEEN POST OFFICE",
                    "locationType": "POST_OFFICE",
                    "streetAddress": "475 LENFANT PLZ SW",
                    "city": "WASHINGTON",
                    "state": "DC",
                    "ZIPCode": "20260",
                    "phone": "800-275-8777",
                    "distance": 0.3,
                    "operatingHours": [
                        {
                            "dayOfTheWeek": "Monday",
                            "openTime": "08:30",
                            "closeTime": "17:00",
                            "isClosed": false
                        }
                    ],
                    "services": [
                        "PO_BOXES",
                        "PASSPORT_APPOINTMENTS"
                    ]
                }
            ],
            "totalLocations": 1
        }"#;

        let res: LocationSearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.locations.len(), 1);
        let loc = &res.locations[0];
        assert_eq!(loc.location_id, "1380568");
        assert_eq!(loc.name, "CURLEEN POST OFFICE");
        assert_eq!(loc.city, "WASHINGTON");
        assert_eq!(loc.distance, Some(0.3));
        assert_eq!(loc.operating_hours.len(), 1);
        assert_eq!(loc.operating_hours[0].day_of_the_week, "Monday");
        assert_eq!(loc.services.len(), 2);
    }
}
