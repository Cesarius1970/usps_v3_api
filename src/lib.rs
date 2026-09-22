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
//! - **Arquitectura Modular en Capas:** Separación limpia entre infraestructura transversal ([`core`])
//!   y catálogo de servicios de negocio ([`services`]).
//! - **Gestión Inteligente de Autenticación OAuth 2.0:** Manejo automático del ciclo de vida del token
//!   Bearer (`POST /oauth2/v3/token`), con almacenamiento seguro en memoria mediante [`tokio::sync::RwLock`],
//!   renovación transparente con margen de expiración y patrón double-checked lock.
//! - **Seguridad en Producción:** Protección de credenciales confidenciales (`client_secret`) contra fugas
//!   accidentales en logs de depuración (`[REDACTED]`).
//! - **Servicio de Direcciones (`Addresses v3`):** Estandarización de direcciones, validación de entrega DPV,
//!   búsqueda de códigos postales (ZIP Lookup) y resolución de ciudad/estado.
//! - **Servicio de Seguimiento (`Tracking v3`):** Consulta individual y por lotes (hasta 35 paquetes) de envíos en tránsito,
//!   detalle de eventos históricos, estados y fechas estimadas de entrega.
//! - **Servicio de Precios y Tarifas (`Prices v3`):** Cotización de tarifas postales nacionales e internacionales
//!   por peso, dimensiones y clases postales (`Priority Mail`, `USPS Ground Advantage`, `Global Express Guaranteed`, etc.).
//! - **Servicio de Estándares de Entrega (`Service Standards v3`):** Cálculo de compromisos de entrega, fechas estimadas
//!   (Expected Delivery Date - EDD) y días de tránsito entre códigos postales origen y destino.
//! - **Servicio de Etiquetas (`Labels v3`):** Emisión y cancelación de etiquetas postales con código de barras en
//!   formatos PDF, PNG, TIFF, SVG o Base64 (`POST /labels/v3/label`, `DELETE /labels/v3/label/{id}`).
//! - **Servicio de Manifiestos (`Manifests v3`):** Consolidación de envíos masivos en un Formulario SCAN Form
//!   (PS Form 5630) con código de barras maestro (`POST /manifests/v3/manifest`, `GET /manifests/v3/manifest/{id}`).
//! - **Servicio de Recolección (`Pickup v3`):** Verificación de disponibilidad, programación y cancelación de recolección
//!   de paquetes por cartero a domicilio (`Carrier Pickup`).
//! - **Servicio de Ubicaciones (`Locations v3`):** Búsqueda de oficinas postales, buzones de depósito y quioscos
//!   por código postal o geocordenadas, consulta de horarios semanales y servicios (pasaportes, casilleros).
//! - **Servicio de Webhooks (`Subscriptions v3`):** Registro, consulta y cancelación de callbacks HTTP en tiempo real
//!   para eventos de rastreo y entrega (`POST /subscriptions/v3/subscription`, `DELETE /subscriptions/v3/subscription/{id}`).
//! - **Servicio de Pagos EPS (`Payments v3`):** Consulta de saldos y autorizaciones en el Enterprise Payment System de USPS.
//! - **Declaraciones Aduaneras (`Customs v3`):** Modelado de formularios CN22 y CP72, partidas arancelarias HTS y exenciones AES/ITN.
//! - **Resiliencia y Reintentos:** Política de backoff exponencial con jitter ([`RetryPolicy`]) ante respuestas transitorias
//!   HTTP 429 (Rate Limit), 500, 502, 503 y 504.
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

pub mod core;
pub mod services;

// Re-exportaciones públicas de la capa central (core)
pub use core::{
    ApiErrorDetail, OAuthTokenResponse, Result, RetryPolicy, TokenManager, USPS_CAT_BASE_URL,
    USPS_PROD_BASE_URL, UspsApiErrorResponse, UspsClient, UspsClientBuilder, UspsConfig,
    UspsEnvironment, UspsError, UspsErrorCode,
};

// Re-exportaciones públicas del catálogo de servicios
pub use services::{
    AccountBalanceResponse, AddressResponse, AddressStandardizationRequest, AddressesService,
    CancelLabelResponse, CancelPickupResponse, CityStateResponse, CreateLabelRequest,
    CreateLabelResponse, CreateManifestRequest, CreateManifestResponse, CreateSubscriptionRequest,
    CustomsContentType, CustomsDeclaration, CustomsItem, DailyHours, DeleteSubscriptionResponse,
    DomesticRateRequest, DomesticRateResponse, ExtraServiceRateItem, ExtraServiceType,
    ExtraServicesRateRequest, ExtraServicesRateResponse, ImageInfo, InternationalMailClass,
    InternationalRateRequest, InternationalRateResponse, LabelBrokerRequest, LabelBrokerResponse,
    LabelImageType, LabelPartyAddress, LabelSize, LabelsService, LocationFacility,
    LocationSearchRequest, LocationSearchResponse, LocationServiceType, LocationsService,
    MailClass, ManifestsService, NonDeliveryOption, PackageDescription, PackageLocation,
    PaymentAccountType, PaymentAuthorizationRequest, PaymentAuthorizationResponse, PaymentsService,
    PickupAvailabilityResponse, PickupContactAddress, PickupPackageCount, PickupService,
    PricesService, ProcessingCategory, ProofOfDeliveryFormat, ProofOfDeliveryRequest,
    ProofOfDeliveryResponse, RateItem, SchedulePickupRequest, SchedulePickupResponse,
    ServiceStandardEstimate, ServiceStandardRequest, ServiceStandardResponse,
    ServiceStandardsService, StandardizedAddress, SubscriptionEventType, SubscriptionResponse,
    TrackingEvent, TrackingExpand, TrackingResponse, TrackingService, WebhooksService,
    ZipCodeLookupRequest,
};

// Módulos públicos canónicos para acceso granular
pub use core::auth;
pub use core::client;
pub use core::config;
pub use core::error;
pub use core::retry;
pub use services::addresses;
pub use services::customs;
pub use services::labels;
pub use services::locations;
pub use services::manifests;
pub use services::payments;
pub use services::pickup;
pub use services::prices;
pub use services::standards;
pub use services::tracking;
pub use services::webhooks;
