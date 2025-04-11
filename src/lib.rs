//! # `tagid` - Typed Unique Identifiers for Rust Entities
//!
//! `tagid` provides a robust system for defining and managing typed unique identifiers in Rust.
//! It supports multiple ID generation strategies (CUID, UUID, Snowflake) and integrates with
//! `serde`, `sqlx`, and other frameworks for seamless use in databases and serialization.
//!
//! ## Features
//!
//! - **Typed Identifiers**: Define entity-specific IDs with compile-time safety.
//! - **Multiple ID Generators**:
//!   - **CUID** (`with-cuid` feature) - Compact, collision-resistant IDs.
//!   - **ULID** (`with-ulid` feature) - Universally unique identifiers.
//!   - **UUID** (`with-uuid` feature) - Universally unique identifiers.
//!   - **Snowflake** (`with-snowflake` feature) - Time-based, distributed IDs.
//! - **Entity Labeling**: Labels provide contextual meaning to identifiers.
//! - **Serialization & Database Support**:
//!   - [`serde`] integration for JSON and binary serialization (`serde` feature).
//!   - [`sqlx`] integration for database storage (`sqlx` feature).
//! - **Custom Labeling**: Define custom label formats for entities.
//!
//! ## Installation
//!
//! Add `tagid` to your `Cargo.toml`, enabling the desired features:
//!
//! ```toml
//! [dependencies]
//! tagid = { version = "0.1", features = ["with-uuid", "sqlx"] }
//! ```
//!
//! ## Usage
//!
//! ### Defining an Entity with a Typed ID
//!
//! ```rust,ignore
//! use tagid::{Entity, Id, Label};
//!
//! #[derive(Label)]
//! struct User;
//!
//! impl Entity for User {
//!     type IdGen = tagid::UuidGenerator;
//! }
//!
//! let user_id = User::next_id();
//! println!("User ID: {}", user_id);
//! ```
//!
//! ### Labeling System
//!
//! Labels help associate an identifier with an entity, improving clarity in logs and databases:
//!
//! ```rust,ignore
//! use tagid::{Label, Labeling};
//! use tagid::snowflake::pretty::{IdPrettifier, BASE_23};
//! IdPrettifier::global_initialize(BASE_23.clone());
//!
//! #[derive(Label)]
//! struct Order;
//!
//! let order_labeler = Order::labeler();
//! let order_label = order_labeler.label();
//! assert_eq!(order_label, "Order");
//! ```
//!
//! ## Features Overview
//!
//! | Feature       | Description                                                   |
//! |--------------|---------------------------------------------------------------|
//! | `"derive"`   | Enables `#[derive(Label)]` macro for automatic labeling.      |
//! | `"with-cuid"`     | Enables the [`CuidGenerator`] for CUID-based IDs.             |
//! | `"with-ulid"`     | Enables the [`UlidGenerator`] for ULID-based IDs.             |
//! | `"with-uuid"`     | Enables the [`UuidGenerator`] for UUID-based IDs.             |
//! | `"with-snowflake"`| Enables the [`SnowflakeGenerator`] for distributed IDs.       |
//! | `"serde"`    | Enables serialization support via `serde`.                    |
//! | `"sqlx"`     | Enables database integration via `sqlx`.                      |
//! | `"envelope"` | Provides an envelope struct for wrapping IDs with metadata.   |
//!
//! ## Contributing
//!
//! Contributions are welcome! Open an issue or submit a pull request on [GitHub](https://github.com/your-repo/tagid).
//!
//! ## License
//!
//! This project is licensed under the MIT License.

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate tagid_derive;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use tagid_derive::*;

mod label;
mod labeling;

#[cfg(feature = "envelope")]
pub mod envelope;
mod id;

pub use id::{Entity, Id, IdGenerator};
pub use label::Label;
pub use labeling::{CustomLabeling, Labeling, MakeLabeling, NoLabeling};

#[cfg(feature = "with-cuid")]
pub use id::{CuidGenerator, CuidId};

#[cfg(feature = "with-ulid")]
pub use id::UlidGenerator;

#[cfg(feature = "with-uuid")]
pub use id::UuidGenerator;

#[cfg(feature = "with-snowflake")]
pub use id::{MachineNode, SnowflakeGenerator, snowflake};

// The default delimiter used to separate entity labels from their ID values.
pub const DELIMITER: &str = "::";
