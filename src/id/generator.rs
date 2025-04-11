//! ID Generation Module
//!
//! This module defines the `IdGenerator` trait and provides implementations for generating unique
//! identifiers using different strategies, including CUID and UUID. The choice of ID generator
//! is controlled through feature flags (`with-cuid`, `with-ulid`, and `with-uuid`).

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
