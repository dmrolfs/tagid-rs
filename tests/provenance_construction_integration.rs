use pretty_assertions::assert_eq;
use tagid::id::provenance::*;
use tagid::{Entity, Id, Label, MakeLabeling, Sourced};

// ============================================================================
// Setup
// ============================================================================

#[derive(Debug)]
struct Application;
impl Label for Application {
    type Labeler = MakeLabeling<Self>;
    fn labeler() -> Self::Labeler {
        MakeLabeling::default()
    }
}

#[derive(Debug)]
struct User;
impl Label for User {
    type Labeler = MakeLabeling<Self>;
    fn labeler() -> Self::Labeler {
        MakeLabeling::default()
    }
}
impl Entity for User {
    type IdGen = tagid::CuidGenerator;
}

#[derive(Debug)]
struct Page;
impl Label for Page {
    type Labeler = MakeLabeling<Self>;
    fn labeler() -> Self::Labeler {
        MakeLabeling::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Slugify;

type AppId = Id<Sourced<Application, External<providers::Spark>>, String>;
type UserId = Id<Sourced<User, Generated>, String>;
type PageSlugId = Id<Sourced<Page, Derived<Slugify>>, String>;
type UserEmailAlias = Id<Sourced<User, AliasOf<UserId>>, String>;
type TenantId = Id<Sourced<User, Generated>, String>;
type ScopedResourceId = Id<Sourced<Page, Scoped<TenantId, Generated>>, String>;

// ============================================================================
// Tests
// ============================================================================

#[test]
fn test_spark_import_workflow() {
    // Simulating a Spark MCP response
    let raw_app_id = "app-202602041830-0001";

    let app_id = AppId::from_source(raw_app_id.to_string());

    assert_eq!(app_id.to_string(), raw_app_id);
    assert_eq!(app_id.as_str(), raw_app_id);

    // Serialization check
    let json = serde_json::to_string(&app_id).unwrap();
    assert_eq!(json, format!("\"{}\"", raw_app_id));
}

#[test]
fn test_slug_generation_workflow() {
    let title = "My New Page";
    let slug_value = title.to_lowercase().replace(" ", "-");

    let slug_id = PageSlugId::derived_from(slug_value.clone());

    assert_eq!(slug_id.to_string(), "my-new-page");
    assert_eq!(slug_id.as_str(), "my-new-page");
}

#[test]
fn test_multi_tenant_scoped_resources() {
    let _tenant1_id = TenantId::from_canonical("tenant-1".to_string());
    let _tenant2_id = TenantId::from_canonical("tenant-2".to_string());

    let res1 = ScopedResourceId::for_scope("resource-A".to_string());
    let res2 = ScopedResourceId::for_scope("resource-A".to_string());

    // They have same canonical ID
    assert_eq!(res1, res2);

    // But they are intended for different scopes (semantic distinction)
    // This test just ensures the construction works.
    assert_eq!(res1.to_string(), "resource-A");
}

#[test]
fn test_alias_and_primary_id_relationship() {
    let _primary_id = UserId::new();
    let email_alias = UserEmailAlias::alias_for("user@example.com".to_string());

    assert_eq!(email_alias.to_string(), "user@example.com");
}

#[test]
fn test_multi_provenance_workflow() {
    // Given: a realistic set of IDs for a single user session
    let user_id = UserId::new();
    let stripe_id =
        Id::<Sourced<User, External<providers::Stripe>>, _>::from_source("cus_123".to_string());
    let session_id = Id::<Sourced<User, Temporary>, _>::for_temporary("sess_abc".to_string());

    // All should be usable and serializable
    assert!(!user_id.to_string().is_empty());
    assert_eq!(stripe_id.to_string(), "cus_123");
    assert_eq!(session_id.to_string(), "sess_abc");

    let ids = vec![
        user_id.to_string(),
        stripe_id.to_string(),
        session_id.to_string(),
    ];

    assert_eq!(ids.len(), 3);
}
