use super::Id;

pub trait IdGenerator {
    type IdType;
    fn next_id_rep() -> Self::IdType;
}

#[allow(dead_code)]
pub type CuidId<T> = Id<T, String>;

pub struct CuidGenerator;

impl IdGenerator for CuidGenerator {
    type IdType = String;

    fn next_id_rep() -> Self::IdType {
        cuid2::create_id()
    }
}

pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    type IdType = uuid::Uuid;

    fn next_id_rep() -> Self::IdType {
        uuid::Uuid::new_v4()
    }
}
