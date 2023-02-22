mod gen;
pub mod snowflake;

pub use self::snowflake::{MachineNode, SnowflakeGenerator};
pub use gen::{CuidGenerator, CuidId, IdGenerator, UuidGenerator};

use crate::{Label, Labeling};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smol_str::SmolStr;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub trait Entity: Label {
    type IdGen: IdGenerator;

    fn next_id() -> Id<Self, <Self::IdGen as IdGenerator>::IdType> {
        Id::new()
    }
}

pub struct Id<T: ?Sized, ID> {
    pub label: SmolStr,
    pub id: ID,
    marker: PhantomData<T>,
}

#[allow(unsafe_code)]
unsafe impl<T: ?Sized, ID: Send> Send for Id<T, ID> {}

#[allow(unsafe_code)]
unsafe impl<T: ?Sized, ID: Sync> Sync for Id<T, ID> {}

impl<E> Id<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: ?Sized + Entity + Label,
{
    pub fn new() -> Self {
        let labeler = <E as Label>::labeler();
        Self {
            label: SmolStr::new(labeler.label()),
            id: E::IdGen::next_id_rep(),
            marker: PhantomData,
        }
    }
}

impl<E: ?Sized + Entity + Label> Default for Id<E, <<E as Entity>::IdGen as IdGenerator>::IdType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized + Label, ID> Id<T, ID> {
    pub fn for_labeled(id: ID) -> Self {
        let labeler = <T as Label>::labeler();
        Self {
            label: SmolStr::new(labeler.label()),
            id,
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized, ID> Id<T, ID> {
    pub fn direct(label: impl AsRef<str>, id: ID) -> Self {
        Self {
            label: SmolStr::new(label.as_ref()),
            id,
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized, ID: Clone> Id<T, ID> {
    pub fn relabel<B: Label>(&self) -> Id<B, ID> {
        let b_labeler = B::labeler();
        Id {
            label: SmolStr::new(b_labeler.label()),
            id: self.id.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized, ID: Clone> Clone for Id<T, ID> {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            id: self.id.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized, ID: fmt::Debug> fmt::Debug for Id<T, ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("Id")
                .field("label", &self.label)
                .field("id", &self.id)
                .finish()
        } else if self.label.is_empty() {
            write!(f, "{:?}", self.id)
        } else {
            write!(f, "{}::{:?}", self.label, self.id)
        }
    }
}

impl<T: ?Sized, ID: fmt::Display> fmt::Display for Id<T, ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() || self.label.is_empty() {
            write!(f, "{}", self.id)
        } else {
            write!(f, "{}::{}", self.label, self.id)
        }
    }
}

impl<T: ?Sized, ID: PartialEq> PartialEq for Id<T, ID> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: ?Sized, ID: Eq> Eq for Id<T, ID> {}

impl<T: ?Sized, ID: Ord> Ord for Id<T, ID> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T: ?Sized, ID: PartialOrd> PartialOrd for Id<T, ID> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<T: ?Sized, ID: Hash> Hash for Id<T, ID> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl<T: ?Sized, ID: Serialize> Serialize for Id<T, ID> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id.serialize(serializer)
    }
}

impl<'de, T: ?Sized + Label, ID: DeserializeOwned> Deserialize<'de> for Id<T, ID> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rep = ID::deserialize(deserializer)?;
        let labeler = <T as Label>::labeler();
        Ok(Self::direct(labeler.label(), rep))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CustomLabeling, MakeLabeling, NoLabeling};
    use claim::*;
    use pretty_assertions::assert_eq;
    use serde_test::{assert_tokens, Token};
    use static_assertions::assert_impl_all;

    #[test]
    fn test_auto_traits() {
        assert_impl_all!(Id<u32, u32>: Send, Sync);
        assert_impl_all!(Id<std::rc::Rc<u32>, String>: Send, Sync);
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
        type IdGen = CuidGenerator;
    }

    impl Label for Foo {
        type Labeler = CustomLabeling;

        fn labeler() -> Self::Labeler {
            CustomLabeling::new("MyFooferNut")
        }
    }

    #[test]
    fn test_display() {
        let a: CuidId<Foo> = Foo::next_id();
        assert_eq!(format!("{a}"), format!("MyFooferNut::{}", a.id));
    }

    #[test]
    fn test_alternate_display() {
        let a: Id<Foo, i64> = Id::direct(Foo::labeler().label(), 13);
        assert_eq!(format!("{a:#}"), a.id.to_string());

        let id = 98734021;
        let a: Id<Foo, u64> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(format!("{a:#}"), a.id.to_string());

        let id = uuid::Uuid::new_v4();
        let a: Id<Foo, uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(format!("{a:#}"), a.id.to_string());
    }

    #[test]
    fn test_debug() {
        let a: CuidId<Foo> = Foo::next_id();
        assert_eq!(format!("{a:?}"), format!("MyFooferNut::{:?}", a.id));

        let id = 98734021;
        let a: Id<Foo, u64> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(format!("{a:?}"), format!("MyFooferNut::{:?}", a.id));

        let id = uuid::Uuid::new_v4();
        let a: Id<Foo, uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(format!("{a:?}"), format!("MyFooferNut::{:?}", a.id));
    }

    #[test]
    fn test_alternate_debug() {
        let a: CuidId<Foo> = Foo::next_id();
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

        let id = uuid::Uuid::new_v4();
        let a: Id<Foo, uuid::Uuid> = Id::direct(Foo::labeler().label(), id);
        assert_eq!(
            format!("{a:#?}"),
            format!("Id {{\n    label: \"{}\",\n    id: {},\n}}", a.label, a.id,)
        );
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
        let id = CuidId::<Foo>::direct(labeler.label(), cuid);
        assert_tokens(&id, &vec![Token::Str("ig6wv6nezj0jg51lg53dztqy")]);

        let id = Id::<Foo, u64>::direct(labeler.label(), 17);
        assert_tokens(&id, &vec![Token::U64(17)]);
    }

    #[test]
    fn test_id_serde_json() {
        let labeler = <Foo as Label>::labeler();

        let cuid = "ig6wv6nezj0jg51lg53dztqy".to_string();
        let id = CuidId::<Foo>::direct(labeler.label(), cuid);
        let json = assert_ok!(serde_json::to_string(&id));
        let actual: CuidId<Foo> = assert_ok!(serde_json::from_str(&json));
        assert_eq!(actual, id);

        let id = Id::<Foo, u64>::direct(labeler.label(), 17);
        let json = assert_ok!(serde_json::to_string(&id));
        let actual: Id<Foo, u64> = assert_ok!(serde_json::from_str(&json));
        assert_eq!(actual, id);

        let uuid = uuid::Uuid::new_v4();
        let id = Id::<Foo, uuid::Uuid>::direct(labeler.label(), uuid.clone());
        let json = assert_ok!(serde_json::to_string(&id));
        let actual: Id<Foo, uuid::Uuid> = assert_ok!(serde_json::from_str(&json));
        assert_eq!(actual, id);
    }
}
