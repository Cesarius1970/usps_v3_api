// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Cliente principal de conexión y orquestación para la API REST v3 de USPS.

use std::sync::Arc;
use std::time::Duration;

use reqwest::Client as HttpClient;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tracing::{debug, instrument};

use crate::addresses::AddressesService;
use crate::auth::TokenManager;
use crate::config::{UspsConfig, UspsEnvironment};
use crate::error::{Result, UspsError};
use crate::tracking::TrackingService;

/// Cliente principal asíncrono y thread-safe para interactuar con la API REST v3 de USPS.
///
/// Implementa clonado económico (`Arc` interno) para compartir entre múltiples tareas de Tokio.
#[derive(Debug, Clone)]
pub struct UspsClient {
    inner: Arc<UspsClientInner>,
}

#[derive(Debug)]
struct UspsClientInner {
    config: UspsConfig,
    http_client: HttpClient,
    token_manager: TokenManager,
}

impl UspsClient {
    /// Inicia la construcción de un [`UspsClient`] mediante el patrón Builder.
    #[must_use]
    pub fn builder() -> UspsClientBuilder {
        UspsClientBuilder::new()
    }

    /// Inicializa un nuevo cliente con la configuración dada.
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::Http`] si la inicialización del cliente HTTP de reqwest falla.
    pub fn new(config: UspsConfig) -> Result<Self> {
        let http_client = HttpClient::builder()
            .timeout(config.timeout)
            .build()
            .map_err(UspsError::Http)?;

        let token_manager = TokenManager::new(config.clone(), http_client.clone());

        Ok(Self {
            inner: Arc::new(UspsClientInner {
                config,
                http_client,
                token_manager,
            }),
        })
    }

    /// Retorna una referencia a la configuración activa del cliente.
    #[must_use]
    pub fn config(&self) -> &UspsConfig {
        &self.inner.config
    }

    /// Retorna el servicio de consulta y estandarización de direcciones (`Addresses v3`).
    #[must_use]
    pub fn addresses(&self) -> AddressesService {
        AddressesService::new(self.clone())
    }

    /// Retorna el servicio de seguimiento y rastreo de envíos (`Tracking v3`).
    #[must_use]
    pub fn tracking(&self) -> TrackingService {
        TrackingService::new(self.clone())
    }

    /// Retorna el servicio de consulta de precios y tarifas (`Prices v3`).
    #[must_use]
    pub fn prices(&self) -> crate::prices::PricesService {
        crate::prices::PricesService::new(self.clone())
    }

    /// Retorna el servicio de emisión y anulación de etiquetas postales (`Labels v3`).
    #[must_use]
    pub fn labels(&self) -> crate::labels::LabelsService {
        crate::labels::LabelsService::new(self.clone())
    }

    /// Retorna el servicio de programación y gestión de recolección de paquetes (`Pickup v3`).
    #[must_use]
    pub fn pickup(&self) -> crate::pickup::PickupService {
        crate::pickup::PickupService::new(self.clone())
    }

    /// Retorna el servicio de búsqueda y consulta de instalaciones y oficinas postales (`Locations v3`).
    #[must_use]
    pub fn locations(&self) -> crate::locations::LocationsService {
        crate::locations::LocationsService::new(self.clone())
    }

    /// Retorna el gestor interno de autenticación para consultar o forzar tokens.
    #[must_use]
    pub fn token_manager(&self) -> &TokenManager {
        &self.inner.token_manager
    }

    /// Ejecuta una solicitud HTTP GET autenticada con parámetros de consulta (`query string`).
    #[instrument(skip(self, query), name = "usps_get_with_query")]
    pub async fn get_with_query<Q, T>(&self, endpoint: &str, query: &Q) -> Result<T>
    where
        Q: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let url = format!("{}{endpoint}", self.inner.config.environment.base_url());
        let token = self.inner.token_manager.get_token().await?;

        debug!("Enviando GET autenticado a {}", url);
        let response = self
            .inner
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", "application/json")
            .query(query)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(UspsError::from_response(status, &body));
        }

        serde_json::from_str::<T>(&body).map_err(UspsError::Serialization)
    }

    /// Ejecuta una solicitud HTTP POST autenticada con cuerpo JSON.
    #[instrument(skip(self, body), name = "usps_post_json")]
    pub async fn post_json<B, T>(&self, endpoint: &str, body: &B) -> Result<T>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let url = format!("{}{endpoint}", self.inner.config.environment.base_url());
        let token = self.inner.token_manager.get_token().await?;

        debug!("Enviando POST JSON autenticado a {}", url);
        let response = self
            .inner
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Accept", "application/json")
            .json(body)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(UspsError::from_response(status, &body));
        }

        serde_json::from_str::<T>(&body).map_err(UspsError::Serialization)
    }
}

/// Constructor fluido (Builder) para [`UspsClient`].
#[derive(Debug, Default)]
pub struct UspsClientBuilder {
    client_id: Option<String>,
    client_secret: Option<String>,
    environment: UspsEnvironment,
    timeout: Option<Duration>,
}

impl UspsClientBuilder {
    /// Inicia un nuevo builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Define las credenciales de la aplicación USPS.
    #[must_use]
    pub fn credentials(
        mut self,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Self {
        self.client_id = Some(client_id.into());
        self.client_secret = Some(client_secret.into());
        self
    }

    /// Establece el entorno de ejecución (Sandbox, Producción o Custom).
    #[must_use]
    pub fn environment(mut self, env: UspsEnvironment) -> Self {
        self.environment = env;
        self
    }

    /// Establece el tiempo de espera por llamada HTTP.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Construye y valida la instancia de [`UspsClient`].
    ///
    /// # Errores
    ///
    /// Retorna [`UspsError::InvalidInput`] si faltan las credenciales.
    pub fn build(self) -> Result<UspsClient> {
        let client_id = self.client_id.ok_or_else(|| {
            UspsError::InvalidInput("Se requiere client_id para construir el cliente".to_string())
        })?;
        let client_secret = self.client_secret.ok_or_else(|| {
            UspsError::InvalidInput(
                "Se requiere client_secret para construir el cliente".to_string(),
            )
        })?;

        let mut config = UspsConfig::new(client_id, client_secret, self.environment)?;
        if let Some(timeout) = self.timeout {
            config = config.with_timeout(timeout);
        }

        UspsClient::new(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_should_create_client_with_valid_config() {
        let client = UspsClientBuilder::new()
            .credentials("test_id", "test_secret")
            .environment(UspsEnvironment::Sandbox)
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap();

        assert_eq!(client.config().client_id, "test_id");
        assert_eq!(client.config().timeout, Duration::from_secs(15));
        assert_eq!(client.config().environment, UspsEnvironment::Sandbox);
    }

    #[test]
    fn builder_should_fail_when_credentials_missing() {
        let err = UspsClientBuilder::new().build().unwrap_err();
        assert!(matches!(err, UspsError::InvalidInput(_)));
    }
}
