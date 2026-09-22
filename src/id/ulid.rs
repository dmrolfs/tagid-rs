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
    String: ::sqlx::Type<DB>,
{
    fn type_info() -> <DB as ::sqlx::Database>::TypeInfo {
        <String as ::sqlx::Type<DB>>::type_info()
    }
}

#[cfg(feature = "sqlx")]
impl ::sqlx::postgres::PgHasArrayType for Ulid {
    fn array_type_info() -> ::sqlx::postgres::PgTypeInfo {
        <String as ::sqlx::postgres::PgHasArrayType>::array_type_info()
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
        // Route through `Ulid`'s own canonical base32 text (`Display`/`to_string`), matching
        // what `Type::type_info()` above now declares -- a `text`-shaped column storing the
        // ULID's own lexically-sortable, human-readable rendering.
        //
        // Corrected 2026-09-22 from an intermediate version (1.2.1) that instead routed through
        // `sqlx::types::Uuid`, declaring the column type as Postgres `uuid`: that fixed an
        // internal Type/Encode inconsistency, but on reflection was the wrong side of that
        // inconsistency to fix. Reusing `uuid`'s fixed 16-byte slot is lossless and
        // DB-sort-preserving (verified below), but it is not a real RFC 4122 UUID -- the
        // version/variant bits are never set, since this simply reinterprets the ULID's raw 128
        // bits -- and it discards the human-readable, lexically-sortable string form that is
        // this type's entire reason to exist over a plain random UUID. A `Ulid` is not a `Uuid`
        // wearing a different hat; it is its own identifier shape with its own canonical text
        // encoding, and its storage type should say so.
        let encoded: String = self.to_string();
        <String as ::sqlx::Encode<'q, DB>>::encode_by_ref(&encoded, buf)
    }
}

#[cfg(feature = "sqlx")]
impl<'q, DB> ::sqlx::decode::Decode<'q, DB> for Ulid
where
    DB: ::sqlx::Database,
    String: ::sqlx::decode::Decode<'q, DB>,
{
    fn decode(
        value: <DB as ::sqlx::Database>::ValueRef<'q>,
    ) -> Result<Self, ::sqlx::error::BoxDynError> {
        // The paired half of the `Encode` fix above: decode the `text`-shaped wire value this
        // column actually holds, then parse it via `Self::from_string`, which expects exactly
        // this canonical base32 form. `ulid::DecodeError` implements `std::error::Error`, so `?`
        // converts it into `BoxDynError` via that trait's own blanket `From` impl.
        let encoded = <String as ::sqlx::decode::Decode<DB>>::decode(value)?;
        Ok(Ulid::from_string(&encoded)?)
    }
}

pub struct UlidGenerator;

impl IdGenerator for UlidGenerator {
    type IdType = Ulid;

    fn next_id_rep() -> Self::IdType {
        Ulid::new()
    }
}

#[cfg(all(test, feature = "sqlx"))]
mod tests {
    use super::*;

    // These test the pure Rust conversions the `Encode`/`Decode` impls above delegate to --
    // `<String as sqlx::Encode/Decode>` is sqlx's own, already-tested implementation, so there is
    // nothing left to test at the wire-protocol level here; what this crate owns and must prove
    // is that `Ulid -> String -> Ulid` is lossless and that the canonical base32 string's own
    // lexical ordering matches `Ulid`'s own `Ord` -- sortability is this type's whole reason for
    // existing over a plain `Uuid`, and it is exactly this property a `uuid`-backed storage path
    // could not honestly claim (see the `Encode` impl's own doc comment for why that path was
    // reverted).

    #[test]
    fn test_ulid_to_string_round_trip_is_lossless() {
        let original = Ulid::new();
        let encoded = original.to_string();
        let round_tripped =
            Ulid::from_string(&encoded).expect("a Ulid's own Display output always parses");
        assert_eq!(original, round_tripped);
    }

    #[test]
    fn test_ulid_to_string_round_trip_is_lossless_for_a_fixed_value() {
        // A fixed, non-random value alongside the random one above: a round-trip bug tied to a
        // specific bit pattern would not necessarily show up on every random draw.
        let original = Ulid::from_parts(1_700_000_000_000, 0x0123_4567_89ab_cdef);
        let encoded = original.to_string();
        let round_tripped =
            Ulid::from_string(&encoded).expect("a Ulid's own Display output always parses");
        assert_eq!(original, round_tripped);
    }

    #[test]
    fn test_ulid_ordering_survives_the_string_round_trip() {
        // Sortability is this type's own reason for existing over a plain random UUID. ULID's
        // canonical base32 encoding is specifically designed so that lexical (byte-for-byte)
        // string comparison matches numeric/chronological ordering -- unlike a UUID's
        // hex-dashed rendering, which does not preserve any embedded timestamp's own order in
        // its canonical *text* form. This is the property a `uuid`-typed storage path discarded
        // and this type's storage path must actually keep, since a caller running
        // `ORDER BY <ulid column>` in raw SQL depends on it, not only a caller comparing
        // already-decoded `Ulid` values in Rust.
        let earlier = Ulid::from_parts(1_700_000_000_000, 0);
        let later = Ulid::from_parts(1_700_000_000_001, 0);
        assert!(
            earlier < later,
            "precondition: Ulid's own Ord must already order these two"
        );

        let earlier_encoded = earlier.to_string();
        let later_encoded = later.to_string();
        assert!(
            earlier_encoded < later_encoded,
            "the canonical base32 string's own lexical order must preserve Ulid's own \
             chronological order"
        );
    }

    #[test]
    fn test_encode_declares_the_same_column_type_string_itself_would() {
        // `Type::type_info()` for `Ulid` must equal `String`'s own -- the property the previous
        // (reverted) `uuid`-backed version violated in the opposite direction from the original
        // 1.2.0 bug: that version's `Type` and `Encode`/`Decode` agreed with each other, but on a
        // storage shape (`uuid`) that discarded this type's own defining property. Compared
        // against sqlx's Postgres backend directly, the one this crate actually ships support
        // for.
        let ulid_type = <Ulid as ::sqlx::Type<::sqlx::Postgres>>::type_info();
        let string_type = <String as ::sqlx::Type<::sqlx::Postgres>>::type_info();
        assert_eq!(ulid_type, string_type);
    }
}
