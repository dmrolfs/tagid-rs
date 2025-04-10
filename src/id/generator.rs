//! ID Generation Module
//!
//! This module defines the `IdGenerator` trait and provides implementations for generating unique
//! identifiers using different strategies, including CUID and UUID. The choice of ID generator
//! is controlled through feature flags (`cuid` and `uuid`).
//!
//! ## Feature Flags
//! - **`cuid`**: Enables the `CuidGenerator` which generates unique CUIDs.
//! - **`uuid`**: Enables the `UuidGenerator` which generates random UUIDs (v4).

/// A trait for generating unique identifiers.
///
/// Implementations of this trait define how new unique IDs are created.
/// The associated type `IdType` represents the type of the generated ID.
///
/// # Example
/// ```rust
/// use tagid::IdGenerator;
///
/// struct MyIdGenerator;
///
/// impl IdGenerator for MyIdGenerator {
///     type IdType = u64;
///
///     fn next_id_rep() -> Self::IdType {
///         42 // Example static ID (replace with a real generator)
///     }
/// }
/// ```
pub trait IdGenerator {
    /// The type of the generated identifier.
    type IdType: Send;

    /// Generates and returns the next unique identifier.
    fn next_id_rep() -> Self::IdType;
}

#[cfg(feature = "cuid")]
pub use self::cuid::{CuidGenerator, CuidId};

#[cfg(feature = "uuid")]
pub use self::uuid::UuidGenerator;

#[cfg(feature = "ulid")]
pub use self::ulid::UlidGenerator;

#[cfg(feature = "cuid")]
mod cuid {
    use super::*;
    use crate::Id;

    /// A type alias for a CUID-based identifier wrapped in the `Id` struct.
    ///
    /// This is useful when IDs are represented as `Id<T, String>`, where `T` is
    /// the entity type and `String` is the CUID.
    #[allow(dead_code)]
    pub type CuidId<T> = Id<T, String>;

    /// A generator for creating CUID-based unique identifiers.
    ///
    /// This implementation uses the [`cuid2`](https://docs.rs/cuid2) crate to generate collision-resistant
    /// unique IDs optimized for distributed systems.
    ///
    /// # Example
    /// ```
    /// use tagid::{IdGenerator, CuidGenerator};
    ///
    /// let new_id = CuidGenerator::next_id_rep();
    /// println!("Generated CUID: {}", new_id);
    /// ```
    pub struct CuidGenerator;

    impl IdGenerator for CuidGenerator {
        type IdType = String;

        /// Generates a new CUID.
        ///
        /// Uses `cuid2::create_id()` to generate a compact, unique identifier.
        ///
        /// # Returns
        /// * A unique `String` identifier in CUID format.
        fn next_id_rep() -> Self::IdType {
            ::cuid2::create_id()
        }
    }
}

#[cfg(feature = "uuid")]
mod uuid {
    use super::*;

    /// A generator for creating UUID-based unique identifiers.
    ///
    /// This implementation generates random version 4 UUIDs (UUID v4) using the
    /// [`uuid`](https://docs.rs/uuid) crate.
    ///
    /// # Example
    /// ```rust
    /// use tagid::{IdGenerator, UuidGenerator};
    ///
    /// let new_id = UuidGenerator::next_id_rep();
    /// println!("Generated UUID: {}", new_id);
    /// ```
    pub struct UuidGenerator;

    impl IdGenerator for UuidGenerator {
        type IdType = ::uuid::Uuid;

        /// Generates a new UUID v4.
        ///
        /// Uses `uuid::Uuid::new_v4()` to generate a random UUID.
        ///
        /// # Returns
        /// * A unique `Uuid` identifier.
        fn next_id_rep() -> Self::IdType {
            ::uuid::Uuid::new_v4()
        }
    }
}

#[cfg(feature = "ulid")]
mod ulid {
    use super::*;

    pub struct UlidGenerator;

    impl IdGenerator for UlidGenerator {
        type IdType = ::ulid::Ulid;

        fn next_id_rep() -> Self::IdType {
            ::ulid::Ulid::from_datetime(std::time::SystemTime::now())
        }
    }
}