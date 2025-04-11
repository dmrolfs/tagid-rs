use super::*;
use serde::{Deserialize, Serialize};

/// A type alias for a ULID-based identifier wrapped in the `Id` struct.
///
/// This is useful when IDs are represented as `Id<T, Ulid>`, where `T` is the entity type.
#[allow(dead_code)]
pub type UlidId<T> = Id<T, Ulid>;

#[derive(
    Debug, Default, PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize,
)]
pub struct Ulid(::ulid::Ulid);

impl Ulid {
    pub fn new() -> Self {
        let ulid = ::ulid::Ulid::new();
        Self::from_ulid(ulid)
    }

    pub const fn from_ulid(ulid: ::ulid::Ulid) -> Self {
        Self(ulid)
    }

    pub const fn from_parts(timestamp_ms: u64, random: u128) -> Self {
        let ulid = ::ulid::Ulid::from_parts(timestamp_ms, random);
        Self::from_ulid(ulid)
    }

    pub fn with_source<R: rand::Rng>(source: &mut R) -> Self {
        let ulid = ::ulid::Ulid::with_source(source);
        Self::from_ulid(ulid)
    }

    pub fn from_datetime(datetime: std::time::SystemTime) -> Self {
        let ulid = ::ulid::Ulid::from_datetime(datetime);
        Self::from_ulid(ulid)
    }

    pub fn from_datetime_with_source<R>(datetime: std::time::SystemTime, source: &mut R) -> Self
    where
        R: rand::Rng + ?Sized,
    {
        let ulid = ::ulid::Ulid::from_datetime_with_source(datetime, source);
        Self::from_ulid(ulid)
    }

    pub const fn from_string(encoded: &str) -> Result<Self, ::ulid::DecodeError> {
        match ::ulid::Ulid::from_string(encoded) {
            Ok(ulid) => Ok(Self::from_ulid(ulid)),
            Err(err) => Err(err),
        }
    }

    pub const fn nil() -> Self {
        Self::from_ulid(::ulid::Ulid::nil())
    }

    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self::from_ulid(::ulid::Ulid::from_bytes(bytes))
    }

    pub fn into_inner(self) -> ::ulid::Ulid {
        self.0
    }
}

impl AsRef<::ulid::Ulid> for Ulid {
    fn as_ref(&self) -> &::ulid::Ulid {
        &self.0
    }
}

impl std::ops::Deref for Ulid {
    type Target = ::ulid::Ulid;

    fn deref(&self) -> &::ulid::Ulid {
        &self.0
    }
}

impl From<Ulid> for String {
    fn from(ulid: Ulid) -> String {
        ulid.to_string()
    }
}

impl From<(u64, u64)> for Ulid {
    fn from((msb, lsb): (u64, u64)) -> Self {
        Self::from_ulid((msb, lsb).into())
    }
}

impl From<Ulid> for (u64, u64) {
    fn from(ulid: Ulid) -> Self {
        ulid.into_inner().into()
    }
}

impl From<u128> for Ulid {
    fn from(u: u128) -> Self {
        Self::from_ulid(u.into())
    }
}

impl From<Ulid> for u128 {
    fn from(ulid: Ulid) -> Self {
        ulid.into_inner().into()
    }
}

impl From<[u8; 16]> for Ulid {
    fn from(bytes: [u8; 16]) -> Self {
        Self::from_ulid(bytes.into())
    }
}

impl From<Ulid> for [u8; 16] {
    fn from(ulid: Ulid) -> Self {
        ulid.into_inner().into()
    }
}

impl std::str::FromStr for Ulid {
    type Err = ::ulid::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ulid = ::ulid::Ulid::from_str(s)?;
        Ok(Self::from_ulid(ulid))
    }
}

impl TryFrom<&'_ str> for Ulid {
    type Error = ::ulid::DecodeError;

    fn try_from(s: &'_ str) -> Result<Self, Self::Error> {
        let ulid = ::ulid::Ulid::from_string(s)?;
        Ok(Self::from_ulid(ulid))
    }
}

impl std::fmt::Display for Ulid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "sqlx")]
impl<DB> ::sqlx::Type<DB> for Ulid
where
    DB: ::sqlx::Database,
    ::sqlx::types::Uuid: ::sqlx::Type<DB>,
{
    fn type_info() -> <DB as ::sqlx::Database>::TypeInfo {
        <::sqlx::types::Uuid as ::sqlx::Type<DB>>::type_info()
    }
}

#[cfg(feature = "sqlx")]
impl ::sqlx::postgres::PgHasArrayType for Ulid {
    fn array_type_info() -> ::sqlx::postgres::PgTypeInfo {
        <::sqlx::types::Uuid as ::sqlx::postgres::PgHasArrayType>::array_type_info()
    }
}

#[cfg(feature = "sqlx")]
impl From<Ulid> for ::sqlx::types::Uuid {
    fn from(ulid: Ulid) -> Self {
        let rep: u128 = ulid.into();
        ::sqlx::types::Uuid::from_u128(rep)
    }
}

#[cfg(feature = "sqlx")]
impl From<::sqlx::types::Uuid> for Ulid {
    fn from(uuid: ::sqlx::types::Uuid) -> Self {
        Ulid::from(uuid.as_u128())
    }
}

#[cfg(feature = "sqlx")]
impl<'q, DB> ::sqlx::Encode<'q, DB> for Ulid
where
    DB: ::sqlx::Database,
    String: ::sqlx::Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as ::sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<::sqlx::encode::IsNull, ::sqlx::error::BoxDynError> {
        let rep = self.to_string();
        // rep.encode_by_ref(buf)
        <String as ::sqlx::Encode<'q, DB>>::encode_by_ref(&rep, buf)
    }
}

#[cfg(feature = "sqlx")]
impl<'q, DB> ::sqlx::decode::Decode<'q, DB> for Ulid
where
    DB: ::sqlx::Database,
    for<'a> &'a str: ::sqlx::Decode<'q, DB>,
{
    fn decode(
        value: <DB as ::sqlx::Database>::ValueRef<'q>,
    ) -> Result<Self, ::sqlx::error::BoxDynError> {
        let rep = <&str as ::sqlx::decode::Decode<DB>>::decode(value)?;
        Self::from_string(rep).map_err(Into::into)
    }
}

pub struct UlidGenerator;

impl IdGenerator for UlidGenerator {
    type IdType = Ulid;

    fn next_id_rep() -> Self::IdType {
        Ulid::new()
    }
}
