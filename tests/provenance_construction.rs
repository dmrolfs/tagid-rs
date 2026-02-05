#[cfg(test)]
mod provenance_construction_tests {
    use pretty_assertions::assert_eq;
    use tagid::id::provenance::*;
    use tagid::{Id, Label, MakeLabeling, Sourced};

    // --- Setup: Mock entity and types ---

    #[derive(Debug)]
    struct User;
    impl Label for User {
        type Labeler = MakeLabeling<Self>;
        fn labeler() -> Self::Labeler {
            MakeLabeling::default()
        }
    }

    type ExternalUserId = Id<Sourced<User, External<()>>, String>;
    type GeneratedUserId = Id<Sourced<User, Generated<()>>, String>;
    type DerivedUserId = Id<Sourced<User, Derived<()>>, String>;
    type ClientUserId = Id<Sourced<User, ClientProvided>, String>;
    type TemporaryUserId = Id<Sourced<User, Temporary>, String>;

    // --- Tests ---

    #[test]
    fn test_from_source_creates_id() {
        let id = ExternalUserId::from_source("ext-123".to_string());
        assert_eq!(id.to_string(), "ext-123");
    }

    #[test]
    fn test_from_source_preserves_exact_value() {
        let original = "cus_L3H8Z6K9j2";
        let id = ExternalUserId::from_source(original.to_string());
        assert_eq!(id.as_str(), original);
    }

    #[test]
    fn test_derived_from_creates_id() {
        let id = DerivedUserId::derived_from("user-slug".to_string());
        assert_eq!(id.to_string(), "user-slug");
    }

    #[test]
    fn test_from_client_creates_id() {
        let key = "client-key-123";
        let id = ClientUserId::from_client(key.to_string());
        assert_eq!(id.to_string(), key);
    }

    #[test]
    fn test_for_temporary_creates_id() {
        let id = TemporaryUserId::for_temporary("temp-123".to_string());
        assert_eq!(id.to_string(), "temp-123");
    }

    #[test]
    fn test_for_test_creates_id() {
        let id = GeneratedUserId::for_test("test-user".to_string());
        assert_eq!(id.to_string(), "test-user");
    }

    #[test]
    fn test_all_methods_produce_same_result_as_from_canonical() {
        let value = "test-id";

        let from_canonical = ExternalUserId::from_canonical(value.to_string());
        let from_source = ExternalUserId::from_source(value.to_string());

        assert_eq!(from_canonical.to_string(), from_source.to_string());
        assert_eq!(from_canonical.as_str(), from_source.as_str());
    }

    #[test]
    fn test_serialization_is_canonical() {
        let id = ExternalUserId::from_source("ext-123".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"ext-123\"");
    }
}
