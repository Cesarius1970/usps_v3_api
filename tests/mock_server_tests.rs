// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Pruebas de integración con servidor HTTP simulado (`wiremock`).
//!
//! Verifica el ciclo completo de vida de:
//! 1. Negociación y almacenamiento en caché del Bearer Token OAuth 2.0.
//! 2. Recuperación y reintento automático ante respuestas HTTP 429 (Rate Limit).
//! 3. Deserialización de respuestas estructuradas de error de la pasarela de USPS.

use std::time::Duration;
use usps_v3_api::*;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn oauth2_handshake_and_token_caching() {
    let server = MockServer::start().await;

    // 1. Mock de autenticación OAuth 2.0 (espera exactamente 1 llamada gracias al caching)
    Mock::given(method("POST"))
        .and(path("/oauth2/v3/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_bearer_token_abc123",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .expect(1)
        .mount(&server)
        .await;

    // 2. Mock del endpoint de estandarización de direcciones (llamado 2 veces)
    Mock::given(method("GET"))
        .and(path("/addresses/v3/address"))
        .and(header("Authorization", "Bearer mock_bearer_token_abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "address": {
                "streetAddress": "475 L'ENFANT PLZ SW",
                "city": "WASHINGTON",
                "state": "DC",
                "ZIPCode": "20260"
            }
        })))
        .expect(2)
        .mount(&server)
        .await;

    let client = UspsClient::builder()
        .credentials("test_client", "test_secret")
        .environment(UspsEnvironment::Custom(server.uri()))
        .build()
        .expect("Cliente válido");

    let req = AddressStandardizationRequest::new("475 L'Enfant Plaza SW")
        .city_state("Washington", "DC")
        .zip_code("20260");

    // Primera llamada: debe solicitar token y luego consultar dirección
    let res1 = client.addresses().standardize(&req).await.unwrap();
    assert_eq!(
        res1.address.as_ref().unwrap().city.as_deref(),
        Some("WASHINGTON")
    );

    // Segunda llamada: debe reutilizar el token en caché sin consultar /oauth2/v3/token
    let res2 = client.addresses().standardize(&req).await.unwrap();
    assert_eq!(
        res2.address.as_ref().unwrap().city.as_deref(),
        Some("WASHINGTON")
    );
}

#[tokio::test]
async fn automatic_retry_on_http_429_too_many_requests() {
    let server = MockServer::start().await;

    // Token OAuth
    Mock::given(method("POST"))
        .and(path("/oauth2/v3/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&server)
        .await;

    // Primer intento devuelve 429 Too Many Requests, segundo intento devuelve 200 OK
    Mock::given(method("GET"))
        .and(path("/tracking/v3/tracking/9400100000000000000000"))
        .respond_with(ResponseTemplate::new(429).set_body_string("Rate limit exceeded"))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/tracking/v3/tracking/9400100000000000000000"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "trackingNumber": "9400100000000000000000",
            "status": "In Transit"
        })))
        .mount(&server)
        .await;

    let retry = RetryPolicy::new(3, Duration::from_millis(50), Duration::from_millis(200));

    let client = UspsClient::builder()
        .credentials("test_client", "test_secret")
        .environment(UspsEnvironment::Custom(server.uri()))
        .retry_policy(retry)
        .build()
        .expect("Cliente válido");

    let tracking = client
        .tracking()
        .track("9400100000000000000000")
        .await
        .unwrap();
    assert_eq!(tracking.status.as_deref(), Some("In Transit"));
}

#[tokio::test]
async fn structured_usps_error_deserialization() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/oauth2/v3/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&server)
        .await;

    // Simulación de error 400 Bad Request estructurado retornado por USPS
    Mock::given(method("POST"))
        .and(path("/labels/v3/label"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": "invalid_request",
            "error_description": "Invalid Destination ZIP Code",
            "errors": [
                {
                    "code": "INVALID_ZIP",
                    "message": "The destination ZIP code provided is unroutable",
                    "source": "toAddress.ZIPCode"
                }
            ]
        })))
        .mount(&server)
        .await;

    let client = UspsClient::builder()
        .credentials("test_client", "test_secret")
        .environment(UspsEnvironment::Custom(server.uri()))
        .build()
        .expect("Cliente válido");

    let from = LabelPartyAddress::new("123 Main St", "Orlando", "FL", "32801");
    let to = LabelPartyAddress::new("456 Broad St", "Nowhere", "XX", "00000");
    let pkg = PackageDescription::new(MailClass::PriorityMail, 1.0);
    let req = CreateLabelRequest::new(from, to, pkg);

    let err = client.labels().create_label(&req).await.unwrap_err();

    match err {
        UspsError::Api {
            status, response, ..
        } => {
            assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
            assert!(response.is_some());
            let resp = response.unwrap();
            assert_eq!(resp.error.as_deref(), Some("invalid_request"));
            assert_eq!(
                resp.error_description.as_deref(),
                Some("Invalid Destination ZIP Code")
            );
            assert_eq!(resp.errors.len(), 1);
            assert_eq!(resp.errors[0].code.as_deref(), Some("INVALID_ZIP"));
        }
        other => panic!("Se esperaba UspsError::Api, se obtuvo: {:?}", other),
    }
}

#[tokio::test]
async fn proof_of_delivery_and_extra_services_mock_flow() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/oauth2/v3/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "mock_token_xyz",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/prices/v3/extra-services"))
        .and(header("authorization", "Bearer mock_token_xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "extraServices": [
                {
                    "name": "Insurance",
                    "serviceId": "100",
                    "price": 3.85,
                    "description": "Coverage up to $200.00"
                }
            ],
            "warnings": []
        })))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/tracking/v3/proof-of-delivery"))
        .and(header("authorization", "Bearer mock_token_xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "trackingNumber": "9400100000000000000000",
            "requestId": "POD-REQ-12345",
            "status": "Request Processed",
            "email": "receiver@example.com"
        })))
        .mount(&server)
        .await;

    let client = UspsClient::builder()
        .credentials("test_client", "test_secret")
        .environment(UspsEnvironment::Custom(server.uri()))
        .build()
        .expect("Cliente válido");

    let extra_req = ExtraServicesRateRequest::new(MailClass::PriorityMail, 9.85, 1.5)
        .declared_value(200.0)
        .add_extra_service(ExtraServiceType::Insurance);

    let extra_res = client
        .prices()
        .calculate_extra_services(&extra_req)
        .await
        .unwrap();
    assert_eq!(extra_res.extra_services.len(), 1);
    assert_eq!(extra_res.extra_services[0].name, "Insurance");
    assert_eq!(extra_res.extra_services[0].price, 3.85);

    let pod_req = ProofOfDeliveryRequest::new(
        "9400100000000000000000",
        "receiver@example.com",
        "John",
        "Smith",
    )
    .format(ProofOfDeliveryFormat::Letter);

    let pod_res = client
        .tracking()
        .request_proof_of_delivery(&pod_req)
        .await
        .unwrap();
    assert_eq!(pod_res.tracking_number, "9400100000000000000000");
    assert_eq!(pod_res.request_id.as_deref(), Some("POD-REQ-12345"));
    assert_eq!(pod_res.status.as_deref(), Some("Request Processed"));
}
