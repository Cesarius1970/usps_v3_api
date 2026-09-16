// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente y modelos para la API REST v3 de Pagos y Cuentas EPS de USPS (`Payments v3`).
//!
//! Permite interactuar con el Sistema de Pago Empresarial de USPS (**Enterprise Payment System - EPS**),
//! verificar saldos disponibles en cuentas prepagas o permisos postales (Permit Imprint),
//! y validar autorizaciones de pago para franqueo postal masivo.

use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::core::client::UspsClient;
use crate::core::error::{Result, UspsError};

/// Tipo de cuenta o instrumento de pago registrado en USPS EPS.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentAccountType {
    /// Cuenta de pago empresarial estándar (Enterprise Payment System).
    Eps,
    /// Cuenta vinculada a un permiso de impresión postal (Permit Imprint).
    PermitImprint,
    /// Cuenta de franqueo con medidor postal o máquina franqueadora.
    PostageMeter,
    /// Cuenta oficial para entidades gubernamentales (OMAS).
    Omas,
}

impl std::fmt::Display for PaymentAccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Eps => write!(f, "EPS"),
            Self::PermitImprint => write!(f, "PERMIT_IMPRINT"),
            Self::PostageMeter => write!(f, "POSTAGE_METER"),
            Self::Omas => write!(f, "OMAS"),
        }
    }
}

/// Consulta de saldo para una cuenta EPS o permiso postal (`GET /payments/v3/payment-accounts/{accountId}/balance`).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountBalanceResponse {
    /// Número de cuenta o identificador en el sistema EPS.
    pub account_id: String,
    /// Tipo de cuenta postal.
    #[serde(default)]
    pub account_type: Option<String>,
    /// Saldo monetario actualmente disponible para franqueo (USD).
    pub available_balance: f64,
    /// Monto actualmente retenido o en autorización pendiente (USD).
    #[serde(default)]
    pub pending_balance: Option<f64>,
    /// Código de moneda ISO 4217 (por defecto "USD").
    #[serde(default)]
    pub currency: Option<String>,
    /// Estado operativo de la cuenta (ej. "ACTIVE", "SUSPENDED", "CLOSED").
    pub status: String,
    /// Identificador del titular o razón social de la cuenta EPS.
    #[serde(default)]
    pub account_holder_name: Option<String>,
}

/// Solicitud de autorización de fondos o débito previo para emisión de etiquetas masivas (`POST /payments/v3/payment-authorization`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentAuthorizationRequest {
    /// Número de cuenta EPS a debitar.
    pub account_id: String,
    /// Monto en USD a autorizar o reservar.
    pub amount: f64,
    /// Tipo de cuenta postal.
    pub account_type: PaymentAccountType,
    /// Referencia externa del pedido o transacción en el sistema del cliente.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_reference_id: Option<String>,
}

impl PaymentAuthorizationRequest {
    /// Inicia una solicitud de autorización de fondos requiriendo cuenta, monto y tipo.
    #[must_use]
    pub fn new(
        account_id: impl Into<String>,
        amount: f64,
        account_type: PaymentAccountType,
    ) -> Self {
        Self {
            account_id: account_id.into(),
            amount,
            account_type,
            client_reference_id: None,
        }
    }

    /// Asigna una referencia externa personalizada de transacción.
    #[must_use]
    pub fn client_reference_id(mut self, ref_id: impl Into<String>) -> Self {
        self.client_reference_id = Some(ref_id.into());
        self
    }
}

/// Respuesta de autorización de pago devuelta por USPS.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentAuthorizationResponse {
    /// Código de autorización emitido por USPS EPS.
    pub authorization_id: String,
    /// Número de cuenta debitada o afectada.
    pub account_id: String,
    /// Monto efectivamente autorizado (USD).
    pub authorized_amount: f64,
    /// Estado de la autorización (ej. "APPROVED", "DECLINED").
    pub status: String,
    /// Fecha y hora de caducidad de la reserva de fondos (si aplica).
    #[serde(default)]
    pub expiration_timestamp: Option<String>,
}

/// Servicio de la API v3 de Pagos y Cuentas EPS de USPS.
#[derive(Debug, Clone)]
pub struct PaymentsService {
    client: UspsClient,
}

impl PaymentsService {
    /// Crea un nuevo servicio vinculado al cliente [`UspsClient`].
    #[must_use]
    pub(crate) fn new(client: UspsClient) -> Self {
        Self { client }
    }

    /// Consulta el saldo disponible y estado operativo de una cuenta EPS (`GET /payments/v3/payment-accounts/{accountId}/balance`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si el identificador de cuenta está vacío.
    #[instrument(skip(self), name = "get_account_balance")]
    pub async fn get_account_balance(&self, account_id: &str) -> Result<AccountBalanceResponse> {
        let clean_id = account_id.trim();
        if clean_id.is_empty() {
            return Err(UspsError::InvalidInput(
                "El identificador de cuenta (account_id) no puede estar vacío".to_string(),
            ));
        }

        let endpoint = format!("/payments/v3/payment-accounts/{clean_id}/balance");
        let empty_query: [(&str, &str); 0] = [];
        self.client.get_with_query(&endpoint, &empty_query).await
    }

    /// Solicita una preautorización de fondos sobre una cuenta EPS (`POST /payments/v3/payment-authorization`).
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si la cuenta está vacía o el monto es menor o igual a cero.
    #[instrument(skip(self), name = "authorize_payment")]
    pub async fn authorize_payment(
        &self,
        req: &PaymentAuthorizationRequest,
    ) -> Result<PaymentAuthorizationResponse> {
        if req.account_id.trim().is_empty() {
            return Err(UspsError::InvalidInput(
                "El account_id no puede estar vacío".to_string(),
            ));
        }

        if req.amount <= 0.0 {
            return Err(UspsError::InvalidInput(
                "El monto de autorización debe ser estrictamente mayor a 0.00 USD".to_string(),
            ));
        }

        let endpoint = "/payments/v3/payment-authorization";
        self.client.post_json(endpoint, req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_authorization_request_builder() {
        let req = PaymentAuthorizationRequest::new("EPS-123456", 150.75, PaymentAccountType::Eps)
            .client_reference_id("ORDER-99001");

        assert_eq!(req.account_id, "EPS-123456");
        assert_eq!(req.amount, 150.75);
        assert_eq!(req.account_type, PaymentAccountType::Eps);
        assert_eq!(req.client_reference_id.as_deref(), Some("ORDER-99001"));
    }

    #[test]
    fn account_balance_response_deserialization() {
        let json = r#"{
            "accountId": "EPS-123456",
            "accountType": "EPS",
            "availableBalance": 4520.50,
            "pendingBalance": 120.00,
            "currency": "USD",
            "status": "ACTIVE",
            "accountHolderName": "Acme Logistics Corp"
        }"#;

        let res: AccountBalanceResponse = serde_json::from_str(json).unwrap();
        assert_eq!(res.account_id, "EPS-123456");
        assert_eq!(res.available_balance, 4520.50);
        assert_eq!(res.pending_balance, Some(120.00));
        assert_eq!(res.status, "ACTIVE");
        assert_eq!(
            res.account_holder_name.as_deref(),
            Some("Acme Logistics Corp")
        );
    }

    #[test]
    fn payment_account_type_display() {
        assert_eq!(PaymentAccountType::Eps.to_string(), "EPS");
        assert_eq!(
            PaymentAccountType::PermitImprint.to_string(),
            "PERMIT_IMPRINT"
        );
        assert_eq!(
            PaymentAccountType::PostageMeter.to_string(),
            "POSTAGE_METER"
        );
        assert_eq!(PaymentAccountType::Omas.to_string(), "OMAS");
    }
}
