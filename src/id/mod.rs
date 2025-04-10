//! The `id` module provides a generic mechanism for creating, representing, and managing entity
//! identifiers (`Id`). It supports multiple ID generation strategies (e.g., CUID, UUID, Snowflake)
//! and integrates with external libraries such as `serde`, `sqlx`, and `disintegrate`.
//!
//! # Overview
//!
//! - [`Entity`] Trait: Defines an entity with an associated [`IdGenerator`] for unique ID generation.
//! - [`Id<T, ID>`] Struct: Represents an ID associated with an entity type `T` and an ID value of type `ID`.
//! - ID Generation Strategies:
//!   - **CUID** ([`CuidGenerator`], [`CuidId`]) - Enabled with the `cuid` feature.
//!   - **UUID** ([`UuidGenerator`]) - Enabled with the `uuid` feature.
//!   - **Snowflake** ([`SnowflakeGenerator`]) - Enabled with the `snowflake` feature.
//!
//! ## Features
//!
//! This module integrates with:
//!
//! - **Serde** (`serde` feature): Implements [`Serialize`] and [`Deserialize`] for `Id`.
//! - **SQLx** (`sqlx` feature): Enables database encoding/decoding via [`sqlx::Decode`], [`sqlx::Encode`], and [`sqlx::Type`].
//! - **Disintegrate** (`disintegrate` feature): Supports [`IntoIdentifierValue`] for identifier-based systems.
//!
//! ## Example Usage
//!
//! ### Defining an Entity with ID Generation
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
//! println!("Generated User ID: {}", user_id);
//! ```
//!
//! # ID Generation Strategies
//!
//! The module supports multiple ID generation strategies that can be enabled via Cargo features:
//!
//! | Feature       | Generator               | Description                                      |
//! |--------------|------------------------|--------------------------------------------------|
//! | `"cuid"`     | [`CuidGenerator`]       | Generates CUID-based IDs.                        |
//! | `"uuid"`     | [`UuidGenerator`]       | Generates UUID-based IDs.                        |
//! | `"snowflake"`| [`SnowflakeGenerator`]  | Uses the Snowflake algorithm for distributed ID generation. |
//!
//! To enable a specific ID generation method, add the corresponding feature to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tagid = { version = "0.2", features = ["uuid", "snowflake"] }
//! ```

mod generator;
pub use generator::IdGenerator;

#[cfg(feature = "cuid")]
pub use generator::{CuidGenerator, CuidId};

#[cfg(feature = "uuid")]
pub use generator::UuidGenerator;

#[cfg(feature = "ulid")]
pub use generator::UlidGenerator;

#[cfg(feature = "snowflake")]
pub mod snowflake;

#[cfg(feature = "snowflake")]
#[allow(unused_imports)]
pub use self::snowflake::{MachineNode, SnowflakeGenerator, pretty};

use crate::{DELIMITER, Label, Labeling};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smol_str::SmolStr;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A trait for entities that have a unique identifier.
///
/// Implementing this trait allows an entity type to generate new unique IDs
/// using its associated [`IdGenerator`].
pub trait Entity: Label {
    /// The ID generator type used to create unique IDs.
    type IdGen: IdGenerator;

    /// Generates a new unique ID for the entity.
    fn next_id() -> Id<Self, <Self::IdGen as IdGenerator>::IdType> {
        Id::new()
    }
}

/// A struct representing an identifier for an entity, and supports id labeling in logs and other
/// output.
///
/// `Id<T, ID>` associates an entity type `T` with an ID value of type `ID`.
///
/// # Example
///
/// ```rust,ignore
/// use tagid::{Id, Entity, Label, MakeLabeling};
///
/// struct User;
///
/// impl Label for User {
///     type Labeler = MakeLabeling<Self>;
///
///     fn labeler() -> Self::Labeler {
///         MakeLabeling::default()
///     }
/// }
///
/// impl Entity for User {
///     type IdGen = tagid::UuidGenerator;
/// }
///
/// let user_id = User::next_id();
/// println!("User ID: {}", user_id);
/// ```
pub struct Id<T: ?Sized, ID> {
    /// The label associated with the entity type.
    pub label: SmolStr,

    /// The unique identifier value.
    pub id: ID,

    marker: PhantomData<T>,
}

#[allow(unsafe_code)]
unsafe impl<T: ?Sized, ID: Send> Send for Id<T, ID> {}

#[allow(unsafe_code)]
unsafe impl<T: ?Sized, ID: Sync> Sync for Id<T, ID> {}

impl<T: ?Sized, ID> AsRef<ID> for Id<T, ID> {
    /// Returns the inner string representation of the `ID`.
    ///
    /// This method provides access to the underlying id value as a `&ID`.
    fn as_ref(&self) -> &ID {
        &self.id
    }
}

impl<T: ?Sized + Label, ID> From<ID> for Id<T, ID> {
    fn from(id: ID) -> Self {
        Self::for_labeled(id)
    }
}

impl<E> Id<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: ?Sized + Entity + Label,
{
    /// Generates a new `Id` using the entity's [`IdGenerator`].
    pub fn new() -> Self {
        let labeler = <E as Label>::labeler();
        Self {
            label: SmolStr::new(labeler.label()),
            id: E::IdGen::next_id_rep(),
            marker: PhantomData,
        }
    }
}

impl<E: ?Sized + Entity + Label> Default for Id<E, <<E as Entity>::IdGen as IdGenerator>::IdType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized + Label, ID> Id<T, ID> {
    pub fn for_labeled(id: ID) -> Self {
        let labeler = <T as Label>::labeler();
        Self {
            label: SmolStr::new(labeler.label()),
            id,
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized, ID> Id<T, ID> {
    /// Creates an `Id` with a specific label and ID value.
    pub fn direct(label: impl AsRef<str>, id: ID) -> Self {
        Self {
            label: SmolStr::new(label.as_ref()),
            id,
            marker: PhantomData,
        }
    }

    /// Consumes the `Id<T, ID>`, returning the inner `ID` value.
    pub fn into_inner(self) -> ID {
        self.id
    }
}

impl<T: ?Sized, ID: Clone> Id<T, ID> {
    /// Converts the `Id` to another entity type while retaining the same ID value.
    pub fn relabel<B: Label>(&self) -> Id<B, ID> {
        let b_labeler = B::labeler();
        Id {
            label: SmolStr::new(b_labeler.label()),
            id: self.id.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized, ID: Clone> Clone for Id<T, ID> {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            id: self.id.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized, ID: fmt::Debug> fmt::Debug for Id<T, ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("Id")
                .field("label", &self.label)
                .field("id", &self.id)
                .finish()
        } else if self.label.is_empty() {
            write!(f, "{:?}", self.id)
        } else {
            write!(f, "{}{DELIMITER}{:?}", self.label, self.id)
        }
    }
}

impl<T: ?Sized, ID: fmt::Display> fmt::Display for Id<T, ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() || self.label.is_empty() {
            write!(f, "{}", self.id)
        } else {
            write!(f, "{}{DELIMITER}{}", self.label, self.id)
        }
    }
}

impl<T: ?Sized, ID: PartialEq> PartialEq for Id<T, ID> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: ?Sized, ID: Eq> Eq for Id<T, ID> {}

impl<T: ?Sized, ID: Ord> Ord for Id<T, ID> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T: ?Sized, ID: PartialOrd> PartialOrd for Id<T, ID> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<T: ?Sized, ID: Hash> Hash for Id<T, ID> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl<T: ?Sized, ID: Serialize> Serialize for Id<T, ID> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id.serialize(serializer)
    }
}

impl<'de, T: ?Sized + Label, ID: DeserializeOwned> Deserialize<'de> for Id<T, ID> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rep = ID::deserialize(deserializer)?;
        let labeler = <T as Label>::labeler();
        Ok(Self::direct(labeler.label(), rep))
    }
}

#[cfg(feature = "sqlx")]
impl<'q, T, ID, DB> sqlx::Decode<'q, DB> for Id<T, ID>
where
    T: Label,
    ID: sqlx::Decode<'q, DB>,
    DB: sqlx::Database,
{
    fn decode(
        value: <DB as sqlx::Database>::ValueRef<'q>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <ID as sqlx::Decode<DB>>::decode(value)?;
        Ok(Self::for_labeled(value))
    }
}

#[cfg(feature = "sqlx")]
impl<'q, T, ID, DB> sqlx::Encode<'q, DB> for Id<T, ID>
where
    ID: sqlx::Encode<'q, DB>,
    DB: sqlx::Database,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <ID as sqlx::Encode<DB>>::encode_by_ref(&self.id, buf)
    }
}

#[cfg(feature = "sqlx")]
impl<T, ID, DB> sqlx::Type<DB> for Id<T, ID>
where
    ID: sqlx::Type<DB>,
    DB: sqlx::Database,
{
    fn type_info() -> DB::TypeInfo {
        <ID as sqlx::Type<DB>>::type_info()
    }
}

#[cfg(feature = "disintegrate")]
use disintegrate::{IdentifierType, IdentifierValue, IntoIdentifierValue};

#[cfg(feature = "disintegrate")]
impl<T, ID: fmt::Display> IntoIdentifierValue for Id<T, ID> {
    const TYPE: IdentifierType = IdentifierType::String;

    fn into_identifier_value(self) -> IdentifierValue {
        IdentifierValue::String(self.id.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CustomLabeling, MakeLabeling, NoLabeling};
    use assert_matches2::assert_let;
    use pretty_assertions::assert_eq;
    use serde_test::{Token, assert_tokens};
    use static_assertions::assert_impl_all;

    #[test]
    fn test_auto_traits() {
        assert_impl_all!(Id<u32, u32>: Send, Sync);
        assert_impl_all!(Id<std::rc::Rc<u32>, String>: Send, Sync);
    }

    struct TestGenerator;
    impl IdGenerator for TestGenerator {
        type IdType = String;

        fn next_id_rep() -> Self::IdType {
            std::time::SystemTime::UNIX_EPOCH
                .elapsed()
                .unwrap()
                .as_millis()
                .to_string()
        }
    }

    struct Bar;
    impl Label for Bar {
        type Labeler = MakeLabeling<Self>;

        fn labeler() -> Self::Labeler {
            MakeLabeling::default()
        }
    }

    struct NoLabelZed;

    impl Label for NoLabelZed {
        type Labeler = NoLabeling;

        fn labeler() -> Self::Labeler {
            NoLabeling
        }
    }

    struct Foo;

    impl Entity for Foo {
        type IdGen = TestGenerator;
    }

    impl Label for Foo {
        type Labeler = CustomLabeling;

        fn labeler() -> Self::Labeler {
            CustomLabeling::new("MyFooferNut")
        }
    }

    #[test]
    fn test_display() {
        let a: Id<Foo, String> = Foo::next_id();
        assert_eq!(format!("{a}"), format!("MyFooferNut::{}", a.id));
    }

    #[test]
    fn test_alternate_display() {
        let a: Id<Foo, i64> = Id::direct(Foo::labeler().label(), 13);
        assert_eq!(format!("{a:#}"), a.id.to_string());

        let id = 98734021;
        let a: Id<Foo, u64> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(format!("{a:#}"), a.id.to_string());

        #[cfg(feature = "uuid")]
        {
            let id = uuid::Uuid::new_v4();
            let a: Id<Foo, uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
            assert_eq!(format!("{a:#}"), a.id.to_string());
        }
    }

    #[test]
    fn test_debug() {
        let a: Id<Foo, String> = Foo::next_id();
        assert_eq!(format!("{a:?}"), format!("MyFooferNut::{:?}", a.id));

        let id = 98734021;
        let a: Id<Foo, u64> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(format!("{a:?}"), format!("MyFooferNut::{:?}", a.id));

        #[cfg(feature = "uuid")]
        {
            let id = uuid::Uuid::new_v4();
            let a: Id<Foo, uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
            assert_eq!(format!("{a:?}"), format!("MyFooferNut::{:?}", a.id));
        }
    }

    #[test]
    fn test_alternate_debug() {
        let a: Id<Foo, String> = Foo::next_id();
        assert_eq!(
            format!("{a:#?}"),
            format!(
                "Id {{\n    label: \"{}\",\n    id: \"{}\",\n}}",
                a.label, a.id,
            )
        );

        let id = 98734021;
        let a: Id<Foo, u64> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(
            format!("{a:#?}"),
            format!("Id {{\n    label: \"{}\",\n    id: {},\n}}", a.label, a.id,)
        );

        #[cfg(feature = "uuid")]
        {
            let id = uuid::Uuid::new_v4();
            let a: Id<Foo, uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
            assert_eq!(
                format!("{a:#?}"),
                format!("Id {{\n    label: \"{}\",\n    id: {},\n}}", a.label, a.id,)
            );
        }
    }

    #[test]
    fn test_id_cross_conversion() {
        let a = Foo::next_id();
        let before = format!("{}", a);
        assert_eq!(format!("MyFooferNut::{}", a.id), before);

        let b: Id<NoLabelZed, String> = a.relabel();
        let after_zed = format!("{}", b);
        assert_eq!(format!("{}", a.id), after_zed);

        let c: Id<Bar, String> = a.relabel();
        let after_bar = format!("{}", c);
        assert_eq!(format!("Bar::{}", a.id), after_bar);
    }

    #[test]
    fn test_id_serde_tokens() {
        let labeler = <Foo as Label>::labeler();
        let cuid = "ig6wv6nezj0jg51lg53dztqy".to_string();
        let id = Id::<Foo, String>::direct(labeler.label(), cuid);
        assert_tokens(&id, &[Token::Str("ig6wv6nezj0jg51lg53dztqy")]);

        let id = Id::<Foo, u64>::direct(labeler.label(), 17);
        assert_tokens(&id, &[Token::U64(17)]);
    }

    #[test]
    fn test_id_serde_json() {
        let labeler = <Foo as Label>::labeler();

        let cuid = "ig6wv6nezj0jg51lg53dztqy".to_string();
        let id = Id::<Foo, String>::direct(labeler.label(), cuid);
        assert_let!(Ok(json) = serde_json::to_string(&id));
        assert_let!(Ok(actual) = serde_json::from_str::<Id<Foo, String>>(&json));
        assert_eq!(actual, id);

        let id = Id::<Foo, u64>::direct(labeler.label(), 17);
        assert_let!(Ok(json) = serde_json::to_string(&id));
        assert_let!(Ok(actual) = serde_json::from_str::<Id<Foo, u64>>(&json));
        assert_eq!(actual, id);

        #[cfg(feature = "uuid")]
        {
            let uuid = uuid::Uuid::new_v4();
            let id = Id::<Foo, uuid::Uuid>::direct(labeler.label(), uuid);
            assert_let!(Ok(json) = serde_json::to_string(&id));
            assert_let!(Ok(actual) = serde_json::from_str::<Id<Foo, uuid::Uuid>>(&json));
            assert_eq!(actual, id);
        }
    }
}
