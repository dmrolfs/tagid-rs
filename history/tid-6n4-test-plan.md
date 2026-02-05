# Provenance-Aware Construction API: Detailed Test Plan

**Status**: Test Specification  
**Date**: 2026-02-04  
**Related Docs**: 
- `tid-6n4-provenance-construction-analysis.md` (design)
- `tid-6n4-implementation-plan.md` (implementation)

---

## Table of Contents

1. [Test Strategy](#test-strategy)
2. [Unit Tests](#unit-tests)
3. [Integration Tests](#integration-tests)
4. [Documentation Tests](#documentation-tests)
5. [Backward Compatibility Tests](#backward-compatibility-tests)
6. [Test Implementation Details](#test-implementation-details)
7. [Test Execution & Verification](#test-execution--verification)

---

## Test Strategy

### Philosophy

Use a **multi-layered testing pyramid**:
- **Layer 1 (Unit)**: Each function works correctly in isolation
- **Layer 2 (Integration)**: Functions work together in realistic scenarios
- **Layer 3 (Documentation)**: Examples are correct and executable
- **Layer 4 (Compatibility)**: Backward compatibility maintained

### Approach

1. **Test-First** — write tests as we understand what to test
2. **Comprehensive Rustdoc Examples** — documentation is testable
3. **Real Scenarios** — integration tests use realistic workflows
4. **Canonical Form** — verify serialization is ID-only, no provenance

### Coverage Targets

| Layer | Target | Rationale |
|-------|--------|-----------|
| **Unit** | 100% | Each function is simple; all branches testable |
| **Integration** | 80%+ | Real workflows; some combinations not realistic |
| **Doc Tests** | 100% | All examples must be runnable |
| **Compat** | 100% | Critical: must not break existing code |

---

## Unit Tests

### Test File Location

Create tests in **two places**:

1. **`src/id/identifier.rs`** - Inline tests (simple, quick feedback)
2. **`tests/provenance_construction_unit.rs`** - Dedicated test file (comprehensive)

### Test Module Structure

```rust
#[cfg(test)]
mod provenance_construction_tests {
    use super::*;
    use crate::id::provenance::*;
    use crate::id::sourced::Sourced;
    use crate::{Entity, Id, Label, MakeLabeling};
    use pretty_assertions::assert_eq;
    use serde_json::json;

    // --- Setup: Entities and Type Aliases ---
    
    // --- Test Groups (organized by function) ---
    mod from_source { ... }
    mod derived_from { ... }
    mod from_client { ... }
    mod for_scope { ... }
    mod alias_for { ... }
    mod for_temporary { ... }
    mod for_test { ... }

    // --- Shared Utilities ---
}
```

### Test 1: from_source()

**Tests**:
```rust
mod from_source {
    use super::*;

    type ExternalId = Id<Sourced<TestEntity, External<()>>, String>;

    #[test]
    fn test_from_source_creates_id() {
        // Given: a value from an external source
        let value = "external-123";
        
        // When: creating an ID using from_source()
        let id = ExternalId::from_source(value.to_string());
        
        // Then: ID is created with that value
        assert_eq!(id.to_string(), value);
        assert_eq!(id.as_str(), value);
    }

    #[test]
    fn test_from_source_preserves_opaque_value() {
        // Given: an opaque external value (Stripe customer ID)
        let stripe_id = "cus_L3H8Z6K9j2";
        
        // When: creating an ID
        let id = ExternalId::from_source(stripe_id.to_string());
        
        // Then: value is preserved exactly (no parsing, no transformation)
        assert_eq!(id.to_string(), stripe_id);
    }

    #[test]
    fn test_from_source_equivalence_with_for_labeled() {
        // Given: same value
        let value = "test-123";
        
        // When: creating IDs both ways
        let from_source = ExternalId::from_source(value.to_string());
        let for_labeled = ExternalId::for_labeled(value.to_string());
        
        // Then: both produce identical IDs
        assert_eq!(from_source.to_string(), for_labeled.to_string());
        assert_eq!(from_source.as_str(), for_labeled.as_str());
    }

    #[test]
    fn test_from_source_serializes_canonically() {
        // Given: an ID from external source
        let id = ExternalId::from_source("ext-123".to_string());
        
        // When: serializing to JSON
        let json = serde_json::to_string(&id).unwrap();
        
        // Then: JSON is just the value, no provenance or label
        assert_eq!(json, "\"ext-123\"");
    }

    #[test]
    fn test_from_source_with_special_characters() {
        // Given: external value with special characters (e.g., URL, ARN)
        let aws_arn = "arn:aws:s3:::bucket/key?version=123";
        
        // When: creating ID from ARN
        let id = ExternalId::from_source(aws_arn.to_string());
        
        // Then: ARN is preserved exactly
        assert_eq!(id.to_string(), aws_arn);
    }

    #[test]
    fn test_from_source_numeric_type() {
        // Given: external value as u64 (e.g., Spark Job ID)
        type ExternalJobId = Id<Sourced<Job, External<()>>, u64>;
        
        let job_num = 42u64;
        
        // When: creating ID
        let id = ExternalJobId::from_source(job_num);
        
        // Then: numeric value is preserved
        assert_eq!(*id.as_ref(), job_num);
    }
}
```

### Test 2: derived_from()

**Tests**:
```rust
mod derived_from {
    use super::*;

    type DerivedId = Id<Sourced<TestEntity, Derived<()>>, String>;

    #[test]
    fn test_derived_from_creates_id() {
        // Given: a source value to derive from
        let source = "my-page-title";
        
        // When: creating a derived ID
        let id = DerivedId::derived_from(source.to_string());
        
        // Then: ID contains the derived value
        assert_eq!(id.to_string(), source);
    }

    #[test]
    fn test_derived_from_is_deterministic() {
        // Given: same source data
        let source = "identical-source";
        
        // When: creating derived IDs multiple times
        let id1 = DerivedId::derived_from(source.to_string());
        let id2 = DerivedId::derived_from(source.to_string());
        
        // Then: both IDs are identical (deterministic)
        assert_eq!(id1.to_string(), id2.to_string());
    }

    #[test]
    fn test_derived_from_equivalence_with_for_labeled() {
        // Given: same value
        let value = "derived-123";
        
        // When: creating IDs both ways
        let derived = DerivedId::derived_from(value.to_string());
        let for_labeled = DerivedId::for_labeled(value.to_string());
        
        // Then: both are identical
        assert_eq!(derived.to_string(), for_labeled.to_string());
    }

    #[test]
    fn test_derived_from_with_slug() {
        // Given: a slug-like value (from slugify function)
        let slug = "my-page-title-123";
        
        // When: creating derived ID
        let id = DerivedId::derived_from(slug.to_string());
        
        // Then: slug is preserved exactly
        assert_eq!(id.to_string(), slug);
    }

    #[test]
    fn test_derived_from_serializes_canonically() {
        // Given: derived ID
        let id = DerivedId::derived_from("slug".to_string());
        
        // When: serializing
        let json = serde_json::to_string(&id).unwrap();
        
        // Then: JSON is just the value
        assert_eq!(json, "\"slug\"");
    }
}
```

### Test 3: from_client()

**Tests**:
```rust
mod from_client {
    use super::*;

    type ClientId = Id<Sourced<TestEntity, ClientProvided>, String>;

    #[test]
    fn test_from_client_creates_id() {
        // Given: client-provided value
        let client_value = "client-supplied-123";
        
        // When: creating from client
        let id = ClientId::from_client(client_value.to_string());
        
        // Then: ID is created
        assert_eq!(id.to_string(), client_value);
    }

    #[test]
    fn test_from_client_preserves_exact_value() {
        // Given: client idempotency key
        let key = "req-2026-02-04-12345-abcde";
        
        // When: creating ID from client key
        let id = ClientId::from_client(key.to_string());
        
        // Then: key is preserved exactly
        assert_eq!(id.as_str(), key);
    }

    #[test]
    fn test_from_client_equivalence_with_for_labeled() {
        // Given: same value
        let value = "test-key";
        
        // When: creating both ways
        let from_client = ClientId::from_client(value.to_string());
        let for_labeled = ClientId::for_labeled(value.to_string());
        
        // Then: identical
        assert_eq!(from_client.to_string(), for_labeled.to_string());
    }

    #[test]
    fn test_from_client_with_uuid() {
        // Given: UUID from client (common in idempotency keys)
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        
        // When: creating from client
        let id = ClientId::from_client(uuid.to_string());
        
        // Then: UUID is preserved
        assert_eq!(id.to_string(), uuid);
    }
}
```

### Test 4: for_scope()

**Tests**:
```rust
mod for_scope {
    use super::*;

    type ScopedId = Id<Sourced<TestEntity, Scoped<(), Generated<()>>>, String>;

    #[test]
    fn test_for_scope_creates_id() {
        // Given: a scoped value
        let scope_value = "scope-123";
        
        // When: creating scoped ID
        let id = ScopedId::for_scope(scope_value.to_string());
        
        // Then: ID is created
        assert_eq!(id.to_string(), scope_value);
    }

    #[test]
    fn test_for_scope_preserves_scope_context() {
        // Given: scope ID (e.g., tenant ID)
        let scope = "tenant-abc123";
        
        // When: creating resource ID within that scope
        let id = ScopedId::for_scope(scope.to_string());
        
        // Then: scope is preserved in the ID value
        assert_eq!(id.to_string(), scope);
    }

    #[test]
    fn test_for_scope_equivalence_with_for_labeled() {
        // Given: same scope value
        let scope = "scope-value";
        
        // When: creating both ways
        let for_scope = ScopedId::for_scope(scope.to_string());
        let for_labeled = ScopedId::for_labeled(scope.to_string());
        
        // Then: identical
        assert_eq!(for_scope.to_string(), for_labeled.to_string());
    }
}
```

### Test 5: alias_for()

**Tests**:
```rust
mod alias_for {
    use super::*;

    type AliasId = Id<Sourced<TestEntity, AliasOf<()>>, String>;

    #[test]
    fn test_alias_for_creates_id() {
        // Given: an alias value
        let alias = "user@example.com";
        
        // When: creating alias ID
        let id = AliasId::alias_for(alias.to_string());
        
        // Then: ID is created
        assert_eq!(id.to_string(), alias);
    }

    #[test]
    fn test_alias_for_with_email() {
        // Given: user email (common alias)
        let email = "user@example.com";
        
        // When: creating email alias
        let id = AliasId::alias_for(email.to_string());
        
        // Then: email is preserved
        assert_eq!(id.as_str(), email);
    }

    #[test]
    fn test_alias_for_equivalence_with_for_labeled() {
        // Given: same value
        let value = "alias-123";
        
        // When: creating both ways
        let alias = AliasId::alias_for(value.to_string());
        let for_labeled = AliasId::for_labeled(value.to_string());
        
        // Then: identical
        assert_eq!(alias.to_string(), for_labeled.to_string());
    }
}
```

### Test 6: for_temporary()

**Tests**:
```rust
mod for_temporary {
    use super::*;

    type TempId = Id<Sourced<TestEntity, Temporary>, String>;

    #[test]
    fn test_for_temporary_creates_id() {
        // Given: a temporary ID value
        let temp = "optimistic-123";
        
        // When: creating temporary ID
        let id = TempId::for_temporary(temp.to_string());
        
        // Then: ID is created
        assert_eq!(id.to_string(), temp);
    }

    #[test]
    fn test_for_temporary_with_uuid() {
        // Given: UUID for optimistic ID
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        
        // When: creating temporary ID
        let id = TempId::for_temporary(uuid.to_string());
        
        // Then: UUID is preserved (but this ID should not be persisted!)
        assert_eq!(id.to_string(), uuid);
    }

    #[test]
    fn test_for_temporary_equivalence_with_for_labeled() {
        // Given: same value
        let value = "temp-123";
        
        // When: creating both ways
        let temp = TempId::for_temporary(value.to_string());
        let for_labeled = TempId::for_labeled(value.to_string());
        
        // Then: identical
        assert_eq!(temp.to_string(), for_labeled.to_string());
    }

    #[test]
    fn test_for_temporary_serializes_canonically() {
        // Given: temporary ID
        let id = TempId::for_temporary("temp".to_string());
        
        // When: serializing
        let json = serde_json::to_string(&id).unwrap();
        
        // Then: JSON is just the value (not persisting provenance)
        assert_eq!(json, "\"temp\"");
    }
}
```

### Test 7: for_test()

**Tests**:
```rust
mod for_test {
    use super::*;

    type GeneratedId = Id<Sourced<TestEntity, Generated<()>>, String>;

    #[test]
    fn test_for_test_creates_id() {
        // Given: a test ID value
        let test_value = "test-123";
        
        // When: creating for tests
        let id = GeneratedId::for_test(test_value.to_string());
        
        // Then: ID is created
        assert_eq!(id.to_string(), test_value);
    }

    #[test]
    fn test_for_test_equivalence_with_for_labeled() {
        // Given: same value
        let value = "test-id";
        
        // When: creating both ways
        let for_test = GeneratedId::for_test(value.to_string());
        let for_labeled = GeneratedId::for_labeled(value.to_string());
        
        // Then: identical
        assert_eq!(for_test.to_string(), for_labeled.to_string());
    }

    #[test]
    fn test_for_test_in_fixture() {
        // Given: a test fixture needing a specific ID
        let fixture_id = GeneratedId::for_test("fixture-user-1".to_string());
        
        // When: using the ID in a test
        let result = fixture_id.to_string();
        
        // Then: ID is available for assertions
        assert_eq!(result, "fixture-user-1");
    }
}
```

### Cross-Functional Tests

```rust
#[test]
fn test_all_methods_are_semantic_aliases() {
    // Given: different provenance types
    type ExternalId = Id<Sourced<TestEntity, External<()>>, String>;
    type DerivedId = Id<Sourced<TestEntity, Derived<()>>, String>;
    
    let value = "test-123";
    
    // When: creating IDs with different methods on same provenance
    let ext1 = ExternalId::from_source(value.to_string());
    let ext2 = ExternalId::for_labeled(value.to_string());
    
    let der1 = DerivedId::derived_from(value.to_string());
    let der2 = DerivedId::for_labeled(value.to_string());
    
    // Then: all produce identical results (all are aliases)
    assert_eq!(ext1.to_string(), ext2.to_string());
    assert_eq!(der1.to_string(), der2.to_string());
}

#[test]
fn test_all_methods_preserve_label() {
    // Given: an entity with a specific label
    type TestId = Id<Sourced<TestEntity, External<()>>, String>;
    
    // When: creating IDs using different methods
    let from_source = TestId::from_source("id".to_string());
    let for_labeled = TestId::for_labeled("id".to_string());
    
    // Then: both have the same label
    assert_eq!(from_source.label, for_labeled.label);
}
```

---

## Integration Tests

### Test File Location

**File**: `tests/provenance_construction_integration.rs`

### Test 1: Multi-Provenance Workflow

**Scenario**: Realistic workflow using multiple provenance types together

```rust
#[test]
fn test_user_creation_workflow_with_multiple_provenances() {
    // Setup
    type GeneratedUserId = Id<Sourced<User, Generated<()>>, String>;
    type EmailAlias = Id<Sourced<User, AliasOf<()>>, String>;
    type TempToken = Id<Sourced<Session, Temporary>, String>;

    // When: user is created and registered
    let user_id = GeneratedUserId::for_test("user-123".to_string());
    let email = EmailAlias::alias_for("user@example.com".to_string());
    let session_token = TempToken::for_temporary("session-xyz".to_string());

    // Then: all IDs coexist correctly
    assert!(!user_id.to_string().is_empty());
    assert_eq!(email.to_string(), "user@example.com");
    assert_eq!(session_token.to_string(), "session-xyz");

    // And: they serialize independently
    let user_json = serde_json::to_value(&user_id).unwrap();
    let email_json = serde_json::to_value(&email).unwrap();
    let session_json = serde_json::to_value(&session_token).unwrap();

    assert!(user_json.is_string());
    assert!(email_json.is_string());
    assert!(session_json.is_string());
}
```

### Test 2: External Data Import

**Scenario**: Importing data from external system (e.g., Spark History Server)

```rust
#[test]
fn test_spark_import_workflow() {
    // Setup
    type AppId = Id<Sourced<Application, External<()>>, String>;
    type JobId = Id<Sourced<Job, External<()>>, u64>;

    // When: importing from Spark
    let app = AppId::from_source("app-20260204-001".to_string());
    let job = JobId::from_source(42u64);

    // Then: values are preserved exactly
    assert_eq!(app.to_string(), "app-20260204-001");
    assert_eq!(*job.as_ref(), 42u64);

    // And: serialization is canonical (no provenance)
    let app_json = serde_json::to_string(&app).unwrap();
    let job_json = serde_json::to_string(&job).unwrap();

    assert_eq!(app_json, "\"app-20260204-001\"");
    assert_eq!(job_json, "42");
}
```

### Test 3: Scoped Resource in Multi-Tenant System

**Scenario**: Resources scoped to tenants

```rust
#[test]
fn test_tenant_scoped_resources() {
    // Setup
    type TenantId = Id<Sourced<Tenant, Generated<()>>, String>;
    type ResourceId = Id<Sourced<Resource, Scoped<(), Generated<()>>>, String>;

    // When: creating resources in different tenants
    let tenant1_id = TenantId::for_test("tenant-1".to_string());
    let tenant2_id = TenantId::for_test("tenant-2".to_string());

    let resource1 = ResourceId::for_scope(tenant1_id.to_string());
    let resource2 = ResourceId::for_scope(tenant2_id.to_string());

    // Then: resources are scoped to their tenants
    assert_eq!(resource1.to_string(), "tenant-1");
    assert_eq!(resource2.to_string(), "tenant-2");

    // And: different tenants have different resources
    assert_ne!(resource1.to_string(), resource2.to_string());
}
```

### Test 4: Derived IDs (Slugs)

**Scenario**: Creating derived IDs from source data

```rust
#[test]
fn test_slug_generation_workflow() {
    // Setup
    type PageSlugId = Id<Sourced<Page, Derived<()>>, String>;

    // When: deriving IDs from page titles
    let slug1 = PageSlugId::derived_from("my-first-post".to_string());
    let slug2 = PageSlugId::derived_from("my-first-post".to_string());  // Same title
    let slug3 = PageSlugId::derived_from("different-title".to_string());

    // Then: same source produces same ID (deterministic)
    assert_eq!(slug1.to_string(), slug2.to_string());
    
    // And: different sources produce different IDs
    assert_ne!(slug1.to_string(), slug3.to_string());
}
```

---

## Documentation Tests

### Approach

All examples in rustdoc are **automatically tested** by `cargo test --doc`.

### Verification

Ensure all examples:
1. Are **syntactically correct** (compile without errors)
2. Are **semantically meaningful** (show real usage)
3. Pass `cargo test --doc` (output is correct)
4. Use `ignore` **only with explicit comments** explaining why

### Example

In method rustdoc:

```rust
/// # Examples
///
/// ```rust,ignore
/// // This would require setting up a full entity with Label impl
/// // For working example, see tests/provenance_construction_integration.rs
/// ```
```

Or better, provide a **simpler working example**:

```rust
/// # Examples
///
/// ```
/// use tagid::id::provenance::External;
/// use tagid::{Id, Sourced};
///
/// // In practice, your entity type would implement Label and Entity
/// // This shows the semantic usage:
///
/// let id = Id::from_source("external-value");
/// assert_eq!(id.to_string(), "external-value");
/// ```
```

---

## Backward Compatibility Tests

### Verification

Ensure **100% of existing tests pass** without modification.

```bash
# Run all existing tests
cargo test --lib

# Run all tests including doctests
cargo test

# Verify no new clippy warnings
cargo clippy --all-targets
```

### Key Tests

1. **`from_labeled()` still works** (regression test)
   ```rust
   #[test]
   fn test_for_labeled_still_works() {
       let id = TestId::for_labeled("value".to_string());
       assert_eq!(id.to_string(), "value");
   }
   ```

2. **Existing usage patterns unchanged**
   - All tests that used `for_labeled()` continue to work
   - No API changes to public methods
   - No breaking changes to traits

3. **Serialization format unchanged**
   - JSON serialization is still canonical (ID value only)
   - No provenance in serialized form
   - No label in serialized form

---

## Test Implementation Details

### Test Utilities

Create a module for common test helpers:

```rust
#[cfg(test)]
mod test_utils {
    use super::*;
    use crate::{Entity, Id, Label, MakeLabeling, Sourced};
    use crate::id::provenance::*;

    // Mock entity for testing
    #[derive(Debug)]
    pub struct TestEntity;

    impl Label for TestEntity {
        type Labeler = MakeLabeling<Self>;
        fn labeler() -> Self::Labeler {
            MakeLabeling::default()
        }
    }

    // Type aliases for each provenance
    pub type ExternalTestId = Id<Sourced<TestEntity, External<()>>, String>;
    pub type GeneratedTestId = Id<Sourced<TestEntity, Generated<()>>, String>;
    pub type DerivedTestId = Id<Sourced<TestEntity, Derived<()>>, String>;
    pub type ClientTestId = Id<Sourced<TestEntity, ClientProvided>, String>;
    pub type AliasTestId = Id<Sourced<TestEntity, AliasOf<()>>, String>;
    pub type TempTestId = Id<Sourced<TestEntity, Temporary>, String>;
    pub type ScopedTestId = Id<Sourced<TestEntity, Scoped<(), Generated<()>>>, String>;

    // Helper: assert ID properties
    pub fn assert_id_is_canonical(id: &str, expected: &str) {
        assert_eq!(id, expected, "ID should be canonical (no label, no provenance)");
    }
}
```

### Naming Conventions

- `test_<function>_<behavior>` — descriptive names
- `test_<function>_equivalence_with_for_labeled` — backward compat tests
- `test_<function>_serializes_canonically` — format tests
- `test_<function>_with_<scenario>` — real-world examples

---

## Test Execution & Verification

### Command Checklist

```bash
# Phase 1: Unit tests
cargo test --lib provenance_construction

# Phase 2: Documentation tests
cargo test --doc

# Phase 3: All tests
cargo test --all

# Phase 4: Code quality
cargo clippy --all-targets --all-features -- -D warnings

# Phase 5: Coverage (requires tarpaulin)
cargo tarpaulin --out Html --timeout 120
```

### Expected Results

| Test Type | Expected |
|-----------|----------|
| **Unit** | All pass, ≥100% coverage |
| **Doc Tests** | All pass (examples compile & run) |
| **Integration** | All pass, realistic scenarios work |
| **Compat** | All existing tests pass unchanged |
| **Clippy** | Zero warnings on new code |

### Failure Handling

If tests fail:

1. **Unit test failure** → Fix the implementation (likely a bug)
2. **Doc test failure** → Fix the example (syntax or output)
3. **Integration failure** → Debug multi-function interaction
4. **Compat failure** → Revert change (API regression)
5. **Clippy warning** → Address code quality issue

---

## Test Coverage Matrix

| Function | Unit | Integration | Doc | Compat |
|----------|------|-------------|-----|--------|
| `from_source()` | ✅ | ✅ | ✅ | ✅ |
| `derived_from()` | ✅ | ✅ | ✅ | ✅ |
| `from_client()` | ✅ | ✅ | ✅ | ✅ |
| `for_scope()` | ✅ | ✅ | ✅ | ✅ |
| `alias_for()` | ✅ | ✅ | ✅ | ✅ |
| `for_temporary()` | ✅ | ✅ | ✅ | ✅ |
| `for_test()` | ✅ | ✅ | ✅ | ✅ |
| Cross-functional | ✅ | ✅ | ✅ | ✅ |
| Backward compat | N/A | ✅ | ✅ | ✅ |

---

## Test Acceptance Criteria

- [ ] All unit tests pass: `cargo test --lib`
- [ ] All doc tests pass: `cargo test --doc`
- [ ] All integration tests pass: `cargo test --all`
- [ ] Code coverage ≥ 85%
- [ ] Zero new clippy warnings
- [ ] 100% of existing tests pass unchanged
- [ ] All examples in rustdoc compile and run
- [ ] No regression in backward compatibility

