// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Catálogo de servicios de la API REST v3 de USPS.

pub mod addresses;
pub mod customs;
pub mod labels;
pub mod locations;
pub mod manifests;
pub mod payments;
pub mod pickup;
pub mod prices;
pub mod standards;
pub mod tracking;
pub mod webhooks;

pub use addresses::{
    AddressResponse, AddressStandardizationRequest, AddressesService, CityStateResponse,
    StandardizedAddress, ZipCodeLookupRequest,
};
pub use customs::{CustomsContentType, CustomsDeclaration, CustomsItem, NonDeliveryOption};
pub use labels::{
    CancelLabelResponse, CreateLabelRequest, CreateLabelResponse, ImageInfo, LabelBrokerRequest,
    LabelBrokerResponse, LabelImageType, LabelPartyAddress, LabelSize, LabelsService,
    PackageDescription,
};
pub use locations::{
    DailyHours, LocationFacility, LocationSearchRequest, LocationSearchResponse,
    LocationServiceType, LocationsService,
};
pub use manifests::{CreateManifestRequest, CreateManifestResponse, ManifestsService};
pub use payments::{
    AccountBalanceResponse, PaymentAccountType, PaymentAuthorizationRequest,
    PaymentAuthorizationResponse, PaymentsService,
};
pub use pickup::{
    CancelPickupResponse, PackageLocation, PickupAvailabilityResponse, PickupContactAddress,
    PickupPackageCount, PickupService, SchedulePickupRequest, SchedulePickupResponse,
};
pub use prices::{
    DomesticRateRequest, DomesticRateResponse, InternationalMailClass, InternationalRateRequest,
    InternationalRateResponse, MailClass, PricesService, ProcessingCategory, RateItem,
};
pub use standards::{
    ServiceStandardEstimate, ServiceStandardRequest, ServiceStandardResponse,
    ServiceStandardsService,
};
pub use tracking::{TrackingEvent, TrackingExpand, TrackingResponse, TrackingService};
pub use webhooks::{
    CreateSubscriptionRequest, DeleteSubscriptionResponse, SubscriptionEventType,
    SubscriptionResponse, WebhooksService,
};
