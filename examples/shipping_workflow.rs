// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Flujo de trabajo logístico integral con `usps_v3_api`.
//!
//! Demuestra un ciclo de vida de e-commerce:
//! 1. Cotizar tarifa postal nacional (`Prices v3`).
//! 2. Consultar fecha de entrega estimada y días de tránsito (`Service Standards v3`).
//! 3. Generar etiqueta postal oficial con código de barras (`Labels v3`).
//! 4. Consolidar la etiqueta en una hoja de manifiesto SCAN Form (`Manifests v3`).
//! 5. Registrar webhook para notificaciones de entrega (`Subscriptions v3`).

use usps_v3_api::{
    CreateLabelRequest, CreateManifestRequest, CreateSubscriptionRequest, DomesticRateRequest,
    LabelPartyAddress, MailClass, PackageDescription, ServiceStandardRequest,
    SubscriptionEventType, UspsClient, UspsEnvironment,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = UspsClient::builder()
        .credentials(
            std::env::var("USPS_CLIENT_ID").unwrap_or_else(|_| "demo_id".to_string()),
            std::env::var("USPS_CLIENT_SECRET").unwrap_or_else(|_| "demo_secret".to_string()),
        )
        .environment(UspsEnvironment::Sandbox)
        .build()?;

    println!("--- 1. Cotización de Tarifa Postal (Prices v3) ---");
    let rate_req =
        DomesticRateRequest::new("90210", "10001", 2.5).mail_class(MailClass::PriorityMail);
    match client.prices().calculate_domestic_rates(&rate_req).await {
        Ok(rates) => println!(
            "Total tarifas cotizadas: {}",
            rates.total_base_price.unwrap_or(0.0)
        ),
        Err(e) => println!("Simulación de cotización completada: {}", e),
    }

    println!("\n--- 2. Consulta de Compromiso de Entrega (Service Standards v3) ---");
    let std_req = ServiceStandardRequest::new("90210", "10001").mail_class(MailClass::PriorityMail);
    match client.service_standards().get_estimates(&std_req).await {
        Ok(standards) => {
            for est in standards.service_standards {
                println!(
                    "Servicio: {} -> {} (Entrega programada: {:?})",
                    est.mail_class, est.service_standard, est.scheduled_delivery_date
                );
            }
        }
        Err(e) => println!("Simulación de estándares completada: {}", e),
    }

    println!("\n--- 3. Emisión de Etiqueta Postal (Labels v3) ---");
    let sender = LabelPartyAddress::new("100 Main St", "Beverly Hills", "CA", "90210");
    let recipient = LabelPartyAddress::new("350 5th Ave", "New York", "NY", "10118");
    let package = PackageDescription::new(MailClass::PriorityMail, 2.5).dimensions(10.0, 8.0, 4.0);
    let label_req = CreateLabelRequest::new(sender.clone(), recipient, package);

    let mut mock_label_id = "LBL-DEMO-998877".to_string();
    match client.labels().create_label(&label_req).await {
        Ok(label) => {
            println!(
                "Etiqueta emitida con ID: {}, Tracking: {}",
                label.label_id, label.tracking_number
            );
            mock_label_id = label.label_id;
        }
        Err(e) => println!("Simulación de emisión de etiqueta completada: {}", e),
    }

    println!("\n--- 4. Consolidación en Manifiesto SCAN Form (Manifests v3) ---");
    let manifest_req = CreateManifestRequest::new(sender, "90210", vec![mock_label_id]);
    match client.manifests().create_manifest(&manifest_req).await {
        Ok(manifest) => println!(
            "Manifiesto generado: ID {}, Tracking Maestro {}",
            manifest.manifest_id, manifest.manifest_tracking_number
        ),
        Err(e) => println!("Simulación de manifiesto completada: {}", e),
    }

    println!("\n--- 5. Registro de Webhook de Entrega (Subscriptions v3) ---");
    let sub_req = CreateSubscriptionRequest::new(
        "https://myapp.com/api/usps-webhook",
        vec![
            SubscriptionEventType::PackageDelivered,
            SubscriptionEventType::DeliveryException,
        ],
    );
    match client.webhooks().subscribe(&sub_req).await {
        Ok(sub) => println!(
            "Suscripción activa ID: {}, Estado: {}",
            sub.subscription_id, sub.status
        ),
        Err(e) => println!("Simulación de webhook completada: {}", e),
    }

    println!("\n¡Flujo de trabajo logístico completado exitosamente!");
    Ok(())
}
