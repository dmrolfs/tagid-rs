//! Pretty-printed Snowflake ID module.
//!
//! This module provides a `PrettySnowflakeId` type that encodes Snowflake IDs
//! into a human-readable format using an `IdPrettifier`. It also includes
//! `PrettySnowflakeGenerator`, an ID generator that produces `PrettySnowflakeId` values.

mod codec;
mod damm;
mod prettifier;

pub use codec::{Alphabet, AlphabetCodec, Codec, BASE_23};
pub use prettifier::{ConversionError, IdPrettifier};

use crate::id::IdGenerator;
use crate::SnowflakeGenerator;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use std::fmt;

/// A human-readable representation of a Snowflake ID.
///
/// `PrettySnowflakeId` wraps a `SmolStr`, which contains an encoded form of a
/// standard Snowflake ID. It is primarily used for easier readability and user
/// interaction while retaining the ability to convert back to a numeric Snowflake ID.
///
/// # Examples
///
/// ```rust
/// use pretty_assertions::assert_eq;
/// use tagid::snowflake::pretty::{AlphabetCodec, IdPrettifier, PrettySnowflakeId, BASE_23};
/// IdPrettifier::<AlphabetCodec>::global_initialize(BASE_23.clone());
///
/// let snowflake_id: i64 = 123456789012345;
/// let pretty_id = PrettySnowflakeId::from_snowflake(snowflake_id);
///
/// assert_eq!("AAAB-23456-GMDM-23450".to_string(), pretty_id.to_string());
/// ```
#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct PrettySnowflakeId(SmolStr);

impl PrettySnowflakeId {
    /// Converts a numeric Snowflake ID into a `PrettySnowflakeId`.
    ///
    /// # Parameters
    /// - `snowflake`: The `i64` Snowflake ID to be encoded.
    ///
    /// # Returns
    /// A `PrettySnowflakeId` containing a human-readable representation.
    ///
    /// # Example
    /// ```rust, ignore
    /// use tagid::snowflake::pretty::PrettySnowflakeId;
    ///
    /// let id = PrettySnowflakeId::from_snowflake(987654321);
    /// println!("Pretty ID: {}", id);
    /// ```
    pub fn from_snowflake(snowflake: i64) -> Self {
        let pretty_id = encoder().prettify(snowflake);
        Self(pretty_id.into())
    }
}

/// Retrieves the global `IdPrettifier` for encoding and decoding Snowflake IDs.
#[inline]
fn encoder() -> &'static IdPrettifier<AlphabetCodec> {
    IdPrettifier::<AlphabetCodec>::summon()
}

impl fmt::Debug for PrettySnowflakeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_tuple("PrettySnowflakeId").field(&self.0).finish()
        } else {
            write!(f, "PrettySnowflakeId({})", self.0)
        }
    }
}

impl fmt::Display for PrettySnowflakeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for PrettySnowflakeId {
    /// Returns the inner string representation of the `PrettySnowflakeId`.
    ///
    /// This method provides access to the underlying id value as a `&str`.
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl std::ops::Deref for PrettySnowflakeId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl From<PrettySnowflakeId> for String {
    fn from(id: PrettySnowflakeId) -> Self {
        id.0.to_string()
    }
}

impl From<PrettySnowflakeId> for i64 {
    /// Converts `PrettySnowflakeId` back into a numeric Snowflake ID.
    ///
    /// # Panics
    /// Panics if the `PrettySnowflakeId` cannot be converted back into a valid
    /// `i64` Snowflake ID.
    fn from(id: PrettySnowflakeId) -> Self {
        encoder()
            .to_id_seed(&id)
            .expect("failed to convert pretty id into snowflake i64")
    }
}

/// A generator that produces `PrettySnowflakeId` values.
///
/// This struct implements the `IdGenerator` trait and wraps the standard
/// `SnowflakeGenerator`, encoding generated IDs into a human-readable form.
///
/// # Example
///
/// ```rust, ignore
/// use tagid::IdGenerator;
/// use tagid::snowflake::pretty::{AlphabetCodec, IdPrettifier, PrettySnowflakeGenerator, BASE_23};
/// IdPrettifier::<AlphabetCodec>::global_initialize(BASE_23.clone());
///
/// let pretty_id = PrettySnowflakeGenerator::next_id_rep();
/// println!("Generated Pretty ID: {}", pretty_id);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct PrettySnowflakeGenerator;

impl IdGenerator for PrettySnowflakeGenerator {
    type IdType = PrettySnowflakeId;

    /// Generates a new `PrettySnowflakeId`.
    ///
    /// Uses `SnowflakeGenerator` to produce a numeric Snowflake ID, which is
    /// then converted into a pretty-printed form.
    ///
    /// # Returns
    /// A `PrettySnowflakeId` representing a unique identifier.
    fn next_id_rep() -> Self::IdType {
        let snowflake = SnowflakeGenerator::next_id_rep();
        PrettySnowflakeId::from_snowflake(snowflake)
    }
}
