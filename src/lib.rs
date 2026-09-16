// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # usps_v3_api
//!
//! SDK en Rust idiomático, fuertemente tipado, asíncrono y listo para producción
//! para interactuar con el ecosistema de APIs REST v3 del Servicio Postal de los
//! Estados Unidos (USPS - United States Postal Service).
//!
//! ## Características Principales
//!
//! - **Arquitectura Asíncrona:** Construido sobre [`tokio`] y [`reqwest`], diseñado para alta concurrencia.
//! - **Gestión Inteligente de Autenticación OAuth 2.0:** Manejo automático del ciclo de vida del token
//!   Bearer (`POST /oauth2/v3/token`), con almacenamiento seguro en memoria mediante [`tokio::sync::RwLock`],
//!   renovación transparente con margen de expiración y patrón double-checked lock.
//! - **Seguridad en Producción:** Protección de credenciales confidenciales (`client_secret`) contra fugas
//!   accidentales en logs de depuración (`[REDACTED]`).
//! - **Servicio de Direcciones (`Addresses v3`):** Estandarización de direcciones, validación de entrega DPV,
//!   búsqueda de códigos postales (ZIP Lookup) y resolución de ciudad/estado.
//! - **Servicio de Seguimiento (`Tracking v3`):** Consulta de paquetes en tránsito, detalle de eventos históricos,
//!   estados y fechas estimadas de entrega.
//! - **Servicio de Precios y Tarifas (`Prices v3`):** Cotización de tarifas postales nacionales (`POST /prices/v3/base-rates/search`)
//!   por peso, dimensiones y clases postales (`Priority Mail`, `USPS Ground Advantage`, etc.).
//! - **Servicio de Etiquetas (`Labels v3`):** Emisión y cancelación de etiquetas postales con código de barras en
//!   formatos PDF, PNG o Base64 (`POST /labels/v3/label`, `DELETE /labels/v3/label/{id}`).
//! - **Servicio de Recolección (`Pickup v3`):** Verificación de disponibilidad, programación y cancelación de recolección
//!   de paquetes por cartero a domicilio (`Carrier Pickup`).
//! - **Servicio de Ubicaciones (`Locations v3`):** Búsqueda de oficinas postales, buzones de depósito y quioscos
//!   por código postal o geocordenadas, consulta de horarios semanales y servicios (pasaportes, casilleros).
//! - **Manejo Exhaustivo de Errores:** Jerarquía fuertemente tipada con [`UspsError`] y deserialización
//!   de errores estructurados devueltos por la pasarela de USPS.
//!
//! ## Ejemplo de Uso Rápido
//!
//! ```no_run
//! use usps_v3_api::{
//!     UspsClient, UspsEnvironment,
//!     AddressStandardizationRequest, TrackingExpand,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Inicializar el cliente SDK
//!     let client = UspsClient::builder()
//!         .credentials("TU_CLIENT_ID", "TU_CLIENT_SECRET")
//!         .environment(UspsEnvironment::Sandbox)
//!         .build()?;
//!
//!     // 2. Estandarizar una dirección postal
//!     let address_req = AddressStandardizationRequest::new("475 L'Enfant Plaza SW")
//!         .city_state("Washington", "DC")
//!         .zip_code("20260");
//!
//!     let address_info = client.addresses().standardize(&address_req).await?;
//!     println!("Dirección estandarizada: {:?}", address_info.address);
//!
//!     // 3. Rastrear un paquete
//!     let tracking = client.tracking().track("9400100000000000000000").await?;
//!     println!("Estado del paquete: {:?}", tracking.status);
//!
//!     Ok(())
//! }
//! ```

pub mod addresses;
pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod labels;
pub mod locations;
pub mod pickup;
pub mod prices;
pub mod tracking;

// Re-exportaciones públicas principales
pub use addresses::{
    AddressResponse, AddressStandardizationRequest, AddressesService, CityStateResponse,
    StandardizedAddress, ZipCodeLookupRequest,
};
pub use auth::{OAuthTokenResponse, TokenManager};
pub use client::{UspsClient, UspsClientBuilder};
pub use config::{USPS_CAT_BASE_URL, USPS_PROD_BASE_URL, UspsConfig, UspsEnvironment};
pub use error::{ApiErrorDetail, Result, UspsApiErrorResponse, UspsError};
pub use labels::{
    CancelLabelResponse, CreateLabelRequest, CreateLabelResponse, ImageInfo, LabelImageType,
    LabelPartyAddress, LabelSize, LabelsService, PackageDescription,
};
pub use locations::{
    DailyHours, LocationFacility, LocationSearchRequest, LocationSearchResponse,
    LocationServiceType, LocationsService,
};
pub use pickup::{
    CancelPickupResponse, PackageLocation, PickupAvailabilityResponse, PickupContactAddress,
    PickupPackageCount, PickupService, SchedulePickupRequest, SchedulePickupResponse,
};
pub use prices::{
    DomesticRateRequest, DomesticRateResponse, InternationalMailClass, InternationalRateRequest,
    InternationalRateResponse, MailClass, PricesService, ProcessingCategory, RateItem,
};
pub use tracking::{TrackingEvent, TrackingExpand, TrackingResponse, TrackingService};
