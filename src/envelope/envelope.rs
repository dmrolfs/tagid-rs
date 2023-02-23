use crate::envelope::metadata::MetaData;
use crate::envelope::{Correlation, ReceivedAt};
use crate::id::IdGenerator;
use crate::{Entity, Id, Label, Labeling};
#[cfg(feature = "functional")]
use frunk::{Monoid, Semigroup};
use iso8601_timestamp::Timestamp;
use pretty_type_name::pretty_type_name;
use serde::{de, ser::SerializeStruct, Deserialize, Serialize, Serializer};
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;

pub trait IntoEnvelope {
    type Content: Label;
    type IdGen: IdGenerator;

    fn into_envelope(self) -> Envelope<Self::Content, <Self::IdGen as IdGenerator>::IdType>;
    fn metadata(&self) -> &MetaData<Self::Content, <Self::IdGen as IdGenerator>::IdType>;
}

/// A metadata wrapper for a data set
#[derive(Clone)]
pub struct Envelope<T, ID>
where
    T: ?Sized,
{
    metadata: MetaData<T, ID>,
    content: T,
}

impl<T, ID> fmt::Debug for Envelope<T, ID>
where
    T: fmt::Debug,
    ID: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("[{}]{{ {:?} }}", self.metadata, self.content))
    }
}

impl<T, ID> fmt::Display for Envelope<T, ID>
where
    T: fmt::Display,
    ID: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]({})", self.metadata, self.content)
    }
}

impl<E> Envelope<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: Entity,
{
    pub fn from_entity(content: E) -> Self {
        Self {
            metadata: MetaData::default(),
            content,
        }
    }
}

impl<T, ID> Envelope<T, ID>
where
    T: Label,
{
    /// Create a new enveloped data.
    pub fn new<G>(content: T) -> Self
    where
        G: IdGenerator<IdType = ID>,
    {
        let correlation_id = Id::direct(T::labeler().label(), G::next_id_rep());

        Self {
            metadata: MetaData::from_parts(correlation_id, Timestamp::now_utc(), None),
            content,
        }
    }
}

impl<T, ID> Envelope<T, ID> {
    /// Directly create enveloped data with given metadata.
    pub const fn direct(content: T, metadata: MetaData<T, ID>) -> Self {
        Self { metadata, content }
    }

    /// Get a reference to the sensor data metadata.
    pub const fn metadata(&self) -> &MetaData<T, ID> {
        &self.metadata
    }

    /// Consumes self, returning the data item
    #[allow(clippy::missing_const_for_fn)]
    #[inline]
    pub fn into_inner(self) -> T {
        self.content
    }

    #[allow(clippy::missing_const_for_fn)]
    #[inline]
    pub fn into_parts(self) -> (MetaData<T, ID>, T) {
        (self.metadata, self.content)
    }

    #[inline]
    pub const fn from_parts(metadata: MetaData<T, ID>, content: T) -> Self {
        Self { metadata, content }
    }
}

impl<T, ID> Envelope<T, ID>
where
    T: Label,
    ID: Clone,
{
    pub fn adopt_metadata<U>(&mut self, new_metadata: MetaData<U, ID>) -> MetaData<T, ID>
    where
        U: Label,
    {
        let old_metadata = self.metadata.clone();
        self.metadata = new_metadata.relabel();
        old_metadata
    }

    pub fn map<F, U>(self, f: F) -> Envelope<U, ID>
    where
        U: Label,
        F: FnOnce(T) -> U,
    {
        let metadata = self.metadata.clone().relabel();
        Envelope {
            metadata,
            content: f(self.content),
        }
    }

    pub fn flat_map<F, U>(self, f: F) -> Envelope<U, ID>
    where
        U: Label,
        F: FnOnce(Self) -> U,
    {
        let metadata = self.metadata.clone().relabel();
        Envelope {
            metadata,
            content: f(self),
        }
    }
}

impl<T, ID> Envelope<T, ID>
where
    T: Label + Send,
    ID: Clone + Send,
{
    pub async fn and_then<Op, Fut, U>(self, f: Op) -> Envelope<U, ID>
    where
        U: Label + Send,
        Fut: Future<Output = U> + Send,
        Op: FnOnce(T) -> Fut + Send,
    {
        let metadata = self.metadata.clone().relabel();
        Envelope {
            metadata,
            content: f(self.content).await,
        }
    }
}

impl<E> Correlation for Envelope<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: Entity + Sync,
{
    type Correlated = E;
    type IdType = <<E as Entity>::IdGen as IdGenerator>::IdType;

    fn correlation(&self) -> &Id<Self::Correlated, Self::IdType> {
        self.metadata.correlation()
    }
}

impl<T, ID> ReceivedAt for Envelope<T, ID> {
    fn recv_timestamp(&self) -> Timestamp {
        self.metadata.recv_timestamp()
    }
}

impl<T, ID> Label for Envelope<T, ID>
where
    T: Label,
{
    type Labeler = <T as Label>::Labeler;

    fn labeler() -> Self::Labeler {
        <T as Label>::labeler()
    }
}

impl<T, ID> std::ops::Add for Envelope<T, ID>
where
    T: std::ops::Add<Output = T>,
    ID: PartialOrd,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_parts(self.metadata + rhs.metadata, self.content + rhs.content)
    }
}

#[cfg(feature = "functional")]
impl<E> Monoid for Envelope<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: Entity + Monoid,
    <<E as Entity>::IdGen as IdGenerator>::IdType: PartialOrd + Clone,
{
    fn empty() -> Self {
        Self::from_parts(
            <MetaData<E, <<E as Entity>::IdGen as IdGenerator>::IdType> as Monoid>::empty(),
            <E as Monoid>::empty(),
        )
    }
}

#[cfg(feature = "functional")]
impl<E> Semigroup for Envelope<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: Entity + Semigroup,
    <<E as Entity>::IdGen as IdGenerator>::IdType: PartialOrd + Clone,
{
    fn combine(&self, other: &Self) -> Self {
        Self::from_parts(
            self.metadata().combine(other.metadata()),
            self.content.combine(&other.content),
        )
    }
}

impl<E> IntoEnvelope for Envelope<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: Entity,
{
    type Content = E;
    type IdGen = <E as Entity>::IdGen;

    fn into_envelope(self) -> Envelope<Self::Content, <Self::IdGen as IdGenerator>::IdType> {
        self
    }

    fn metadata(&self) -> &MetaData<Self::Content, <Self::IdGen as IdGenerator>::IdType> {
        &self.metadata
    }
}

impl<T, ID> std::ops::Deref for Envelope<T, ID> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<T, ID> std::ops::DerefMut for Envelope<T, ID> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<T, ID> AsRef<T> for Envelope<T, ID> {
    fn as_ref(&self) -> &T {
        &self.content
    }
}

impl<T, ID> AsMut<T> for Envelope<T, ID> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.content
    }
}

impl<T, ID> PartialEq for Envelope<T, ID>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.content == other.content
    }
}

impl<T, ID> PartialEq<T> for Envelope<T, ID>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        &self.content == other
    }
}

impl<T, ID> Envelope<Option<T>, ID>
where
    T: Label,
    ID: Clone,
{
    /// Transposes an `Envelope` of an [`Option`] into an [`Option`] of `Envelope`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tagid::{CuidGenerator, Entity, Label, MakeLabeling};
    /// use tagid::envelope::{Envelope, MetaData};
    ///
    /// #[derive(Debug, Label, PartialEq)]
    /// struct Foo(pub i32);
    /// impl Entity for Foo { type IdGen = CuidGenerator; }
    ///
    /// let meta: MetaData<Foo, String> = MetaData::default();
    ///
    /// let x: Option<Envelope<Foo, String>> = Some(Envelope::from_parts(meta.clone(), Foo(5)));
    /// let y: Envelope<Option<Foo>, String> = Envelope::from_parts(meta.relabel(), Some(Foo(5)));
    /// assert_eq!(x, y.transpose());
    /// ```
    #[inline]
    pub fn transpose(self) -> Option<Envelope<T, ID>> {
        match self.content {
            Some(d) => Some(Envelope {
                content: d,
                metadata: self.metadata.relabel(),
            }),
            None => None,
        }
    }
}

impl<T, E, ID> Envelope<Result<T, E>, ID>
where
    T: Label,
    ID: Clone,
{
    /// Transposes a `Envelope` of a [`Result`] into a [`Result`] of `Envelope`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use tagid::{CuidGenerator, Entity, Label, MakeLabeling};
    /// use tagid::envelope::{Envelope, MetaData};
    ///
    /// #[derive(Debug, Label, PartialEq)]
    /// struct Foo(pub i32);
    /// impl Entity for Foo { type IdGen = CuidGenerator; }
    ///
    /// let meta: MetaData<Foo, String> = MetaData::default();
    ///
    /// #[derive(Debug, Eq, PartialEq)]
    /// struct SomeErr;
    ///
    /// let x: Result<Envelope<Foo, String>, SomeErr> = Ok(Envelope::from_parts(meta.clone(), Foo(5)));
    /// let y: Envelope<Result<Foo, SomeErr>, String> = Envelope::from_parts(meta.relabel(), Ok(Foo(5)));
    /// assert_eq!(x, y.transpose());
    /// ```
    #[inline]
    pub fn transpose(self) -> Result<Envelope<T, ID>, E> {
        match self.content {
            Ok(content) => Ok(Envelope {
                content,
                metadata: self.metadata.relabel(),
            }),
            Err(e) => Err(e),
        }
    }
}

const ENV_METADATA: &str = "metadata";
const ENV_CONTENT: &str = "content";
const FIELDS: [&str; 2] = [ENV_METADATA, ENV_CONTENT];

impl<T, ID> Serialize for Envelope<T, ID>
where
    T: Serialize,
    ID: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Envelope", 2)?;
        state.serialize_field(ENV_METADATA, &self.metadata)?;
        state.serialize_field(ENV_CONTENT, &self.content)?;
        state.end()
    }
}

impl<'de, T, ID> Deserialize<'de> for Envelope<T, ID>
where
    T: Label + de::DeserializeOwned,
    ID: de::DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        enum Field {
            MetaData,
            Content,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> de::Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("`metadata` or `content`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            ENV_METADATA => Ok(Field::MetaData),
                            ENV_CONTENT => Ok(Field::Content),
                            _ => Err(de::Error::unknown_field(value, &FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct EnvelopeVisitor<T0, ID0> {
            marker: PhantomData<(T0, ID0)>,
        }

        impl<T0, ID0> EnvelopeVisitor<T0, ID0> {
            pub const fn new() -> Self {
                Self {
                    marker: PhantomData,
                }
            }
        }

        impl<'de, T0, ID0> de::Visitor<'de> for EnvelopeVisitor<T0, ID0>
        where
            T0: Label + de::DeserializeOwned,
            ID0: de::DeserializeOwned,
        {
            type Value = Envelope<T0, ID0>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(
                    format!(
                        "struct Envelope<{}, {}>",
                        pretty_type_name::<T0>(),
                        pretty_type_name::<ID0>(),
                    )
                    .as_str(),
                )
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let metadata: MetaData<T0, ID0> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let content: T0 = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                Ok(Envelope::from_parts(metadata, content))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut metadata = None;
                let mut content = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::MetaData => {
                            if metadata.is_some() {
                                return Err(de::Error::duplicate_field(ENV_METADATA));
                            }
                            metadata = Some(map.next_value()?);
                        }
                        Field::Content => {
                            if content.is_some() {
                                return Err(de::Error::duplicate_field(ENV_CONTENT));
                            }
                            content = Some(map.next_value()?);
                        }
                    }
                }

                let metadata: MetaData<T0, ID0> =
                    metadata.ok_or_else(|| de::Error::missing_field(ENV_METADATA))?;
                let content: T0 = content.ok_or_else(|| de::Error::missing_field(ENV_CONTENT))?;
                Ok(Envelope::from_parts(metadata, content))
            }
        }

        deserializer.deserialize_struct("Envelope", &FIELDS, EnvelopeVisitor::<T, ID>::new())
    }
}
