// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Ejemplo rápido de uso básico del SDK `usps_v3_api`.
//!
//! Demuestra:
//! 1. Inicialización de [`UspsClient`] con entorno Sandbox.
//! 2. Normalización de una dirección postal de EE.UU.
//! 3. Búsqueda de ciudad y estado a partir de un código postal.
//! 4. Rastreo individual y consulta por lotes de paquetes.

use std::time::Duration;
use usps_v3_api::{AddressStandardizationRequest, TrackingExpand, UspsClient, UspsEnvironment};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Configurar credenciales y cliente SDK
    let client_id =
        std::env::var("USPS_CLIENT_ID").unwrap_or_else(|_| "demo_client_id".to_string());
    let client_secret =
        std::env::var("USPS_CLIENT_SECRET").unwrap_or_else(|_| "demo_client_secret".to_string());

    let client = UspsClient::builder()
        .credentials(client_id, client_secret)
        .environment(UspsEnvironment::Sandbox)
        .timeout(Duration::from_secs(10))
        .build()?;

    println!("Cliente USPS v3 inicializado con éxito.");

    // 2. Normalizar una dirección postal
    let addr_req = AddressStandardizationRequest::new("475 L'Enfant Plaza SW")
        .city_state("Washington", "DC")
        .zip_code("20260");

    match client.addresses().standardize(&addr_req).await {
        Ok(res) => {
            println!("Dirección estandarizada: {:?}", res.address);
        }
        Err(e) => {
            println!(
                "Consulta simulada de dirección completada (error esperado con credenciales demo: {})",
                e
            );
        }
    }

    // 3. Resolver Ciudad y Estado a partir de código postal
    match client.addresses().lookup_city_state("90210").await {
        Ok(res) => println!("Código 90210 -> {}, {}", res.city, res.state),
        Err(e) => println!("Consulta ciudad/estado (error esperado con demo): {}", e),
    }

    // 4. Rastrear paquete individual
    match client.tracking().track("9400100000000000000000").await {
        Ok(res) => println!("Estado del paquete: {:?}", res.status),
        Err(e) => println!("Consulta tracking (error esperado con demo): {}", e),
    }

    // 5. Rastrear múltiples paquetes por lote (batch)
    let batch = ["9400100000000000000001", "9400100000000000000002"];
    match client
        .tracking()
        .track_batch(&batch, TrackingExpand::Summary)
        .await
    {
        Ok(list) => println!("Total paquetes en lote rastreados: {}", list.len()),
        Err(e) => println!("Consulta batch tracking (error esperado con demo): {}", e),
    }

    Ok(())
}
