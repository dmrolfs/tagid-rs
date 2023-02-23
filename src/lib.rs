#![warn(clippy::cargo, clippy::nursery, future_incompatible, rust_2018_idioms)]
#![allow(clippy::multiple_crate_versions)]

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate tagid_derive;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use tagid_derive::*;

mod label;
mod labeling;

#[cfg(feature = "envelope")]
pub mod envelope;
mod id;

pub use id::{Entity, Id, IdGenerator};
pub use label::Label;
pub use labeling::{CustomLabeling, Labeling, MakeLabeling, NoLabeling};

#[cfg(feature = "cuid")]
pub use id::CuidGenerator;

#[cfg(feature = "uuid")]
pub use id::UuidGenerator;

#[cfg(feature = "snowflake")]
pub use id::snowflake::{pretty, MachineNode, SnowflakeGenerator};
