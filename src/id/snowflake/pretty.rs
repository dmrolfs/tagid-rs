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

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct PrettySnowflakeId(SmolStr);

impl PrettySnowflakeId {
    pub fn from_snowflake(snowflake: i64) -> Self {
        let pretty_id = encoder().prettify(snowflake);
        Self(pretty_id.into())
    }
}

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
    fn from(id: PrettySnowflakeId) -> Self {
        encoder()
            .to_id_seed(&id)
            .expect("failed to convert pretty id into snowflake i64")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PrettySnowflakeGenerator;

impl IdGenerator for PrettySnowflakeGenerator {
    type IdType = PrettySnowflakeId;

    fn next_id_rep() -> Self::IdType {
        let snowflake = SnowflakeGenerator::next_id_rep();
        PrettySnowflakeId::from_snowflake(snowflake)
    }
}
