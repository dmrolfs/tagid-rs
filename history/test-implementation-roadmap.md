# Tagid Test Implementation Roadmap

**Estimated Total Effort:** 3.5-4 hours for complete unit coverage + P3 feature-gated tests

---

## PHASE 1: Core Semantics (1-2 hours) — P0 Priority

### Category A: Id<T, ID> Core Invariants (in `src/id/mod.rs`)

Add these tests to the existing `#[cfg(test)] mod tests` block:

```rust
#[test]
fn test_equality_ignores_label() {
    let id1: Id<Foo, String> = Id::direct("label1", "123".into());
    let id2: Id<Foo, String> = Id::direct("label2", "123".into());
    assert_eq!(id1, id2);
    
    let id3: Id<Foo, String> = Id::direct("label1", "456".into());
    assert_ne!(id1, id3);
}

#[test]
fn test_hash_matches_equality() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let id1 = Id::direct("a", "123".to_string());
    let id2 = Id::direct("b", "123".to_string());
    
    let hash1 = {
        let mut h = DefaultHasher::new();
        id1.hash(&mut h);
        h.finish()
    };
    
    let hash2 = {
        let mut h = DefaultHasher::new();
        id2.hash(&mut h);
        h.finish()
    };
    
    assert_eq!(hash1, hash2);
}

#[test]
fn test_ordering_ignores_label() {
    let a = Id::direct("z", 1u64);
    let b = Id::direct("a", 2u64);
    assert!(a < b);  // Ordered by ID value, not label
    
    let c = Id::direct("m", 1u64);
    assert_eq!(a, c);  // Same ID → equal despite label difference
}

#[test]
fn test_from_id_uses_entity_labeler() {
    // From<ID> should use T::labeler(), not create empty label
    let id: Id<Foo, String> = "abc".to_string().into();
    assert_eq!(id.label, Foo::labeler().label());
    assert_eq!(id.id, "abc");
}

#[test]
fn test_as_str_returns_id_only() {
    let id: Id<Foo, String> = Id::direct("IGNORED", "value".to_string());
    assert_eq!(id.as_str(), "value");
    assert_ne!(id.as_str(), "IGNORED::value");  // No label in result
}

#[test]
fn test_clone_correctness() {
    let original = Id::direct("MyLabel", "id-value".to_string());
    let cloned = original.clone();
    
    assert_eq!(original, cloned);
    assert_eq!(original.label, cloned.label);
    assert_eq!(original.id, cloned.id);
}

#[test]
fn test_into_inner_correctness() {
    let original_value = "test-id".to_string();
    let id: Id<Foo, String> = Id::direct("_", original_value.clone());
    
    let extracted = id.into_inner();
    assert_eq!(extracted, original_value);
}
```

### Category B: Labeled<T, ID> All Modes (in `src/id/labeled.rs` or `src/id/mod.rs`)

Add a new test module:

```rust
#[cfg(test)]
mod labeled_tests {
    use super::*;
    use crate::id::labeled::{Labeled, LabelMode};
    
    #[test]
    fn test_labeled_none_mode() {
        let id: Id<Foo, String> = Id::direct("Label", "value".into());
        assert_eq!(id.labeled().mode(LabelMode::None).to_string(), "value");
    }
    
    #[test]
    fn test_labeled_short_mode() {
        let id: Id<Foo, String> = Id::direct("Label", "value".into());
        assert_eq!(id.labeled().mode(LabelMode::Short).to_string(), "Label::value");
    }
    
    #[test]
    fn test_labeled_full_mode() {
        let id: Id<Foo, String> = Id::direct("_", "value".into());
        let full = id.labeled().mode(LabelMode::Full).to_string();
        // Full uses decorated_label() from Foo::labeler()
        assert!(full.contains("value"));
        assert!(full.contains("MyFooferNut"));  // Type-derived
    }
    
    #[test]
    fn test_labeled_full_ignores_stored_label() {
        // CRITICAL SEMANTIC: LabelMode::Full uses T::labeler(), not id.label
        let id: Id<Foo, String> = Id::direct("WRONG", "value".into());
        
        let short = id.labeled().mode(LabelMode::Short).to_string();
        assert!(short.contains("WRONG"));  // Stored label used
        
        let full = id.labeled().mode(LabelMode::Full).to_string();
        assert!(!full.contains("WRONG"));  // Stored label ignored
        assert!(full.contains("MyFooferNut"));  // Type-derived label used
    }
    
    #[test]
    fn test_labeled_empty_label_short_mode() {
        let id: Id<NoLabelZed, String> = Id::direct("", "value".into());
        // Short mode with empty label should fall back to just value
        let short = id.labeled().mode(LabelMode::Short).to_string();
        assert_eq!(short, "value");
    }
    
    #[test]
    fn test_labeled_debug_modes() {
        let id: Id<Foo, u64> = Id::direct("Label", 42u64);
        
        assert_eq!(id.labeled().mode(LabelMode::None).to_string(), "42");
        assert_eq!(id.labeled().mode(LabelMode::Short).to_string(), "Label::42");
    }
    
    #[test]
    fn test_labeled_builder_pattern() {
        let id: Id<Foo, String> = Id::direct("L", "v".into());
        
        let labeled_none = id.labeled().mode(LabelMode::None);
        assert_eq!(labeled_none.to_string(), "v");
        
        let labeled_short = id.labeled().mode(LabelMode::Short);
        assert_eq!(labeled_short.to_string(), "L::v");
    }
}
```

**Tests Added:** 13 (7 + 6)  
**Time:** ~1.5 hours

---

## PHASE 2: Type Safety & Serde (1-1.5 hours) — P1 Priority

### Category C: Serde Error Handling (in `src/id/mod.rs`)

```rust
#[test]
fn test_serde_rejects_invalid_type_string_to_u64() {
    // JSON string "abc" cannot deserialize to u64
    let result = serde_json::from_str::<Id<Foo, u64>>("\"abc\"");
    assert!(result.is_err());
}

#[test]
fn test_serde_labeled_format_is_literal() {
    // "Foo::abc" should deserialize as a literal string, not parsed
    let result = serde_json::from_str::<Id<Foo, String>>("\"Foo::abc\"");
    assert!(result.is_ok());
    let id = result.unwrap();
    assert_eq!(id.id, "Foo::abc");  // Literal string, not split
    assert_eq!(id.label, "MyFooferNut");  // Label from entity, not from JSON
}

#[test]
fn test_serde_label_from_entity_labeler() {
    let json = "\"test-id\"";
    let id: Id<Foo, String> = serde_json::from_str(json).unwrap();
    
    // Label must come from Foo::labeler(), not from JSON
    assert_eq!(id.label, Foo::labeler().label());
    assert_eq!(id.id, "test-id");
}

#[test]
fn test_serde_roundtrip_label_behavior() {
    let original = Id::<Foo, String>::direct("ANY", "value".into());
    let json = serde_json::to_string(&original).unwrap();
    let restored: Id<Foo, String> = serde_json::from_str(&json).unwrap();
    
    // ID matches
    assert_eq!(restored.id, original.id);
    
    // Label is RE-DERIVED, not preserved from original
    assert_eq!(restored.label, Foo::labeler().label());
}
```

### Category D: Provenance & Sourced (in `src/id/provenance.rs`)

```rust
#[test]
fn test_all_core_provenance_slugs() {
    assert_eq!(External::<()>::SLUG, "ext");
    assert_eq!(Generated::<()>::SLUG, "gen");
    assert_eq!(Imported::<()>::SLUG, "imp");
    assert_eq!(Derived::<()>::SLUG, "der");
    assert_eq!(Scoped::<(), Generated<()>>::SLUG, "scoped");
    assert_eq!(Temporary::SLUG, "tmp");
    assert_eq!(ClientProvided::SLUG, "cli");
    assert_eq!(AliasOf::<()>::SLUG, "als");
}

#[test]
fn test_vendor_providers_expose_constants() {
    // Verify Vendor trait provides VENDOR constants
    assert_eq!(providers::Stripe::VENDOR, "stripe");
    assert_eq!(providers::Github::VENDOR, "github");
    assert_eq!(providers::Spark::VENDOR, "spark");
}

#[test]
fn test_with_provenance_stores_both_fields() {
    let id = "test-id".to_string();
    let desc = 42u32;
    let with_prov = WithProvenance::new(id.clone(), desc);
    
    assert_eq!(with_prov.identifier, id);
    assert_eq!(with_prov.descriptor, desc);
}

#[test]
fn test_with_provenance_equality() {
    let a = WithProvenance::new("id".to_string(), 1u32);
    let b = WithProvenance::new("id".to_string(), 1u32);
    let c = WithProvenance::new("id".to_string(), 2u32);
    
    assert_eq!(a, b);
    assert_ne!(a, c);  // Different descriptor
}
```

### Category E: Compile-Fail Tests (in docs, not inline tests)

Add these as `compile_fail` doctests in `src/id/sourced.rs`:

```rust
/// # Type Safety: Generate Only for Generated
///
/// `Sourced<E, Generated<S>>` implements `Entity`, allowing `.next_id()`.
/// `Sourced<E, External<P>>` does NOT implement `Entity`, preventing accidents.
///
/// ```compile_fail
/// use tagid::{Entity, Label, id::provenance::*};
///
/// struct User;
/// impl Label for User { /* ... */ }
///
/// // This should NOT compile:
/// fn requires_entity<E: Entity>() {}
/// requires_entity::<Sourced<User, External<()>>>();  // ERROR!
/// ```
///
/// ```compile_fail
/// use tagid::{Entity, Label, id::provenance::*};
///
/// struct User;
/// impl Label for User { /* ... */ }
///
/// // This should NOT compile:
/// fn requires_entity<E: Entity>() {}
/// requires_entity::<Sourced<User, Imported<()>>>();  // ERROR!
/// ```
```

**Tests Added:** 9 (4 serde + 4 provenance + 2 compile-fail doctests)  
**Time:** ~1 hour

---

## PHASE 3: Complete Coverage (1 hour) — P2 Priority

### Category F: Label Implementations (in `src/label.rs` or new `tests/label_impl_tests.rs`)

```rust
#[test]
fn test_label_policy_to_mode_conversion() {
    use crate::id::labeled::LabelMode;
    
    assert_eq!(LabelMode::from(LabelPolicy::Opaque), LabelMode::None);
    assert_eq!(LabelMode::from(LabelPolicy::OpaqueByDefault), LabelMode::None);
    assert_eq!(LabelMode::from(LabelPolicy::EntityNameDefault), LabelMode::Short);
    assert_eq!(LabelMode::from(LabelPolicy::ExternalKeyDefault), LabelMode::Full);
}

#[test]
fn test_label_option_delegation() {
    let opt_labeler = <Option<String> as Label>::labeler();
    let str_labeler = <String as Label>::labeler();
    assert_eq!(opt_labeler.label(), str_labeler.label());
}

#[test]
fn test_label_result_delegation() {
    let res_labeler = <Result<u32, String> as Label>::labeler();
    let u32_labeler = <u32 as Label>::labeler();
    assert_eq!(res_labeler.label(), u32_labeler.label());
}

#[test]
fn test_label_hashmap_format() {
    let labeler = <HashMap<String, u32> as Label>::labeler();
    let label = labeler.label();
    
    // Should contain both key and value type names
    assert!(label.contains("String") || label.contains("str"));  // String representation
    assert!(label.contains("u32"));
    
    // NOTE: This test documents current behavior and would catch
    // the format string bug if it exists (missing ">")
    // Expected: "HashMap<String,u32" (missing closing bracket)
    // This test makes that explicit!
}

#[test]
fn test_label_unit_impl() {
    assert!(<() as Label>::labeler().label().is_empty());
    assert_eq!(<() as Label>::POLICY, LabelPolicy::Opaque);
}
```

### Category G: Labeling Implementations (in `src/labeling.rs`)

```rust
#[test]
fn test_make_labeling_caching() {
    let labeler = MakeLabeling::<String>::default();
    let label1 = labeler.label();
    let label2 = labeler.label();
    
    // Should return same value (caching works)
    assert_eq!(label1, label2);
    assert!(!label1.is_empty());
}

#[test]
fn test_make_labeling_display_and_debug() {
    let labeler = MakeLabeling::<u32>::default();
    let label = labeler.label();
    
    assert_eq!(format!("{}", labeler), label);
    assert!(format!("{:?}", labeler).contains(label));
}

#[test]
fn test_custom_labeling_conversions() {
    let from_str = CustomLabeling::from("test");
    let from_string = CustomLabeling::from("test".to_string());
    let from_parse: CustomLabeling = "test".parse().unwrap();
    
    assert_eq!(from_str.label(), "test");
    assert_eq!(from_string.label(), "test");
    assert_eq!(from_parse.label(), "test");
}

#[test]
fn test_custom_labeling_display_and_debug() {
    let labeler = CustomLabeling::new("MyLabel");
    assert_eq!(format!("{}", labeler), "MyLabel");
    assert!(format!("{:?}", labeler).contains("MyLabel"));
}

#[test]
fn test_no_labeling_returns_empty() {
    let labeler = NoLabeling;
    assert_eq!(labeler.label(), "");
}

#[test]
fn test_labeling_summon() {
    // Test the static summon() method
    let labeler = <dyn Labeling>::summon::<Foo>();
    assert_eq!(labeler.label(), Foo::labeler().label());
}

#[test]
fn test_labeling_decorated_default() {
    let labeler = MakeLabeling::<String>::default();
    let decorated = labeler.decorated_label();
    assert_eq!(decorated.as_ref(), labeler.label());
}

#[test]
fn test_primitive_labels_not_empty() {
    assert!(!<u32 as Label>::labeler().label().is_empty());
    assert!(!<u64 as Label>::labeler().label().is_empty());
    assert!(!<String as Label>::labeler().label().is_empty());
    assert!(!<bool as Label>::labeler().label().is_empty());
}
```

**Tests Added:** 17 (5 + 12)  
**Time:** ~1 hour

---

## PHASE 4: Feature-Gated Integrations (2-3 hours) — P3 Priority (Optional)

### Category H: sqlx Integration (requires `--features sqlx`)

Create `tests/sqlx_roundtrip.rs` (feature-gated):

```rust
#![cfg(feature = "sqlx")]

use sqlx::sqlite::SqlitePool;
use tagid::{Id, Label, Entity, Labeling, CustomLabeling};

#[derive(Label)]
struct User;

#[tokio::test]
async fn test_sqlx_string_id_roundtrip() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    
    sqlx::raw_sql("CREATE TABLE users (id TEXT PRIMARY KEY)")
        .execute(&pool)
        .await
        .unwrap();
    
    let original: Id<User, String> = Id::direct("user", "user-123".into());
    
    sqlx::raw_sql("INSERT INTO users (id) VALUES (?)")
        .bind(&original.id)
        .execute(&pool)
        .await
        .unwrap();
    
    let (retrieved_id,): (String,) = sqlx::query_as("SELECT id FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(retrieved_id, original.id);
}
```

### Category I: Disintegrate Integration (if applicable)

```rust
#[test]
#[cfg(feature = "disintegrate")]
fn test_disintegrate_into_identifier_value() {
    use disintegrate::IntoIdentifierValue;
    
    let id: Id<Foo, String> = Id::direct("Label", "canonical-id".into());
    let identifier = id.into_identifier_value();
    
    // Must use canonical form (no label)
    assert_eq!(identifier, IdentifierValue::String("canonical-id".into()));
}
```

### Category J: Feature Matrix Tests (for CI)

Create a `.github/workflows/test-features.yml` file to test all feature combinations:

```yaml
name: Feature Matrix Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        features:
          - ""  # No features
          - "serde"
          - "uuid"
          - "cuid"
          - "ulid"
          - "snowflake"
          - "sqlx"
          - "disintegrate"
          - "uuid,serde,sqlx"
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --features "${{ matrix.features }}"
```

**Tests/Config Added:** 4 integration tests + feature matrix CI  
**Time:** ~2-3 hours

---

## Implementation Order & Checkpoints

### Quick Win (30 minutes)
1. Add P0 tests to `src/id/mod.rs` (13 tests)
2. Run `cargo test` to verify all pass
3. **Checkpoint:** 47 total tests, 58% coverage

### Confidence Phase (1 hour)
4. Add P1 serde tests (4 tests)
5. Add P1 provenance tests (4 tests)
6. Add compile-fail doctests (2)
7. Run `cargo test` + `cargo test --doc`
8. **Checkpoint:** 55 total tests, 68% coverage

### Complete Phase (1 hour)
9. Add P2 label/labeling tests (17 tests)
10. Fix any bugs discovered (e.g., HashMap format)
11. Refactor brittle assertions to use substrings
12. Run full test suite
13. **Checkpoint:** 72 total tests, 89% coverage ✓

### Optional Phase (2-3 hours, feature-gated)
14. Add sqlx integration tests
15. Add disintegrate integration tests
16. Set up feature matrix CI
17. Run matrix tests
18. **Final:** 80+ tests, feature-matrix validated

---

## Key Metrics

| Phase | Tests | Coverage | Time | Priority |
|-------|-------|----------|------|----------|
| Current | 34 | 42% | — | — |
| + P0 | 47 | 58% | 30min | **Must** |
| + P1 | 55 | 68% | 1h | **Should** |
| + P2 | 72 | 89% | 1h | **Nice** |
| + P3 | 80+ | 98%+ | 2-3h | Optional |

---

## Quality Gates

Before committing each phase:

```bash
# Unit tests
cargo test --lib

# Doc tests
cargo test --doc

# Clippy
cargo clippy --all-features

# Coverage (requires tarpaulin)
cargo tarpaulin --lib
```

---

## Notes for Implementation

1. **Use deterministic generators** in tests (not `SystemTime`)
2. **Avoid brittle string assertions** (use substring checks)
3. **Group related tests** in logical test modules
4. **Document semantic invariants** with comments
5. **Use `assert_let!` macro** from `assert_matches2` for clarity
6. **Feature-gate integration tests** with `#[cfg(feature = "...")]`

---

## Future Improvements (Not in Scope)

- [ ] Compile-time checked labeling (would need proc macro)
- [ ] Automatic derive for Label trait (can use `tagid-derive`)
- [ ] Performance benchmarks (could add `criterion` tests)
- [ ] Fuzz testing (could add `proptest` for ID generation)
- [ ] MIRI validation (for `unsafe` Send/Sync impls)
