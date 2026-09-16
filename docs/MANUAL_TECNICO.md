# Manual Técnico — usps_v3_api

## 1. Identificación y Copyright

- **Nombre del Proyecto:** `usps_v3_api`
- **Descripción:** Cliente SDK asíncrono, fuertemente tipado e idiomático para el ecosistema completo de APIs REST v3 de USPS (United States Postal Service) implementado en Rust.
- **Autor y Titular de Derechos de Autor:** César A Vergara Buenaventura (`cesarvergarab@gmail.com`).
- **Copyright:** Copyright (c) 2026 César A Vergara Buenaventura. Todos los derechos reservados.
- **Licenciamiento:** Doble licencia estándar del ecosistema oficial de Rust:
  - [MIT License](../LICENSE-MIT)
  - [Apache License 2.0](../LICENSE-APACHE)
- **Aviso de Atribución:** [NOTICE](../NOTICE)

---

## 2. Reglas de Desarrollo y Estándares Obligatorios

### 2.1. Regla de Ciclo de Vida y Versionado en Git

> [!IMPORTANT]
> **Regla Obligatoria:** Debe generarse un commit en el repositorio Git al finalizar cada fase o interacción con el agente o desarrollador.

#### Estándar para Nombres de Ramas
Se sigue el flujo estándar de Git estructurado por prefijos en minúsculas separados por slash (`/`):

- `main`: Rama principal de producción / releases estables.
- `develop`: Rama de integración de desarrollo continuo (cuando aplique).
- `feature/<nombre-descriptivo>`: Nuevas funcionalidades o módulos de la API USPS (ej. `feature/oauth2-client`, `feature/tracking-service`).
- `bugfix/<nombre-descriptivo>`: Correcciones de errores o bugs.
- `hotfix/<nombre-descriptivo>`: Correcciones urgentes y directas sobre releases.
- `docs/<nombre-descriptivo>`: Cambios exclusivamente dedicados a documentación técnica o bitácoras.
- `refactor/<nombre-descriptivo>`: Reestructuraciones de código o arquitectura de módulos.
- `chore/<nombre-descriptivo>`: Tareas de mantenimiento, dependencias o configuración de tooling.

#### Estándar para Mensajes de Commit (Conventional Commits)
Cada commit debe estructurarse obligatoriamente bajo el estándar **Conventional Commits v1.0.0**:

```text
<tipo>(<ámbito opcional>): <descripción imperativa en presente o infinitivo>

[cuerpo opcional detallando el motivo, contexto y cambios técnicos]

[pie opcional con referencias a issues o breaking changes]
```

Tipos permitidos:
- `feat`: Nueva característica para el usuario o consumidor de la librería.
- `fix`: Corrección de un fallo o error en el código.
- `docs`: Modificaciones en documentación (`README.md`, `MANUAL_TECNICO.md`, `HISTORICO_SOLICITUDES.md`, rustdoc).
- `style`: Cambios de formato o espaciado que no afectan el significado del código.
- `refactor`: Refactorización de código sin añadir funcionalidades ni corregir bugs.
- `perf`: Mejoras de rendimiento en algoritmos o alocaciones.
- `test`: Adición o corrección de pruebas unitarias, de integración o doctests.
- `chore`: Tareas auxiliares, actualización de `Cargo.toml`, `.gitignore`, scripts o configuración CI/CD.

---

### 2.2. Lineamientos de Calidad e Ingeniería

1. **Directrices de Karpathy (Karpathy Guidelines):**
   - **Pensar antes de codificar:** Declarar supuestos explícitos, aclarar ambigüedades, plantear compensaciones de diseño antes de ejecutar.
   - **Simplicidad primero:** Código mínimo necesario que resuelva el problema sin abstracciones prematuras ni sobreingeniería.
   - **Cambios quirúrgicos:** Modificar estrictamente lo necesario, conservando estilo existente y sin tocar código ajeno que funcione.
   - **Ejecución orientada a objetivos:** Metas con criterios de verificación medibles (`cargo test`, `cargo clippy`).

2. **Mejores Prácticas de Rust (Apollo Guidelines):**
   - Arquitectura modular en capas con separación nítida entre infraestructura (`core`) y servicios de dominio (`services`).
   - Priorizar referencias (`&str`, `&[T]`) sobre clones y traspasos de propiedad innecesarios.
   - Jerarquía de errores fuertemente tipada con `thiserror`.
   - Prohibido el uso de `unwrap()` o `expect()` fuera del alcance de tests (`#[cfg(test)]`).
   - Mantenimiento con cero advertencias: `cargo clippy --all-targets --all-features -- -D warnings`.

3. **Patrones Asíncronos con Tokio:**
   - Estricta no-bloqueabilidad del hilo asíncrono (uso de `spawn_blocking` para I/O bloqueante o CPU intensivo).
   - No retener cerrojos sincrónicos (`std::sync::Mutex`) a través de llamadas `.await`.
   - Concurrencia controlada mediante `tokio::sync::RwLock` con patrón *double-checked locking* para renovación segura de tokens.

---

## 3. Arquitectura y Componentes del Software

### 3.1. Estructura de Directorios Refactorizada en Capas
El proyecto aplica el patrón de diseño enterprise en capas (Layered / Domain-Driven Architecture), separando la infraestructura transversal de los servicios de negocio de USPS:

```text
usps_v3_api/
├── .github/
│   └── workflows/
│       └── ci.yml              # Pipeline de integración continua (CI) en GitHub Actions
├── Cargo.toml                  # Manifiesto y metadatos del paquete Rust
├── LICENSE-APACHE              # Licencia Apache 2.0
├── LICENSE-MIT                 # Licencia MIT
├── NOTICE                      # Atribución y copyright oficial
├── README.md                   # Resumen del proyecto y guía de inicio
├── docs/
│   ├── MANUAL_TECNICO.md       # Este manual técnico vivo
│   └── histórico/
│       └── HISTORICO_SOLICITUDES.md # Bitácora cronológica de interacciones
└── src/
    ├── lib.rs                  # Raíz del crate, re-exportaciones canónicas y doctests
    ├── core/                   # CAPA CENTRAL (Infraestructura y Transporte)
    │   ├── mod.rs              # Re-exportaciones públicas de la capa core
    │   ├── auth.rs             # Gestor OAuth 2.0 con auto-refresh y RwLock
    │   ├── client.rs           # Cliente central UspsClient y UspsClientBuilder
    │   ├── config.rs           # Configuración, entornos y saneamiento de secretos
    │   ├── error.rs            # Jerarquía de errores UspsError y deserialización API
    │   └── retry.rs            # Política de reintentos con backoff exponencial y jitter
    └── services/               # CAPA DE SERVICIOS (Dominios de Negocio USPS v3)
        ├── mod.rs              # Re-exportaciones públicas del catálogo de servicios
        ├── addresses.rs        # Módulo de Direcciones v3 (Addresses v3)
        ├── labels.rs           # Módulo de Etiquetas Postales v3 (Labels v3)
        ├── locations.rs        # Módulo de Ubicaciones e Instalaciones v3 (Locations v3)
        ├── manifests.rs        # Módulo de Manifiestos SCAN Form v3 (Manifests v3)
        ├── pickup.rs           # Módulo de Recolección de Paquetes v3 (Pickup v3)
        ├── prices.rs           # Módulo de Precios y Tarifas Nacionales/Internacionales (Prices v3)
        ├── tracking.rs         # Módulo de Seguimiento de Envíos v3 (Tracking v3)
        └── webhooks.rs         # Módulo de Suscripciones y Webhooks v3 (Subscriptions v3)
```

---

## 4. Detalle de Módulos, Patrones y Algoritmos

### 4.1. Capa Central (`src/core/`)

#### 4.1.1. Módulo de Errores (`src/core/error.rs`)
- **Propósito:** Proporcionar una jerarquía tipada que permita al consumidor inspeccionar la causa exacta de una falla sin conversiones de cadenas opacas.
- **Tipos clave:**
  - `UspsError`: Enum que agrupa errores de red (`reqwest::Error`), serialización (`serde_json::Error`), autenticación OAuth 2.0 (`UspsError::Auth`), datos de entrada inválidos (`UspsError::InvalidInput`) y errores de API (`UspsError::Api`).
  - `UspsApiErrorResponse`: Modela la carga JSON de respuesta de error oficial de USPS (códigos, descripciones y vectores de `ApiErrorDetail`).
- **Algoritmo `UspsError::from_response(status, body)`:**
  Evalúa el cuerpo HTTP retornado; si es un JSON estructurado, extrae los detalles técnicos y advertencias de USPS; si no lo es (ej. error 502 de gateway intermedio), realiza fallback seguro sin entrar en pánico.

#### 4.1.2. Módulo de Configuración (`src/core/config.rs`)
- **Propósito:** Gestionar credenciales, timeouts y selección de endpoints según el ambiente.
- **Ambientes soportados (`UspsEnvironment`):**
  - `Sandbox`: `https://api-cat.usps.com` (Entorno oficial de pruebas CAT de USPS).
  - `Production`: `https://api.usps.com` (Entorno de producción en vivo).
  - `Custom(String)`: Para proxies empresariales, balanceadores o servidores mock locales.
- **Patrón de Seguridad (Sanitización en Debug):**
  Se implementa `std::fmt::Debug` manualmente para `UspsConfig`, sustituyendo el campo sensible `client_secret` por `"[REDACTED]"`. Esto previene fugas accidentales de secretos en sistemas de telemetría y logs.

#### 4.1.3. Módulo de Autenticación (`src/core/auth.rs`)
- **Propósito:** Automatizar la obtención y el refresco transparente del Bearer Token OAuth 2.0 (`POST /oauth2/v3/token`).
- **Algoritmo de Concurrencia Segura:**
  1. Adquiere un bloqueo de lectura (`read().await`) sobre `cached_token` (`tokio::sync::RwLock`). Si el token existe y aún no ha alcanzado su margen de expiración (`is_valid()`), se retorna de inmediato sin bloquear a otras tareas concurrentes.
  2. Si el token está ausente o próximo a expirar (margen `EXPIRATION_BUFFER_SECS = 60s`), adquiere el bloqueo de escritura (`write().await`).
  3. Ejecuta el patrón *double-checked locking*: verifica si otra tarea concurrente ya renovó el token mientras se esperaba el bloqueo.
  4. Si continúa inválido, emite la llamada HTTP `POST /oauth2/v3/token` con `grant_type=client_credentials`, almacena el nuevo token con su timestamp de caducidad calculada y lo retorna.

#### 4.1.4. Módulo de Cliente Central (`src/core/client.rs`)
- **Propósito:** Punto único de orquestación, conexión HTTP y despacho de peticiones.
- **Diseño con Puntero Atómico (`Arc`):**
  `UspsClient` encapsula un `Arc<UspsClientInner>`, permitiendo su clonación a costo insignificante (incremento de puntero atómico) para distribuirlo entre múltiples hilos o tareas concurrentes de Tokio.
- **Patrón Builder (`UspsClientBuilder`):**
  Permite configuración fluida de credenciales, timeout, política de reintentos y entorno con validación previa de datos obligatorios.
- **Métodos HTTP Reutilizables:**
  - `get_with_query(endpoint, query)`: Despacho de peticiones GET autenticadas con bucle de reintentos y deserialización JSON.
  - `post_json(endpoint, body)`: Despacho de peticiones POST autenticadas con cuerpo JSON, bucle de reintentos y deserialización.
  - `delete(endpoint)`: Despacho de peticiones DELETE autenticadas con bucle de reintentos y soporte de respuestas vacías (204/200).
- **Servicios Integrados:**
  - `client.addresses()` -> `AddressesService`
  - `client.tracking()` -> `TrackingService`
  - `client.prices()` -> `PricesService`
  - `client.labels()` -> `LabelsService`
  - `client.locations()` -> `LocationsService`
  - `client.manifests()` -> `ManifestsService`
  - `client.pickup()` -> `PickupService`
  - `client.webhooks()` -> `WebhooksService`

#### 4.1.5. Módulo de Reintentos y Resiliencia (`src/core/retry.rs`)
- **Propósito:** Manejo automático y transparente de fallos transitorios de red y límites de velocidad de la API de USPS.
- **Estructura `RetryPolicy`:**
  - `max_retries`: Número máximo de intentos (por defecto 3).
  - `initial_backoff`: Demora base inicial (por defecto 200 ms).
  - `max_backoff`: Límite superior de espera (por defecto 5.000 ms).
  - `backoff_factor`: Factor multiplicador exponencial (por defecto 2.0).
  - `jitter`: Variación aleatoria pseudo-determinística para mitigar el problema de *thundering herd* hacia la infraestructura de USPS.
- **Algoritmo de Detección de Códigos Reintentables:**
  Evalúa el código de estado HTTP y reintenta ante:
  - `429 Too Many Requests`: Respeto a ventanas de límite de cuota o rate limiting.
  - `500 Internal Server Error`: Fallos transitorios de los servidores de USPS.
  - `502 Bad Gateway`, `503 Service Unavailable`, `504 Gateway Timeout`: Inestabilidad temporal de proxies y balanceadores intermedios.

---

### 4.2. Capa de Servicios de Negocio (`src/services/`)

#### 4.2.1. Módulo de Direcciones (`src/services/addresses.rs`)
- **Propósito:** Normalizar y validar direcciones postales en Estados Unidos según la base de datos de USPS.
- **Servicios:**
  - `standardize(&AddressStandardizationRequest) -> Result<AddressResponse>`: Consulta `GET /addresses/v3/address`. Retorna la dirección en formato estándar de USPS, códigos ZIP+4, confirmación DPV (`DPVConfirmation`), indicación de entrega comercial (`DPVCMRA`), indicador de negocio y vacancia.
  - `lookup_zip_code(&ZipCodeLookupRequest) -> Result<AddressResponse>`: Consulta `GET /addresses/v3/zipcode` para resolver el código postal correspondiente a una dirección.
  - `lookup_city_state(zip_code) -> Result<CityStateResponse>`: Consulta `GET /addresses/v3/city-state` validando previamente que el código postal conste de 5 dígitos numéricos.

#### 4.2.2. Módulo de Seguimiento (`src/services/tracking.rs`)
- **Propósito:** Seguimiento de envíos postales en tiempo real.
- **Servicios:**
  - `track(tracking_number) -> Result<TrackingResponse>`: Consulta detallada de la línea de tiempo completa del paquete (`TrackingExpand::Detail`).
  - `track_with_expand(tracking_number, TrackingExpand) -> Result<TrackingResponse>`: Permite seleccionar entre historial detallado (`TrackingExpand::Detail`) o resumen del estado actual (`TrackingExpand::Summary`).

#### 4.2.3. Módulo de Precios y Tarifas (`src/services/prices.rs`)
- **Propósito:** Cálculo y cotización de tarifas de franqueo para envíos nacionales e internacionales.
- **Servicios:**
  - `calculate_domestic_rates(&DomesticRateRequest) -> Result<DomesticRateResponse>`: Despacha `POST /prices/v3/base-rates/search`. Soporta `MailClass` (*Priority Mail, USPS Ground Advantage, Priority Mail Express, etc.*) y `ProcessingCategory`.
  - `calculate_international_rates(&InternationalRateRequest) -> Result<InternationalRateResponse>`: Despacha `POST /prices/v3/international-base-rates/search`. Valida el código de país de 2 caracteres ISO (ej. `CA`, `GB`, `MX`, `ES`) y soporta `InternationalMailClass` (*Global Express Guaranteed, Priority Mail International, First-Class Package International, etc.*).

#### 4.2.4. Módulo de Etiquetas Postales (`src/services/labels.rs`)
- **Propósito:** Generación, emisión y cancelación de etiquetas postales con código de barras USPS.
- **Servicios:**
  - `create_label(&CreateLabelRequest) -> Result<CreateLabelResponse>`: Despacha `POST /labels/v3/label`. Soporta formatos gráficos `LabelImageType` (*PDF, PNG, TIFF, SVG*) y entrega de imagen Base64 o URL de descarga directa.
  - `cancel_label(label_id) -> Result<CancelLabelResponse>`: Despacha `DELETE /labels/v3/label/{labelId}` para anular etiquetas y tramitar reembolsos de franqueo utilizando el cliente HTTP centralizado con reintentos.

#### 4.2.5. Módulo de Recolección de Paquetes (`src/services/pickup.rs`)
- **Propósito:** Gestión integral de recolección de paquetes por el cartero a domicilio (`Carrier Pickup`).
- **Servicios:**
  - `check_availability(zip_code) -> Result<PickupAvailabilityResponse>`: Consulta `GET /pickup/v3/carrier-pickup/availability?ZIPCode={zip_code}`.
  - `schedule(&SchedulePickupRequest) -> Result<SchedulePickupResponse>`: Despacha `POST /pickup/v3/carrier-pickup`. Permite designar ubicación (`PackageLocation`: `FrontDoor`, `BackDoor`, `InMailbox`, etc.) y conteo de paquetes por clase (`PickupPackageCount`).
  - `cancel(confirmation_number) -> Result<CancelPickupResponse>`: Despacha `DELETE /pickup/v3/carrier-pickup/{confirmationNumber}` utilizando el cliente HTTP centralizado con reintentos.

#### 4.2.6. Módulo de Ubicaciones e Instalaciones (`src/services/locations.rs`)
- **Propósito:** Búsqueda y consulta de instalaciones físicas de USPS, buzones de depósito y quioscos automatizados.
- **Servicios:**
  - `search(&LocationSearchRequest) -> Result<LocationSearchResponse>`: Consulta `GET /locations/v3/location`. Permite búsqueda por código postal (`from_zip_code`) o coordenadas geográficas (`from_coordinates`), radio en millas y filtrado por servicios (`LocationServiceType`: `PassportAppointments`, `PoBoxes`, `RetailServices`, `CollectionBox`, `SelfServiceKiosks`, etc.).
  - `get_details(location_id) -> Result<LocationFacility>`: Consulta `GET /locations/v3/location/{locationId}` para obtener datos de contacto, coordenadas precisas, servicios habilitados y horarios semanales detallados (`DailyHours`).

#### 4.2.7. Módulo de Manifiestos y Formularios SCAN Form (`src/services/manifests.rs`)
- **Propósito:** Consolidación de múltiples envíos postales individuales en una única hoja de manifiesto oficial de entrega (**USPS SCAN Form - PS Form 5630**) con un único código de barras maestro de aceptación.
- **Servicios:**
  - `create_manifest(&CreateManifestRequest) -> Result<CreateManifestResponse>`: Despacha `POST /manifests/v3/manifest`. Requiere la dirección de origen (`LabelPartyAddress`), código postal de 5 dígitos de la oficina de ingreso (`entryFacilityZIPCode`) y la lista de `label_ids` a consolidar.
  - `get_manifest(manifest_id) -> Result<CreateManifestResponse>`: Consulta `GET /manifests/v3/manifest/{manifestId}` para recuperar el manifiesto emitido previamente.

#### 4.2.8. Módulo de Suscripciones y Webhooks (`src/services/webhooks.rs`)
- **Propósito:** Registro, administración y baja de callbacks HTTP/HTTPS para recibir notificaciones asíncronas en tiempo real sobre eventos de paquetes y entrega de USPS.
- **Servicios:**
  - `subscribe(&CreateSubscriptionRequest) -> Result<SubscriptionResponse>`: Despacha `POST /subscriptions/v3/subscription`. Valida que la URL receptora sea un endpoint HTTP/HTTPS válido y soporta eventos (`SubscriptionEventType`: `TrackingEvents`, `PackageDelivered`, `DeliveryException`, `ReturnToSender`) y clave secreta opcional para verificación de firma HMAC.
  - `get_subscription(subscription_id) -> Result<SubscriptionResponse>`: Consulta `GET /subscriptions/v3/subscription/{subscriptionId}`.
  - `delete_subscription(subscription_id) -> Result<DeleteSubscriptionResponse>`: Despacha `DELETE /subscriptions/v3/subscription/{subscriptionId}` utilizando el cliente HTTP centralizado con reintentos.

---

## 5. Guía de Compilación, Pruebas y Calidad

El proyecto se valida de extremo a extremo mediante el conjunto de herramientas oficiales de Rust y CI automatizado:

```bash
# Compilar todo el SDK
cargo build

# Ejecutar las 31 pruebas unitarias y doctests interactivos
cargo test

# Verificar cumplimiento de formato oficial con rustfmt
cargo fmt --check

# Ejecutar el linter estricto de Rust sin advertencias permitidas
cargo clippy --all-targets --all-features -- -D warnings

# Generar documentación local en HTML
cargo doc --no-deps --open
```

### 5.1. Pipeline de Integración Continua (CI/CD)
El proyecto incluye un flujo de trabajo de GitHub Actions en `.github/workflows/ci.yml` ejecutado en cada `push` y `pull_request` sobre la rama `main`:
- **Verificación de Formato:** `cargo fmt --all -- --check`
- **Análisis Estático:** `cargo clippy --all-targets --all-features -- -D warnings`
- **Generación de Documentación:** `cargo doc --no-deps --all-features`
- **Suite de Pruebas:** `cargo test --all-targets --all-features`
- **Caché Eficiente:** Integración con `Swatinem/rust-cache@v2` para tiempos de compilación mínimos en CI.

---

## 6. Mantenimiento Continuo

Cada vez que se extienda el SDK:
1. Añadir los contratos de datos y endpoints en el submódulo correspondiente dentro de `src/services/` (o en `src/core/` si es infraestructura).
2. Exponer el servicio en `src/services/mod.rs` y re-exportar en `src/lib.rs`.
3. Escribir pruebas unitarias de serialización/deserialización y constructores en `tests`.
4. Actualizar la sección 4 de este documento con las firmas de API y algoritmos.
5. Generar el commit correspondiente en Git bajo el estándar **Conventional Commits v1.0.0**.

---

## 7. Historial de Versiones

### Versión 0.1.0 (Lanzamiento Inicial)
- **Fecha:** 2026-09-15
- **Git Tag:** `v0.1.0`
- **Registro en crates.io:** `usps_v3_api = "0.1.0"`
- **Alcance Completo:**
  - Capa de infraestructura transversal (`core`): OAuth 2.0 Client Credentials con auto-refresh seguro mediante `RwLock`, `UspsClient`, `UspsConfig` (con sanitización de secretos en logs) y jerarquía `UspsError`.
  - Capa de servicios (`services`):
    - `AddressesService` (`addresses/v3`): Estandarización de direcciones, validación DPV, búsqueda de ZIP codes y resolución de ciudad/estado.
    - `TrackingService` (`tracking/v3`): Rastreo en tiempo real, eventos históricos de escaneo y fechas estimadas de entrega.
    - `PricesService` (`prices/v3`): Tarifas nacionales base y dimensionales, y tarifas internacionales con validación ISO.
    - `LabelsService` (`labels/v3`): Emisión de etiquetas oficiales con código de barras (PDF, PNG, TIFF, SVG, Base64) y cancelación de etiquetas.
    - `PickupService` (`pickup/v3`): Disponibilidad de recolección de cartero, programación a domicilio y cancelación.
    - `LocationsService` (`locations/v3`): Búsqueda de oficinas postales y buzones por código postal o geocordenadas, horarios y catálogo de servicios.
  - Batería de 25 pruebas unitarias y doctests interactivos con 100% de aprobación y 0 advertencias de Clippy.

### Fase Actual (Camino hacia v0.2.0)
- **Mejoras de Infraestructura y Resiliencia:**
  - Incorporación de `RetryPolicy` con backoff exponencial y jitter aleatorio configurable en `UspsConfig`.
  - Soporte transversal de reintentos para peticiones HTTP GET, POST y DELETE en `UspsClient`.
  - Método unificado `client.delete()` que reutiliza el pool de conexiones y timeouts configurados.
- **Nuevos Servicios USPS v3:**
  - `ManifestsService` (`manifests/v3`): Emisión y consulta de formularios SCAN Form (PS Form 5630) con código maestro.
  - `WebhooksService` (`subscriptions/v3`): Gestión de suscripciones webhook para eventos de rastreo y entrega en tiempo real.
- **Control de Calidad & CI/CD:**
  - Automatización con GitHub Actions (`.github/workflows/ci.yml`).
  - Cobertura incrementada a 31 pruebas unitarias y doctests con 0 errores y 0 advertencias.
