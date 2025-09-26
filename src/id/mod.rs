//! The `id` module provides a generic mechanism for creating, representing, and managing entity
//! identifiers (`Id`). It supports multiple ID generation strategies (e.g., CUID, UUID, Snowflake)
//! and integrates with external libraries such as `serde`, `sqlx`, and `disintegrate`.
//!
//! # Overview
//!
//! - [`Entity`] Trait: Defines an entity with an associated [`IdGenerator`] for unique ID generation.
//! - [`Id<T, ID>`] Struct: Represents an ID associated with an entity type `T` and an ID value of type `ID`.
//! - ID Generation Strategies:
//!   - **CUID** ([`CuidGenerator`], [`CuidId`]) - Enabled with the `with-cuid` feature.
//!   - **ULID** ([`UlidGenerator`]) - Enabled with the `with-ulid` feature.
//!   - **UUID** ([`UuidGenerator`]) - Enabled with the `with-uuid` feature.
//!   - **Snowflake** ([`SnowflakeGenerator`]) - Enabled with the `with-snowflake` feature.
//!
//! ## Features
//!
//! This module integrates with:
//!
//! - **Serde** (`serde` feature): Implements [`Serialize`] and [`Deserialize`] for `Id`.
//! - **SQLx** (`sqlx` feature): Enables database encoding/decoding via [`sqlx::Decode`], [`sqlx::Encode`], and [`sqlx::Type`].
//! - **Disintegrate** (`disintegrate` feature): Supports [`IntoIdentifierValue`] for identifier-based systems.
//!
//! ## Example Usage
//!
//! ### Defining an Entity with ID Generation
//!
//! ```rust,ignore
//! use tagid::{Entity, Id, Label};
//!
//! #[derive(Label)]
//! struct User;
//!
//! impl Entity for User {
//!     type IdGen = tagid::UuidGenerator;
//! }
//!
//! let user_id = User::next_id();
//! println!("Generated User ID: {}", user_id);
//! ```
//!
//! # ID Generation Strategies
//!
//! The module supports multiple ID generation strategies that can be enabled via Cargo features:
//!
//! | Feature           | Generator               | Description                                      |
//! |-------------------|-------------------------|--------------------------------------------------|
//! | `"with-cuid"`     | [`CuidGenerator`]       | Generates CUID-based IDs.                        |
//! | `"with-ulid"`     | [`UlidGenerator`]       | Generates ULID-based IDs.                        |
//! | `"with-uuid"`     | [`UuidGenerator`]       | Generates UUID-based IDs.                        |
//! | `"with-snowflake"`| [`SnowflakeGenerator`]  | Uses the Snowflake algorithm for distributed ID generation. |
//!
//! To enable a specific ID generation method, add the corresponding feature to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! tagid = { version = "0.2", features = ["with-uuid", "with-snowflake"] }
//! ```

mod generator;
pub use generator::IdGenerator;

#[cfg(feature = "with-cuid")]
pub mod cuid;
#[cfg(feature = "with-cuid")]
pub use cuid::{CuidGenerator, CuidId};

#[cfg(feature = "with-ulid")]
pub mod ulid;
#[cfg(feature = "with-ulid")]
#[allow(unused_imports)]
pub use ulid::{Ulid, UlidGenerator, UlidId};

#[cfg(feature = "with-uuid")]
mod uuid;
#[cfg(feature = "with-uuid")]
#[allow(unused_imports)]
pub use uuid::{UuidGenerator, UuidId};

#[cfg(feature = "with-snowflake")]
pub mod snowflake;
#[cfg(feature = "with-snowflake")]
#[allow(unused_imports)]
pub use self::snowflake::{MachineNode, SnowflakeGenerator, pretty};

mod identifier;
pub use identifier::Id;

use crate::Label;

/// A trait for entities that have a unique identifier.
///
/// Implementing this trait allows an entity type to generate new unique IDs
/// using its associated [`IdGenerator`].
pub trait Entity: Label {
    /// The ID generator type used to create unique IDs.
    type IdGen: IdGenerator;

    /// Generates a new unique ID for the entity.
    fn next_id() -> Id<Self, <Self::IdGen as IdGenerator>::IdType> {
        Id::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Labeling;
    use crate::id::ulid::Ulid;
    use crate::{CustomLabeling, MakeLabeling, NoLabeling};
    use assert_matches2::assert_let;
    use pretty_assertions::assert_eq;
    use serde_test::{Token, assert_tokens};
    use static_assertions::assert_impl_all;

    #[test]
    fn test_auto_traits() {
        assert_impl_all!(Id<u32, u32>: Send, Sync);
        assert_impl_all!(Id<std::rc::Rc<u32>, String>: Send, Sync);
    }

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

    struct Bar;
    impl Label for Bar {
        type Labeler = MakeLabeling<Self>;

        fn labeler() -> Self::Labeler {
            MakeLabeling::default()
        }
    }

    struct NoLabelZed;

    impl Label for NoLabelZed {
        type Labeler = NoLabeling;

        fn labeler() -> Self::Labeler {
            NoLabeling
        }
    }

    struct Foo;

    impl Entity for Foo {
        type IdGen = TestGenerator;
    }

    impl Label for Foo {
        type Labeler = CustomLabeling;

        fn labeler() -> Self::Labeler {
            CustomLabeling::new("MyFooferNut")
        }
    }

    #[test]
    fn test_display() {
        let a: Id<Foo, String> = Foo::next_id();
        assert_eq!(format!("{a}"), format!("MyFooferNut::{}", a.id));
    }

    #[test]
    fn test_alternate_display() {
        let a: Id<Foo, i64> = Id::direct(Foo::labeler().label(), 13);
        assert_eq!(format!("{a:#}"), a.id.to_string());

        let id = 98734021;
        let a: Id<Foo, u64> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(format!("{a:#}"), a.id.to_string());

        #[cfg(feature = "with-uuid")]
        {
            let id = ::uuid::Uuid::new_v4();
            let a: Id<Foo, ::uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
            assert_eq!(format!("{a:#}"), a.id.to_string());
        }
    }

    #[test]
    fn test_debug() {
        let a: Id<Foo, String> = Foo::next_id();
        assert_eq!(format!("{a:?}"), format!("MyFooferNut::{:?}", a.id));

        let id = 98734021;
        let a: Id<Foo, u64> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(format!("{a:?}"), format!("MyFooferNut::{:?}", a.id));

        #[cfg(feature = "with-uuid")]
        {
            let id = ::uuid::Uuid::new_v4();
            let a: Id<Foo, ::uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
            assert_eq!(format!("{a:?}"), format!("MyFooferNut::{:?}", a.id));
        }
    }

    #[test]
    fn test_alternate_debug() {
        let a: Id<Foo, String> = Foo::next_id();
        assert_eq!(
            format!("{a:#?}"),
            format!(
                "Id {{\n    label: \"{}\",\n    id: \"{}\",\n}}",
                a.label, a.id,
            )
        );

        let id = 98734021;
        let a: Id<Foo, u64> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(
            format!("{a:#?}"),
            format!("Id {{\n    label: \"{}\",\n    id: {},\n}}", a.label, a.id,)
        );

        #[cfg(feature = "with-uuid")]
        {
            let id = ::uuid::Uuid::new_v4();
            let a: Id<Foo, ::uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
            assert_eq!(
                format!("{a:#?}"),
                format!("Id {{\n    label: \"{}\",\n    id: {},\n}}", a.label, a.id,)
            );
        }
    }

    #[test]
    fn test_id_cross_conversion() {
        let a = Foo::next_id();
        let before = format!("{}", a);
        assert_eq!(format!("MyFooferNut::{}", a.id), before);

        let b: Id<NoLabelZed, String> = a.relabel();
        let after_zed = format!("{}", b);
        assert_eq!(format!("{}", a.id), after_zed);

        let c: Id<Bar, String> = a.relabel();
        let after_bar = format!("{}", c);
        assert_eq!(format!("Bar::{}", a.id), after_bar);
    }

    #[test]
    fn test_id_serde_tokens() {
        let labeler = <Foo as Label>::labeler();
        let cuid = "ig6wv6nezj0jg51lg53dztqy".to_string();
        let id = Id::<Foo, String>::direct(labeler.label(), cuid);
        assert_tokens(&id, &[Token::Str("ig6wv6nezj0jg51lg53dztqy")]);

        let id = Id::<Foo, u64>::direct(labeler.label(), 17);
        assert_tokens(&id, &[Token::U64(17)]);
    }

    #[test]
    fn test_id_serde_json() {
        let labeler = <Foo as Label>::labeler();

        let cuid = "ig6wv6nezj0jg51lg53dztqy".to_string();
        let id = Id::<Foo, String>::direct(labeler.label(), cuid);
        assert_let!(Ok(json) = serde_json::to_string(&id));
        assert_let!(Ok(actual) = serde_json::from_str::<Id<Foo, String>>(&json));
        assert_eq!(actual, id);

        let id = Id::<Foo, u64>::direct(labeler.label(), 17);
        assert_let!(Ok(json) = serde_json::to_string(&id));
        assert_let!(Ok(actual) = serde_json::from_str::<Id<Foo, u64>>(&json));
        assert_eq!(actual, id);

        #[cfg(feature = "with-ulid")]
        {
            let ulid = Ulid::new();
            let id = crate::id::ulid::UlidId::<Foo>::direct(labeler.label(), ulid);
            assert_let!(Ok(json) = serde_json::to_string(&id));
            assert_let!(Ok(actual) = serde_json::from_str::<Id<Foo, Ulid>>(&json));
            assert_eq!(actual, id);
        }
    }
}
