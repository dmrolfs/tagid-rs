use crate::{DELIMITER, Entity, IdGenerator, Label, Labeling};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smol_str::SmolStr;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[cfg(feature = "disintegrate")]
use disintegrate::{IdentifierType, IdentifierValue, IntoIdentifierValue};

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

impl<E> Default for Id<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: Entity + Label + ?Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, ID> Id<T, ID>
where
    T: Label + ?Sized,
{
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

impl<'de, T, ID> Deserialize<'de> for Id<T, ID>
where
    T: Label + ?Sized,
    ID: DeserializeOwned,
{
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
impl<T, ID: fmt::Display> IntoIdentifierValue for Id<T, ID> {
    const TYPE: IdentifierType = IdentifierType::String;

    fn into_identifier_value(self) -> IdentifierValue {
        IdentifierValue::String(self.id.to_string())
    }
}
