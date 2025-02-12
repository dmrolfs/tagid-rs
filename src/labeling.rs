//! Labeling System for Identifiable Types
//!
//! This module defines the `Labeling` trait and several implementations that
//! provide structured labels for different types. Labeling is used to attach
//! human-readable or type-derived identifiers to objects in a lightweight and
//! efficient manner.
//!
//! ## Features:
//! - **`Labeling` trait**: Defines a common interface for label retrieval.
//! - **`MakeLabeling<T>`**: Automatically derives labels from Rust type names.
//! - **`CustomLabeling`**: Allows user-defined labels.
//! - **`NoLabeling`**: Represents the absence of a label.
//!
//! This system helps categorize objects efficiently with minimal runtime overhead.

use crate::Label;
use once_cell::sync::OnceCell;
use pretty_type_name::pretty_type_name;
use smol_str::SmolStr;
use std::convert::Infallible;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

/// A trait representing types that can provide a label.
///
/// Implementors of this trait define a `label` method that returns a reference
/// to the label associated with the type.
///
/// # Example
/// ```rust
/// use tagid::{Labeling, CustomLabeling};
///
/// let custom = CustomLabeling::new("MyLabel");
/// assert_eq!(custom.label(), "MyLabel");
/// ```
pub trait Labeling {
    /// Returns the label associated with the type.
    fn label(&self) -> &str;
}

impl dyn Labeling {
    /// Summon an instance of the labeler for a given type `T`.
    ///
    /// # Example
    /// ```rust
    /// use tagid::{Label, Labeling};
    ///
    /// struct MyType;
    ///
    /// impl Label for MyType {
    ///     type Labeler = tagid::MakeLabeling<Self>;
    ///
    ///     fn labeler() -> Self::Labeler {
    ///         Self::Labeler::default()
    ///     }
    /// }
    ///
    /// let labeler = <dyn Labeling>::summon::<MyType>();
    /// assert!(!labeler.label().is_empty());
    /// ```
    pub fn summon<T: Label>() -> <T as Label>::Labeler {
        T::labeler()
    }
}

/// A labeling implementation that derives labels from Rust type names.
///
/// This struct lazily computes and stores the type name as a label.
#[derive(Clone)]
pub struct MakeLabeling<T: ?Sized> {
    label: OnceCell<SmolStr>,
    marker: PhantomData<T>,
}

impl<T: ?Sized> MakeLabeling<T> {
    /// Creates a new `MakeLabeling` instance.
    pub const fn new() -> Self {
        Self {
            label: OnceCell::new(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized> Default for MakeLabeling<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Labeling for MakeLabeling<T> {
    /// Retrieves the label, which is the pretty-printed type name of `T`.
    fn label(&self) -> &str {
        self.label
            .get_or_init(|| SmolStr::new(pretty_type_name::<T>()))
            .as_str()
    }
}

impl<T: ?Sized> fmt::Debug for MakeLabeling<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MakeLabeling({})", self.label())
    }
}

impl<T: ?Sized> fmt::Display for MakeLabeling<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

/// A user-defined labeling mechanism.
///
/// This struct allows explicitly setting a label that does not depend on type names.
#[derive(Clone)]
pub struct CustomLabeling {
    label: SmolStr,
}

impl CustomLabeling {
    /// Creates a new `CustomLabeling` instance with the given label.
    pub fn new(label: impl AsRef<str>) -> Self {
        Self {
            label: SmolStr::new(label),
        }
    }
}

impl Labeling for CustomLabeling {
    /// Retrieves the custom label.
    fn label(&self) -> &str {
        self.label.as_str()
    }
}

impl fmt::Debug for CustomLabeling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CustomLabeling({})", self.label())
    }
}

impl fmt::Display for CustomLabeling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

impl From<&str> for CustomLabeling {
    fn from(label: &str) -> Self {
        Self {
            label: SmolStr::new(label),
        }
    }
}

impl From<String> for CustomLabeling {
    fn from(label: String) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl FromStr for CustomLabeling {
    type Err = Infallible;

    fn from_str(label: &str) -> Result<Self, Self::Err> {
        Ok(label.into())
    }
}

/// A marker type representing the absence of a label.
///
/// This is useful for cases where labeling is optional or unnecessary.
#[derive(Debug, Copy, Clone)]
pub struct NoLabeling;

impl Labeling for NoLabeling {
    fn label(&self) -> &str {
        ""
    }
}

impl fmt::Display for NoLabeling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}
