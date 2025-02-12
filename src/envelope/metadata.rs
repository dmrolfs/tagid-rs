use crate::envelope::{Correlation, ReceivedAt};
use crate::id::IdGenerator;
use crate::{Entity, Id, Label, Labeling};
use iso8601_timestamp::Timestamp;
use pretty_type_name::pretty_type_name;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

#[cfg(feature = "functional")]
use frunk::{Monoid, Semigroup};

/// This key represents the standard metadata correlation attribute in message envelopes.
pub const CORRELATION_ID_KEY: &str = "correlation_id";

/// This key represents the standard metadata timestamp attribute used in message envelopes.
pub const RECV_TIMESTAMP_KEY: &str = "recv_timestamp";

/// Converts an object into `MetaData`, extracting correlation IDs and received timestamps.
pub trait IntoMetaData {
    /// The type of entity associated with the metadata.
    type CorrelatedType: Label;

    /// Converts the object into a `MetaData` instance.
    fn into_metadata<G>(self) -> MetaData<Self::CorrelatedType, G::IdType>
    where
        G: IdGenerator,
        G::IdType: FromStr;
}

impl IntoMetaData for HashMap<String, String> {
    type CorrelatedType = ();

    fn into_metadata<G>(mut self) -> MetaData<Self::CorrelatedType, G::IdType>
    where
        G: IdGenerator,
        G::IdType: FromStr,
    {
        let id_rep = self
            .remove(CORRELATION_ID_KEY)
            .and_then(|rep| G::IdType::from_str(&rep).ok())
            .unwrap_or_else(|| G::next_id_rep());
        let correlation_id = Id::direct(<() as Label>::labeler().label(), id_rep);

        let recv_timestamp = self
            .remove(RECV_TIMESTAMP_KEY)
            .map_or_else(Timestamp::now_utc, |ts| {
                Timestamp::parse(ts.as_str()).unwrap_or_else(Timestamp::now_utc)
            });

        let custom = if !self.is_empty() { Some(self) } else { None };

        MetaData::from_parts(correlation_id, recv_timestamp, custom)
    }
}

/// Represents metadata for an envelope, including correlation IDs, timestamps, and custom attributes.
#[derive(Serialize)]
pub struct MetaData<T, ID>
where
    T: ?Sized,
{
    /// A unique identifier for correlating messages.
    correlation_id: Id<T, ID>,

    /// The timestamp when the message was received.
    recv_timestamp: Timestamp,

    /// A key-value store for additional metadata.
    custom: HashMap<String, String>,
}

impl<T, ID> fmt::Debug for MetaData<T, ID>
where
    T: ?Sized,
    ID: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug = f.debug_struct("MetaData");
        debug.field("correlation", &self.correlation_id);
        debug.field("recv_timestamp", &self.recv_timestamp.to_string());

        if !self.custom.is_empty() {
            debug.field("custom", &self.custom);
        }

        debug.finish()
    }
}

impl<T, ID> fmt::Display for MetaData<T, ID>
where
    ID: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let custom_rep = format!("{:?}", self.custom);
        write!(
            f,
            "{} @ {}{}",
            self.correlation_id, self.recv_timestamp, custom_rep
        )
    }
}

impl<E> Default for MetaData<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: Entity + Label,
{
    fn default() -> Self {
        Self::from_parts(<E as Entity>::next_id(), Timestamp::now_utc(), None)
    }
}

impl<T, ID> MetaData<T, ID> {
    /// Creates a `MetaData` instance from given correlation ID, timestamp, and optional custom metadata.
    pub fn from_parts(
        correlation_id: Id<T, ID>,
        recv_timestamp: Timestamp,
        custom: Option<HashMap<String, String>>,
    ) -> Self {
        Self {
            correlation_id,
            recv_timestamp,
            custom: custom.unwrap_or_default(),
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn with_recv_timestamp(self, recv_timestamp: Timestamp) -> Self {
        Self {
            recv_timestamp,
            ..self
        }
    }

    #[allow(clippy::missing_const_for_fn)]
    pub fn into_parts(self) -> (Id<T, ID>, Timestamp, HashMap<String, String>) {
        (self.correlation_id, self.recv_timestamp, self.custom)
    }
}

impl<T, ID> MetaData<T, ID>
where
    ID: Clone,
{
    pub fn relabel<U: Label>(self) -> MetaData<U, ID> {
        MetaData {
            correlation_id: self.correlation_id.relabel(),
            recv_timestamp: self.recv_timestamp,
            custom: self.custom,
        }
    }
}

impl<T, ID> Correlation for MetaData<T, ID> {
    type Correlated = T;
    type IdType = ID;

    fn correlation(&self) -> &Id<Self::Correlated, Self::IdType> {
        &self.correlation_id
    }
}

impl<T, ID> ReceivedAt for MetaData<T, ID> {
    fn recv_timestamp(&self) -> Timestamp {
        self.recv_timestamp
    }
}

impl<T, ID> Clone for MetaData<T, ID>
where
    ID: Clone,
{
    fn clone(&self) -> Self {
        Self {
            correlation_id: self.correlation_id.clone(),
            recv_timestamp: self.recv_timestamp,
            custom: self.custom.clone(),
        }
    }
}

impl<T, ID> PartialEq for MetaData<T, ID>
where
    ID: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.correlation_id == other.correlation_id
    }
}

impl<T, ID> Eq for MetaData<T, ID> where ID: Eq {}

impl<T, ID> PartialOrd for MetaData<T, ID>
where
    ID: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.recv_timestamp.partial_cmp(&other.recv_timestamp)
    }
}

impl<T, ID> Ord for MetaData<T, ID>
where
    ID: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.recv_timestamp.cmp(&other.recv_timestamp)
    }
}

impl<T, ID> std::hash::Hash for MetaData<T, ID>
where
    ID: std::hash::Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.correlation_id.hash(state)
    }
}

impl<T, ID> std::ops::Add for MetaData<T, ID>
where
    ID: PartialOrd,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self < rhs {
            rhs
        } else {
            self
        }
    }
}

#[cfg(feature = "functional")]
impl<E> Monoid for MetaData<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: Entity,
    <<E as Entity>::IdGen as IdGenerator>::IdType: PartialOrd + Clone,
{
    fn empty() -> Self {
        Self::from_parts(<E as Entity>::next_id(), Timestamp::UNIX_EPOCH, None)
    }
}

#[cfg(feature = "functional")]
impl<E> Semigroup for MetaData<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: Entity,
    <<E as Entity>::IdGen as IdGenerator>::IdType: PartialOrd + Clone,
{
    fn combine(&self, other: &Self) -> Self {
        if self < other {
            other.clone()
        } else {
            self.clone()
        }
    }
}

impl<T, ID> From<MetaData<T, ID>> for HashMap<String, String>
where
    ID: fmt::Display,
{
    fn from(meta: MetaData<T, ID>) -> Self {
        let mut core = Self::with_capacity(2);
        core.insert(
            CORRELATION_ID_KEY.to_string(),
            meta.correlation_id.id.to_string(),
        );
        core.insert(
            RECV_TIMESTAMP_KEY.to_string(),
            meta.recv_timestamp.to_string(),
        );

        let mut result = meta.custom;
        result.extend(core);

        result
    }
}

const META_CORRELATION_ID: &str = "correlation_id";
const META_RECV_TIMESTAMP: &str = "recv_timestamp";
const META_CUSTOM: &str = "custom";
const FIELDS: [&str; 3] = [META_CORRELATION_ID, META_RECV_TIMESTAMP, META_CUSTOM];

impl<'de, T, ID> Deserialize<'de> for MetaData<T, ID>
where
    T: Label,
    ID: de::DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            CorrelationId,
            RecvTimestamp,
            Custom,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D0>(deserializer: D0) -> Result<Self, D0::Error>
            where
                D0: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl de::Visitor<'_> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        f.write_str("`correlation_id`, `recv_timestamp` or `custom`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            META_CORRELATION_ID => Ok(Self::Value::CorrelationId),
                            META_RECV_TIMESTAMP => Ok(Self::Value::RecvTimestamp),
                            META_CUSTOM => Ok(Self::Value::Custom),
                            _ => Err(de::Error::unknown_field(value, &FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct MetaVisitor<T0: Label, ID0> {
            marker: PhantomData<(T0, ID0)>,
        }

        impl<T0: Label, ID0> MetaVisitor<T0, ID0> {
            pub const fn new() -> Self {
                Self {
                    marker: PhantomData,
                }
            }
        }

        impl<'de, T0, ID0> de::Visitor<'de> for MetaVisitor<T0, ID0>
        where
            T0: Label,
            ID0: de::DeserializeOwned,
        {
            type Value = MetaData<T0, ID0>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(format!("struct MetaData<{}>", pretty_type_name::<T0>()).as_str())
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let correlation_id: Id<T0, ID0> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let recv_timestamp: Timestamp = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let custom: HashMap<String, String> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(MetaData::from_parts(
                    correlation_id,
                    recv_timestamp,
                    Some(custom),
                ))
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut correlation_id = None;
                let mut recv_timestamp = None;
                let mut custom = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::CorrelationId => {
                            if correlation_id.is_some() {
                                return Err(de::Error::duplicate_field(META_CORRELATION_ID));
                            }
                            correlation_id = Some(map.next_value()?);
                        }

                        Field::RecvTimestamp => {
                            if recv_timestamp.is_some() {
                                return Err(de::Error::duplicate_field(META_RECV_TIMESTAMP));
                            }
                            recv_timestamp = Some(map.next_value()?);
                        }

                        Field::Custom => {
                            if custom.is_some() {
                                return Err(de::Error::duplicate_field(META_CUSTOM));
                            }
                            custom = Some(map.next_value()?);
                        }
                    }
                }

                let correlation_id: Id<T0, ID0> =
                    correlation_id.ok_or_else(|| de::Error::missing_field(META_CORRELATION_ID))?;
                let recv_timestamp: Timestamp =
                    recv_timestamp.ok_or_else(|| de::Error::missing_field(META_RECV_TIMESTAMP))?;
                let custom: HashMap<String, String> =
                    custom.ok_or_else(|| de::Error::missing_field(META_CUSTOM))?;
                Ok(MetaData::from_parts(
                    correlation_id,
                    recv_timestamp,
                    Some(custom),
                ))
            }
        }

        deserializer.deserialize_struct("MetaData", &FIELDS, MetaVisitor::<T, ID>::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::Envelope;
    use crate::{Entity, Label, Labeling, MakeLabeling};
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use serde_test::Configure;
    use serde_test::{assert_tokens, Token};

    const METADATA_TS: &str = "2022-11-30T03:43:18.068Z";

    static META_DATA: Lazy<MetaData<TestData, String>> = Lazy::new(|| {
        let ts = Timestamp::parse(METADATA_TS).unwrap();
        MetaData::default().with_recv_timestamp(ts)
    });

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

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct TestData(i32);

    impl Entity for TestData {
        type IdGen = TestGenerator;
    }

    impl Label for TestData {
        type Labeler = MakeLabeling<Self>;

        fn labeler() -> Self::Labeler {
            MakeLabeling::default()
        }
    }

    #[derive(Debug, PartialEq)]
    struct TestContainer(TestData);

    impl Label for TestContainer {
        type Labeler = MakeLabeling<Self>;

        fn labeler() -> Self::Labeler {
            MakeLabeling::default()
        }
    }

    #[derive(Debug, PartialEq)]
    struct TestEnvelopeContainer(Envelope<TestData, String>);

    impl Label for TestEnvelopeContainer {
        type Labeler = MakeLabeling<Self>;

        fn labeler() -> Self::Labeler {
            MakeLabeling::default()
        }
    }

    #[test]
    fn test_envelope_map() {
        let data = TestData(13);

        let metadata = MetaData::from_parts(
            Id::direct(<TestData as Label>::labeler().label(), "zero".to_string()),
            Timestamp::now_utc(),
            None,
        );
        let enveloped_data = Envelope::from_parts(metadata.clone(), data);
        let expected = TestContainer(enveloped_data.clone().into_inner());
        let actual = enveloped_data.map(TestContainer);

        assert_eq!(
            actual.metadata().correlation().id,
            metadata.correlation().id
        );
        assert_eq!(
            actual.metadata().recv_timestamp(),
            metadata.recv_timestamp()
        );
        assert_eq!(actual.as_ref(), &expected);
    }

    #[test]
    fn test_envelope_flat_map() {
        let data = TestData(13);
        let mut custom = HashMap::default();
        custom.insert("cat".to_string(), "Otis".to_string());

        let metadata = MetaData::from_parts(
            Id::direct(<TestData as Label>::labeler().label(), "zero".to_string()),
            Timestamp::now_utc(),
            Some(custom),
        );
        let enveloped_data = Envelope::from_parts(metadata.clone(), data);
        let expected = TestEnvelopeContainer(enveloped_data.clone());
        let actual = enveloped_data.flat_map(TestEnvelopeContainer);

        assert_eq!(
            actual.metadata().correlation().id,
            metadata.correlation().id
        );
        assert_eq!(
            actual.metadata().recv_timestamp(),
            metadata.recv_timestamp()
        );
        assert_eq!(actual.as_ref(), &expected);
    }

    #[test]
    fn test_envelope_serde_tokens() {
        let data = TestData(17);
        let actual = Envelope::from_parts(META_DATA.clone(), data);
        static_assertions::assert_impl_all!(TestData: Serialize);
        static_assertions::assert_impl_all!(Envelope<TestData, String>: Serialize);

        assert_tokens(
            &actual.readable(),
            &vec![
                Token::Struct {
                    name: "Envelope",
                    len: 2,
                },
                Token::Str("metadata"),
                Token::Struct {
                    name: "MetaData",
                    len: 3,
                },
                Token::Str("correlation_id"),
                Token::Str(META_DATA.correlation_id.id.as_str()),
                Token::Str("recv_timestamp"),
                Token::Str(METADATA_TS),
                Token::Str("custom"),
                Token::Map { len: Some(0) },
                Token::MapEnd,
                Token::StructEnd,
                Token::Str("content"),
                Token::NewtypeStruct { name: "TestData" },
                Token::I32(17),
                Token::StructEnd,
            ],
        )
    }
}
