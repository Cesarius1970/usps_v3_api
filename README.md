# usps_v3_api

[![Crates.io](https://img.shields.io/crates/v/usps_v3_api.svg)](https://crates.io/crates/usps_v3_api)
[![Documentation](https://docs.rs/usps_v3_api/badge.svg)](https://docs.rs/usps_v3_api)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![CI](https://github.com/Cesarius1970/usps_v3_api/actions/workflows/ci.yml/badge.svg)](https://github.com/Cesarius1970/usps_v3_api/actions)

Cliente SDK en Rust idiomático, fuertemente tipado, asíncrono y listo para producción para el ecosistema de **APIs REST v3 de USPS (United States Postal Service)**.

---

## Características Principales

- **Arquitectura Enterprise en Capas:** Separación limpia entre infraestructura transversal (`core`) y servicios de negocio (`services`).
- **Autenticación Inteligente OAuth 2.0:** Auto-refresh en memoria con `tokio::sync::RwLock`, renovación transparente previa a caducidad (`double-checked locking`) y sanitización de secretos en logs (`[REDACTED]`).
- **Resiliencia Automática (`RetryPolicy`):** Backoff exponencial con jitter determinístico para manejar transparentemente `429 Too Many Requests`, `500`, `502`, `503` y `504`.
- **Rastreo Masivo por Lotes:** Soporte de rastreo simultáneo de hasta 35 paquetes por llamada en `TrackingService`.
- **Soporte Completo de Servicios USPS REST v3:**

| Servicio | Módulo | Endpoint Base | Descripción |
| :--- | :--- | :--- | :--- |
| **Addresses v3** | `services::addresses` | `/addresses/v3` | Estandarización, validación DPV, búsqueda de ZIP codes y ciudad/estado. |
| **Tracking v3** | `services::tracking` | `/tracking/v3` | Seguimiento individual y por lotes (hasta 35 paquetes), eventos históricos y Prueba Electrónica de Entrega (ePOD). |
| **Prices v3** | `services::prices` | `/prices/v3` | Tarifas nacionales base/dimensionales, internacionales y servicios especiales (Extra Services). |
| **Labels v3** | `services::labels` | `/labels/v3` | Emisión (PDF, PNG, TIFF, SVG, Base64), anulación y código QR **Label Broker**. |
| **Service Standards v3** | `services::standards` | `/service-standards/v3` | Días de tránsito, compromisos y fechas estimadas de entrega (EDD). |
| **Manifests v3** | `services::manifests` | `/manifests/v3` | Hojas de manifiesto oficial de entrega **SCAN Form (PS Form 5630)**. |
| **Pickup v3** | `services::pickup` | `/pickup/v3` | Disponibilidad, programación a domicilio y cancelación de recolección de cartero. |
| **Locations v3** | `services::locations` | `/locations/v3` | Búsqueda de oficinas postales y quioscos por ZIP o coordenadas, horarios semanales. |
| **Subscriptions v3** | `services::webhooks` | `/subscriptions/v3` | Webhooks y callbacks HTTP en tiempo real para eventos de rastreo y entrega. |
| **Payments v3** | `services::payments` | `/payments/v3` | Cuentas EPS (Enterprise Payment System), consulta de saldos y autorizaciones. |
| **Customs v3** | `services::customs` | `/customs/v3` | Declaraciones aduaneras internacionales CN22/CP72, códigos HTS y AES/ITN. |

---

## Instalación

Agrega `usps_v3_api` a tu `Cargo.toml`:

```toml
[dependencies]
# Por defecto utiliza 'rustls-tls' (cero dependencias de C / OpenSSL)
usps_v3_api = "0.2.1"
tokio = { version = "1", features = ["full"] }

# O si prefieres utilizar los certificados nativos del sistema operativo (OpenSSL / SChannel / SecurityFramework):
# usps_v3_api = { version = "0.2.1", default-features = false, features = ["native-tls"] }
```

---

## Ejemplo Rápido

```rust
use usps_v3_api::{
    AddressStandardizationRequest, TrackingExpand, UspsClient, UspsEnvironment,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Inicializar cliente con credenciales de USPS Developer Portal
    let client = UspsClient::builder()
        .credentials("TU_CLIENT_ID", "TU_CLIENT_SECRET")
        .environment(UspsEnvironment::Sandbox) // o UspsEnvironment::Production
        .build()?;

    // 2. Normalizar una dirección postal
    let req = AddressStandardizationRequest::new("475 L'Enfant Plaza SW")
        .city_state("Washington", "DC")
        .zip_code("20260");

    let res = client.addresses().standardize(&req).await?;
    println!("Dirección estandarizada: {:?}", res.address);

    // 3. Rastrear un paquete
    let tracking = client.tracking().track("9400100000000000000000").await?;
    println!("Estado del paquete: {:?}", tracking.status);

    // 4. Rastrear hasta 35 paquetes por lote
    let batch = ["9400100000000000000001", "9400100000000000000002"];
    let results = client.tracking().track_batch(&batch, TrackingExpand::Summary).await?;
    println!("Total paquetes rastreados en lote: {}", results.len());

    Ok(())
}
```

---

## Ejemplos Incluidos

El repositorio incluye ejemplos ejecutables en el directorio [`examples/`](examples/):

- **Inicio Rápido:**
  ```bash
  cargo run --example quickstart
  ```
- **Flujo de Trabajo Logístico Completo:**
  ```bash
  cargo run --example shipping_workflow
  ```

---

## Compilación y Pruebas

```bash
# Compilar proyecto y dependencias
cargo build

# Ejecutar las más de 50 pruebas unitarias, de integración y con servidor mock HTTP (wiremock)
cargo test --all-targets --all-features

# Verificar lints estrictos
cargo clippy --all-targets --all-features -- -D warnings

# Verificar formateo oficial
cargo fmt --check
```

---

## Licencia y Copyright

Copyright (c) 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>.

Distribuido bajo la doble licencia estándar del ecosistema oficial de Rust:
- **Licencia Apache, Versión 2.0** ([LICENSE-APACHE](LICENSE-APACHE) o <http://www.apache.org/licenses/LICENSE-2.0>)
- **Licencia MIT** ([LICENSE-MIT](LICENSE-MIT) o <http://opensource.org/licenses/MIT>)

Cualquier contribución enviada intencionalmente para su inclusión se licenciará bajo estos mismos términos duales.
