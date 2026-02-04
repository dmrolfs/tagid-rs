//! Labeling Mechanism for Identifiable Types
//!
//! This module provides a trait `Label` that associates types with a labeling mechanism,
//! determining how an entity should be labeled. The labeling system is flexible and supports:
//! - **Primitive types** (e.g., integers, floats, booleans, strings).
//! - **Container types** (e.g., `Option<T>`, `Result<T, E>`, `HashMap<K, V>`).
//! - **Custom labeling mechanisms** via the `CustomLabeling`, `MakeLabeling`, and `NoLabeling` strategies.
//!
//! ## Labeling Traits
//! - **`Label`**: Defines how a type provides a labeler instance.
//! - **`Labeling`**: The base trait for labeling logic.
//! - **`MakeLabeling<T>`**: A default labeling implementation for primitive types.
//! - **`CustomLabeling`**: A customizable labeler for composite types.
//! - **`NoLabeling`**: A marker for types that require no labels.

use crate::{CustomLabeling, Labeling, MakeLabeling, NoLabeling};
use std::collections::HashMap;

/// Controls how IDs are displayed to humans (via `.labeled()`).
///
/// # Scope
///
/// This enum affects **only** human-facing output. It does **NOT** affect:
/// - Display/to_string() → always canonical
/// - Serialize/serde → always canonical
/// - Database storage → always canonical
/// - ID construction → no complexity added
/// - Equality/hashing → based on canonical ID only
///
/// # Usage
///
/// Specified on the `Label` trait (and delegated from `Provenance`) as `POLICY` constant.
/// Controls the default behavior of `.labeled()` when called with no arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelPolicy {
    /// No preference; defaults to canonical output but encourages explicit mode selection.
    ///
    /// Used when the provenance doesn't have a strong default preference.
    ///
    /// **Output Scenarios:**
    /// - `id.to_string()`: `"value"` (Canonical)
    /// - `id.labeled().to_string()`: `"value"` (Default = None)
    /// - `id.labeled().mode(LabelMode::None).to_string()`: `"value"`
    /// - `id.labeled().mode(LabelMode::Short).to_string()`: `"Entity::value"`
    /// - `id.labeled().mode(LabelMode::Full).to_string()`: `"Entity@prov::value"`
    Opaque,

    /// Default to showing the entity type name in human output.
    ///
    /// Used for internal origins (Generated, Derived) where the entity type
    /// provides sufficient context without needing to show provenance details.
    ///
    /// **Output Scenarios:**
    /// - `id.to_string()`: `"value"` (Canonical)
    /// - `id.labeled().to_string()`: `"Entity::value"` (Default = Short)
    /// - `id.labeled().mode(LabelMode::None).to_string()`: `"value"`
    /// - `id.labeled().mode(LabelMode::Short).to_string()`: `"Entity::value"`
    /// - `id.labeled().mode(LabelMode::Full).to_string()`: `"Entity@prov::value"`
    EntityNameDefault,

    /// Default to showing both entity and provenance in human output.
    ///
    /// Used for external origins (External, Imported) where knowing the source system
    /// is critical for understanding and identifying the ID.
    ///
    /// **Output Scenarios:**
    /// - `id.to_string()`: `"value"` (Canonical)
    /// - `id.labeled().to_string()`: `"Entity@prov::value"` (Default = Full)
    /// - `id.labeled().mode(LabelMode::None).to_string()`: `"value"`
    /// - `id.labeled().mode(LabelMode::Short).to_string()`: `"Entity::value"`
    /// - `id.labeled().mode(LabelMode::Full).to_string()`: `"Entity@prov::value"`
    ExternalKeyDefault,

    /// Hide all labels by default; only show on explicit request.
    ///
    /// Used for sensitive or transient sources (Temporary, ClientProvided) where the ID
    /// itself might be sensitive and context shouldn't leak in logs unless necessary.
    ///
    /// **Output Scenarios:**
    /// - `id.to_string()`: `"value"` (Canonical)
    /// - `id.labeled().to_string()`: `"value"` (Default = None)
    /// - `id.labeled().mode(LabelMode::None).to_string()`: `"value"`
    /// - `id.labeled().mode(LabelMode::Short).to_string()`: `"Entity::value"`
    /// - `id.labeled().mode(LabelMode::Full).to_string()`: `"Entity@prov::value"`
    OpaqueByDefault,
}

/// Trait for types that can provide a labeling mechanism.
///
/// Types implementing `Label` define a corresponding `Labeler` type that
/// determines how their labels are generated.
///
/// # Example
/// ```rust
/// use tagid::{Label, Labeling, MakeLabeling};
///
/// struct MyType;
///
/// impl Label for MyType {
///     type Labeler = MakeLabeling<Self>;
///
///     fn labeler() -> Self::Labeler {
///         MakeLabeling::<Self>::default()
///     }
/// }
/// ```
pub trait Label {
    type Labeler: Labeling;

    /// The default labeling policy for this type.
    const POLICY: LabelPolicy = LabelPolicy::EntityNameDefault;

    /// Returns an instance of the labeler for the type.
    fn labeler() -> Self::Labeler;
}

/// Implementation for the unit type `()`, which has no labeling.
impl Label for () {
    type Labeler = NoLabeling;
    const POLICY: LabelPolicy = LabelPolicy::Opaque;

    fn labeler() -> Self::Labeler {
        NoLabeling
    }
}

/// Implementation for `Option<T>`, using the same labeling as `T`.
impl<T: Label> Label for Option<T> {
    type Labeler = <T as Label>::Labeler;
    const POLICY: LabelPolicy = T::POLICY;

    fn labeler() -> Self::Labeler {
        <T as Label>::labeler()
    }
}

/// Implementation for `Result<T, E>`, using the same labeling as `T`.
impl<T: Label, E> Label for Result<T, E> {
    type Labeler = <T as Label>::Labeler;
    const POLICY: LabelPolicy = T::POLICY;

    fn labeler() -> Self::Labeler {
        <T as Label>::labeler()
    }
}

/// Implementation for `HashMap<K, V>`, constructing a composite label.
///
/// This uses `CustomLabeling` with a label format like `"HashMap<K,V>"`,
/// where `K` and `V` use their respective labelers.
impl<K: Label, V: Label> Label for HashMap<K, V> {
    type Labeler = CustomLabeling;

    fn labeler() -> Self::Labeler {
        let k_labeler = <K as Label>::labeler();
        let v_labeler = <V as Label>::labeler();
        CustomLabeling::from(format!(
            "HashMap<{},{}",
            k_labeler.label(),
            v_labeler.label()
        ))
    }
}

/// Implements `Label` for primitive types using `MakeLabeling<Self>`.
///
/// This macro reduces boilerplate by automatically implementing `Label`
/// for each primitive type, using `MakeLabeling<Self>` as its labeler.
macro_rules! primitive_label {
    ($i:ty) => {
        impl Label for $i {
            type Labeler = MakeLabeling<Self>;

            fn labeler() -> Self::Labeler {
                MakeLabeling::<Self>::default()
            }
        }
    };
}

// Apply `primitive_label!` macro to standard primitive types.
primitive_label!(bool);
primitive_label!(char);
primitive_label!(f32);
primitive_label!(f64);
primitive_label!(i8);
primitive_label!(i32);
primitive_label!(i64);
primitive_label!(i128);
primitive_label!(isize);
primitive_label!(u8);
primitive_label!(u16);
primitive_label!(u32);
primitive_label!(u64);
primitive_label!(u128);
primitive_label!(usize);
primitive_label!(String);

/// Implementation for string slices (`&str`), using `MakeLabeling<Self>`.
///
/// Since `&str` is a reference type, this implementation helps ensure
/// consistent labeling when working with string slices.
impl Label for &str {
    type Labeler = MakeLabeling<Self>;

    fn labeler() -> Self::Labeler {
        MakeLabeling::<Self>::default()
    }
}
