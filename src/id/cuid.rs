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
