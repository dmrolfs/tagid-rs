use crate::{CustomLabeling, Labeling, MakeLabeling, NoLabeling};
use std::collections::HashMap;

pub trait Label {
    type Labeler: Labeling;
    fn labeler() -> Self::Labeler;
}

impl Label for () {
    type Labeler = NoLabeling;

    fn labeler() -> Self::Labeler {
        NoLabeling
    }
}

impl<T: Label> Label for Option<T> {
    type Labeler = <T as Label>::Labeler;

    fn labeler() -> Self::Labeler {
        <T as Label>::labeler()
    }
}

impl<T: Label, E> Label for Result<T, E> {
    type Labeler = <T as Label>::Labeler;

    fn labeler() -> Self::Labeler {
        <T as Label>::labeler()
    }
}

impl<K: Label, V: Label> Label for HashMap<K, V> {
    type Labeler = CustomLabeling;

    fn labeler() -> Self::Labeler {
        let k_labeler = <K as Label>::labeler();
        let v_labeler = <V as Label>::labeler();
        CustomLabeling::from(format!(
            "HashMap<{},{}",
            k_labeler.label(),
            v_labeler.label()
        ))
    }
}

macro_rules! primitive_label {
    ($i:ty) => {
        impl Label for $i {
            type Labeler = MakeLabeling<Self>;

            fn labeler() -> Self::Labeler {
                MakeLabeling::<Self>::default()
            }
        }
    };
}

primitive_label!(bool);
primitive_label!(char);
primitive_label!(f32);
primitive_label!(f64);
primitive_label!(i8);
primitive_label!(i32);
primitive_label!(i64);
primitive_label!(i128);
primitive_label!(isize);
primitive_label!(u8);
primitive_label!(u16);
primitive_label!(u32);
primitive_label!(u64);
primitive_label!(u128);
primitive_label!(usize);
primitive_label!(String);

impl<'a> Label for &'a str {
    type Labeler = MakeLabeling<Self>;

    fn labeler() -> Self::Labeler {
        MakeLabeling::<Self>::default()
    }
}
