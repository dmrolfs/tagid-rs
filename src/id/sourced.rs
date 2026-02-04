//! Sourced wrapper: Combines an entity with provenance tracking.
//!
//! This module provides `Sourced<E, S>`, a type wrapper that associates an entity `E`
//! with provenance information `S`. This allows tagging IDs with their origin (External,
//! Generated, etc.) while maintaining zero runtime overhead through type-level encoding.
//!
//! # Overview
//!
//! - [`Sourced<E, S>`] struct: Wraps entity E with provenance S (PhantomData only)
//! - [`Label`] implementation: Delegates to E but allows decoration with provenance
//! - [`Entity`] implementation: Available only when S is Generated to prevent accidents
//! - [`SourceLabeler<L, S>`] struct: Composes labeling strategies with provenance
//!
//! # Philosophy
//!
//! Provenance is purely **type-level** with zero runtime cost:
//! - `Sourced<User, Generated<UuidV7>>` encodes the provenance in types
//! - No runtime wrapper overhead
//! - Compile-time prevents generating External or Imported IDs
//!
//! # Example
//!
//! ```ignore
//! use tagid::{Entity, Label};
//! use tagid::id::provenance::*;
//!
//! // Stripe customer ID (external)
//! pub type StripeCustomerId = Sourced<Customer, External<Stripe>>;
//! let id = StripeCustomerId::new("cus_L3H8Z6K9j2");
//!
//! // Generated user ID
//! pub type UserId = Sourced<User, Generated<UuidV7>>;
//! let id = UserId::generate();  // Only works because S = Generated<_>
//! ```

use std::marker::PhantomData;

use crate::{Entity, Label, Labeling};

use super::provenance::{Generated, Provenance};

/// A wrapper associating an entity `E` with provenance `S`.
///
/// # Type Parameters
///
/// - `E`: The entity type (User, Customer, etc.)
/// - `S`: The provenance type (External<Provider>, Generated<Strategy>, etc.)
///
/// # Zero Runtime Overhead
///
/// `Sourced` is a compile-time construct using only PhantomData.
/// It adds no fields, no memory cost, no runtime overhead.
///
/// # Design
///
/// Provenance information flows through:
/// 1. `Sourced<E, S>::labeler()` returns a `SourceLabeler<L, S>`
/// 2. The labeler decorates the entity's label with provenance context
/// 3. Display and serde always use canonical (label-free) form via `{:#}`
/// 4. Explicit `.labeled()` calls use the decorated form
///
/// # Note on Size
///
/// Since `Sourced` only uses `PhantomData`, `size_of::<Sourced<E, S>>()` is 0.
/// The entire provenance system has zero runtime cost.
#[derive(Debug, Clone, Copy)]
pub struct Sourced<E: ?Sized, S: Provenance> {
    /// Entity type (unused at runtime)
    _entity: PhantomData<E>,

    /// Provenance type (unused at runtime)
    _source: PhantomData<S>,
}

impl<E: ?Sized, S: Provenance> Sourced<E, S> {
    /// Creates a new Sourced instance.
    ///
    /// This is a compile-time operation with zero runtime cost.
    ///
    /// # Note
    ///
    /// Typically, you won't call this directly. Instead, you'll define type aliases:
    ///
    /// ```ignore
    /// pub type StripeCustomerId = Sourced<Customer, External<Stripe>>;
    /// pub type UserId = Sourced<User, Generated<UuidV7>>;
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self {
            _entity: PhantomData,
            _source: PhantomData,
        }
    }
}

impl<E: ?Sized, S: Provenance> Default for Sourced<E, S> {
    fn default() -> Self {
        Self::new()
    }
}

/// Labeling wrapper that composes entity labeling with provenance context.
///
/// Implements the `Labeling` trait to provide the canonical entity label.
/// Provenance decoration is handled separately via explicit method calls.
///
/// # Design Notes
///
/// - `.label()` returns the canonical entity label (delegates to entity's labeler)
/// - Provenance context is handled at a higher level (e.g., in a `.labeled()` method on wrapper types)
/// - This keeps SourceLabeler simple and focused on composition
///
/// # Example
///
/// With `SourceLabeler<MakeLabeling<Customer>, External<Stripe>>`:
/// - `.label()` → "Customer" (canonical, delegates to entity labeler)
pub struct SourceLabeler<L: Labeling, S: Provenance> {
    /// The entity's original labeler
    entity_labeler: L,

    /// Provenance information (type-level only, no runtime cost)
    _provenance: PhantomData<S>,
}

impl<L: Labeling, S: Provenance> SourceLabeler<L, S> {
    /// Creates a new SourceLabeler from an entity labeler and provenance.
    ///
    /// # Arguments
    ///
    /// * `entity_labeler` - The labeler from the wrapped entity type
    ///
    /// # Returns
    ///
    /// A SourceLabeler that can produce canonical labels and provenance context.
    pub fn new(entity_labeler: L) -> Self {
        Self {
            entity_labeler,
            _provenance: PhantomData,
        }
    }

    /// Returns the provenance label for this sourced entity.
    ///
    /// This is used for human-facing output to show where the ID comes from.
    ///
    /// # Examples
    ///
    /// - For `SourceLabeler<_, External<Stripe>>` → "external/stripe"
    /// - For `SourceLabeler<_, Generated<UuidV7>>` → "generated/uuid-v7"
    /// - For `SourceLabeler<_, Temporary>` → "temporary"
    pub fn provenance_label(&self) -> String {
        provenance_display_name::<S>()
    }

    /// Returns the full decorated label combining entity and provenance.
    ///
    /// Format: `Entity@provenance/provider` or `Entity@provenance` (depending on type)
    ///
    /// # Examples
    ///
    /// - For `Sourced<Customer, External<Stripe>>` → "Customer@external/stripe"
    /// - For `Sourced<User, Generated<UuidV7>>` → "User@generated/uuid-v7"
    pub fn decorated_label(&self) -> String {
        format!(
            "{}@{}",
            self.entity_labeler.label(),
            self.provenance_label()
        )
    }
}

impl<L: Labeling, S: Provenance> Labeling for SourceLabeler<L, S> {
    /// Returns the canonical entity label (delegated to entity's labeler).
    ///
    /// This is the base label, without provenance decoration.
    /// Used for canonical ID strings (Display, serde).
    ///
    /// # Invariant
    ///
    /// The canonical label should always be stable and label-free,
    /// matching what Display and serialization produce.
    ///
    /// # Example
    ///
    /// For `Sourced<Customer, External<Stripe>>`:
    /// - `.label()` → "Customer"
    fn label(&self) -> &str {
        self.entity_labeler.label()
    }
}

impl<E: ?Sized + Label, S: Provenance> Label for Sourced<E, S> {
    type Labeler = SourceLabeler<E::Labeler, S>;

    fn labeler() -> Self::Labeler {
        SourceLabeler::new(E::labeler())
    }
}

impl<E: ?Sized + Entity, S> Entity for Sourced<E, Generated<S>>
where
    S: 'static + Send + Sync + Default + Clone,
{
    type IdGen = E::IdGen;
}

// ============================================================================
// Helper functions for provenance context formatting
// ============================================================================

/// Formats the provenance display name for a provenance type.
///
/// Returns a human-readable string representing the provenance.
/// This is used in decorated labels.
///
/// # Note
///
/// This currently just returns the provenance NAME. In a future phase,
/// this could be extended to include provider-specific information
/// (e.g., "external/stripe" instead of just "external").
fn provenance_display_name<S: Provenance>() -> String {
    S::NAME.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock types for testing
    struct TestEntity;
    impl Label for TestEntity {
        type Labeler = crate::MakeLabeling<Self>;

        fn labeler() -> Self::Labeler {
            crate::MakeLabeling::default()
        }
    }

    #[test]
    fn test_sourced_is_zero_sized() {
        use std::mem::size_of;
        assert_eq!(size_of::<Sourced<TestEntity, super::super::provenance::Generated<()>>>(), 0);
    }

    #[test]
    fn test_sourced_label_delegation() {
        use super::super::provenance::Generated;

        let labeler = Sourced::<TestEntity, Generated<()>>::labeler();
        assert_eq!(labeler.label(), "TestEntity");
    }

    #[test]
    fn test_provenance_display_name() {
        use super::super::provenance::{External, Temporary};
        assert_eq!(provenance_display_name::<External<()>>(), "external");
        assert_eq!(provenance_display_name::<Generated<()>>(), "generated");
        assert_eq!(provenance_display_name::<Temporary>(), "temporary");
    }
}
