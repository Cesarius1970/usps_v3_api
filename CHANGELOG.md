# Changelog

Todos los cambios notables en este proyecto serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto adhiere a [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-09-15

### Añadido
- **Servicios Oficiales USPS v3:**
  - `ServiceStandardsService` (`service-standards/v3`): estimación de compromisos y fechas estimadas de entrega (EDD) con `get_estimates`.
  - `PaymentsService` (`payments/v3`): consulta de saldos en cuentas EPS (`get_account_balance`) y autorización de pagos (`authorize_payment`).
  - `CustomsDeclaration` y `CustomsItem` (`customs/v3`): modelado estructurado para declaraciones aduaneras internacionales CN22/CP72.
  - `ManifestsService` (`manifests/v3`): consolidación de envíos en manifiestos oficiales SCAN Form (PS Form 5630).
  - `WebhooksService` (`subscriptions/v3`): gestión y registro de suscripciones a eventos de paquetes por webhook.
  - `LabelBroker`: emisión de códigos QR y códigos de mostrador (`create_label_broker`) y descarga de datos de etiquetas (`get_label_data`).
  - `Extra Services Pricing`: cotización de servicios adicionales (`calculate_extra_services`) como seguros, firma y acuse de recibo.
  - `Electronic Proof of Delivery (ePOD)`: solicitud de comprobante oficial de entrega firmado vía email (`request_proof_of_delivery`).
  - `track_batch`: consulta masiva de hasta 35 números de seguimiento en una sola petición.
- **Resiliencia y Concurrencia:**
  - `RetryPolicy` con retroceso exponencial (*exponential backoff*) y *jitter* determinístico ante códigos HTTP transitorios (429, 500, 502, 503, 504).
  - Integración transparente en `UspsClient` para reintentos sin intervención del consumidor.
- **Pruebas y Verificación:**
  - Suite de pruebas de servidor HTTP simulado con `wiremock` (`tests/mock_server_tests.rs`).
  - Suite de pruebas de integración y concurrencia (`tests/integration_tests.rs`).
  - Banco ampliado a más de 50 pruebas automatizadas sin advertencias de linter (`-D warnings`).
- **Ejemplos y Documentación:**
  - `examples/quickstart.rs`: guía rápida para direcciones y rastreo.
  - `examples/shipping_workflow.rs`: flujo logístico e-commerce de extremo a extremo.
  - Badges oficiales y guía de uso renovada en `README.md`.
  - Flujo de trabajo de GitHub Actions CI en `.github/workflows/ci.yml`.

## [0.1.0] - 2026-09-15

### Añadido
- Versión inicial pública del SDK `usps_v3_api`.
- Gestor de autenticación OAuth 2.0 con auto-renovación concurrente en memoria (`RwLock`).
- Módulo `Addresses v3`: estandarización de direcciones, validación DPV y búsqueda de códigos postales.
- Módulo `Tracking v3`: rastreo individual y línea de tiempo de eventos postales.
- Módulo `Prices v3`: cálculo de tarifas de franqueo nacionales e internacionales.
- Módulo `Labels v3`: emisión de etiquetas postales con código de barras y anulación de franqueo.
- Módulo `Pickup v3`: consulta de disponibilidad y programación/cancelación de recolección a domicilio.
- Módulo `Locations v3`: búsqueda de instalaciones postales, buzones y horarios de atención.
- Manejo tipado de errores con `UspsError` y `UspsApiErrorResponse`.
- Doble licencia MIT / Apache 2.0.
