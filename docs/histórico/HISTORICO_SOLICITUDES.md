# Histórico de Solicitudes y Respuestas

Este documento registra de manera numerada y secuencial todas las interacciones, solicitudes del usuario y respuestas del agente técnico para el proyecto `usps_v3_api`.

---

## Interacción #1

### Solicitud del Usuario
```text
/karpathy-guidelines
```

### Respuesta del Agente
- Carga y adopción de los lineamientos de Andrej Karpathy:
  1. **Pensar antes de codificar (Think Before Coding):** No asumir, explicitar dudas, exponer disyuntivas antes de escribir código.
  2. **Simplicidad primero (Simplicity First):** Código mínimo que solucione el requerimiento sin abstracciones prematuras ni sobreingeniería.
  3. **Cambios quirúrgicos (Surgical Changes):** Tocar exclusivamente lo necesario y conservar la integridad del código circundante.
  4. **Ejecución guiada por objetivos (Goal-Driven Execution):** Criterios de verificación comprobables mediante pruebas antes de dar por cerrada la tarea.

---

## Interacción #2

### Solicitud del Usuario
```text
/rust-best-practices
```

### Respuesta del Agente
- Carga y adopción de las mejores prácticas de Rust (basadas en el compendio de Apollo GraphQL):
  - Priorizar referencias (`&str`, `&[T]`) y préstamos sobre clones innecesarios.
  - Gestión robusta de errores mediante `Result<T, E>` y operador `?`; prohibido el uso de `unwrap()` y `expect()` fuera de pruebas.
  - Mentalidad de alto rendimiento y costo cero en abstracciones.
  - Estricta verificación de lints (`cargo clippy --all-targets --all-features -- -D warnings`).
  - Nombres descriptivos de tests y pruebas documentadas (doctests).
  - Documentación `///` e integridad de comentarios.

---

## Interacción #3

### Solicitud del Usuario
```text
/rust-async-patterns
```

### Respuesta del Agente
- Carga y adopción de patrones de concurrencia y programación asíncrona con el runtime Tokio:
  - Nunca bloquear hilos del runtime Tokio (delegar tareas intensivas o bloqueantes a `tokio::task::spawn_blocking`).
  - No retener mutexes sincrónicos a través de puntos de suspensión `.await`.
  - Orquestación concurrente con `tokio::select!`, `JoinSet` y canales acotados (`mpsc`, `watch`, `oneshot`).
  - Manejo adecuado de cancelación (`CancellationToken`) y propagación segura de errores con `Send + 'static`.

---

## Interacción #4

### Solicitud del Usuario
```text
Genera una carpeta "docs\histórico" y guarda en ella un histórico de los prompts y solicitudes, HISTORICO_SOLICITUDES.md así como tus respuestas de manera numerada y secuencializada. Mantén actualizada esta carpeta con la interacción que se genere respecto a este proyecto

Pon como regla que debe generarse un commit al finalizar cada fase o interacción con el agente. Usar los estándares de git para nombrar los commit y las ramas

Genera comentarios pertinentes en los archivos y algoritmos de manera de generar un manual técnico en “docs\MANUAL_TECNICO.md”. Mantén actualizado este documento

Modifica Cargo.toml y añade descripciones y campos obligatorios. Crea la documentación necesaria para el copyright de esta librería a nombre de "César A Vergara Buenaventura" cesarvergarab@gmail.com. Sigue obligatoriamente el estándar del equipo de desarrollo de Rust. Modifica o crea los archivos que sean necesarios.
```

### Respuesta del Agente
1. **Creación de Carpeta de Histórico y Registro Cronológico:**
   - Creación de la carpeta `docs/histórico/` y el presente archivo `HISTORICO_SOLICITUDES.md`, documentando todas las interacciones secuencialmente desde el inicio del proyecto.
2. **Definición de Reglas de Git:**
   - Se estableció como norma obligatoria la creación de un commit al concluir cada fase o interacción con el agente.
   - Adopción de Conventional Commits (`feat:`, `fix:`, `docs:`, `chore:`, etc.) y convención estándar de nomenclatura de ramas (`feature/*`, `bugfix/*`, `docs/*`, etc.), documentado detalladamente en `docs/MANUAL_TECNICO.md`.
3. **Comentarios de Código y Manual Técnico:**
   - Documentación exhaustiva con estándares de Rust (`//!`, `///`, `#[must_use]`, doctests) en `src/lib.rs`.
   - Creación del manual técnico completo en `docs/MANUAL_TECNICO.md` describiendo autoría, copyright, estándares, arquitectura, módulos, algoritmos y procedimientos de compilación/prueba.
4. **Modificación de `Cargo.toml` y Estándar de Copyright:**
   - Configuración completa de metadatos en `Cargo.toml` (`authors`, `description`, `license`, `repository`, `homepage`, `documentation`, `keywords`, `categories`, `edition = "2024"`, `rust-version = "1.85.0"`).
   - Generación de los archivos oficiales de licencia dual del ecosistema Rust: `LICENSE-MIT` y `LICENSE-APACHE`.
   - Creación del archivo `NOTICE` con la atribución explícita a César A Vergara Buenaventura (`cesarvergarab@gmail.com`).
   - Creación de `README.md` con descripción y términos de licencia y contribución.
5. **Verificación y Pruebas:**
   - Ejecución exitosa de `cargo test` (unit tests + doctests pasando con 0 errores).
   - Ejecución de `cargo clippy -- -D warnings` con 0 advertencias.
   - Generación del commit inicial en Git cumpliendo la regla establecida.

---

## Interacción #5

### Solicitud del Usuario
```text
Inicializar y desarrollar una librería cliente SDK en Rust fuertemente tipada y lista para producción para el ecosistema de APIs REST USPS v3.
```

### Clarificación y Selección de Alcance
- Se consultó al usuario sobre la priorización de servicios para la primera fase.
- El usuario seleccionó la opción recomendada:
  `Fase 1: Arquitectura base (Config, OAuth2 auto-refresh, UspsClient) + Addresses v3 y Tracking v3 completos.`

### Respuesta del Agente
1. **Configuración de Dependencias de Producción (`Cargo.toml`):**
   - Incorporación de dependencias: `reqwest` (con TLS multiplataforma rustls y json), `tokio`, `serde`, `serde_json`, `thiserror`, `tracing` y `chrono`.
2. **Jerarquía Tipada de Errores (`src/error.rs`):**
   - Definición de `UspsError` utilizando `thiserror` para modelar fallas de red, serialización, autenticación, entradas inválidas y errores HTTP devueltos por USPS.
   - Implementación de `UspsApiErrorResponse` y `ApiErrorDetail` con análisis resiliente y fallback en `UspsError::from_response`.
3. **Gestión Segura de Configuración y Ambientes (`src/config.rs`):**
   - Modelado de `UspsEnvironment` (`Sandbox` - `https://api-cat.usps.com`, `Production` - `https://api.usps.com`, `Custom`).
   - Implementación de `UspsConfig` con constructor validado y sanitización de seguridad en `fmt::Debug` (`[REDACTED]` para `client_secret`).
4. **Autenticación OAuth 2.0 con Auto-Refresh Concurrente (`src/auth.rs`):**
   - Implementación de `TokenManager` con `tokio::sync::RwLock` y patrón *double-checked locking*.
   - Renovación automática y transparente del token con margen de anticipación de 60 segundos (`EXPIRATION_BUFFER_SECS`).
5. **Cliente Central y Patrón Builder (`src/client.rs`):**
   - Implementación de `UspsClient` con puntero interno atómico `Arc` para clonado de costo cero.
   - `UspsClientBuilder` para inicialización ergonómica y validada.
   - Métodos autenticados genéricos `get_with_query` y `post_json`.
6. **Módulo de Direcciones v3 (`src/addresses.rs`):**
   - Modelos fuertemente tipados: `AddressStandardizationRequest`, `StandardizedAddress`, `AddressResponse`, `ZipCodeLookupRequest`, `CityStateResponse`.
   - Métodos de servicio: `standardize`, `lookup_zip_code`, `lookup_city_state`.
7. **Módulo de Seguimiento v3 (`src/tracking.rs`):**
   - Modelos fuertemente tipados: `TrackingResponse`, `TrackingEvent`, `TrackingExpand`.
   - Métodos de servicio: `track`, `track_with_expand`.
8. **Exposición Pública y Documentación Viva:**
   - Actualización de `src/lib.rs` con re-exportaciones de alto nivel y doctests interactivos.
   - Actualización exhaustiva del manual técnico en `docs/MANUAL_TECNICO.md`.
9. **Verificación de Calidad y Cierre:**
   - Ejecución de `cargo fmt --check`: Cumplimiento total de formato.
   - Ejecución de `cargo test`: 13 pruebas unitarias y 1 doctest aprobados (0 fallos).
   - Ejecución de `cargo clippy --all-targets --all-features -- -D warnings`: 0 advertencias.
   - Generación de commit bajo estándar Conventional Commits.

---

## Interacción #6

### Solicitud del Usuario
```text
Continua con el resto de fases secuencialmente
```

### Respuesta del Agente
Se desarrollaron e integraron secuencialmente los módulos restantes de la suite oficial USPS v3 REST API:

1. **Fase 2: Precios y Tarifas Nacionales (`src/prices.rs` - `Prices v3`):**
   - Modelado de solicitud `DomesticRateRequest` con soporte de dimensiones, clases postales (`MailClass`: Priority Mail, Ground Advantage, Priority Mail Express, etc.) y categorías de procesamiento (`ProcessingCategory`).
   - Implementación de `PricesService::calculate_domestic_rates` (`POST /prices/v3/base-rates/search`).
   - Parseo tipado de respuestas con `DomesticRateResponse` y `RateItem`.

2. **Fase 3: Emisión y Cancelación de Etiquetas Postales (`src/labels.rs` - `Labels v3`):**
   - Modelado exhaustivo de solicitud `CreateLabelRequest`, `LabelPartyAddress` (remitente/destinatario) y `PackageDescription`.
   - Soporte de formatos gráficos `LabelImageType` (`PDF`, `PNG`, `TIFF`, `SVG`) y tamaños térmicos (`4X6`, `4X4`).
   - Implementación de `LabelsService::create_label` (`POST /labels/v3/label`) para generación con imagen Base64 y código de barras.
   - Implementación de `LabelsService::cancel_label` (`DELETE /labels/v3/label/{labelId}`) para anulación y reembolso de etiquetas.

3. **Fase 4: Recolección de Paquetes en Domicilio (`src/pickup.rs` - `Package Pickup v3`):**
   - Consulta de disponibilidad geográfica de recolección: `PickupService::check_availability` (`GET /pickup/v3/carrier-pickup/availability`).
   - Programación de visita del cartero: `PickupService::schedule` (`POST /pickup/v3/carrier-pickup`) con especificación de ubicación (`PackageLocation`) y conteo discriminado de paquetes (`PickupPackageCount`).
   - Cancelación de recolección programada: `PickupService::cancel` (`DELETE /pickup/v3/carrier-pickup/{confirmationNumber}`).

4. **Integración en Cliente Central y API Pública:**
   - Métodos agregados en `UspsClient`: `.prices()`, `.labels()` y `.pickup()`.
   - Actualización de `src/lib.rs` con re-exportaciones de alto nivel y documentación de características.

5. **Documentación Viva y Verificación Integral:**
   - Actualización completa de `docs/MANUAL_TECNICO.md` con los nuevos módulos, algoritmos y contratos de servicio.
   - Ejecución de `cargo fmt --check`: Cumplimiento 100% con `rustfmt`.
   - Ejecución de `cargo test`: 21 pruebas unitarias y 1 doctest aprobados (0 fallos).
   - Ejecución de `cargo clippy --all-targets --all-features -- -D warnings`: 0 advertencias.
   - Generación de commit en Git bajo la convención establecida.

---

## Interacción #7

### Solicitud del Usuario
```text
Continua con el resto de fases secuencialmente
```

### Respuesta del Agente
Se desarrollaron e integraron secuencialmente las capacidades avanzadas de cobertura internacional e infraestructura física de USPS:

1. **Fase 5: Ubicaciones e Instalaciones Físicas (`src/locations.rs` - `Locations v3`):**
   - Búsqueda geográfica por código postal de 5 dígitos o por coordenadas de latitud/longitud con radio en millas (`LocationSearchRequest`).
   - Filtrado por tipos de servicio especializado (`LocationServiceType`: pasaportes, apartados postales, quioscos automáticos, etc.).
   - Modelado de horarios de atención semanales (`DailyHours`) y ficha completa de la instalación (`LocationFacility`).
   - Métodos de servicio: `LocationsService::search` y `LocationsService::get_details`.

2. **Fase 6: Tarifas y Precios Internacionales (`src/prices.rs` - `International Rates v3`):**
   - Modelado de solicitud `InternationalRateRequest` con soporte de país de destino (código ISO de 2 letras), código postal foráneo, peso y dimensiones.
   - Clases de servicio internacional (`InternationalMailClass`: Global Express Guaranteed, Priority Mail Express International, Priority Mail International, First-Class Package International).
   - Implementación del método `PricesService::calculate_international_rates` (`POST /prices/v3/international-base-rates/search`).

3. **Integración en Cliente Central y API Pública:**
   - Método `.locations()` expuesto en `UspsClient`.
   - Re-exportación de todos los tipos y servicios nuevos en `src/lib.rs`.

4. **Documentación Técnica y Verificación de Calidad:**
   - Actualización de `docs/MANUAL_TECNICO.md` reflejando la cobertura completa del ecosistema.
   - Ejecución de `cargo fmt --check`: 100% compliant.
   - Ejecución de `cargo test`: 25 pruebas unitarias y 1 doctest aprobados (0 fallos).
   - Ejecución de `cargo clippy --all-targets --all-features -- -D warnings`: 0 advertencias.
   - Generación de commit en Git bajo la convención establecida.

---

## Interacción #8

### Solicitud del Usuario
```text
Refactoriza el orden de las carpetas de src y ordenalas segun las mejores practicas
```

### Respuesta del Agente
Se llevó a cabo una refactorización arquitectónica completa del directorio `src/`, aplicando el estándar de diseño en capas para SDKs enterprise en Rust:

1. **Separación de Responsabilidades en Capas (SoC):**
   - **`src/core/` (Capa de Infraestructura y Transporte):**
     - `auth.rs`: Autenticación OAuth 2.0, token caching thread-safe con `RwLock`.
     - `client.rs`: `UspsClient` y `UspsClientBuilder`, despacho de peticiones HTTP autenticadas.
     - `config.rs`: `UspsConfig`, ambientes `UspsEnvironment` y saneamiento de secretos en logs.
     - `error.rs`: Jerarquía tipada `UspsError`, `UspsApiErrorResponse` y deserialización de errores.
     - `mod.rs`: Re-exportaciones públicas de la capa de infraestructura.
   - **`src/services/` (Capa de Servicios de Negocio USPS v3):**
     - `addresses.rs`: Normalización postal y validación DPV (`Addresses v3`).
     - `labels.rs`: Generación de etiquetas con código de barras y anulación (`Labels v3`).
     - `locations.rs`: Búsqueda de oficinas y buzones (`Locations v3`).
     - `pickup.rs`: Programación y cancelación de recolección a domicilio (`Pickup v3`).
     - `prices.rs`: Cotizaciones de tarifas nacionales e internacionales (`Prices v3`).
     - `tracking.rs`: Seguimiento y eventos de tránsito (`Tracking v3`).
     - `mod.rs`: Re-exportaciones públicas del catálogo de servicios.
2. **Preservación Total de Compatibilidad de la API Pública (`src/lib.rs`):**
   - Se mantuvieron las re-exportaciones de alto nivel en la raíz del crate (`usps_v3_api::*`).
   - Se agregaron módulos públicos canónicos (`usps_v3_api::core::*` y `usps_v3_api::services::*`) y aliases de retrocompatibilidad directa (`usps_v3_api::addresses::*`, etc.).
3. **Actualización de Documentación Técnica:**
   - Actualización de `docs/MANUAL_TECNICO.md` reflejando el nuevo árbol modular y la justificación de la separación de capas.
4. **Verificación de Calidad y Cierre:**
   - Ejecución de `cargo fmt --check`: 100% compliant.
   - Ejecución de `cargo test`: 25 pruebas unitarias y 1 doctest aprobados (0 fallos).
   - Ejecución de `cargo clippy --all-targets --all-features -- -D warnings`: 0 advertencias.
   - Generación de commit bajo estándar Conventional Commits (`refactor(architecture): ...`).

---

## Interacción #9

### Solicitud del Usuario
```text
Dar por cerrada esta version. Subela a git con el tag v0.1.0 y publicala en crates.io
```

### Respuesta del Agente
1. **Cierre y Congelamiento de Versión v0.1.0:**
   - La suite completa de clientes para la API REST v3 de USPS queda consolidada en la versión `0.1.0`.
   - Se actualizó el manual técnico en `docs/MANUAL_TECNICO.md` integrando la Sección 7 con el registro del release v0.1.0.
2. **Validación Exhaustiva de Empaquetado para crates.io:**
   - Ejecución de `cargo publish --dry-run`: Empaquetado satisfactorio de 24 archivos (47 KiB comprimidos), verificación de dependencias y compilación limpia sin advertencias.
3. **Control de Versiones en Git:**
   - Generación del commit de cierre: `chore(release): prepare and tag version 0.1.0`.
   - Creación del tag anotado oficial: `v0.1.0`.
   - Verificación de remotos de Git (`git remote -v`).
4. **Publicación en crates.io:**
   - Publicación formal ejecutada con `cargo publish`.

---

## Interacción #10

### Solicitud del Usuario
```text
Continuar con las siguientes fases del desarrollo segun el plan
```

### Respuesta del Agente
Se continuó con la siguiente fase integral de desarrollo del SDK, incorporando infraestructura de resiliencia avanzada, nuevos servicios oficiales de la API v3 de USPS, pipeline automatizado de CI/CD y ampliación del banco de pruebas:

1. **Infraestructura de Resiliencia y Reintentos (`src/core/retry.rs`):**
   - Creación de la estructura [`RetryPolicy`] configurable en [`UspsConfig`] y en [`UspsClientBuilder`].
   - Algoritmo de backoff exponencial con cálculo determinístico de *jitter* para evitar saturación (*thundering herd*).
   - Detección inteligente de estados HTTP transitorios reintentables: `429 Too Many Requests`, `500 Internal Server Error`, `502 Bad Gateway`, `503 Service Unavailable`, `504 Gateway Timeout`.
   - Soporte transparente de reintentos automáticos integrado en los despachos HTTP centrales (`get_with_query`, `post_json` y `delete`).
   - Implementación del método unificado `client.delete()` reutilizando el pool de conexiones existente.

2. **Servicio de Manifiestos Postales SCAN Form (`src/services/manifests.rs` - `Manifests v3`):**
   - Modelado de solicitud `CreateManifestRequest` con validación estricta de bultos y código postal de oficina de ingreso de 5 dígitos.
   - Modelado de respuesta `CreateManifestResponse` con código de barras maestro y soporte para formulario PS Form 5630 en PDF o imagen.
   - Implementación de los métodos `ManifestsService::create_manifest` (`POST /manifests/v3/manifest`) y `ManifestsService::get_manifest` (`GET /manifests/v3/manifest/{id}`).
   - Re-exportación e integración accesible desde `client.manifests()`.

3. **Servicio de Suscripciones y Notificaciones Webhook (`src/services/webhooks.rs` - `Subscriptions v3`):**
   - Modelado de tipos de eventos postales con [`SubscriptionEventType`] (`TRACKING_EVENTS`, `PACKAGE_DELIVERED`, `DELIVERY_EXCEPTION`, `RETURN_TO_SENDER`).
   - Solicitud `CreateSubscriptionRequest` con validación de URL HTTPS y token secreto opcional para validación de firmas HMAC.
   - Respuestas fuertemente tipadas `SubscriptionResponse` y `DeleteSubscriptionResponse`.
   - Implementación de métodos `WebhooksService::subscribe`, `WebhooksService::get_subscription` y `WebhooksService::delete_subscription`.
   - Re-exportación e integración accesible desde `client.webhooks()`.

4. **Automatización de Integración Continua (CI/CD):**
   - Creación del flujo de trabajo de GitHub Actions en `.github/workflows/ci.yml`.
   - Verificaciones automáticas en cada `push` y `pull_request` a `main`: formateo (`cargo fmt`), análisis de linter sin advertencias (`cargo clippy`), validación de documentación (`cargo doc`) y suite de pruebas (`cargo test`) con caché de dependencias.

5. **Documentación Técnica y Verificación de Calidad:**
   - Actualización exhaustiva del árbol de componentes, algoritmos y descripciones de servicios en `docs/MANUAL_TECNICO.md`.
   - Incorporación de nuevas pruebas unitarias: suite expandida a 31 pruebas unitarias y 1 doctest, todas superadas con éxito.
   - Verificación estricta de formato (`cargo fmt --check`) y análisis de lints (`cargo clippy --all-targets --all-features -- -D warnings` con 0 advertencias).
   - Generación de commit bajo estándar Conventional Commits (`feat(sdk): add Manifests v3, Subscriptions v3, and RetryPolicy resilience`).

---

## Interacción #11

### Solicitud del Usuario
```text
Continuar con las siguientes fases del desarrollo segun el plan
```

### Respuesta del Agente
Se ejecutó la siguiente fase del plan de desarrollo, consolidando el soporte para estándares de entrega postal, rastreo masivo por lotes, suite de pruebas de integración externa y recetas prácticas de uso:

1. **Servicio de Estándares de Entrega y Tiempos de Tránsito (`src/services/standards.rs` - `Service Standards v3`):**
   - Implementación de [`ServiceStandardsService`] para consulta del endpoint `GET /service-standards/v3/estimates`.
   - Modelado de solicitud [`ServiceStandardRequest`] con validación de códigos postales de 5 dígitos de origen y destino, fecha de depósito y filtro por clase postal (`MailClass`).
   - Modelado de respuesta [`ServiceStandardResponse`] y estimaciones individuales [`ServiceStandardEstimate`], con fechas programadas (EDD), días de tránsito y horarios de corte de admisión.
   - Integración fluida accesible desde `client.service_standards()`.

2. **Rastreo Masivo por Lotes (`src/services/tracking.rs` - `Tracking v3`):**
   - Incorporación del método `TrackingService::track_batch`, permitiendo consultar hasta 35 números de seguimiento en una única solicitud HTTP con parámetros de expansión (`TrackingExpand`).
   - Validación estricta en el cliente para listas vacías o listas que superen la cuota de 35 envíos por llamada.

3. **Ejemplos Prácticos y Recetas de Uso (`examples/`):**
   - `examples/quickstart.rs`: Demostración concisa de inicio rápido con estandarización de direcciones, resolución de ciudad/estado y rastreo individual y por lotes.
   - `examples/shipping_workflow.rs`: Ciclo completo de logística e-commerce (cotización de tarifas, verificación de compromisos de entrega, emisión de etiqueta postal, consolidación en manifiesto SCAN Form y registro de webhook).

4. **Suite de Pruebas de Integración y Concurrencia (`tests/integration_tests.rs`):**
   - Pruebas desde la perspectiva de un consumidor externo del crate (`usps_v3_api::*`).
   - Verificación de concurrencia y clonado seguro de [`UspsClient`] a través de múltiples tareas de Tokio.
   - Validación de configuración personalizada (`RetryPolicy` con delays y jitter, entornos custom).
   - Pruebas de fallo preventivo en validaciones de entrada previas a despachos de red en todos los servicios.

5. **Actualización de Versión y Métricas de Calidad:**
   - Incremento de versión en `Cargo.toml` a `0.2.0`.
   - Banco de pruebas incrementado a **38 pruebas automáticas (35 unitarias + 3 de integración) y 1 doctest**, todas superadas al 100%.
   - Verificación de formato (`cargo fmt --check`) y análisis de lints estricto (`cargo clippy --all-targets --all-features -- -D warnings` con 0 advertencias).
   - Actualización del Manual Técnico `docs/MANUAL_TECNICO.md` y bitácora histórica.
   - Generación de commit bajo estándar Conventional Commits (`feat(sdk): add Service Standards v3, batch tracking, examples, and bump to v0.2.0`).

---

## Interacción #12

### Solicitud del Usuario
```text
Continuar con las siguientes fases del desarrollo segun el plan
```

### Respuesta del Agente
Se continuó con la siguiente fase avanzada del desarrollo del SDK, completando la cobertura de servicios corporativos e internacionales de la API REST v3 de USPS con la integración de pagos empresariales y declaraciones de aduana:

1. **Servicio de Pagos y Cuentas EPS (`src/services/payments.rs` - `Payments v3`):**
   - Implementación de [`PaymentsService`] para la interacción con el Enterprise Payment System (EPS) de USPS.
   - Consulta de saldos disponibles, montos retenidos y estados operativos de cuentas EPS (`AccountBalanceResponse`, método `get_account_balance`).
   - Gestión de solicitudes de autorización y reserva de fondos (`PaymentAuthorizationRequest`, `PaymentAuthorizationResponse`, método `authorize_payment`).
   - Soporte para instrumentos de pago [`PaymentAccountType`]: `EPS`, `PERMIT_IMPRINT`, `POSTAGE_METER` y `OMAS`.
   - Accesible desde el cliente SDK mediante `client.payments()`.

2. **Módulo de Declaraciones de Aduana Internacional (`src/services/customs.rs` - `Customs v3`):**
   - Modelado de declaraciones para formularios aduaneros oficiales CN22 y CP72 (**PS Form 2976 / 2976-A**).
   - Estructuración de artículos individuales [`CustomsItem`] con descripciones comerciales, cantidades, pesos, valores y partidas arancelarias HTS (Harmonized Tariff Schedule).
   - Agregador [`CustomsDeclaration`] con clasificación de contenidos [`CustomsContentType`] (`Merchandise`, `Gift`, `Documents`, `Sample`, etc.), opciones ante imposibilidad de entrega [`NonDeliveryOption`] (`Return`, `Abandon`), identificadores fiscales (IOSS/VAT) y exenciones aduaneras AES/ITN (ej. `NOEEI 30.37(a)`).
   - Métodos utilitarios de cálculo automático: `total_declared_value()` y `total_weight_lbs()`.

3. **Ampliación de Pruebas Unitarias y de Integración:**
   - Pruebas unitarias de serialización/deserialización, constructores y agregaciones en `payments.rs` y `customs.rs`.
   - Integración en `tests/integration_tests.rs` validando rechazo preventivo ante cuentas vacías o montos de débito menores o iguales a cero.
   - Suite total expandida a **44 pruebas automáticas (40 unitarias + 4 de integración) y 1 doctest**, todas superadas al 100%.

4. **Calidad de Código y Documentación:**
   - Verificación de formato (`cargo fmt --check`) y análisis estricto de linter (`cargo clippy --all-targets --all-features -- -D warnings` con 0 advertencias).
   - Actualización del Manual Técnico `docs/MANUAL_TECNICO.md` y bitácora histórica.
   - Generación de commit bajo estándar Conventional Commits (`feat(sdk): add Payments v3 (EPS) and Customs v3 international declarations`).

---

## Interacción #13

### Solicitud del Usuario
```text
Continuar con las siguientes fases del desarrollo segun el plan
```

### Respuesta del Agente
Se completó la siguiente fase del plan de desarrollo, agregando capacidades de impresión física sin hardware mediante Label Broker, una suite de pruebas HTTP determinística con simulador de servidor y renovación integral de la documentación del proyecto:

1. **Soporte Oficial de USPS Label Broker v3 (`src/services/labels.rs`):**
   - Implementación de [`LabelBrokerRequest`] y [`LabelBrokerResponse`] para la emisión de códigos QR y códigos de Label Broker (`POST /labels/v3/label-broker`).
   - Permite a los remitentes presentar un código QR en cualquier mostrador postal de USPS para que el empleado imprima la etiqueta sin requerir impresora personal o térmica.
   - Implementación del método `LabelsService::get_label_data` (`GET /labels/v3/label/{labelId}`) para la recuperación posterior de imágenes o metadatos de etiquetas emitidas.
   - Re-exportación completa de los tipos y métodos en `src/services/mod.rs` y `src/lib.rs`.

2. **Suite de Pruebas con Servidor HTTP Simulado (`tests/mock_server_tests.rs`):**
   - Incorporación de `wiremock` (`0.6.5`) en `dev-dependencies` para pruebas sin red externa real ni secretos productivos.
   - Test de negociación y almacenamiento en caché de token Bearer OAuth 2.0: valida que múltiples llamadas consecutivas utilicen el token almacenado en `RwLock` sin emitir peticiones redundantes.
   - Test de resiliencia ante `HTTP 429 Too Many Requests`: valida el reintento automático transparente gobernado por `RetryPolicy` ante respuestas de saturación de cuota.
   - Test de mapeo tipado de errores de API: valida que errores HTTP estructurados devueltos por USPS se mapeen adecuadamente a variantes `UspsError::Api` con `UspsApiErrorResponse`.

3. **Renovación Integral de la Documentación (`README.md`):**
   - Incorporación de insignias oficiales (Crates.io, Docs.rs, Licencia dual MIT/Apache 2.0, CI Build Status).
   - Tabla exhaustiva de cobertura de todos los servicios v3 de USPS implementados con sus endpoints y módulos correspondientes.
   - Guía clara de instalación, configuración por variables de entorno y ejemplos reproducibles de uso.
   - Documentación de pruebas unitarias, de integración y con `wiremock`.

4. **Calidad de Código y Métricas de Calidad:**
   - Verificación de formato con `cargo fmt --check` (100% compliant).
   - Análisis estático estricto con `cargo clippy --all-targets --all-features -- -D warnings` (0 advertencias).
   - Ejecución de la suite completa de pruebas: **49 pruebas automáticas (42 unitarias + 4 de integración + 3 de wiremock) y 1 doctest**, todas superadas al 100%.
   - Actualización sincrónica del manual técnico `docs/MANUAL_TECNICO.md`.
   - Generación de commit bajo estándar Conventional Commits (`feat(sdk): add Label Broker support, wiremock testing suite, and enhanced README`).

---

## Interacción #14

### Solicitud del Usuario
```text
Continuar con las siguientes fases del desarrollo segun el plan
```

### Respuesta del Agente
Se continuó con la siguiente fase integral de desarrollo del SDK, agregando capacidades de cotización de servicios adicionales (Extra Services), soporte para la Prueba Electrónica de Entrega (ePOD) con firma, creación del changelog estandarizado del proyecto y expansión de las suites de prueba:

1. **Cotización de Servicios Especiales y Adicionales (`src/services/prices.rs` - `Prices v3`):**
   - Implementación del método `PricesService::calculate_extra_services` (`POST /prices/v3/extra-services`).
   - Modelado de opciones [`ExtraServiceType`]: seguro (`Insurance`), acuse de recibo (`ReturnReceipt`), confirmación de firma (`SignatureConfirmation`), entrega a adultos (`AdultSignatureRequired`), entrega restringida (`RestrictedDelivery`) y correo registrado (`RegisteredMail`).
   - Soporte de solicitud [`ExtraServicesRateRequest`] con validación preventiva de precio base y peso no negativos, y cálculo de cobertura según valor declarado.
   - Modelado de respuesta [`ExtraServicesRateResponse`] y desglose por ítem [`ExtraServiceRateItem`].

2. **Prueba Electrónica de Entrega ePOD (`src/services/tracking.rs` - `Tracking v3`):**
   - Implementación del método `TrackingService::request_proof_of_delivery` (`POST /tracking/v3/proof-of-delivery`).
   - Solicitud estructurada [`ProofOfDeliveryRequest`] con selección de formato [`ProofOfDeliveryFormat`] (`Letter` o `Signature`) y validaciones preventivas de número de rastreo, email con `@` y nombres del solicitante.
   - Modelado de respuesta [`ProofOfDeliveryResponse`] con confirmación de ID de solicitud y estado de procesamiento.

3. **Registro Estandarizado de Versiones (`CHANGELOG.md`):**
   - Creación del archivo `CHANGELOG.md` en la raíz del repositorio siguiendo la especificación [Keep a Changelog](https://keepachangelog.com/) y Semantic Versioning.
   - Documentación exhaustiva de las versiones `0.1.0` y `0.2.0`.

4. **Ampliación de Pruebas y Validación:**
   - Nuevas pruebas unitarias en `prices.rs` y `tracking.rs`.
   - Nuevo test con servidor HTTP simulado en `tests/mock_server_tests.rs`: `proof_of_delivery_and_extra_services_mock_flow`.
   - Nuevas validaciones preventivas de entrada en `tests/integration_tests.rs`.
   - Actualización sincrónica del manual técnico `docs/MANUAL_TECNICO.md` y `README.md`.
   - Ejecución de las pruebas y verificación de calidad con cero advertencias.
   - Generación de commit bajo estándar Conventional Commits (`feat(sdk): add Extra Services pricing, electronic proof of delivery, and CHANGELOG`).

---

## Interacción #15

### Solicitud del Usuario
```text
Continuar con las siguientes fases del desarrollo segun el plan
```

### Respuesta del Agente
Se completó la siguiente fase del plan de desarrollo, dotando al SDK de soporte para múltiples motores TLS configurables mediante Cargo Feature Flags e integrando su documentación en el manual y changelog:

1. **Soporte Multi-Motor TLS (`Cargo.toml` - Feature Flags):**
   - Configuración de la característica `rustls-tls` activada por defecto: implementación moderna 100% Rust basada en Rustls, libre de dependencias de compilación en C/OpenSSL, idónea para contenedores Alpine/Scratch y compilación cruzada `musl`.
   - Configuración de la característica opcional `native-tls`: permite enlazar con la pila criptográfica y almacén de certificados nativos del sistema operativo (`default-features = false, features = ["native-tls"]`).
   - Configuración desacoplada en `reqwest` asegurando compatibilidad mutua.

2. **Verificación y Pruebas Cruzadas:**
   - Compilación y ejecución exitosa de pruebas con el motor por defecto `rustls-tls` (55 pruebas aprobadas).
   - Compilación y ejecución exitosa de pruebas con `native-tls` (`cargo test --all-targets --no-default-features --features native-tls`).
   - Compilación y ejecución exhaustiva con `--all-targets --all-features` superada al 100%.
   - Análisis de linter estricto `cargo clippy --all-targets --all-features -- -D warnings` con 0 advertencias.
   - Formato validado al 100% con `cargo fmt --check`.

3. **Documentación Técnica y Bitácora:**
   - Actualización de [`README.md`] con guía de selección de TLS para `Cargo.toml`.
   - Actualización de [`CHANGELOG.md`] detallando las *feature flags* criptográficas.
   - Incorporación de la sección 5.3 en [`docs/MANUAL_TECNICO.md`].
   - Generación de commit bajo estándar Conventional Commits (`feat(core): add configurable TLS engine feature flags (rustls-tls and native-tls)`).

---

## Interacción #16

### Solicitud del Usuario
```text
si, etiquerla, subirla a github y publicarla
```

### Respuesta del Agente
Se llevó a cabo la consolidación, etiquetado formal, sincronización con GitHub y publicación en el registro oficial `crates.io` de la versión `0.2.0` del SDK:

1. **Ajuste de Metadatos y Enlaces Oficiales (`Cargo.toml` y `README.md`):**
   - Actualización de las URLs de repositorio y homepage a `https://github.com/Cesarius1970/usps_v3_api`.
   - Ajuste de insignias de CI apuntando al flujo de trabajo del repositorio remoto del usuario.

2. **Control de Versiones y Etiquetado en Git:**
   - Generación del commit de cierre del release: `chore(release): prepare and finalize release v0.2.0`.
   - Creación del tag anotado oficial: `v0.2.0`.
   - Publicación de ramas y etiquetas en GitHub (`git push origin main --tags`).

3. **Publicación en crates.io:**
   - Empaquetado y verificación de integridad final con `cargo publish`.
   - Paquete publicado exitosamente con la versión `0.2.0` para toda la comunidad de Rust.

4. **Documentación y Cierre de Ciclo:**
   - Actualización sincrónica de `docs/MANUAL_TECNICO.md` y de la bitácora histórica.

---

## Interacción #17

### Solicitud del Usuario
```text
Analiza este proyecto y dame los detalles completos. Recomienda refactorizaciones necesarias y alguna nueva funcionalidad
-> Documenta profundamente la implementacion de los 3 Refactorizaciones recomendadas. No implementes nada hasta recibir la aprobacion del plan
-> [Approved] plan_refactorizaciones.md
```

### Respuesta del Agente
Se llevó a cabo un análisis integral de arquitectura de la base de código tras la versión `0.2.0`, identificando oportunidades de mejora estructural y formulando un plan de refactorización (`plan_refactorizaciones.md`) que fue aprobado por el usuario. Posteriormente, se implementaron con éxito las 3 refactorizaciones planificadas:

1. **R1: Unificación del Despacho HTTP y Reintentos (`src/core/client.rs`):**
   - Creación del método privado centralizado `execute_with_retry(method, endpoint, query, body, needs_body)`.
   - Consolidación del bucle de reintentos exponencial con *jitter*, resolución automática de tokens OAuth 2.0, inyección de timeout por petición y manejo de códigos HTTP y errores de API.
   - Refactorización de `get_with_query`, `post_json` y `delete` a métodos concisos de 1-3 líneas, eliminando ~95 líneas de lógica duplicada.

2. **R2: Modularización de Servicios Extensos (`src/services/prices/` y `src/services/labels/`):**
   - División de `src/services/prices.rs` en el módulo `src/services/prices/` compuesto por `types.rs` (estructuras, DTOs y enums de tarifas) y `mod.rs` (lógica del servicio `PricesService` y pruebas unitarias).
   - División de `src/services/labels.rs` en el módulo `src/services/labels/` compuesto por `types.rs` (DTOs, imágenes, metadatos y aduanas) y `mod.rs` (lógica de `LabelsService` y pruebas unitarias).
   - Preservación del 100% de retrocompatibilidad y exportaciones públicas sin rupturas de API (`breaking changes`).

3. **R3: Formalización Algorítmica de Jitter en Reintentos (`src/core/retry.rs`):**
   - Reemplazo del multiplicador estático pseudoaleatorio por un algoritmo formal de *Equal Jitter* en el intervalo `[max/2, max]`.
   - Implementación de un generador pseudoaleatorio *SplitMix64* nativo con semilla atómica `AtomicU64`, garantizando seguridad multi-hilo (*thread-safe*) sin introducir dependencias de crates de números aleatorios pesados.
   - Actualización y ampliación de las pruebas unitarias de `calculate_backoff`.

4. **Verificación y Control de Calidad:**
   - Formateo de código con `cargo fmt --check` sin discrepancias.
   - Análisis estático con `cargo clippy --all-targets --all-features -- -D warnings` aprobado con cero advertencias.
   - Batería de 55 pruebas automatizadas (47 unitarias + 4 de integración + 4 de wiremock) aprobadas al 100%.
   - Actualización de `CHANGELOG.md` (sección `[Unreleased]`), `docs/MANUAL_TECNICO.md` y de la presente bitácora.

