// Copyright 2026 César A Vergara Buenaventura <cesarvergarab@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # usps_v3_api
//!
//! `usps_v3_api` es una biblioteca idiomática y asíncrona para interactuar
//! con la API REST v3 de USPS (United States Postal Service).
//!
//! ## Estructura del proyecto
//!
//! Esta biblioteca sigue las directrices oficiales del equipo de desarrollo de Rust,
//! los lineamientos de Karpathy (simplicidad, cambios quirúrgicos, pensar antes de codificar)
//! y las mejores prácticas de Apollo GraphQL para Rust.

/// Suma dos enteros de 64 bits sin signo (`u64`).
///
/// Función utilitaria base y de prueba para verificar la integridad del compilador y el crate.
///
/// # Parámetros
///
/// * `left`: Primer operando de tipo [`u64`].
/// * `right`: Segundo operando de tipo [`u64`].
///
/// # Retorno
///
/// Retorna la suma de `left` y `right`.
///
/// # Ejemplos
///
/// ```
/// use usps_v3_api::add;
///
/// let total = add(2, 3);
/// assert_eq!(total, 5);
/// ```
#[must_use]
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_should_return_sum_of_two_numbers() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
