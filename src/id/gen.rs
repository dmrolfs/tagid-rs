pub trait IdGenerator {
    type IdType;
    fn next_id_rep() -> Self::IdType;
}

#[cfg(feature = "cuid")]
pub use self::cuid::{CuidGenerator, CuidId};

#[cfg(feature = "uuid")]
pub use self::uuid::UuidGenerator;

#[cfg(feature = "cuid")]
mod cuid {
    use super::*;
    use crate::Id;

    #[allow(dead_code)]
    pub type CuidId<T> = Id<T, String>;

    pub struct CuidGenerator;

    impl IdGenerator for CuidGenerator {
        type IdType = String;

        fn next_id_rep() -> Self::IdType {
            ::cuid2::create_id()
        }
    }
}

#[cfg(feature = "uuid")]
mod uuid {
    use super::*;

    pub struct UuidGenerator;

    impl IdGenerator for UuidGenerator {
        type IdType = ::uuid::Uuid;

        fn next_id_rep() -> Self::IdType {
            ::uuid::Uuid::new_v4()
        }
    }
}
