//! Labeled wrapper: Human-readable ID presentation.
//!
//! This module provides `Labeled<T, ID>`, a wrapper that attaches presentation
//! metadata to an `Id`. This controls how the ID is displayed to humans in
//! logs, error messages, and UIs, without affecting its canonical form.
//!
//! # Overview
//!
//! - [`LabelMode`] enum: Controls the level of detail (None, Short, Full)
//! - [`Labeled`] struct: Wraps an `Id` with a specific presentation mode
//! - [`.labeled(mode)`] method: Explicit opt-in for labeled presentation
//!
//! # Philosophy
//!
//! Labeling is strictly for **presentation**. It should never affect:
//! - Equality/Hashing (always based on the inner ID)
//! - Serialization (always uses canonical form)
//! - Database storage (always uses canonical form)
//!
//! By making labeling an explicit wrapper, we ensure that developers must
//! consciously opt-in to non-canonical representations, preventing accidental
//! leakage of labels into stable formats.

use super::Id;
use crate::{DELIMITER, Label, LabelPolicy, Labeling};
use std::fmt;

/// Controls the level of detail in human-readable ID presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LabelMode {
    /// No labeling; displays only the canonical ID value.
    ///
    /// Example: `550e8400-e29b-41d4-a716-446655440000`
    #[default]
    None,

    /// Short labeling; displays `Entity::value`.
    ///
    /// Example: `User::550e8400-e29b-41d4-a716-446655440000`
    Short,

    /// Full labeling; displays `Entity@provenance::value`.
    ///
    /// Example: `User@generated/uuid-v7::550e8400-e29b-41d4-a716-446655440000`
    Full,
}

impl From<LabelPolicy> for LabelMode {
    fn from(policy: LabelPolicy) -> Self {
        match policy {
            LabelPolicy::Opaque | LabelPolicy::OpaqueByDefault => LabelMode::None,
            LabelPolicy::EntityNameDefault => LabelMode::Short,
            LabelPolicy::ExternalKeyDefault => LabelMode::Full,
        }
    }
}

/// A wrapper that attaches a [`LabelMode`] to an [`Id`].
///
/// This struct is created via [`Id::labeled`]. It implements [`fmt::Display`]
/// and [`fmt::Debug`] to respect the chosen presentation mode.
pub struct Labeled<'a, T: ?Sized, ID> {
    pub(crate) id: &'a Id<T, ID>,
    pub(crate) mode: LabelMode,
}

impl<'a, T: ?Sized, ID> Labeled<'a, T, ID> {
    /// Creates a new `Labeled` wrapper.
    pub fn new(id: &'a Id<T, ID>, mode: LabelMode) -> Self {
        Self { id, mode }
    }

    /// Sets the label mode for this wrapper (builder pattern).
    pub fn mode(mut self, mode: LabelMode) -> Self {
        self.mode = mode;
        self
    }
}

impl<T: ?Sized + Label, ID: fmt::Display> fmt::Display for Labeled<'_, T, ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mode {
            LabelMode::None => write!(f, "{}", self.id.id),
            LabelMode::Short => {
                if self.id.label.is_empty() {
                    write!(f, "{}", self.id.id)
                } else {
                    write!(f, "{}{DELIMITER}{}", self.id.label, self.id.id)
                }
            }
            LabelMode::Full => {
                let labeler = T::labeler();
                let decorated = labeler.decorated_label();
                if decorated.is_empty() {
                    write!(f, "{}", self.id.id)
                } else {
                    write!(f, "{decorated}{DELIMITER}{}", self.id.id)
                }
            }
        }
    }
}

impl<T: ?Sized + Label, ID: fmt::Debug> fmt::Debug for Labeled<'_, T, ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mode {
            LabelMode::None => write!(f, "{:?}", self.id.id),
            LabelMode::Short => {
                if self.id.label.is_empty() {
                    write!(f, "{:?}", self.id.id)
                } else {
                    write!(f, "{}{DELIMITER}{:?}", self.id.label, self.id.id)
                }
            }
            LabelMode::Full => {
                let labeler = T::labeler();
                let decorated = labeler.decorated_label();
                if decorated.is_empty() {
                    write!(f, "{:?}", self.id.id)
                } else {
                    write!(f, "{decorated}{DELIMITER}{:?}", self.id.id)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CustomLabeling;
    use crate::NoLabeling;

    struct Foo;
    impl Label for Foo {
        type Labeler = CustomLabeling;
        fn labeler() -> Self::Labeler {
            CustomLabeling::new("MyFooferNut")
        }
    }

    struct NoLabelZed;
    impl Label for NoLabelZed {
        type Labeler = NoLabeling;
        fn labeler() -> Self::Labeler {
            NoLabeling
        }
    }

    #[test]
    fn test_label_policy_to_mode_conversion() {
        assert_eq!(LabelMode::from(LabelPolicy::Opaque), LabelMode::None);
        assert_eq!(
            LabelMode::from(LabelPolicy::OpaqueByDefault),
            LabelMode::None
        );
        assert_eq!(
            LabelMode::from(LabelPolicy::EntityNameDefault),
            LabelMode::Short
        );
        assert_eq!(
            LabelMode::from(LabelPolicy::ExternalKeyDefault),
            LabelMode::Full
        );
    }

    #[test]
    fn test_labeled_none_mode() {
        let id: Id<Foo, String> = Id::direct("Label", "value".into());
        assert_eq!(id.labeled().mode(LabelMode::None).to_string(), "value");
    }

    #[test]
    fn test_labeled_short_mode() {
        let id: Id<Foo, String> = Id::direct("Label", "value".into());
        assert_eq!(
            id.labeled().mode(LabelMode::Short).to_string(),
            "Label::value"
        );
    }

    #[test]
    fn test_labeled_full_mode() {
        let id: Id<Foo, String> = Id::direct("_", "value".into());
        let full = id.labeled().mode(LabelMode::Full).to_string();
        // Full uses decorated_label() from Foo::labeler()
        assert!(full.contains("value"));
        assert!(full.contains("MyFooferNut")); // Type-derived
    }

    #[test]
    fn test_labeled_full_ignores_stored_label() {
        // CRITICAL SEMANTIC: LabelMode::Full uses T::labeler(), not id.label
        let id: Id<Foo, String> = Id::direct("WRONG", "value".into());

        let short = id.labeled().mode(LabelMode::Short).to_string();
        assert!(short.contains("WRONG")); // Stored label used

        let full = id.labeled().mode(LabelMode::Full).to_string();
        assert!(!full.contains("WRONG")); // Stored label ignored
        assert!(full.contains("MyFooferNut")); // Type-derived label used
    }

    #[test]
    fn test_labeled_empty_label_short_mode() {
        let id: Id<NoLabelZed, String> = Id::direct("", "value".into());
        // Short mode with empty label should fall back to just value
        let short = id.labeled().mode(LabelMode::Short).to_string();
        assert_eq!(short, "value");
    }

    #[test]
    fn test_labeled_debug_modes() {
        let id: Id<Foo, u64> = Id::direct("Label", 42u64);

        assert_eq!(id.labeled().mode(LabelMode::None).to_string(), "42");
        assert_eq!(id.labeled().mode(LabelMode::Short).to_string(), "Label::42");
    }

    #[test]
    fn test_labeled_builder_pattern() {
        let id: Id<Foo, String> = Id::direct("L", "v".into());

        let labeled_none = id.labeled().mode(LabelMode::None);
        assert_eq!(labeled_none.to_string(), "v");

        let labeled_short = id.labeled().mode(LabelMode::Short);
        assert_eq!(labeled_short.to_string(), "L::v");
    }
}
