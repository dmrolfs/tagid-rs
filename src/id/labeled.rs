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
