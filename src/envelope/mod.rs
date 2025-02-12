//! Provides types and traits for structuring enveloped data with metadata.
//!
//! The `envelope` module defines abstractions for working with data that includes
//! additional metadata, such as correlation identifiers and timestamps. This can
//! be useful for event-driven systems, logging, and message passing.
//!
//! ## Features
//!
//! This module is enabled via the `"envelope"` feature in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tagid = { version = "0.2", features = ["envelope"] }
//! ```
//!
//! ## Overview
//!
//! - [`Envelope`](Envelope): A wrapper that encapsulates an entity with metadata.
//! - [`IntoEnvelope`](IntoEnvelope): A trait for converting entities into envelopes.
//! - [`MetaData`](MetaData): Stores additional metadata associated with an entity.
//! - [`Correlation`](Correlation): Defines a correlation ID for tracking related entities.
//! - [`ReceivedAt`](ReceivedAt): Ensures an entity has a timestamp indicating when it was received.

#[allow(clippy::module_inception)]
mod envelope;
mod metadata;

pub use envelope::{Envelope, IntoEnvelope};
pub use metadata::{IntoMetaData, MetaData};

use crate::Id;
use iso8601_timestamp::Timestamp;

/// Defines a correlation identifier for tracking related entities.
///
/// This trait is typically implemented by messages or events that belong
/// to a larger workflow or transaction.
///
/// # Example
///
/// ```rust
/// use tagid::{Id, envelope::Correlation};
///
/// struct Event {
///     correlation_id: Id<Event, String>,
/// }
///
/// impl Correlation for Event {
///     type Correlated = Event;
///     type IdType = String;
///
///     fn correlation(&self) -> &Id<Self::Correlated, Self::IdType> {
///         &self.correlation_id
///     }
/// }
/// ```
pub trait Correlation {
    /// The type being correlated.
    type Correlated: Sized;

    /// The type used as the correlation identifier.
    type IdType;

    /// Returns a reference to the correlation ID.
    fn correlation(&self) -> &Id<Self::Correlated, Self::IdType>;
}

/// Provides a timestamp indicating when an entity was received.
///
/// This trait is useful in event-driven systems where it is important
/// to track when data was received.
///
/// # Example
///
/// ```rust, ignore
/// use tagid::envelope::ReceivedAt;
/// use iso8601_timestamp::Timestamp;
///
/// struct Message {
///     timestamp: Timestamp,
/// }
///
/// impl ReceivedAt for Message {
///     fn recv_timestamp(&self) -> Timestamp {
///         self.timestamp
///     }
/// }
/// ```
pub trait ReceivedAt {
    /// Returns the timestamp indicating when the entity was received.
    fn recv_timestamp(&self) -> Timestamp;
}
