# Provenance-Aware Construction Patterns

**Status**: Stable  
**Last Updated**: 2026-02-04  
**Related Tasks**: tid-6n4 (Core Implementation)  
**Audience**: tagid users (Turtle, downstream projects, all AI assistants)

---

## Table of Contents

1. [Overview](#overview)
2. [Decision Tree](#decision-tree)
3. [Eight Provenance Patterns](#eight-provenance-patterns)
4. [Migration Guide](#migration-guide)
5. [FAQ](#faq)
6. [Quick Reference](#quick-reference)

---

## Overview

### What is Provenance-Aware Construction?

Each ID in your system **originates from somewhere**. The **provenance** is where it comes from:
- **External** — from a third-party system (Stripe, Spark, AWS)
- **Generated** — we create it internally
- **Imported** — from a legacy/migration system
- **Derived** — computed from other data
- **ClientProvided** — the user/client provides it
- **Scoped** — unique within a context, not globally
- **AliasOf** — secondary identifier
- **Temporary** — ephemeral, don't persist

### Why It Matters

**Semantic clarity via function names:**

| Function | Meaning |
|----------|---------|
| `from_canonical()` | Create from a generic canonical ID value (internal use) |
| `from_source()` | This ID comes **FROM** an external source (don't transform) |
| `derived_from()` | This ID is **DERIVED FROM** data (deterministic) |
| `from_client()` | This ID comes **FROM** the client/user (accept as-is) |
| `for_scope()` | This ID is scoped **TO** a context (context-relative) |
| `alias_for()` | This ID is an **ALIAS FOR** the entity (secondary) |
| `for_temporary()` | This ID is **FOR TEMPORARY** use (don't persist!) |
| `for_test()` | This ID is **FOR TESTING** (fixtures only) |

**Benefits:**
- ✅ **Self-documenting code** — function name documents intent
- ✅ **Prevents mistakes** — hard to use wrong construction method
- ✅ **Faster reviews** — reviewers immediately see provenance intent
- ✅ **Type-safe** — compiler enforces some constraints

---

## Decision Tree

**Choose the right construction function:**

```
Does the ID come from an external system?
├─ YES (Stripe, AWS, Spark, etc.) → use from_source()
│   └─ Is this migration/legacy data?
│       └─ Type marks it as Imported<From> → still use from_source()
└─ NO, we create it ourselves
   ├─ Does it compute from other data?
   │  └─ YES (slug, hash, composite key) → use derived_from()
   └─ Does the user/client supply it?
      └─ YES (idempotency keys, user IDs) → use from_client()
      └─ NO, we generate it
         ├─ Is this production code?
         │  └─ YES → use .generate() (Entity trait)
         │  └─ NO (test/fixture) → use for_test()
         └─ Is it temporary?
            └─ YES (optimistic ID, session token) → use for_temporary()
         └─ Is it scoped to a context?
            └─ YES (tenant-local, org-local) → use for_scope()
         └─ Is this a secondary ID for the same entity?
            └─ YES (email as alias for user_id) → use alias_for()
```

---

## Eight Provenance Patterns

### 1. External: from_source()

**What**: ID originates from an external system; we don't create it.

**When to use**:
- ✅ Stripe customer IDs (`cus_L3H8Z6K9j2`)
- ✅ AWS S3 bucket ARNs
- ✅ Spark Job IDs from History Server
- ✅ GitHub user IDs
- ✅ Any third-party API response

**Key rule**: **The value is opaque.** Don't parse, transform, or validate it. Preserve it exactly.

**Example**:

```rust
use tagid::id::provenance::External;
use tagid::{Id, Sourced};

pub struct Stripe;

pub type StripeCustomerId = Id<Sourced<Customer, External<Stripe>>, String>;

// Parsing MCP response from Stripe
fn parse_stripe_response(response: &str) -> StripeCustomerId {
    let customer_id = response;  // Just the raw ID, no parsing
    StripeCustomerId::from_source(customer_id.to_string())
    // ✅ from_source() signals: external, don't transform
}

// Usage
let id = parse_stripe_response("cus_L3H8Z6K9j2");
assert_eq!(id.to_string(), "cus_L3H8Z6K9j2");  // Preserved exactly
```

**Common mistake**:
```rust
// ❌ Wrong: parsing/transforming external value
let id = StripeCustomerId::from_source(
    customer_id.strip_prefix("cus_").unwrap_or(customer_id).to_string()
);
// This transforms the ID, losing information!

// ✅ Correct: preserve as-is
let id = StripeCustomerId::from_source(customer_id.to_string());
```

**In Turtle context**: Spark IDs are all External, use `from_source()`:
```rust
pub fn app_id(value: impl Into<String>) -> AppId {
    AppId::from_source(value.into())
}

pub fn job_id(value: u64) -> JobId {
    JobId::from_source(value)
}
```

---

### 2. Generated: .generate() or for_test()

**What**: ID is created internally by our system.

**When to use**:
- ✅ New user IDs (production)
- ✅ Request IDs
- ✅ Tenant IDs
- ✅ Any ID we generate (UUID, ULID, Snowflake, etc.)

**Two construction methods:**

| Method | When | Purpose |
|--------|------|---------|
| `.generate()` | **Production** | Creates new ID using Entity trait |
| `.for_test()` | **Tests/fixtures** | Explicit ID for test data |

**Example**:

```rust
use tagid::{Entity, Id, Label, MakeLabeling};
use tagid::id::provenance::Generated;
use tagid::id::snowflake::SnowflakeGenerator;

#[derive(Debug)]
pub struct User;

impl Label for User {
    type Labeler = MakeLabeling<Self>;
    fn labeler() -> Self::Labeler { MakeLabeling::default() }
}

impl Entity for User {
    type IdGen = SnowflakeGenerator;
}

pub type UserId = Id<Sourced<User, Generated<Snowflake>>, u64>;

// Production: generate new ID
let user_id = UserId::generate();  // ← use in production

// Tests: explicit test ID
#[test]
fn test_user_creation() {
    let user_id = UserId::for_test(42u64);  // ← use in tests
    assert_eq!(user_id.to_string(), "42");
}
```

**Common mistake**:
```rust
// ❌ Wrong: using for_test() in production
let id = UserId::for_test(uuid::Uuid::new_v7().to_string());
// This signals "test data" to reviewers!

// ✅ Correct: use generate() in production
let id = UserId::generate();

// ✅ Correct: use for_test() in tests
#[test]
fn test() {
    let id = UserId::for_test("user-1".to_string());
}
```

---

### 3. Imported: from_source()

**What**: ID from legacy/migration data; similar to External but marked as imported.

**When to use**:
- ✅ Migration from old database
- ✅ Data backfill from legacy system
- ✅ Import from another organization/system

**Key rule**: Semantically **identical to External** — value is opaque, preserve exactly. The type parameter marks this as migration data.

**Example**:

```rust
use tagid::id::provenance::Imported;
use tagid::{Id, Sourced};

pub struct LegacyDatabase;

pub type LegacyJobId = Id<Sourced<Job, Imported<LegacyDatabase>>, u64>;

// During migration from old database
fn migrate_job(legacy_id: u64) -> LegacyJobId {
    LegacyJobId::from_source(legacy_id)
    // ✅ from_source() works for both External and Imported
    // Type system tracks: this came from LegacyDatabase
}

let id = migrate_job(42u64);
assert_eq!(*id.as_ref(), 42u64);  // Preserved exactly
```

**Difference from External:**
```rust
// External: from a current, live system (Spark API)
pub type CurrentJobId = Id<Sourced<Job, External<Spark>>, u64>;

// Imported: from a migration/legacy system
pub type LegacyJobId = Id<Sourced<Job, Imported<LegacyDb>>, u64>;

// Both use from_source() (same semantics)
// But the type tells reviewers: this is migration data
```

---

### 4. Derived: derived_from()

**What**: ID is **computed** deterministically from source data.

**When to use**:
- ✅ URL slugs (from page title)
- ✅ Content hashes (from file contents)
- ✅ Composite keys (from multiple fields)
- ✅ Any deterministically-derived value

**Key rule**: **Same source → same ID** (deterministic). Different from Generated (which is random).

**Example**:

```rust
use tagid::id::provenance::Derived;
use tagid::{Id, Sourced};

pub struct Slugify;

pub type PageSlugId = Id<Sourced<Page, Derived<Slugify>>, String>;

// Derive ID from page title
fn create_page_slug(title: &str) -> PageSlugId {
    let slug = slugify(title);  // "My Page" -> "my-page"
    PageSlugId::derived_from(slug)
    // ✅ derived_from() signals: computed from source, deterministic
}

// Same title always produces same ID
let slug1 = create_page_slug("My Page");
let slug2 = create_page_slug("My Page");
assert_eq!(slug1.to_string(), slug2.to_string());  // ✅ Deterministic
```

**Difference from External and Generated**:

| Type | Computation | Deterministic | Use Case |
|------|-------------|---------------|----------|
| **External** | None (given) | Always | External system provides |
| **Generated** | Random/time | No | We generate new |
| **Derived** | Algorithmic | Yes | Computed from data |

**Common mistake**:
```rust
// ❌ Wrong: using from_canonical() doesn't show intent
let slug = PageSlugId::from_canonical(slugify(title));

// ✅ Correct: derived_from() shows computation
let slug = PageSlugId::derived_from(slugify(title));
```

---

### 5. ClientProvided: from_client()

**What**: User or client supplies the ID directly.

**When to use**:
- ✅ Idempotency keys in APIs
- ✅ User-defined identifiers (bring-your-own-ID)
- ✅ Customer-provided reference numbers
- ✅ Client-side-generated request IDs

**Key rule**: Different from External (which comes from a **system**). This comes from **user/client input**.

**Example**:

```rust
use tagid::id::provenance::ClientProvided;
use tagid::{Id, Sourced};

pub type IdempotencyKey = Id<Sourced<Request, ClientProvided>, String>;

// API endpoint accepting idempotency key
async fn create_payment(
    client_key: String,
    amount: u64,
) -> Result<PaymentId> {
    let idempotency_key = IdempotencyKey::from_client(client_key);
    // ✅ from_client() signals: trust but verify
    
    // Check if already processed (idempotency)
    if let Some(existing) = cache.get(idempotency_key.as_str()) {
        return Ok(existing);
    }
    
    // Process payment
    let payment_id = process_payment(amount).await?;
    cache.set(idempotency_key.as_str(), payment_id.clone());
    Ok(payment_id)
}

let key = IdempotencyKey::from_client("client-provided-key-123".to_string());
```

**Difference from External**:
```rust
// External: Stripe API provides customer ID
pub type StripeCustomerId = Id<Sourced<Customer, External<Stripe>>, String>;
let id = StripeCustomerId::from_source(stripe_api_response.customer_id);

// ClientProvided: user provides their own reference
pub type UserReferenceId = Id<Sourced<Order, ClientProvided>, String>;
let id = UserReferenceId::from_client(user_provided_reference);

// Both are "external to us" but from different origins
```

---

### 6. Scoped: for_scope()

**What**: ID is **unique within a scope**, not globally unique.

**When to use**:
- ✅ Tenant-local resource IDs
- ✅ Organization-scoped identifiers
- ✅ Workspace-specific IDs
- ✅ Any multi-tenant scenario

**Key rule**: Combine with another provenance (usually Generated). The inner provenance determines allowed operations.

**Example**:

```rust
use tagid::id::provenance::{Scoped, Generated};
use tagid::{Entity, Id, Label, MakeLabeling, Sourced};

pub type TenantId = Id<Sourced<Tenant, Generated<Snowflake>>, u64>;
pub type TenantResourceId =
    Id<Sourced<Resource, Scoped<TenantId, Generated<Snowflake>>>, u64>;

// Generate a resource unique within a tenant
fn create_resource_in_tenant(tenant_id: TenantId) -> TenantResourceId {
    TenantResourceId::for_scope(tenant_id.to_string())
    // ✅ for_scope() marks this as context-dependent
}

// Or use the inner provenance's method
fn generate_resource_in_tenant() -> TenantResourceId {
    TenantResourceId::generate()  // Inner Generated provides this
}

// Usage: same resource ID in different tenants can mean different things
let tenant1 = TenantId::for_test(1u64);
let tenant2 = TenantId::for_test(2u64);

let resource1 = create_resource_in_tenant(tenant1);
let resource2 = create_resource_in_tenant(tenant2);

// Both might have same value internally, but scope is different
// DB might store: (tenant_id=1, resource_id=42) and (tenant_id=2, resource_id=42)
```

---

### 7. AliasOf: alias_for()

**What**: Secondary identifier for the same entity.

**When to use**:
- ✅ Email as alias for user ID
- ✅ URL slug as alias for page ID
- ✅ Username as alias for user ID
- ✅ Phone number as alias for customer ID

**Key rule**: This is **not the primary ID**, just an alternative way to identify the same entity.

**Example**:

```rust
use tagid::id::provenance::AliasOf;
use tagid::{Id, Sourced};

pub type UserId = Id<Sourced<User, Generated<Snowflake>>, u64>;
pub type UserEmailAlias = Id<Sourced<User, AliasOf<UserId>>, String>;

// Creating a user with email alias
struct NewUser {
    email: String,
}

fn create_user(req: NewUser) -> User {
    let user_id = UserId::generate();  // Primary ID
    let email_alias = UserEmailAlias::alias_for(req.email.clone());  // Secondary
    
    User {
        id: user_id,
        email: email_alias,
    }
}

// Usage: both refer to same user
let user = create_user(NewUser { email: "user@example.com".to_string() });

// Can lookup by primary ID or email alias
db.find_user_by_id(user.id);           // Primary lookup
db.find_user_by_email_alias(user.email);  // Secondary lookup
```

**Often combined with Derived**:
```rust
use tagid::id::provenance::{AliasOf, Derived};

pub type UserSlugAlias = Id<Sourced<User, AliasOf<UserId>>, String>;

// Email is often derived from user data
pub type UserEmailAlias = Id<
    <Sourced<User, Derived<EmailHash>>,  // Derived
    String,
>;
```

---

### 8. Temporary: for_temporary()

**What**: Ephemeral ID, valid only short-term. **Do not persist.**

**When to use**:
- ✅ Optimistic IDs during edit workflows
- ✅ Session tokens
- ✅ Temporary request IDs
- ✅ Client-side placeholder IDs

**Key rule**: **Never store this in the database.** It's only for the current session/request.

**Example**:

```rust
use tagid::id::provenance::Temporary;
use tagid::{Id, Sourced};

pub type OptimisticItemId = Id<Sourced<Item, Temporary>, String>;

// Frontend creates optimistic ID for offline/optimistic UI
async fn create_item_with_optimistic_id(
    title: String,
) -> (OptimisticItemId, ItemId) {
    // Generate temporary ID for optimistic UI
    let temp_id = OptimisticItemId::for_temporary(
        uuid::Uuid::new_v4().to_string()
    );
    // ✅ for_temporary() flags: don't persist this!

    // Send to backend
    let real_id = backend.create_item(title).await?;
    // Backend returns real, persistent ID

    // Frontend replaces optimistic ID with real ID
    (temp_id, real_id)
}

// Usage: optimistic update
let (temp_id, real_id) = create_item("New Item".to_string()).await?;

// Show temp_id in UI immediately (optimistic)
ui.render_item(temp_id);

// When response arrives, replace with real_id
ui.replace_item(temp_id, real_id);

// ⚠️ Important: never save temp_id to local storage or database!
```

**Common mistake**:
```rust
// ❌ Wrong: trying to persist temporary ID
db.save_item(OptimisticItemId::for_temporary(id), data);
// This is semantically wrong; the type says "don't persist"!

// ✅ Correct: use real ItemId
let real_id = backend.create_item(data).await?;
db.save_item(real_id, data);
```

---

## Migration Guide

### If you're using `from_canonical()` (formerly `for_labeled()`) everywhere

**Current code**:
```rust
pub fn app_id(value: impl Into<String>) -> AppId {
    AppId::from_canonical(value.into())
}

pub fn user_id(value: String) -> UserId {
    UserId::from_canonical(value)
}
```

**Step 1: Identify the provenance type**

Check your type definitions:
```rust
pub type AppId = Id<Sourced<Application, External<providers::Spark>>, String>;
// ^^ External → use from_source()

pub type UserId = Id<Sourced<User, Generated<Snowflake>>, String>;
// ^^ Generated → use for_test() (or remove, use .generate())
```

**Step 2: Update to provenance-aware functions**

```rust
// External: use from_source()
pub fn app_id(value: impl Into<String>) -> AppId {
    AppId::from_source(value.into())  // ← Updated
}

// Generated: use for_test() or .generate()
pub fn user_id_test(value: String) -> UserId {
    UserId::for_test(value)  // ← Updated, explicit that it's for testing
}

// In production:
let user_id = UserId::generate();  // ← Use Entity trait method
```

**Step 3: Update call sites**

```rust
// Before
let app_id = ids::app_id(response.id);

// After (no change needed! Same function, semantically clearer)
let app_id = ids::app_id(response.id);  // ← Still called ids::app_id()
// But implementation now uses from_source() internally
```

### Checklist

- [ ] Identify all ID types and their provenance
- [ ] Categorize by provenance (External, Generated, etc.)
- [ ] Update helper functions to use appropriate methods
- [ ] Update production code to use correct methods (.generate() for Generated)
- [ ] Update tests to use for_test() explicitly
- [ ] Run `cargo test` to verify no regressions
- [ ] Code review: ensure semantics match provenance

---

## FAQ

### Q: Can I use `from_canonical()` instead of these new functions?

**A**: Yes, `from_canonical()` still works and is backward compatible. But the new functions are **semantically clearer**. Use them for new code.

### Q: Which function should I use if I'm not sure?

**A**: Use the **decision tree** above. Or:
1. Does an external system provide this ID? → `from_source()`
2. Do we compute it from data? → `derived_from()`
3. Does the user provide it? → `from_client()`
4. Do we generate it? → `generate()` (production) or `for_test()` (tests)
5. Is it temporary? → `for_temporary()`
6. Is it scoped? → `for_scope()`
7. Is it secondary? → `alias_for()`

### Q: What if my provenance doesn't fit these 8?

**A**: tagid's 8 provenances cover all common scenarios. If you genuinely need custom provenance:
1. Implement the `Provenance` trait (see `src/id/provenance.rs`)
2. Add a helper function in your codebase
3. File an issue if it's a common pattern

### Q: Does this affect serialization?

**A**: No. All methods are **aliases** to `from_canonical()`. Serialization format is unchanged (canonical ID only, no label, no provenance).

### Q: Does this affect performance?

**A**: No. All methods are **zero-cost** — they compile to the same code.

### Q: What about my existing type annotations?

**A**: Unchanged. The new functions don't change `Id<T, ID>` itself, just how you construct it.

---

## Quick Reference

| Scenario | Function | Example |
|----------|----------|---------|
| **From Stripe API** | `from_source()` | `StripeId::from_source("cus_123")` |
| **From Spark History** | `from_source()` | `AppId::from_source("app-001")` |
| **Generate new** | `generate()` | `UserId::generate()` |
| **Test fixture** | `for_test()` | `UserId::for_test("user-1")` |
| **From slug/hash** | `derived_from()` | `SlugId::derived_from(slugify(title))` |
| **From user input** | `from_client()` | `Key::from_client(user_key)` |
| **Migration data** | `from_source()` | `LegacyId::from_source(42u64)` |
| **Scoped resource** | `for_scope()` | `TenantResourceId::for_scope(tenant_id)` |
| **Email/username** | `alias_for()` | `EmailAlias::alias_for("user@example.com")` |
| **Temporary** | `for_temporary()` | `TempId::for_temporary(uuid_v4())` |

---

## Related Documents

- **[Detailed Analysis](../../history/tid-6n4-provenance-construction-analysis.md)** — Full semantic analysis
- **[Implementation Plan](../../history/tid-6n4-implementation-plan.md)** — How it's implemented
- **[Test Plan](../../history/tid-6n4-test-plan.md)** — Testing strategy
- **[Provenance Types](../../src/id/provenance.rs)** — Full trait documentation

