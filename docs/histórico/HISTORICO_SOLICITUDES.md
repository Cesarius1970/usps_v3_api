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
