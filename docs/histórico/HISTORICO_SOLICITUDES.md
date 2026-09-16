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
