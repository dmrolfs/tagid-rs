//! Provenance-Aware Construction API (v1.1.0).
//!
//! This example provides a focused demonstration of the new semantic construction
//! functions added in tagid v1.1.0. These functions clarify the origin and
//! intent of an ID based on its provenance type.

use tagid::id::provenance::{
    AliasOf, ClientProvided, Derived, External, Generated, Scoped, Temporary, providers,
};
use tagid::{Entity, Id, Label, MakeLabeling, Sourced};

// ============================================================================
// Entity Definitions
// ============================================================================

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

struct Page;
impl Label for Page {
    type Labeler = MakeLabeling<Self>;
    fn labeler() -> Self::Labeler {
        MakeLabeling::default()
    }
}

struct Customer;
impl Label for Customer {
    type Labeler = MakeLabeling<Self>;
    fn labeler() -> Self::Labeler {
        MakeLabeling::default()
    }
}

// ============================================================================
// Type Aliases (Best Practice)
// ============================================================================

type UserId = Id<Sourced<User, Generated>, String>;
type StripeCustomerId = Id<Sourced<Customer, External<providers::Stripe>>, String>;
type PageSlugId = Id<Sourced<Page, Derived>, String>;
type UserEmailAlias = Id<Sourced<User, AliasOf<UserId>>, String>;
type IdempotencyKey = Id<Sourced<User, ClientProvided>, String>;
type OptimisticId = Id<Sourced<User, Temporary>, String>;
type TenantId = Id<Sourced<User, Generated>, String>;
type ScopedResourceId = Id<Sourced<Page, Scoped<TenantId, Generated>>, String>;

fn main() {
    println!("=== tagid v1.1.0: Provenance-Aware Construction ===\n");

    // 1. EXTERNAL: from_source()
    // Used for IDs that originate from external systems.
    let stripe_id = StripeCustomerId::from_source("cus_L3H8Z6".to_string());
    println!("1. External (Stripe):   {}", stripe_id.labeled());

    // 2. GENERATED: .new() (via Entity) or .for_test()
    // .new() is for production; .for_test() is for fixtures/tests.
    let user_id = UserId::new();
    let test_user = UserId::for_test("user-123".to_string());
    println!("2a. Generated (Prod):   {}", user_id.labeled());
    println!("2b. Generated (Test):   {}", test_user.labeled());

    // 3. DERIVED: derived_from()
    // Used for IDs computed deterministically from data (slugs, hashes).
    let slug = PageSlugId::derived_from("welcome-to-tagid".to_string());
    println!("3. Derived (Slug):      {}", slug.labeled());

    // 4. CLIENT PROVIDED: from_client()
    // Used for IDs supplied by the user or client (idempotency keys).
    let client_key = IdempotencyKey::from_client("req_12345".to_string());
    println!("4. Client Provided:     {}", client_key.labeled());

    // 5. SCOPED: for_scope()
    // Used for IDs unique within a context (tenant, organization).
    let tenant_id = TenantId::from_canonical("tenant-7".to_string());
    let resource = ScopedResourceId::for_scope("resource-42".to_string());
    println!(
        "5. Scoped (Resource):   {} (in {})",
        resource.labeled(),
        tenant_id
    );

    // 6. ALIAS: alias_for()
    // Used for secondary identifiers (email, username).
    let email = UserEmailAlias::alias_for("user@example.com".to_string());
    println!("6. Alias (Email):       {}", email.labeled());

    // 7. TEMPORARY: for_temporary()
    // Used for ephemeral IDs that should NOT be persisted.
    let temp = OptimisticId::for_temporary("temp-001".to_string());
    println!("7. Temporary (Opaque):  {}", temp.labeled());

    println!("\nSummary: These semantic functions provide zero-cost guidance");
    println!("and ensure your code documents the origin of every identifier.");
}
