//! Traits for conversions between types.
//!
//! # Examples
//!
//! ```no_run
//! use tide::Request;
//! use tide::prelude::*;
//!
//! #[derive(Debug, Deserialize, Validate)]
//! struct Animal {
//!     #[validate(length(min = 4))]
//!     name: String,
//!     #[validate(range(max = 12))]
//!     legs: u8,
//! }
//!
//! #[async_std::main]
//! async fn main() -> tide::Result<()> {
//!     let mut app = tide::new();
//!     app.at("/orders/shoes").post(order_shoes);
//!     app.listen("127.0.0.1:8080").await?;
//!     Ok(())
//! }
//!
//! async fn order_shoes(mut req: Request<()>) -> tide::Result {
//!     let animal: Animal = req.body_json().await?;
//!     animal.validate().status(400)?;
//!     let msg = format!("Hello, {}! I've put in an order for {} shoes", animal.name, animal.legs);
//!     Ok(msg.into())
//! }
//! ```

#[doc(inline)]
pub use http_types::convert::*;

/// Validate a type.
pub use validator::Validate;

/// A list of validation errors.
pub use validator::ValidationErrors;

/// A single validation error.
pub use validator::ValidationError;

/// The kind of errors returned by validating.
pub use validator::ValidationErrorsKind;

#[cfg(test)]
mod test {
    pub use super::*;

    #[derive(Debug, Deserialize, Validate)]
    struct PersonInput {
        #[validate(length(min = 4))]
        name: String,
        #[validate(email)]
        email: String,
    }

    #[test]
    fn validate() -> crate::Result<()> {
        let cat = PersonInput {
            name: "chashu".into(),
            email: "cute@cat.cafe".into(),
        };
        cat.validate()?;
        Ok(())
    }

    #[test]
    fn parse_validate() -> crate::Result<()> {
        let input = r#"{ "name": "chashu", "email": "cute@cat.cafe" }"#;
        let cat: PersonInput = serde_json::from_str(input)?;
        cat.validate()?;
        Ok(())
    }
}
