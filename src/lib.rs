#![warn(clippy::cargo, clippy::nursery, future_incompatible, rust_2018_idioms)]

mod label;
mod labeling;

#[cfg(feature = "envelope")]
pub mod envelope;
mod id;

pub use id::{CuidGenerator, Entity, Id, SnowflakeGenerator, UuidGenerator};
pub use label::Label;
pub use labeling::{CustomLabeling, Labeling, MakeLabeling, NoLabeling};
