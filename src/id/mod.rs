mod generator;
pub use generator::IdGenerator;

#[cfg(feature = "cuid")]
pub mod cuid;
#[cfg(feature = "cuid")]
pub use cuid::{CuidGenerator, CuidId};

#[cfg(feature = "ulid")]
pub mod ulid;
#[cfg(feature = "ulid")]
#[allow(unused_imports)]
pub use ulid::{Ulid, UlidGenerator, UlidId};

#[cfg(feature = "uuid")]
mod uuid;
#[cfg(feature = "uuid")]
#[allow(unused_imports)]
pub use uuid::{UuidGenerator, UuidId};

#[cfg(feature = "snowflake")]
pub mod snowflake;
#[cfg(feature = "snowflake")]
#[allow(unused_imports)]
pub use self::snowflake::{MachineNode, SnowflakeGenerator, pretty};

mod identifier;
pub use identifier::Id;

pub mod labeled;
pub use labeled::{LabelMode, Labeled};
pub mod provenance;
pub mod sourced;
#[allow(unused_imports)]
pub use provenance::{
    AliasOf, ClientProvided, Derived, External, Generated, Imported, LabelPolicy, Provenance,
    Scoped, Temporary, providers, strategies,
};
#[allow(unused_imports)]
pub use sourced::Sourced;

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
    #[cfg(feature = "ulid")]
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
        // Display is ALWAYS canonical (no labels)
        assert_eq!(format!("{a}"), a.id.to_string());
    }

    #[test]
    fn test_alternate_display() {
        let a: Id<Foo, i64> = Id::direct(Foo::labeler().label(), 13);
        assert_eq!(format!("{a:#}"), a.id.to_string());

        let id = 98734021;
        let a: Id<Foo, u64> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(format!("{a:#}"), a.id.to_string());

        #[cfg(feature = "uuid")]
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

        #[cfg(feature = "uuid")]
        {
            let id = ::uuid::Uuid::new_v4();
            let a: Id<Foo, ::uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
            assert_eq!(format!("{a:?}"), format!("MyFooferNut::{:?}", a.id));
        }
    }

    #[test]
    fn test_alternate_debug() {
        let a: Id<Foo, String> = Foo::next_id();
        let debug_str = format!("{a:#?}");
        assert!(debug_str.contains("Id {"));
        assert!(debug_str.contains(a.label.as_str()));
        assert!(debug_str.contains(&a.id));

        let id = 98734021;
        let a: Id<Foo, u64> = Id::direct(Foo::labeler().label(), id);
        let debug_str = format!("{a:#?}");
        assert!(debug_str.contains("Id {"));
        assert!(debug_str.contains(a.label.as_str()));
        assert!(debug_str.contains(&format!("{}", a.id)));

        #[cfg(feature = "uuid")]
        {
            let id = ::uuid::Uuid::new_v4();
            let a: Id<Foo, ::uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
            let debug_str = format!("{a:#?}");
            assert!(debug_str.contains("Id {"));
            assert!(debug_str.contains(a.label.as_str()));
            assert!(debug_str.contains(&format!("{}", a.id)));
        }
    }

    #[test]
    fn test_equality_ignores_label() {
        let id1: Id<Foo, String> = Id::direct("label1", "123".into());
        let id2: Id<Foo, String> = Id::direct("label2", "123".into());
        assert_eq!(id1, id2);

        let id3: Id<Foo, String> = Id::direct("label1", "456".into());
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_hash_matches_equality() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let id1: Id<Foo, String> = Id::direct("a", "123".to_string());
        let id2: Id<Foo, String> = Id::direct("b", "123".to_string());

        let hash1 = {
            let mut h = DefaultHasher::new();
            id1.hash(&mut h);
            h.finish()
        };

        let hash2 = {
            let mut h = DefaultHasher::new();
            id2.hash(&mut h);
            h.finish()
        };

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_ordering_ignores_label() {
        let a: Id<Foo, u64> = Id::direct("z", 1u64);
        let b: Id<Foo, u64> = Id::direct("a", 2u64);
        assert!(a < b); // Ordered by ID value, not label

        let c: Id<Foo, u64> = Id::direct("m", 1u64);
        assert_eq!(a, c); // Same ID → equal despite label difference
    }

    #[test]
    fn test_from_id_uses_entity_labeler() {
        // From<ID> should use T::labeler(), not create empty label
        let id: Id<Foo, String> = "abc".to_string().into();
        assert_eq!(id.label, Foo::labeler().label());
        assert_eq!(id.id, "abc");
    }

    #[test]
    fn test_as_str_returns_id_only() {
        let id: Id<Foo, String> = Id::direct("IGNORED", "value".to_string());
        assert_eq!(id.as_str(), "value");
        assert_ne!(id.as_str(), "IGNORED::value"); // No label in result
    }

    #[test]
    fn test_clone_correctness() {
        let original: Id<Foo, String> = Id::direct("MyLabel", "id-value".to_string());
        let cloned = original.clone();

        assert_eq!(original, cloned);
        assert_eq!(original.label, cloned.label);
        assert_eq!(original.id, cloned.id);
    }

    #[test]
    fn test_into_inner_correctness() {
        let original_value = "test-id".to_string();
        let id: Id<Foo, String> = Id::direct("_", original_value.clone());

        let extracted = id.into_inner();
        assert_eq!(extracted, original_value);
    }

    #[test]
    fn test_id_cross_conversion() {
        let a = Foo::next_id();

        // Display is canonical (no labels)
        let display_a = format!("{a}");
        assert_eq!(a.id.to_string(), display_a); // Canonical = just ID

        // Debug is labeled (includes entity context)
        let debug_a = format!("{a:?}");
        assert_eq!(format!("MyFooferNut::{:?}", a.id), debug_a); // Labeled with entity

        let b: Id<NoLabelZed, String> = a.relabel();

        // Display canonical is same for both (no labels)
        assert_eq!(format!("{a}"), format!("{b}")); // Same canonical

        // Debug labels differ (different entities)
        assert_ne!(format!("{a:?}"), format!("{b:?}")); // Different labels

        let c: Id<Bar, String> = a.relabel();

        // Display canonical is same (no labels)
        assert_eq!(format!("{a}"), format!("{c}")); // Same canonical

        // Debug labels differ
        assert_ne!(format!("{a:?}"), format!("{c:?}")); // Different labels
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

        #[cfg(feature = "ulid")]
        {
            let ulid = Ulid::new();
            let id = crate::id::ulid::UlidId::<Foo>::direct(labeler.label(), ulid);
            assert_let!(Ok(json) = serde_json::to_string(&id));
            assert_let!(Ok(actual) = serde_json::from_str::<Id<Foo, Ulid>>(&json));
            assert_eq!(actual, id);
        }
    }

    #[test]
    fn test_serde_rejects_invalid_type_string_to_u64() {
        // JSON string "abc" cannot deserialize to u64
        let result = serde_json::from_str::<Id<Foo, u64>>("\"abc\"");
        assert!(result.is_err());
    }

    #[test]
    fn test_serde_labeled_format_is_literal() {
        // "Foo::abc" should deserialize as a literal string, not parsed
        let result = serde_json::from_str::<Id<Foo, String>>("\"Foo::abc\"");
        assert!(result.is_ok());
        let id = result.unwrap();
        assert_eq!(id.id, "Foo::abc"); // Literal string, not split
        assert_eq!(id.label, Foo::labeler().label()); // Label from entity, not from JSON
    }

    #[test]
    fn test_serde_label_from_entity_labeler() {
        let json = "\"test-id\"";
        let id: Id<Foo, String> = serde_json::from_str(json).unwrap();

        // Label must come from Foo::labeler(), not from JSON
        assert_eq!(id.label, Foo::labeler().label());
        assert_eq!(id.id, "test-id");
    }

    #[test]
    fn test_serde_roundtrip_label_behavior() {
        let original = Id::<Foo, String>::direct("ANY", "value".into());
        let json = serde_json::to_string(&original).unwrap();
        let restored: Id<Foo, String> = serde_json::from_str(&json).unwrap();

        // ID matches
        assert_eq!(restored.id, original.id);

        // Label is RE-DERIVED, not preserved from original
        assert_eq!(restored.label, Foo::labeler().label());
    }
}
