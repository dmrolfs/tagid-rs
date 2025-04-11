use super::*;

/// A type alias for a UUID-based identifier wrapped in the `Id` struct.
///
/// This is useful when IDs are represented as `Id<T, Uuid>`, where `T` is the entity type.
#[allow(dead_code)]
pub type UuidId<T> = Id<T, ::uuid::Uuid>;

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
