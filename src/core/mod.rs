// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Primitivas centrales de configuración, autenticación OAuth 2.0, cliente HTTP y errores.

pub mod auth;
pub mod client;
pub mod config;
pub mod error;
pub mod retry;

pub use auth::{OAuthTokenResponse, TokenManager};
pub use client::{UspsClient, UspsClientBuilder};
pub use config::{USPS_CAT_BASE_URL, USPS_PROD_BASE_URL, UspsConfig, UspsEnvironment};
pub use error::{ApiErrorDetail, Result, UspsApiErrorResponse, UspsError, UspsErrorCode};
pub use retry::RetryPolicy;
