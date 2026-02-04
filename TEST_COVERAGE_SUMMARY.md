# Tagid Test Coverage Review Summary
**Date:** 2026-02-04  
**Status:** Complete Analysis with Implementation Roadmap

---

## Quick Overview

| Metric | Value |
|--------|-------|
| Current Tests | 34 (all passing) |
| Current Coverage | 42% |
| Gap Analysis | 5 categories × 15-20 tests |
| Recommended Total | 55-72 tests (80-90% coverage) |
| Effort (full unit) | 3-4 hours |
| Highest Priority | 2 hours for P0+P1 fixes |

---

## 5 Critical Gaps Identified

### 🔴 P0: Core Semantic Invariants (1 hour, 13 tests)
**Risk Level: HIGH** — Affects correctness of collections and equality

1. **Id<T, ID> Equality/Hash/Ordering** must ignore labels
   - Current: Untested
   - Impact: Silent label leakage in HashMap/BTreeSet corruption
   - Tests needed: 7 (test_equality_ignores_label, test_hash_matches_equality, etc.)

2. **Labeled<T, ID> Display/Debug Modes** (None/Short/Full)
   - Current: Only indirect testing via provenance
   - Impact: Edge cases (empty labels, full mode using type-derived labels)
   - Tests needed: 6 (test_labeled_*_mode, test_labeled_full_ignores_stored_label)
   - **Critical insight:** `Full` mode uses `T::labeler()`, not `id.label` — this is subtle!

### 🟠 P1: Type Safety & Canonical Guarantee (1.5 hours, 8 tests + 2 compile-fail)
**Risk Level: MEDIUM** — Affects data integrity and type contracts

3. **Serde Error Handling** (type mismatches, labeled format rejection)
   - Current: Only happy paths tested
   - Impact: Could accept invalid types or labeled formats (breaking canonical guarantee)
   - Tests needed: 4 (test_serde_rejects_invalid_types, test_serde_rejects_labeled_format)

4. **Provenance/Sourced Type Safety** (only `Generated` can call `next_id()`)
   - Current: Compile-time contract not documented or protected
   - Impact: `External<_>/Imported<_>` could accidentally become `Entity`
   - Tests needed: 4 + 2 compile-fail doctests
   - Examples: test_all_core_provenance_slugs, compile-fail doctests

### 🟡 P2: Complete Implementation Coverage (1 hour, 17 tests)
**Risk Level: LOW** — Mostly edge cases and completeness

5. **Labeling Implementations** (MakeLabeling, CustomLabeling, Label trait)
   - Current: Zero test coverage
   - Impact: Likely bug in HashMap<K,V> label format (missing ">")
   - Tests needed: 13 (test_make_labeling_*, test_custom_labeling_*, test_label_*_impl)

---

## Quick Start Guide

### Path 1: Minimal (30 min) — P0 Only
**Goal:** Protect core semantic invariants

```bash
# Add 13 tests to src/id/mod.rs
# Run: cargo test --lib
# Expected: 47 passing tests (58% coverage)
```

Documents in: `history/test-implementation-roadmap.md` → "PHASE 1"

### Path 2: Solid (1.5 hours) — P0 + P1
**Goal:** Protect semantics + type safety

```bash
# Add 13 + 8 tests to src/id/mod.rs + src/id/provenance.rs
# Add 2 compile-fail doctests
# Run: cargo test --lib && cargo test --doc
# Expected: 55 passing tests (68% coverage)
```

Documents in: `history/test-implementation-roadmap.md` → "PHASE 1 + 2"

### Path 3: Complete (3 hours) — P0 + P1 + P2
**Goal:** Full unit test coverage

```bash
# Add all 38 tests across id/mod.rs, provenance.rs, labeling.rs, label.rs
# Refactor brittle formatting assertions
# Run: cargo test --lib && cargo clippy --all-features
# Expected: 72 passing tests (89% coverage)
```

Documents in: `history/test-implementation-roadmap.md` → "PHASE 1 + 2 + 3"

---

## Key Test Opportunities

### Highest-Value Tests
1. `test_equality_ignores_label()` — Prevents collection corruption
2. `test_labeled_full_ignores_stored_label()` — Documents subtle semantic
3. `test_serde_rejects_labeled_format()` — Enforces canonical-only guarantee
4. `test_label_hashmap_format()` — Catches likely bug
5. Compile-fail doctests — Protects type safety at compile time

### Most Surprising Findings
- `Labeled::Full` uses `T::labeler().decorated_label()`, NOT `id.label` — unwritten contract
- HashMap<K,V> label format likely incomplete (missing ">")
- No tests ensure `External<_>` can't call `next_id()` (type-level safety)
- Serde accepts literally any string as ID value (includes "Foo::abc" if that's the value)

---

## Documentation Artifacts Created

1. **test-coverage-analysis-20260204.md** (20KB, 553 lines)
   - Comprehensive gap analysis with risk assessment
   - Coverage matrix by module
   - Rationale and trade-offs
   - 6-7 example tests per gap

2. **test-implementation-roadmap.md** (17KB, 594 lines)
   - Step-by-step implementation with actual test code
   - 4 phases: P0 (30min), P1 (1h), P2 (1h), P3 (2-3h, optional)
   - Quality gates and verification commands
   - Feature matrix CI template

3. **This summary** (quick reference)

All saved in `history/` for easy discovery and historical tracking.

---

## Coverage by Component

```
src/id/mod.rs           10/20 tests (50%)  → +10 tests needed (P0)
src/id/labeled.rs        0/8 tests  (0%)   → +6 tests needed (P0)
src/id/provenance.rs    10/15 tests (67%)  → +4 tests needed (P1)
src/id/sourced.rs        5/7 tests  (71%)  → +2 compile-fail (P1)
src/label.rs             0/10 tests (0%)   → +10 tests needed (P2)
src/labeling.rs          0/8 tests  (0%)   → +8 tests needed (P2)
Serde integration        3/8 tests  (38%)  → +4 tests needed (P1)
───────────────────────────────────────────
TOTAL                   34/81 tests (42%)  → +48 tests for 89% coverage
```

---

## High-Impact Risk Mitigation

### Risk 1: Label Leaks Into Equality/Hash
**Current:** Untested
**Mitigation:** `test_equality_ignores_label()` + `test_hash_matches_equality()`
**Impact:** Prevents silent HashMap/BTreeSet corruption

### Risk 2: Labeled Format Deserializes
**Current:** Untested
**Mitigation:** `test_serde_rejects_labeled_format()`
**Impact:** Enforces canonical-only wire format guarantee

### Risk 3: Type-Level Safety Unchecked
**Current:** Indirect assertion only
**Mitigation:** Compile-fail doctests for `Sourced<E, External<_>>::next_id()`
**Impact:** Protects accidental External/Imported generation at compile time

### Risk 4: HashMap Label Format Bug
**Current:** Untested (likely missing ">")
**Mitigation:** `test_label_hashmap_format()`
**Impact:** Catches runtime panic or silent corruption

---

## Next Steps

1. **Read documentation** (15 min)
   - `history/test-coverage-analysis-20260204.md` — Understand gaps
   - `history/test-implementation-roadmap.md` — See actual test code

2. **Choose path** (5 min)
   - Path 1 (30min): P0 only
   - Path 2 (1.5h): P0 + P1 (recommended)
   - Path 3 (3h): Full coverage

3. **Implement tests** (30min-3h depending on path)
   - Copy test templates from roadmap
   - Run `cargo test --lib` after each phase

4. **Verify** (5 min)
   - `cargo test --lib`
   - `cargo clippy --all-features`
   - `cargo test --doc` (if adding doctests)

---

## Success Metrics

### Minimum (After P0+P1, 1.5h)
- ✓ 55 tests (68% coverage)
- ✓ Core semantic invariants protected
- ✓ Type safety documented
- ✓ All tests passing + clippy clean

### Complete (After P0+P1+P2, 3h)
- ✓ 72 tests (89% coverage)
- ✓ Every public API exercised
- ✓ Edge cases and error paths validated
- ✓ Likely bugs caught (e.g., HashMap format)

### Advanced (With P3, 5-6h total)
- ✓ 80+ tests (98%+ coverage)
- ✓ Feature matrix validated
- ✓ sqlx/disintegrate integration proven
- ✓ Ready for public crate publication

---

## Key Takeaway

The current test suite validates **formatting and happy paths** but misses **semantic invariants** that are central to tagid's value proposition. Adding 15-20 targeted tests (2-3 hours) would close 70% of critical gaps and document core design contracts explicitly.

**Recommended starting point:** Phase 1 (P0, 30 min) for immediate confidence gain.
