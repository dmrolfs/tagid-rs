#[allow(clippy::module_inception)]
mod envelope;
mod metadata;

pub use envelope::{Envelope, IntoEnvelope};
pub use metadata::{IntoMetaData, MetaData};

use crate::Id;
use iso8601_timestamp::Timestamp;

/// Type has correlation identifier.
pub trait Correlation {
    type Correlated: Sized + Sync;
    type IdType;

    fn correlation(&self) -> &Id<Self::Correlated, Self::IdType>;
}

/// Type has received at timestamp.
pub trait ReceivedAt {
    fn recv_timestamp(&self) -> Timestamp;
}
