# Tagid Test Coverage & Quality Analysis
**Date:** 2026-02-04  
**Current Test Count:** 34 unit tests  
**Coverage Assessment:** Narrow (formatting + serde only); missing core semantic invariants  

---

## Executive Summary

The current test suite validates formatting and serde serialization but **leaves critical gaps in core semantic invariants** that are central to tagid's value proposition:
- ID equality/hash independence from labels ❌
- Labeling policy behavior and mode overrides ❌
- Serde deserialization error handling ❌
- Type-level provenance safety guarantees ❌
- Feature-gated integrations (`sqlx`, `disintegrate`) ❌

**Recommendation:** Add ~15-20 targeted unit tests across 5 categories (see below). Effort: 1-3 hours for major confidence uplift.

---

## Current Coverage Assessment

### ✅ What's Well-Tested
- **Display/Debug formatting** with labels (3 tests covering None/Empty/Multiple modes)
- **Serde tokens & JSON roundtrips** for String, u64, and ULID types
- **Provenance trait implementation** across all 8 core types
- **Slug/vendor display formatting** for External providers
- **Basic labeling delegation** in Sourced wrapper
- **Zero-sized guarantee** for Sourced (PhantomData-only)

### ❌ Critical Gaps

#### 1. **Core `Id<T, ID>` Semantic Invariants (Untested)**

**Problem:** `Id`'s most important properties are unvalidated:

| Property | Current State | Risk |
|----------|---------------|------|
| `Eq`/`PartialEq` (ID-only) | Only tested indirectly | Could silently include label in equality |
| `Hash` (must match Eq) | Untested | Hash-map corruption if label leaks into hash |
| `Ord`/`PartialOrd` (ID-only) | Untested | Sorting could accidentally use labels |
| `Clone` | Untested | Could fail on large ID types |
| `From<ID>` label source | Untested | Could use wrong labeler |
| `Default` (for Entity IDs) | Untested | Could differ from `new()` |
| `into_inner()` | Untested | Could transform/corrupt the ID |
| `as_str()` for `AsRef<str>` IDs | Untested | Could include label in string form |

**Test Opportunities:**
```rust
#[test]
fn test_equality_ignores_label() {
    let id1: Id<Foo, String> = Id::direct("label1", "123".into());
    let id2: Id<Foo, String> = Id::direct("label2", "123".into());
    assert_eq!(id1, id2);  // Same ID, different labels
}

#[test]
fn test_hash_matches_equality() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let id1 = Id::direct("a", "123".to_string());
    let id2 = Id::direct("b", "123".to_string());
    
    let mut h1 = DefaultHasher::new();
    id1.hash(&mut h1);
    let mut h2 = DefaultHasher::new();
    id2.hash(&mut h2);
    
    assert_eq!(h1.finish(), h2.finish());  // Same hash
}

#[test]
fn test_ordering_ignores_label() {
    let a = Id::direct("z", 1u64);
    let b = Id::direct("a", 2u64);
    assert!(a < b);  // Ordered by ID, not label
}

#[test]
fn test_from_id_uses_entity_labeler() {
    let id: Id<Foo, String> = "abc".to_string().into();
    assert_eq!(id.label, "MyFooferNut");  // Should come from Foo::labeler()
}

#[test]
fn test_as_str_returns_id_only() {
    let id: Id<Foo, String> = Id::direct("IGNORED", "value".to_string());
    assert_eq!(id.as_str(), "value");  // No label in string form
}
```

---

#### 2. **Labeled<T, ID> Behavior (Mostly Indirect)**

**Problem:** The `Labeled` wrapper is only tested via provenance output formatting; direct mode testing is missing.

| Aspect | Current State | Gap |
|--------|---------------|-----|
| `LabelMode::None` display | Tested via provenance | Only for specific types, not exhaustive |
| `LabelMode::Short` display | Tested via provenance | Doesn't test empty label case |
| `LabelMode::Full` display | Tested via provenance | Doesn't test `decorated_label()` edge cases |
| `LabelMode` for Debug output | Untested | |
| `Labeled::mode()` builder | Untested | |
| `LabelPolicy` → default `LabelMode` | Tested indirectly | No direct unit test |
| Empty label behavior | Partially tested | Missing Short/Full empty-label cases |

**Critical Semantic:** `Labeled::Full` uses `T::labeler().decorated_label()` (type-driven), NOT `id.label`. This is subtle and easy to break.

**Test Opportunities:**
```rust
#[test]
fn test_labeled_display_modes() {
    let id: Id<Foo, String> = Id::direct("MyFooferNut", "123".into());
    
    assert_eq!(id.labeled().mode(LabelMode::None).to_string(), "123");
    assert_eq!(id.labeled().mode(LabelMode::Short).to_string(), "MyFooferNut::123");
    // Full uses decorated_label(), which includes provenance
}

#[test]
fn test_labeled_full_ignores_stored_label() {
    let id: Id<Foo, String> = Id::direct("WRONG", "123".into());
    
    // Short uses the stored label
    assert_eq!(id.labeled().mode(LabelMode::Short).to_string(), "WRONG::123");
    
    // Full uses type-derived labeler (Foo::labeler()), not stored label
    let full = id.labeled().mode(LabelMode::Full).to_string();
    assert!(full.contains("MyFooferNut"));  // Type-derived
    assert!(!full.contains("WRONG"));       // Stored label ignored
}

#[test]
fn test_labeled_empty_label_behavior() {
    let id: Id<NoLabelZed, String> = Id::direct("", "123".into());
    
    // Short with empty label should fall back to just value
    assert_eq!(id.labeled().mode(LabelMode::Short).to_string(), "123");
    
    // Full with empty decorated_label should also fall back
    assert_eq!(id.labeled().mode(LabelMode::Full).to_string(), "123");
}

#[test]
fn test_labeled_debug_modes() {
    let id: Id<Foo, u64> = Id::direct("Label", 42u64);
    
    assert_eq!(id.labeled().mode(LabelMode::None).to_string(), "42");
    assert_eq!(id.labeled().mode(LabelMode::Short).to_string(), "Label::42");
}

#[test]
fn test_label_policy_maps_to_default_mode() {
    use crate::id::labeled::LabelMode;
    
    // Opaque/OpaqueByDefault => None
    assert_eq!(LabelMode::from(LabelPolicy::Opaque), LabelMode::None);
    assert_eq!(LabelMode::from(LabelPolicy::OpaqueByDefault), LabelMode::None);
    
    // EntityNameDefault => Short
    assert_eq!(LabelMode::from(LabelPolicy::EntityNameDefault), LabelMode::Short);
    
    // ExternalKeyDefault => Full
    assert_eq!(LabelMode::from(LabelPolicy::ExternalKeyDefault), LabelMode::Full);
}
```

---

#### 3. **Serde: Only Happy Paths Tested**

**Problem:** No negative tests; doesn't validate that labeled formats are rejected.

| Scenario | Current | Gap |
|----------|---------|-----|
| Valid string deserialization | ✓ | |
| Valid number deserialization | ✓ | |
| Invalid type (string where u64 expected) | ✗ | Could silently accept |
| Invalid JSON structure | ✗ | |
| **Labeled format rejection** | ✗ | **Critical:** Must reject `"Foo::abc"` for `Id<Foo, String>` |
| Label is not preserved | Partially | Need to verify label comes from `T::labeler()`, not from JSON |
| Canonical wire format guarantee | Partially | |

**Test Opportunities:**
```rust
#[test]
fn test_serde_rejects_invalid_types() {
    // For Id<Foo, u64>, JSON string should fail
    let result = serde_json::from_str::<Id<Foo, u64>>("\"not-a-number\"");
    assert!(result.is_err());
    
    // For Id<Foo, String>, JSON number might coerce or fail depending on serde
    // This documents the actual behavior
    let result = serde_json::from_str::<Id<Foo, String>>("42");
    // Assert expected behavior (coerce or fail)
}

#[test]
fn test_serde_rejects_labeled_format() {
    // Critical: "Foo::abc" must NOT deserialize (canonical-only guarantee)
    let result = serde_json::from_str::<Id<Foo, String>>("\"Foo::abc\"");
    // If this succeeds, it means labeled format leaked into serialization (bug!)
    // Expected: succeeds, but parsed value is "Foo::abc" (literal, not split)
    let id: Id<Foo, String> = result.unwrap();
    assert_eq!(id.id, "Foo::abc");  // Literal string, not parsed as Foo/abc
    assert_eq!(id.label, "MyFooferNut");  // Label from Foo::labeler(), not from JSON
}

#[test]
fn test_serde_label_comes_from_entity_labeler() {
    let json = "\"abc\"";
    let id: Id<Foo, String> = serde_json::from_str(json).unwrap();
    
    // Label must come from Foo::labeler(), not from JSON
    assert_eq!(id.label, "MyFooferNut");
    assert_eq!(id.id, "abc");
}

#[test]
fn test_serde_roundtrip_preserves_only_id() {
    let orig = Id::<Foo, String>::direct("ANY_LABEL", "abc".into());
    let json = serde_json::to_string(&orig).unwrap();
    let restored: Id<Foo, String> = serde_json::from_str(&json).unwrap();
    
    // ID must match
    assert_eq!(restored.id, orig.id);
    
    // Label must NOT match orig.label; it's re-derived from Foo::labeler()
    assert_eq!(restored.label, Foo::labeler().label());
}

#[test]
fn test_serde_json_structure() {
    let id: Id<Foo, String> = Id::direct("_", "value".into());
    let json = serde_json::to_string(&id).unwrap();
    
    // Must be a plain string, not an object
    assert_eq!(json, "\"value\"");
}
```

---

#### 4. **Provenance & Sourced: Untested Contracts**

**Problem:** Type-level safety guarantees (e.g., "generate only for Generated") are asserted indirectly; not protected by compile-time checks in tests.

| Contract | Current | Gap |
|----------|---------|-----|
| `External` has no generator | Only implicit | Could accidentally implement `Entity` |
| `Generated` has generator | Asserted | |
| `Sourced<E, Generated<_>>` impl | Asserted | |
| `Sourced<E, External<_>>` no Entity | ✗ | Should **fail to compile** if violated |
| `WithProvenance` exists/works | ✗ | Untested |
| All SLUG constants correct | Partially | Only checked for display name |
| All VENDOR constants correct | Partially | Only checked for External<Provider> |
| Scoped inner SLUG/VENDOR | ✗ | Untested |

**Test Opportunities:**
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
fn test_vendor_providers_have_vendor_constants() {
    // These should be available as Vendor trait
    assert_eq!(providers::Stripe::VENDOR, "stripe");
    assert_eq!(providers::Github::VENDOR, "github");
    assert_eq!(providers::Spark::VENDOR, "spark");
    assert_eq!(providers::Okta::VENDOR, "okta");
    // ... more providers
}

#[test]
fn test_external_vendors_appear_in_slug_format() {
    // External<Stripe> should show "ext/stripe" in slug display
    assert_eq!(
        provenance_display_name::<External<providers::Stripe>>(),
        "ext/stripe"
    );
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
    let a = WithProvenance::new("id1".to_string(), 1u32);
    let b = WithProvenance::new("id1".to_string(), 1u32);
    let c = WithProvenance::new("id1".to_string(), 2u32);  // Different descriptor
    
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn test_scoped_provenance_properties() {
    // Scoped is a wrapper around an inner provenance
    let inner = Generated::<()>::SLUG;
    let scoped_slug = Scoped::<(), Generated<()>>::SLUG;
    
    // Scoped should have its own identity
    assert_eq!(scoped_slug, "scoped");
    assert_ne!(scoped_slug, inner);
}

// Compile-fail doctest (in provenance.rs or sourced.rs docs):
/// This should NOT compile:
/// ```compile_fail
/// use tagid::{Entity, Label, id::provenance::*};
/// 
/// struct User;
/// impl Label for User { /* ... */ }
/// 
/// // ERROR: Sourced<User, External<_>> does not implement Entity
/// fn requires_entity<E: Entity>() {}
/// requires_entity::<Sourced<User, External<()>>>();
/// ```
```

---

#### 5. **Labeling & Label Trait (Almost No Coverage)**

**Problem:** Only 3 core labeling implementations (`MakeLabeling`, `CustomLabeling`, `NoLabeling`) and their edge cases are untested. Also likely bug in `HashMap` impl.

| Impl | Current | Gap |
|------|---------|-----|
| `MakeLabeling` caching | ✗ | Untested; should be stable + non-empty |
| `MakeLabeling` Debug/Display | ✗ | |
| `CustomLabeling` conversions | ✗ | `From<&str>`, `From<String>`, `FromStr` |
| `CustomLabeling` Debug/Display | ✗ | |
| `NoLabeling` | ✗ | Should return `""` |
| `Label for Option<T>` | ✗ | Should delegate to T |
| `Label for Result<T, E>` | ✗ | Should delegate to T |
| `Label for HashMap<K, V>` | ✗ | **Likely bug:** format string is incomplete |
| Primitive label impls | ✗ | Macro-generated; should verify a few |

**Test Opportunities:**
```rust
#[test]
fn test_make_labeling_caching() {
    let labeler1 = MakeLabeling::<String>::default();
    let label1 = labeler1.label();
    let label2 = labeler1.label();
    
    // Should be stable (same pointer or content)
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
fn test_label_option_delegation() {
    // Option<String> should use String's labeler
    let opt_labeler = <Option<String> as Label>::labeler();
    let str_labeler = <String as Label>::labeler();
    
    assert_eq!(opt_labeler.label(), str_labeler.label());
}

#[test]
fn test_label_result_delegation() {
    // Result<u32, E> should use u32's labeler
    let res_labeler = <Result<u32, String> as Label>::labeler();
    let u32_labeler = <u32 as Label>::labeler();
    
    assert_eq!(res_labeler.label(), u32_labeler.label());
}

#[test]
fn test_label_hashmap_format() {
    let labeler = <HashMap<String, u32> as Label>::labeler();
    let label = labeler.label();
    
    // Should contain both key and value labels
    assert!(label.contains("String"));
    assert!(label.contains("u32"));
    
    // Note: Current impl appears to be "HashMap<K,V" (missing ">")
    // This test documents/catches that bug
}

#[test]
fn test_primitive_labels_not_empty() {
    assert!(!<u32 as Label>::labeler().label().is_empty());
    assert!(!<String as Label>::labeler().label().is_empty());
    assert!(!<bool as Label>::labeler().label().is_empty());
}
```

---

## Summary Table: Test Gaps

| Category | Untested | Priority | Effort | Tests to Add |
|----------|----------|----------|--------|--------------|
| **Core `Id<T, ID>` semantics** | Eq/Hash/Ord, From, Default, as_str, clone, into_inner | **P0** | 30min | 7 |
| **`Labeled` modes & edge cases** | All LabelMode variants, empty labels, policy→mode mapping | **P0** | 30min | 6 |
| **Serde error handling** | Type mismatches, labeled format rejection | **P1** | 20min | 5 |
| **Provenance/Sourced safety** | Compile-fail contracts, all SLUG/VENDOR, WithProvenance | **P1** | 45min | 8 |
| **Labeling implementations** | All 3 impls + Option/Result/HashMap + primitives | **P2** | 45min | 9 |
| **Feature integrations** | sqlx, disintegrate, generators | **P2** | 2-3h | 5-10 (integration) |

**Total recommended additions:** 20-25 unit tests + 2-3 compile-fail doctests + 5-10 integration tests (feature-gated).

---

## Quality Improvements (Non-Feature-Driven)

### 1. **Reduce Brittle Formatting Assertions**
Current approach:
```rust
assert_eq!(
    format!("{a:#?}"),
    format!("Id {{\n    label: \"{}\",\n    id: {},\n}}", a.label, a.id,)
);
```
Problems: Fragile across Rust versions, whitespace changes, derive changes.

Improved approach:
```rust
let debug_str = format!("{a:#?}");
assert!(debug_str.contains("Id {"));
assert!(debug_str.contains(&a.label));
assert!(debug_str.contains(&format!("{}", a.id)));
```

### 2. **Avoid Time-Based Generators in Unit Tests**
Current `TestGenerator`:
```rust
fn next_id_rep() -> String {
    std::time::SystemTime::UNIX_EPOCH
        .elapsed()
        .unwrap()
        .as_millis()
        .to_string()
}
```
Risk: Collisions in fast-running tests or low-resolution timers.

Better: Use a deterministic counter or fixed string.

### 3. **Document Semantic Decisions with Tests**
Current tests mostly validate formatting. Add **explicit semantic invariant tests** with comments explaining *why*:
```rust
#[test]
fn test_equality_ignores_label_semantic_invariant() {
    // SEMANTIC INVARIANT: Equality is based ONLY on the ID value.
    // The label is presentation metadata and must not affect equality.
    // This allows relabeling the same ID without breaking collections (HashMap, BTreeSet).
    // ...
}
```

---

## Rollout Plan

### Phase 1: Immediate (1-2h) — High-Confidence Wins
1. Add 7 tests for core `Id<T, ID>` semantics (Eq, Hash, Ord, From, Default, as_str)
2. Add 6 tests for `Labeled` modes and edge cases
3. Add 5 serde error/rejection tests
4. **Result:** 18 new tests, ~100 lines; closes 70% of critical gaps

### Phase 2: Short-Term (1-2h) — Type Safety Documentation
5. Add 8 provenance/sourced tests (SLUG, VENDOR, WithProvenance)
6. Add compile-fail doctests for "generate only for Generated" invariant
7. **Result:** 8 new tests + 2 doctests; fully documents type-level safety

### Phase 3: Medium-Term (45min-1h) — Complete Coverage
8. Add 9 labeling implementation tests
9. Refactor existing formatting assertions to use substring checks (reduce brittleness)
10. **Result:** Full unit test coverage

### Phase 4: Long-Term (2-3h, feature-gated) — Integration Assurance
11. Add `sqlx` roundtrip tests (if crate supports `sqlx`)
12. Add feature matrix CI/tests to catch feature-specific regressions
13. **Result:** High confidence across feature combinations

---

## Appendix: Key Design Invariants to Protect

These are the core semantic guarantees tagid makes and why they need tests:

1. **Canonical IDs are label-free** → serde rejection tests
2. **Equality/hash ignore labels** → Eq/Hash tests
3. **Labeling is opt-in via `.labeled()`** → Labeled mode tests
4. **Type-level provenance safety** → compile-fail doctests
5. **Zero-sized Sourced** → size_of tests (already have)
6. **Labeling policy → default mode** → LabelPolicy conversion tests

All these are currently either untested or indirectly validated. Codifying them in tests prevents future regressions and makes design intent explicit for future maintainers.

---

## References

- **Rust testing best practices:** https://doc.rust-lang.org/book/ch11-00-testing.html
- **serde testing:** https://docs.rs/serde_test/1.0/serde_test/
- **Hash contract:** https://doc.rust-lang.org/std/hash/trait.Hash.html#hash-and-eq
