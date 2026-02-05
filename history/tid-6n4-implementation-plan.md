# Provenance-Aware Construction API: Detailed Implementation Plan

**Status**: Specification & Implementation Roadmap  
**Date**: 2026-02-04  
**Related Analysis**: `tid-6n4-provenance-construction-analysis.md`  

---

## Table of Contents

1. [Overview](#overview)
2. [Phase 1: Core Implementation](#phase-1-core-implementation)
3. [Phase 2: Documentation & Lessons](#phase-2-documentation--lessons)
4. [Phase 3: Testing Strategy](#phase-3-testing-strategy)
5. [Phase 4: Downstream Integration](#phase-4-downstream-integration)
6. [Risk Mitigation](#risk-mitigation)
7. [Timeline & Dependencies](#timeline--dependencies)

---

## Overview

### Objective
Add provenance-appropriate construction functions to `tagid::Id<T, ID>` that clarify the **origin** and **semantics** of each ID based on its provenance type.

### Scope
- **Affected Module**: `src/id/identifier.rs`
- **New Methods**: 7 (all aliases to existing `for_labeled()`)
- **Breaking Changes**: None (backward compatible)
- **Affected Downstream**: turtle-core, turtle-spark (updates, not breaks)

### Key Principles
1. **Zero-cost abstractions** — all new methods are simple aliases
2. **Backward compatible** — `for_labeled()` remains and works
3. **Semantic clarity** — function names document provenance intent
4. **Type-safe** — provenance types guide toward correct usage

---

## Phase 1: Core Implementation

### 1.1 Add Provenance-Aware Methods to `Id<T, ID>`

**File**: `src/id/identifier.rs`

**Current Code** (around line 111-118):
```rust
impl<T, ID> Id<T, ID>
where
    T: Label + ?Sized,
{
    pub fn for_labeled(id: ID) -> Self {
        let labeler = <T as Label>::labeler();
        Self {
            label: SmolStr::new(labeler.label()),
            id,
            marker: PhantomData,
        }
    }
}
```

**Add After** (before or after `for_labeled`, doesn't matter):

```rust
impl<T: Label + ?Sized, ID> Id<T, ID> {
    // ========== EXTERNAL / IMPORTED ==========
    
    /// Create an ID from an external system or legacy source.
    ///
    /// Used for IDs that originate from external systems (APIs, databases,
    /// data imports, migrations) where the value is **opaque** and should
    /// be preserved exactly as received.
    ///
    /// # Semantics
    /// - The ID comes **FROM** an external source
    /// - The value is **opaque** — don't parse, transform, or validate it
    /// - Used with `External<Provider>` or `Imported<From>` provenance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::External;
    ///
    /// pub struct Stripe;
    /// pub type StripeCustomerId = Id<Sourced<Customer, External<Stripe>>, String>;
    ///
    /// let id = StripeCustomerId::from_source("cus_L3H8Z6K9j2");
    /// assert_eq!(id.to_string(), "cus_L3H8Z6K9j2");
    /// ```
    ///
    /// # See Also
    /// - `derived_from()` — for IDs computed from data
    /// - `from_client()` — for user-provided IDs
    /// - For `External<Provider>` provenance
    pub fn from_source(id: ID) -> Self {
        Self::for_labeled(id)
    }

    // ========== DERIVED ==========
    
    /// Create an ID derived deterministically from source data.
    ///
    /// Used for IDs that are **computed** from other fields where the same
    /// source always produces the same ID (unlike `generate()` which is random).
    ///
    /// # Semantics
    /// - The ID is **derived FROM** source data
    /// - Creation is **deterministic** — same input → same output
    /// - Different from `Generated` (which is random)
    /// - Different from `External` (which is opaque)
    /// - Used with `Derived<Method>` provenance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::Derived;
    ///
    /// pub struct Slugify;
    /// pub type PageSlugId = Id<Sourced<Page, Derived<Slugify>>, String>;
    ///
    /// let slug = PageSlugId::derived_from(slugify("My Page Title"));
    /// assert_eq!(slug.to_string(), "my-page-title");
    /// ```
    ///
    /// # See Also
    /// - `from_source()` — for external/legacy IDs
    /// - `generate()` — for random IDs
    /// - For `Derived<Method>` provenance
    pub fn derived_from(id: ID) -> Self {
        Self::for_labeled(id)
    }

    // ========== CLIENT PROVIDED ==========
    
    /// Create an ID provided by a user or client.
    ///
    /// Used for IDs that are **supplied by the client or user** rather than
    /// generated or imported by the system.
    ///
    /// # Semantics
    /// - The ID comes **FROM** the user/client
    /// - Different from `External` which comes from a **system** or **API**
    /// - Used with `ClientProvided` provenance
    /// - Often validated but not transformed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::ClientProvided;
    ///
    /// pub type IdempotencyKey = Id<Sourced<Request, ClientProvided>, String>;
    ///
    /// let key = IdempotencyKey::from_client(client_provided_key);
    /// assert_eq!(key.to_string(), client_provided_key);
    /// ```
    ///
    /// # See Also
    /// - `from_source()` — for external system IDs
    /// - For `ClientProvided` provenance
    pub fn from_client(id: ID) -> Self {
        Self::for_labeled(id)
    }

    // ========== SCOPED ==========
    
    /// Create a context-scoped ID (unique within a scope, not globally).
    ///
    /// Used for IDs that are **unique within a context** (tenant, organization,
    /// workspace) but not globally unique.
    ///
    /// # Semantics
    /// - The ID is **scoped TO** a context
    /// - Uniqueness is **context-relative**, not global
    /// - Used with `Scoped<Scope, Inner>` provenance
    /// - The inner provenance determines allowed operations
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::{Scoped, Generated};
    ///
    /// pub type TenantId = Id<Sourced<Tenant, Generated<UuidV7>>, String>;
    /// pub type TenantResourceId =
    ///     Id<Sourced<Resource, Scoped<TenantId, Generated<UuidV7>>>, String>;
    ///
    /// let resource = TenantResourceId::for_scope(tenant_id);
    /// // Or use the inner provenance's method:
    /// let resource = TenantResourceId::generate();  // Generates, still scoped
    /// ```
    ///
    /// # See Also
    /// - Inner provenance determines `.generate()`, `.from_source()`, etc.
    /// - For `Scoped<Scope, Inner>` provenance
    pub fn for_scope(id: ID) -> Self {
        Self::for_labeled(id)
    }

    // ========== ALIAS ==========
    
    /// Create a secondary identifier (alias) for an entity.
    ///
    /// Used for IDs that are **aliases** or **secondary identifiers** for the
    /// same entity where the primary ID is defined elsewhere.
    ///
    /// # Semantics
    /// - This is a **secondary ID**, not the primary/canonical ID
    /// - Used with `AliasOf<Canonical>` provenance
    /// - Often combined with `Derived` (e.g., email is derived)
    /// - Enables alternative lookup but primary ID is canonical
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::AliasOf;
    ///
    /// pub type UserId = Id<Sourced<User, Generated<UuidV7>>, String>;
    /// pub type UserEmailAlias = Id<Sourced<User, AliasOf<UserId>>, String>;
    ///
    /// let email_alias = UserEmailAlias::alias_for("user@example.com");
    /// assert_eq!(email_alias.to_string(), "user@example.com");
    /// ```
    ///
    /// # See Also
    /// - Often paired with `Derived` for computed aliases
    /// - For `AliasOf<Canonical>` provenance
    pub fn alias_for(id: ID) -> Self {
        Self::for_labeled(id)
    }

    // ========== TEMPORARY ==========
    
    /// Create a temporary ID (valid only short-term, not for persistence).
    ///
    /// Used for **ephemeral IDs** that should not be persisted to databases
    /// or caches.
    ///
    /// # Semantics
    /// - The ID is **FOR TEMPORARY use only**
    /// - Should **never be persisted** to databases or caches
    /// - Used with `Temporary` provenance
    /// - Examples: optimistic IDs, session tokens, request IDs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::Temporary;
    ///
    /// pub type OptimisticId = Id<Sourced<Item, Temporary>, String>;
    ///
    /// let id = OptimisticId::for_temporary(uuid::Uuid::new_v4().to_string());
    /// // ⚠️  Important: never persist this to the database!
    /// ```
    ///
    /// # See Also
    /// - Different from `Generated` which **should** be persisted
    /// - For `Temporary` provenance
    pub fn for_temporary(id: ID) -> Self {
        Self::for_labeled(id)
    }

    // ========== GENERATED (Testing/Fixtures) ==========
    
    /// Create a Generated ID for testing purposes.
    ///
    /// Used in fixtures and tests where you need to construct IDs explicitly
    /// without actually generating them (which is done via `generate()` or
    /// the `Entity` trait in production).
    ///
    /// # Semantics
    /// - Used **only for tests and fixtures**, not production
    /// - For production ID generation, use `generate()` instead
    /// - Used with `Generated<Strategy>` provenance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::Generated;
    ///
    /// pub type UserId = Id<Sourced<User, Generated<UuidV7>>, String>;
    ///
    /// #[test]
    /// fn test_user_creation() {
    ///     let user_id = UserId::for_test("user-123".to_string());
    ///     assert_eq!(user_id.to_string(), "user-123");
    /// }
    ///
    /// // For production:
    /// let user_id = UserId::generate();  // ← use this instead
    /// ```
    ///
    /// # See Also
    /// - `generate()` — for actual ID generation (production)
    /// - For `Generated<Strategy>` provenance
    pub fn for_test(id: ID) -> Self {
        Self::for_labeled(id)
    }
}
```

**Rationale**:
- All methods are **aliases** to `for_labeled()` (zero cost)
- Each method has **detailed rustdoc** with semantics, examples, and rationale
- Methods are grouped by provenance type (comments between groups)
- `for_test()` is guidance for Generated IDs in tests

---

### 1.2 Add Method to `src/lib.rs` Re-exports

**File**: `src/lib.rs` (if needed for visibility)

Currently, `for_labeled` is accessed via the trait. The new methods are on `Id<T, ID>` directly, so no additional re-exports needed. The pub API is already accessible.

---

### 1.3 Update CHANGELOG

**File**: `CHANGELOG.md`

Add under the next version:

```markdown
### Added

- **Provenance-Aware Construction Functions** (#XXX): Added semantic methods to `Id<T, ID>` that clarify ID origin based on provenance type:
  - `from_source()` — for External/Imported IDs from systems
  - `derived_from()` — for Derived IDs computed from data
  - `from_client()` — for ClientProvided IDs from users
  - `for_scope()` — for Scoped IDs with context
  - `alias_for()` — for AliasOf secondary IDs
  - `for_temporary()` — for Temporary ephemeral IDs
  - `for_test()` — for Generated IDs in tests
  
  These are **zero-cost aliases** to `for_labeled()` and serve as **semantic guidance** for choosing the right construction method based on provenance type.
  All methods are **backward compatible** — existing code continues to work.

### Examples

```rust
// External ID from Spark
let app_id = AppId::from_source("app-123");

// Derived ID from slug
let slug = PageSlugId::derived_from(slugify("Title"));

// Temporary ID (don't persist!)
let opt_id = OptimisticId::for_temporary(uuid_v4());
```
```

---

### 1.4 Implementation Checklist

- [ ] Add all 7 new methods to `Id<T, ID>` in `src/id/identifier.rs`
- [ ] Each method has complete rustdoc with semantics, examples, See Also
- [ ] Add comments grouping methods by provenance type
- [ ] Update CHANGELOG.md
- [ ] Run `cargo doc --no-deps --open` to verify rustdoc renders correctly
- [ ] Verify no clippy warnings
- [ ] Verify backward compatibility: all existing tests still pass

---

## Phase 2: Documentation & Lessons

### 2.1 Create Lessons Document

**File**: `ref/lessons/provenance-construction-patterns.md`

This document is **reusable across projects** (Turtle, others) and explains how to use the new API.

**Content Structure**:
1. **Overview** - what provenance-aware construction is
2. **Decision Tree** - how to choose the right function
3. **Eight Patterns** - one section per provenance type:
   - What it means
   - When to use
   - Examples
   - Common mistakes
4. **Migration Guide** - how to update existing code
5. **FAQ** - common questions

**See Phase 2 specification below for detailed outline**.

### 2.2 Update Examples in `examples/`

**Files**: Any example files that show ID creation

**Updates**:
- Replace `for_labeled()` with provenance-aware functions
- Add comments explaining the provenance choice
- Show realistic scenarios per provenance type

Example:
```rust
// examples/provenance_aware_construction.rs

use tagid::id::provenance::*;

// 1. External from Stripe API
let stripe_id = StripeCustomerId::from_source("cus_123");

// 2. Generated for new tenant
let tenant_id = TenantId::generate();

// 3. Derived from URL slug
let page_id = PageSlugId::derived_from(slugify("my-page"));

// 4. Scoped within tenant
let resource_id = TenantResourceId::for_scope(tenant_id);

println!("Examples of all provenance construction patterns");
```

### 2.3 Documentation Checklist

- [ ] Create `ref/lessons/provenance-construction-patterns.md` (see detailed spec)
- [ ] Update examples in `examples/` directory
- [ ] Verify all examples compile (via doctests)
- [ ] Add cross-references to provenance types in `src/id/provenance.rs` rustdoc
- [ ] Ensure lessons document is in `ref/` for reusability across projects

---

## Phase 3: Testing Strategy

### 3.1 Unit Tests in `src/id/identifier.rs`

Add tests after the existing tests (or in a dedicated test module).

**Test File** (if separate): `tests/provenance_construction.rs`

**Tests to Add**:

```rust
#[cfg(test)]
mod provenance_construction_tests {
    use super::*;
    use crate::{Entity, Id, Label, MakeLabeling, Sourced};
    use crate::id::provenance::*;
    use pretty_assertions::assert_eq;

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
    fn test_all_methods_produce_same_result_as_for_labeled() {
        let value = "test-id";

        let from_labeled = ExternalUserId::for_labeled(value.to_string());
        let from_source = ExternalUserId::from_source(value.to_string());

        assert_eq!(from_labeled.to_string(), from_source.to_string());
        assert_eq!(from_labeled.as_str(), from_source.as_str());
    }

    #[test]
    fn test_serialization_is_canonical() {
        let id = ExternalUserId::from_source("ext-123".to_string());
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"ext-123\"");
    }
}
```

**Rationale**:
- Tests verify each function creates an ID correctly
- Tests verify the result is identical to `for_labeled()` (since they're aliases)
- Tests verify serialization is canonical (not including provenance)
- Tests serve as usage examples

### 3.2 Documentation Tests in Rustdoc

The examples in the method rustdoc (Phase 1.1) are **automatically tested** by `cargo test --doc`.

Ensure all examples:
- [ ] Are correct and runnable (use `ignore` only if necessary with comment why)
- [ ] Show realistic usage
- [ ] Pass `cargo test --doc`

### 3.3 Integration Tests

**File**: `tests/provenance_construction_integration.rs`

Tests that combine multiple provenance types in realistic scenarios:

```rust
#[test]
fn test_multi_provenance_workflow() {
    // Given: mixed provenance IDs in a realistic workflow
    let user_id = GeneratedUserId::generate();  // Generated
    let email = UserEmailAlias::alias_for("user@example.com");  // Alias
    let temp_token = TemporaryUserId::for_temporary(uuid_v4());  // Temporary

    // When: using them together
    // Then: all should serialize canonically
    assert!(!user_id.to_string().is_empty());
    assert_eq!(email.to_string(), "user@example.com");
    assert!(!temp_token.to_string().is_empty());
}
```

### 3.4 Testing Checklist

- [ ] Add unit tests for each of the 7 new methods
- [ ] Tests verify identity with `for_labeled()`
- [ ] All rustdoc examples pass `cargo test --doc`
- [ ] Integration tests cover multi-provenance scenarios
- [ ] All tests pass: `cargo test --all`
- [ ] No coverage regressions

---

## Phase 4: Downstream Integration

### 4.1 Update Turtle-Core

**File**: `crates/turtle-core/src/domain/ids.rs`

**Current** (using `for_labeled()`):
```rust
pub fn app_id(value: impl Into<String>) -> AppId {
    AppId::for_labeled(value.into())
}
```

**Updated** (using `from_source()`):
```rust
pub fn app_id(value: impl Into<String>) -> AppId {
    AppId::from_source(value.into())
}

pub fn job_id(value: u64) -> JobId {
    JobId::from_source(value)
}

pub fn stage_id(value: u64) -> StageId {
    StageId::from_source(value)
}

pub fn task_id(value: u64) -> TaskId {
    TaskId::from_source(value)
}

pub fn executor_id(value: impl Into<String>) -> ExecutorId {
    ExecutorId::from_source(value.into())
}
```

**Rationale**: All Spark IDs are `External<providers::Spark>`, so `from_source()` is semantically correct.

### 4.2 Update Turtle-Spark (mcp_client.rs)

**File**: `crates/turtle-spark/src/mcp_client.rs`

**Changes**:
- Replace `AppId::for_labeled()` with `ids::app_id()`
- Replace `JobId::for_labeled()` with `ids::job_id()`
- Update all ID creation in parsing functions

**Example** (parse_app_info function, line 905):
```rust
// Before
Some(AppInfo {
    id: ids::app_id(id),
    ...
})

// After (already using ids::app_id())
Some(AppInfo {
    id: ids::app_id(id),  // ✅ already correct
    ...
})
```

### 4.3 Verification Checklist

- [ ] All ID construction in turtle-core uses provenance-aware functions
- [ ] All ID construction in turtle-spark uses provenance-aware functions
- [ ] All tests still pass: `cargo test --workspace`
- [ ] Code review confirms semantic correctness
- [ ] No breaking changes to public API

---

## Risk Mitigation

### Risk 1: Confusion about which function to use

**Mitigation**:
- Create decision tree in `ref/lessons/provenance-construction-patterns.md`
- Add examples per provenance type in rustdoc
- Code review checklist: "Is the provenance type and construction function aligned?"

### Risk 2: Developers continue using `for_labeled()`

**Mitigation**:
- Document that `for_labeled()` is still valid but deprecated for new code
- Linter/clippy rule (future): warn if `for_labeled()` is used without provenance context
- Code review: guide toward provenance-aware functions

### Risk 3: Incomplete rustdoc examples

**Mitigation**:
- All examples must compile and pass `cargo test --doc`
- Use `ignore` attribute only with explicit comment why
- Review all examples before release

### Risk 4: Backward incompatibility

**Mitigation**:
- `for_labeled()` remains unchanged and fully functional
- All new methods are aliases (zero-cost)
- All existing tests must pass
- No breaking changes to traits or public APIs

---

## Timeline & Dependencies

### Critical Path

```
Phase 1: Core Implementation (1-2 days)
    ├─ Add 7 methods to Id<T, ID>
    ├─ Add comprehensive rustdoc
    ├─ Update CHANGELOG
    └─ Run all tests

Phase 2: Documentation (1-2 days)
    ├─ Create ref/lessons/provenance-construction-patterns.md
    ├─ Update examples/
    └─ Review and test all docs

Phase 3: Testing (1 day)
    ├─ Add unit tests for each method
    ├─ Verify doctests pass
    └─ Add integration tests

Phase 4: Downstream (2-3 days)
    ├─ Update turtle-core/ids.rs
    ├─ Update turtle-spark/mcp_client.rs
    └─ Test and verify
```

**Total**: ~5-8 days with concurrent work

### Dependencies

1. **Phase 1** must complete before Phases 2-4 (core API needed)
2. **Phase 2** can start after Phase 1 (documentation uses new API)
3. **Phase 3** can run in parallel with Phase 1-2 (tests are independent)
4. **Phase 4** must wait for Phase 1 (downstream depends on new API)

### Blockers

- None (all work is additive, no breaking changes)

---

## Acceptance Criteria (Cross-Phase)

### Phase 1 (Core)
- [ ] All 7 new methods implemented in `src/id/identifier.rs`
- [ ] Each method has complete rustdoc with examples
- [ ] CHANGELOG updated
- [ ] All existing tests pass
- [ ] `cargo doc --no-deps` generates without warnings
- [ ] Zero clippy warnings on new code

### Phase 2 (Documentation)
- [ ] `ref/lessons/provenance-construction-patterns.md` complete
- [ ] Examples in `examples/` updated and tested
- [ ] All rustdoc examples pass `cargo test --doc`
- [ ] Cross-references between docs and code verified

### Phase 3 (Testing)
- [ ] Unit tests cover all 7 new methods
- [ ] Integration tests cover multi-provenance scenarios
- [ ] All tests pass: `cargo test --all`
- [ ] Code coverage ≥ 85%

### Phase 4 (Downstream)
- [ ] All Spark ID creation in turtle-core uses new API
- [ ] All ID parsing in turtle-spark uses new API
- [ ] All tests pass: `cargo test --workspace`
- [ ] Code review confirms semantic correctness

---

## Success Criteria

| Criterion | Measure | Target |
|-----------|---------|--------|
| **API Completeness** | All 8 provenance types have guidance | 100% |
| **Documentation** | Doctests pass, examples are clear | 100% pass |
| **Test Coverage** | Unit + integration tests | ≥85% |
| **Backward Compat** | Existing code still works | 100% pass |
| **Downstream Adoption** | turtle-core/spark use new API | 100% |
| **Code Quality** | Clippy warnings, doc warnings | 0 |

