# Manual Técnico — usps_v3_api

## 1. Identificación y Copyright

- **Nombre del Proyecto:** `usps_v3_api`
- **Descripción:** Cliente asíncrono e idiomático para la API REST v3 de USPS (United States Postal Service) implementado en Rust.
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
   - **Simplicidad primero:** Código mínimo necesario que resuelva el problema sin abstracciones prematuras ni características especulativas.
   - **Cambios quirúrgicos:** Modificar estrictamente lo necesario, conservando estilo existente y sin tocar código ajeno que funcione.
   - **Ejecución orientada a objetivos:** Metas con criterios de verificación medibles (`cargo test`, `cargo clippy`).

2. **Mejores Prácticas de Rust (Apollo Guidelines):**
   - Priorizar referencias (`&str`, `&[T]`) sobre clones y traspasos de propiedad innecesarios.
   - Tipos de error claros con `thiserror` para la librería.
   - Prohibido el uso de `unwrap()` o `expect()` fuera del alcance de tests (`#[cfg(test)]`).
   - Mantenimiento con cero advertencias: `cargo clippy --all-targets -- -D warnings`.

3. **Patrones Asíncronos con Tokio:**
   - Estricta no-bloqueabilidad del hilo asíncrono (uso de `spawn_blocking` para I/O bloqueante o CPU intensivo).
   - No retener cerrojos sincrónicos (`std::sync::Mutex`) a través de llamadas `.await`.
   - Concurrencia controlada con semáforos, canales acotados y orquestación con `JoinSet` / `select!`.

---

## 3. Arquitectura y Componentes del Software

### 3.1. Estructura de Directorios
```text
usps_v3_api/
├── Cargo.toml                  # Manifiesto y metadatos del paquete Rust
├── LICENSE-APACHE              # Licencia Apache 2.0
├── LICENSE-MIT                 # Licencia MIT
├── NOTICE                      # Atribución y copyright oficial
├── README.md                   # Resumen del proyecto y guía de inicio
├── docs/
│   ├── MANUAL_TECNICO.md       # Este documento técnico vivo
│   └── histórico/
│       └── HISTORICO_SOLICITUDES.md # Bitácora cronológica de interacciones
└── src/
    └── lib.rs                  # Raíz de la biblioteca y utilidades base
```

### 3.2. Metadatos de `Cargo.toml`
El manifiesto del crate cumple rigurosamente con los campos exigidos y recomendados por el equipo oficial de desarrollo de Rust:

- `name`: Identificador único en crates.io (`usps_v3_api`).
- `version`: `0.1.0` siguiendo SemVer.
- `edition`: `"2024"`.
- `rust-version`: `"1.85.0"`.
- `authors`: Atribución a César A Vergara Buenaventura.
- `license`: Expresión SPDX válida `"MIT OR Apache-2.0"`.
- `description`, `readme`, `repository`, `homepage`, `documentation`, `keywords`, `categories`.

### 3.3. Detalle de Módulos y Algoritmos Actuales

#### Módulo Raíz: `src/lib.rs`
- **Encabezado legal:** Incluye aviso de copyright y compatibilidad de licencias Apache 2.0 y MIT.
- **Documentación a nivel de crate (`//!`):** Proporciona contexto y marco metodológico a `rustdoc`.
- **Función / Algoritmo `add(left: u64, right: u64) -> u64`:**
  - **Propósito:** Función aritmética básica de validación de compilación, pipeline y sanidad de tipos de 64 bits.
  - **Anotación `#[must_use]`:** Garantiza advertencias del compilador si el resultado no es consumido.
  - **Doctest:** Cuenta con prueba funcional integrada en la documentación para verificar ejemplos vivos con `cargo test`.
  - **Pruebas unitarias:** Módulo `tests` con verificación de casos de prueba (`add_should_return_sum_of_two_numbers`).

---

## 4. Guía de Compilación, Pruebas y Calidad

Para compilar el proyecto y garantizar que cumple con todos los estándares:

```bash
# Compilar en modo desarrollo
cargo build

# Ejecutar pruebas unitarias y pruebas de documentación
cargo test

# Ejecutar linter estricto de Rust sin advertencias permitidas
cargo clippy --all-targets --all-features -- -D warnings

# Generar documentación técnica en formato HTML
cargo doc --no-deps --open
```

---

## 5. Mantenimiento y Actualización de este Manual

Este documento es un manual técnico vivo. Conforme se incorporen clientes HTTP, autenticación OAuth2 con el portal de USPS Developer v3, endpoints de direcciones, tracking y tarifas, este documento debe actualizarse registrando:
1. Diagramas de arquitectura y flujo de autenticación/llamadas.
2. Contratos de tipos y modelos de datos (DTOs).
3. Manejo de códigos de error HTTP y reintentos.
