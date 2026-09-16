// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Pruebas de integración del SDK `usps_v3_api` desde la perspectiva de un consumidor externo.

use std::time::Duration;
use usps_v3_api::*;

#[test]
fn client_builder_with_custom_settings() {
    let retry = RetryPolicy::new(5, Duration::from_millis(150), Duration::from_millis(2000));

    let client = UspsClient::builder()
        .credentials("my_app_id", "my_app_secret")
        .environment(UspsEnvironment::Custom(
            "https://proxy.internal.corp:8443".into(),
        ))
        .timeout(Duration::from_secs(45))
        .retry_policy(retry.clone())
        .build()
        .expect("Error al construir cliente con configuración personalizada");

    assert_eq!(client.config().client_id, "my_app_id");
    assert_eq!(client.config().timeout, Duration::from_secs(45));
    assert_eq!(
        client.config().environment.base_url(),
        "https://proxy.internal.corp:8443"
    );
    assert_eq!(client.config().retry_policy.max_retries, 5);
}

#[tokio::test]
async fn client_can_be_shared_across_tokio_tasks() {
    let client = UspsClient::builder()
        .credentials("test_id", "test_secret")
        .environment(UspsEnvironment::Sandbox)
        .build()
        .expect("Cliente válido");

    let mut handles = Vec::new();
    for _ in 0..5 {
        let client_clone = client.clone();
        handles.push(tokio::spawn(async move {
            assert_eq!(client_clone.config().client_id, "test_id");
        }));
    }

    for handle in handles {
        handle.await.expect("Tarea de Tokio completada");
    }
}

#[tokio::test]
async fn invalid_inputs_should_fail_before_network_dispatch() {
    let client = UspsClient::builder()
        .credentials("test_id", "test_secret")
        .build()
        .unwrap();

    // 1. Direcciones: City/State con código postal inválido
    let err = client
        .addresses()
        .lookup_city_state("123")
        .await
        .unwrap_err();
    assert!(matches!(err, UspsError::InvalidInput(_)));

    // 2. Tracking: Número vacío
    let err = client.tracking().track("   ").await.unwrap_err();
    assert!(matches!(err, UspsError::InvalidInput(_)));

    // 3. Tracking por lotes: Lista vacía
    let empty_batch: [&str; 0] = [];
    let err = client
        .tracking()
        .track_batch(&empty_batch, TrackingExpand::Detail)
        .await
        .unwrap_err();
    assert!(matches!(err, UspsError::InvalidInput(_)));

    // 4. Precios: Peso negativo o cero
    let rate_req = DomesticRateRequest::new("90210", "10001", 0.0);
    let err = client
        .prices()
        .calculate_domestic_rates(&rate_req)
        .await
        .unwrap_err();
    assert!(matches!(err, UspsError::InvalidInput(_)));

    // 5. Precios internacionales: Código de país no ISO (longitud != 2)
    let intl_req = InternationalRateRequest::new("90210", "CAN", 1.0);
    let err = client
        .prices()
        .calculate_international_rates(&intl_req)
        .await
        .unwrap_err();
    assert!(matches!(err, UspsError::InvalidInput(_)));

    // 6. Service Standards: Código de destino inválido (no 5 dígitos)
    let std_req = ServiceStandardRequest::new("90210", "99");
    let err = client
        .service_standards()
        .get_estimates(&std_req)
        .await
        .unwrap_err();
    assert!(matches!(err, UspsError::InvalidInput(_)));

    // 7. Manifiestos: Sin etiquetas
    let party = LabelPartyAddress::new("100 Main St", "Orlando", "FL", "32801");
    let manifest_req = CreateManifestRequest::new(party, "32801", vec![]);
    let err = client
        .manifests()
        .create_manifest(&manifest_req)
        .await
        .unwrap_err();
    assert!(matches!(err, UspsError::InvalidInput(_)));

    // 8. Webhooks: URL HTTP inválida
    let webhook_req = CreateSubscriptionRequest::new(
        "ftp://invalid.url",
        vec![SubscriptionEventType::PackageDelivered],
    );
    let err = client.webhooks().subscribe(&webhook_req).await.unwrap_err();
    assert!(matches!(err, UspsError::InvalidInput(_)));

    // 9. Pagos EPS: Account ID vacío
    let err = client
        .payments()
        .get_account_balance("   ")
        .await
        .unwrap_err();
    assert!(matches!(err, UspsError::InvalidInput(_)));

    // 10. Pagos EPS: Monto inválido
    let pay_req = PaymentAuthorizationRequest::new("EPS-12345", 0.0, PaymentAccountType::Eps);
    let err = client
        .payments()
        .authorize_payment(&pay_req)
        .await
        .unwrap_err();
    assert!(matches!(err, UspsError::InvalidInput(_)));
}

#[test]
fn customs_declaration_aggregation_test() {
    let item1 = CustomsItem::new("T-Shirt", 2, 15.0, 0.8, "US").hs_tariff_number("6109.10");
    let item2 = CustomsItem::new("Baseball Cap", 1, 25.0, 0.4, "US").hs_tariff_number("6505.00");

    let decl = CustomsDeclaration::new(
        CustomsContentType::Merchandise,
        NonDeliveryOption::Return,
        vec![item1, item2],
    )
    .aes_itn("NOEEI 30.37(a)");

    assert_eq!(decl.items.len(), 2);
    // (2 * 15) + (1 * 25) = 55.0
    assert_eq!(decl.total_declared_value(), 55.0);
    // 0.8 + 0.4 = 1.2
    assert!((decl.total_weight_lbs() - 1.2).abs() < 1e-6);
}
