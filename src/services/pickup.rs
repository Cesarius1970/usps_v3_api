// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y modelos para la API REST v3 de Recolección de Paquetes (`Package Pickup v3`).
//!
//! Permite programar recolecciones de paquetes en origen por parte del cartero (Carrier Pickup),
//! verificar la disponibilidad del servicio por código postal y cancelar solicitudes previamente programadas.

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Ubicación donde el remitente dejará los paquetes para el cartero.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PackageLocation {
    /// En la puerta principal o porche delantero.
    #[default]
    FrontDoor,
    /// En la puerta trasera.
    BackDoor,
    /// En la puerta lateral.
    SideDoor,
    /// En el buzón o casilla de correspondencia.
    InMailbox,
    /// En la recepción u oficina principal.
    OfficeReception,
    /// En otra ubicación especificada en notas especiales.
    Other,
}

/// Conteo discriminado de paquetes a recolectar según su clase postal.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickupPackageCount {
    /// Cantidad de paquetes Priority Mail Express.
    #[serde(default)]
    pub priority_mail_express: u32,
    /// Cantidad de paquetes Priority Mail.
    #[serde(default)]
    pub priority_mail: u32,
    /// Cantidad de paquetes USPS Ground Advantage.
    #[serde(default)]
    pub ground_advantage: u32,
    /// Cantidad de paquetes internacionales (ej. Priority Mail International).
    #[serde(default)]
    pub international: u32,
    /// Cantidad de otros paquetes o devoluciones postales.
    #[serde(default)]
    pub other: u32,
}

impl PickupPackageCount {
    /// Retorna la cantidad total de paquetes a recolectar.
    #[must_use]
    pub fn total(&self) -> u32 {
        self.priority_mail_express
            + self.priority_mail
            + self.ground_advantage
            + self.international
            + self.other
    }
}

/// Datos de contacto y dirección postal para la recolección de paquetes.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PickupContactAddress {
    /// Nombre de la persona de contacto.
    pub first_name: String,
    /// Apellido de la persona de contacto.
    pub last_name: String,
    /// Empresa o razón social (opcional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub firm_name: Option<String>,
    /// Línea principal de dirección.
    pub street_address: String,
    /// Línea secundaria (apto, suite, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secondary_address: Option<String>,
    /// Ciudad.
    pub city: String,
    /// Estado (2 caracteres).
    pub state: String,
    /// Código postal (5 dígitos).
    #[serde(rename = "ZIPCode")]
    pub zip_code: String,
    /// Teléfono de contacto.
    pub phone: String,
    /// Correo electrónico.
    pub email: String,
}

impl PickupContactAddress {
    /// Crea un nuevo registro de contacto y dirección física para la recolección.
    #[must_use]
    pub fn new(
        first_name: impl Into<String>,
        last_name: impl Into<String>,
        street_address: impl Into<String>,
        city: impl Into<String>,
        state: impl Into<String>,
        zip_code: impl Into<String>,
    ) -> Self {
        Self {
            first_name: first_name.into(),
            last_name: last_name.into(),
            firm_name: None,
            street_address: street_address.into(),
            secondary_address: None,
            city: city.into(),
            state: state.into(),
            zip_code: zip_code.into(),
            phone: String::new(),
            email: String::new(),
        }
    }

    /// Asigna el teléfono y correo electrónico de notificación.
    #[must_use]
    pub fn contact_info(mut self, phone: impl Into<String>, email: impl Into<String>) -> Self {
        self.phone = phone.into();
        self.email = email.into();
        self
    }

    /// Asigna la razón social o empresa.
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

/// Solicitud de programación de recolección de paquetes (`POST /pickup/v3/carrier-pickup`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulePickupRequest {
    /// Contacto y dirección donde se realizará la recolección.
    #[serde(flatten)]
    pub contact: PickupContactAddress,
    /// Fecha proyectada para la recolección en formato YYYY-MM-DD.
    pub pickup_date: String,
    /// Ubicación física designada donde se encontrarán los paquetes.
    pub package_location: PackageLocation,
    /// Instrucciones o notas especiales para el cartero.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special_instructions: Option<String>,
    /// Desglose y conteo de paquetes a recolectar.
    pub package_count: PickupPackageCount,
}

impl SchedulePickupRequest {
    /// Inicia una nueva solicitud con el contacto, dirección y fecha deseada.
    #[must_use]
    pub fn new(contact: PickupContactAddress, pickup_date: impl Into<String>) -> Self {
        Self {
            contact,
            pickup_date: pickup_date.into(),
            package_location: PackageLocation::FrontDoor,
            special_instructions: None,
            package_count: PickupPackageCount::default(),
        }
    }

    /// Asigna la ubicación donde el cartero encontrará los paquetes.
    #[must_use]
    pub fn location(mut self, location: PackageLocation) -> Self {
        self.package_location = location;
        self
    }

    /// Añade instrucciones especiales para el cartero.
    #[must_use]
    pub fn special_instructions(mut self, notes: impl Into<String>) -> Self {
        self.special_instructions = Some(notes.into());
        self
    }

    /// Asigna el conteo de paquetes a retirar.
    #[must_use]
    pub fn packages(mut self, count: PickupPackageCount) -> Self {
        self.package_count = count;
        self
    }
}

/// Respuesta tras programar una recolección de paquetes con USPS.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulePickupResponse {
    /// Número oficial de confirmación de la recolección generado por USPS.
    pub confirmation_number: String,
    /// Fecha confirmada para la visita del cartero.
    pub pickup_date: String,
    /// Estado de la programación (ej. "Scheduled", "Success").
    pub status: String,
    /// Conteo total de paquetes registrados para retiro.
    #[serde(default)]
    pub total_packages: Option<u32>,
    /// Mensaje o instrucciones adicionales de confirmación.
    #[serde(default)]
    pub message: Option<String>,
}

/// Respuesta de consulta de disponibilidad de recolección en un código postal.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PickupAvailabilityResponse {
    /// Indica si el servicio de recolección de cartero está disponible en el área consultada.
    pub available: bool,
    /// Código postal consultado.
    #[serde(rename = "ZIPCode")]
    pub zip_code: String,
    /// Fechas hábiles o franja horaria tentativa si está disponible.
    #[serde(default)]
    pub message: Option<String>,
}

/// Respuesta de cancelación de una recolección programada.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelPickupResponse {
    /// Estado de la cancelación (ej. "Cancelled").
    pub status: String,
    /// Número de confirmación anulado.
    pub confirmation_number: String,
    /// Mensaje descriptivo de confirmación.
    #[serde(default)]
    pub message: Option<String>,
}

/// Servicio de la API v3 de Recolección de Paquetes (`Package Pickup v3`).
#[derive(Debug, Clone)]
pub struct PickupService {
    client: UspsClient,
}

impl PickupService {
    /// Crea una nueva instancia del servicio de recolección vinculada a [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Consulta si el servicio de recolección por cartero está habilitado en un código postal.
    ///
    /// Realiza una solicitud `GET /pickup/v3/carrier-pickup/availability?ZIPCode={zip_code}`.
    #[instrument(skip(self), name = "check_pickup_availability")]
    pub async fn check_availability(&self, zip_code: &str) -> Result<PickupAvailabilityResponse> {
        let trimmed = zip_code.trim();
        if trimmed.len() != 5 || !trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Err(UspsError::InvalidInput(
                "El código postal debe contener exactamente 5 dígitos".to_string(),
            ));
        }

        let endpoint = "/pickup/v3/carrier-pickup/availability";
        let query = [("ZIPCode", trimmed)];
        self.client.get_with_query(endpoint, &query).await
    }

    /// Programa una nueva recolección de paquetes (`POST /pickup/v3/carrier-pickup`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si faltan campos obligatorios o si no se especificó al menos 1 paquete,
    /// o fallas de red/API si la solicitud es rechazada por USPS.
    #[instrument(skip(self), name = "schedule_pickup")]
    pub async fn schedule(&self, req: &SchedulePickupRequest) -> Result<SchedulePickupResponse> {
        if req.contact.first_name.trim().is_empty() || req.contact.last_name.trim().is_empty() {
            return Err(UspsError::InvalidInput(
                "El nombre y apellido del contacto son obligatorios".to_string(),
            ));
        }

        if req.contact.street_address.trim().is_empty()
            || req.contact.city.trim().is_empty()
            || req.contact.state.trim().is_empty()
        {
            return Err(UspsError::InvalidInput(
                "La dirección física de recolección está incompleta".to_string(),
            ));
        }

        if req.package_count.total() == 0 {
            return Err(UspsError::InvalidInput(
                "Debe indicarse al menos 1 paquete a recolectar".to_string(),
            ));
        }

        let endpoint = "/pickup/v3/carrier-pickup";
        self.client.post_json(endpoint, req).await
    }

    /// Cancela una recolección programada previamente (`DELETE /pickup/v3/carrier-pickup/{confirmationNumber}`).
    #[instrument(skip(self), name = "cancel_pickup")]
    pub async fn cancel(&self, confirmation_number: &str) -> Result<CancelPickupResponse> {
        let clean_num = confirmation_number.trim();
        if clean_num.is_empty() {
            return Err(UspsError::InvalidInput(
                "El número de confirmación no puede estar vacío".to_string(),
            ));
        }

        let endpoint = format!("/pickup/v3/carrier-pickup/{clean_num}");
        let body = self.client.delete(&endpoint).await?;

        if body.trim().is_empty() {
            Ok(CancelPickupResponse {
                status: "CANCELLED".to_string(),
                confirmation_number: clean_num.to_string(),
                message: Some("Recolección cancelada satisfactoriamente".to_string()),
            })
        } else {
            serde_json::from_str::<CancelPickupResponse>(&body).map_err(UspsError::Serialization)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedule_pickup_request_builder() {
        let count = PickupPackageCount {
            priority_mail: 2,
            ground_advantage: 3,
            ..Default::default()
        };

        let contact =
            PickupContactAddress::new("John", "Doe", "123 Maple St", "Chicago", "IL", "60601")
                .contact_info("555-123-4567", "john@example.com");

        let req = SchedulePickupRequest::new(contact, "2026-09-20")
            .location(PackageLocation::FrontDoor)
            .special_instructions("Beware of dog")
            .packages(count);

        assert_eq!(req.contact.first_name, "John");
        assert_eq!(req.package_count.total(), 5);
        assert_eq!(req.package_location, PackageLocation::FrontDoor);
        assert_eq!(req.special_instructions.as_deref(), Some("Beware of dog"));
    }

    #[test]
    fn schedule_pickup_response_deserialization() {
        let json = r#"{
            "confirmationNumber": "WEC123456789",
            "pickupDate": "2026-09-20",
            "status": "Scheduled",
            "totalPackages": 5,
            "message": "Pickup scheduled successfully"
        }"#;

        let res: SchedulePickupResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.confirmation_number, "WEC123456789");
        assert_eq!(res.pickup_date, "2026-09-20");
        assert_eq!(res.status, "Scheduled");
        assert_eq!(res.total_packages, Some(5));
    }
}
