# Epic Summary: Provenance-Aware Construction API (tid-6n4)

**Status**: Planned & Documented  
**Date**: 2026-02-04  
**Priority**: High (1)  
**Epic ID**: tid-6n4  

---

## Quick Links

### Documentation
- **Analysis & Design**: [tid-6n4-provenance-construction-analysis.md](tid-6n4-provenance-construction-analysis.md)
- **Implementation Plan**: [tid-6n4-implementation-plan.md](tid-6n4-implementation-plan.md)
- **Test Plan**: [tid-6n4-test-plan.md](tid-6n4-test-plan.md)
- **Lessons/Guide**: [ref/lessons/provenance-construction-patterns.md](../ref/lessons/provenance-construction-patterns.md)

### Beads Tasks
- **Epic**: tid-6n4
- **Phase 1 (Impl)**: tid-6n4.1
- **Phase 2 (Docs)**: tid-6n4.2
- **Phase 3 (Tests)**: tid-6n4.3
- **Phase 4 (Turtle)**: tid-6n4.4

---

## Executive Summary

### The Problem

Current `tagid` uses a single construction pattern—`for_labeled()`—for **all ID types** regardless of provenance. This is **semantically misleading** because:

1. `for_labeled()` is a **Label operation**, not a provenance operation
2. The argument is the **canonical ID value**, not a "labeled" ID
3. Each **provenance type** has distinct semantics that should be encoded in the construction function name

### The Solution

Introduce **7 provenance-appropriate construction functions** that clarify the **origin** and **semantics** of each ID:

| Function | Provenance | Meaning |
|----------|-----------|---------|
| `from_source()` | External, Imported | Comes FROM external system |
| `derived_from()` | Derived | Computed FROM data |
| `from_client()` | ClientProvided | Comes FROM user/client |
| `for_scope()` | Scoped | Scoped TO context |
| `alias_for()` | AliasOf | ALIAS FOR entity |
| `for_temporary()` | Temporary | FOR TEMPORARY use |
| `for_test()` | Generated | FOR TESTING |

All are **zero-cost aliases** to `for_labeled()`. The benefit is **semantic guidance** via function naming.

### Benefits

✅ **Self-documenting code** — function names document provenance intent  
✅ **Type-safe guidance** — compiler helps steer toward correct usage  
✅ **Faster code reviews** — reviewers see intent immediately  
✅ **Prevents mistakes** — semantic mismatch is obvious  
✅ **Backward compatible** — `for_labeled()` still works  

---

## Scope & Structure

### Four Phases (Sequential Dependency)

```
Phase 1: Core Implementation (1-2 days)
    └─ Add 7 methods to Id<T, ID>
    └─ Update CHANGELOG
    └─ Ready for Phase 2+

Phase 2: Documentation & Lessons (1-2 days)
    ├─ Create ref/lessons/provenance-construction-patterns.md ✅
    ├─ Update examples/
    └─ Ready for downstream adoption

Phase 3: Comprehensive Testing (1 day)
    ├─ Unit tests (7 methods × 5+ tests each)
    ├─ Integration tests (multi-provenance workflows)
    ├─ Doc tests (rustdoc examples)
    └─ Backward compatibility tests

Phase 4: Downstream Integration (2-3 days) [After Phase 1]
    ├─ Update turtle-core/ids.rs
    ├─ Update turtle-spark/mcp_client.rs
    └─ Verify tests pass
```

**Total Time**: ~5-8 days with concurrent work

---

## Detailed Documentation

### 1. Analysis & Design
**File**: `tid-6n4-provenance-construction-analysis.md`

Complete semantic analysis of all 8 provenance types:
- Current state and gap analysis
- Detailed explanation of each provenance type
- Current vs. proposed usage
- Design principles
- Migration strategy

**Key Sections**:
- Executive Summary
- Problem Statement
- Eight Provenance Types (detailed explanations)
- Proposed Construction API (summary table)
- Design Principles (6 principles)
- Migration Strategy (3 phases)

**For**: Architects, designers, reviewers

---

### 2. Implementation Plan
**File**: `tid-6n4-implementation-plan.md`

Detailed, step-by-step implementation instructions:

**Phase 1: Core Implementation**
- Add 7 new methods to `Id<T, ID>`
- Complete rustdoc with examples
- Update CHANGELOG
- Implementation checklist

**Phase 2: Documentation & Lessons**
- Create `ref/lessons/provenance-construction-patterns.md`
- Update examples/
- Cross-references

**Phase 3: Testing Strategy**
- Unit test structure
- Integration test scenarios
- Doc test verification
- Backward compatibility tests
- Test checklist

**Phase 4: Downstream Integration**
- turtle-core updates
- turtle-spark updates
- Verification checklist

**For**: Implementers (can follow step-by-step)

---

### 3. Test Plan
**File**: `tid-6n4-test-plan.md`

Comprehensive testing specification:

**Test Pyramid**:
- **Layer 1 (Unit)**: Each function works correctly (100% coverage)
- **Layer 2 (Integration)**: Functions work together (80%+ coverage)
- **Layer 3 (Documentation)**: Examples are correct (100%)
- **Layer 4 (Compatibility)**: Backward compatible (100%)

**Test Details**:
- Unit test structure for each of 7 methods
- Integration test scenarios (4 realistic workflows)
- Documentation test verification
- Backward compatibility tests
- Test utilities and helpers

**Acceptance Criteria**:
- All unit tests pass
- All doc tests pass
- All integration tests pass
- Code coverage ≥ 85%
- Zero new clippy warnings
- 100% backward compatible

**For**: Test implementers, QA

---

### 4. Lessons & Guide
**File**: `ref/lessons/provenance-construction-patterns.md`

Reusable guide for all tagid users (Turtle, downstream, all AI assistants):

**Sections**:
- Overview (what and why)
- Decision Tree (which function to use)
- Eight Provenance Patterns (detailed with examples)
- Migration Guide (how to update existing code)
- FAQ (common questions)
- Quick Reference (lookup table)

**Features**:
- Realistic code examples for each pattern
- Common mistakes and corrections
- Clear "when to use" for each function
- Cross-references to detailed docs

**For**: Users implementing IDs, code reviewers, future developers

---

## Acceptance Criteria

### Phase 1 (Core Implementation)
- [x] All 7 methods implemented in `src/id/identifier.rs`
- [x] Complete rustdoc with examples
- [x] CHANGELOG updated
- [ ] `cargo doc --no-deps` builds without warnings
- [ ] Zero new clippy warnings

### Phase 2 (Documentation)
- [x] `ref/lessons/provenance-construction-patterns.md` created
- [ ] Examples in `examples/` updated
- [ ] All rustdoc examples pass `cargo test --doc`

### Phase 3 (Testing)
- [ ] Unit tests cover all 7 methods (100%)
- [ ] Integration tests cover multi-provenance scenarios (80%+)
- [ ] All tests pass: `cargo test --all`
- [ ] Code coverage ≥ 85%

### Phase 4 (Downstream)
- [ ] turtle-core/ids.rs updated
- [ ] turtle-spark/mcp_client.rs uses new API
- [ ] `cargo test --workspace` passes
- [ ] Code review confirms semantic correctness

---

## Key Decisions

1. **Zero-cost aliases**: All new methods are aliases to `for_labeled()`, not separate implementations. This keeps the code simple and emphasizes that the benefit is semantic guidance, not functional change.

2. **Backward compatible**: `for_labeled()` remains available and working. Existing code doesn't break. New code uses semantic functions.

3. **Comprehensive rustdoc**: Each method has detailed documentation with semantics, examples, and "See Also" sections. This makes the documentation part of the code.

4. **Reusable lessons**: `ref/lessons/provenance-construction-patterns.md` is in the tagid repo but can be copied/referenced by any downstream project (Turtle, etc.). It's multi-assistant-compatible.

5. **Type-driven guidance**: The provenance type (External, Generated, etc.) guides you toward the right construction function. This aligns semantic types with semantic APIs.

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| **Developers continue using `for_labeled()`** | Code review guidance; future linting rules |
| **Confusion about which function to use** | Decision tree in lessons; examples per type |
| **Incomplete rustdoc** | All examples must pass `cargo test --doc` |
| **Backward compatibility break** | Extensive backward compat tests |
| **Downstream projects miss update** | Copy lessons doc; Phase 4 updates Turtle |

---

## Timeline

### Critical Path (Sequential)

```
Week 1:
  Day 1-2: Phase 1 (Core Implementation) — BLOCKING for others
  Day 2-3: Phase 2 (Documentation) — CAN start after Day 1
  Day 3-4: Phase 3 (Testing) — PARALLEL with Phase 1-2
  Day 5+: Phase 4 (Downstream) — AFTER Phase 1
```

### Parallel Tracks

- **Phase 1 & 2** — can overlap after Phase 1 core is done
- **Phase 3** — can start immediately, independent
- **Phase 4** — must wait for Phase 1 completion

### Estimated Duration

- Phase 1: 1-2 days
- Phase 2: 1-2 days (can overlap Phase 1)
- Phase 3: 1 day (can overlap Phase 1-2)
- Phase 4: 2-3 days (after Phase 1)

**Total**: 5-8 days with optimized scheduling

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| **Semantic clarity** | "for_labeled()" | Function name documents provenance |
| **Type safety** | Generic method | Provenance-specific guidance |
| **Reviewer burden** | Must trace type params | Function name communicates intent |
| **Correctness** | Possible to misuse | Semantic mismatch is obvious |
| **Test coverage** | N/A | 85%+ overall |
| **Backward compat** | N/A | 100% of tests pass |

---

## Implementation Checklist

### Phase 1: Core Implementation (tid-6n4.1)
- [ ] Review analysis doc (`tid-6n4-provenance-construction-analysis.md`)
- [ ] Add 7 new methods to `src/id/identifier.rs`
- [ ] Write comprehensive rustdoc for each method
- [ ] Test each method in isolation
- [ ] Update CHANGELOG.md
- [ ] Run `cargo test --lib` — all pass
- [ ] Run `cargo doc --no-deps` — no warnings
- [ ] Run `cargo clippy` — zero new warnings

### Phase 2: Documentation (tid-6n4.2)
- [ ] Create/review `ref/lessons/provenance-construction-patterns.md`
- [ ] Create `examples/provenance_aware_construction.rs`
- [ ] Verify examples compile: `cargo build --examples`
- [ ] Verify all rustdoc examples pass: `cargo test --doc`
- [ ] Add cross-references in provenance.rs rustdoc
- [ ] Add cross-references in identifier.rs rustdoc

### Phase 3: Testing (tid-6n4.3)
- [ ] Create unit tests (per test plan)
- [ ] Create integration tests (per test plan)
- [ ] Verify backward compatibility tests
- [ ] Run full suite: `cargo test --all`
- [ ] Check coverage: `cargo tarpaulin`
- [ ] Zero clippy warnings: `cargo clippy --all-targets`

### Phase 4: Downstream (tid-6n4.4)
- [ ] Update `crates/turtle-core/src/domain/ids.rs`
- [ ] Update `crates/turtle-spark/src/mcp_client.rs`
- [ ] Verify tests: `cargo test --workspace`
- [ ] Code review for semantic correctness

---

## Related Work

- **Upstream (tagid)**: This epic
- **Downstream (Turtle)**: trtl-tpr (MCP ID parsing migration)
- **Cross-project**: Any project using tagid will benefit

---

## For Implementers

Start here:
1. Read [tid-6n4-provenance-construction-analysis.md](tid-6n4-provenance-construction-analysis.md) (design rationale)
2. Follow [tid-6n4-implementation-plan.md](tid-6n4-implementation-plan.md) (step-by-step)
3. Implement per [tid-6n4-test-plan.md](tid-6n4-test-plan.md) (testing)
4. Reference [ref/lessons/provenance-construction-patterns.md](../ref/lessons/provenance-construction-patterns.md) (examples)

---

## Questions & Clarifications

For clarifications, refer to:
- **What is provenance?** → See analysis doc, section "Eight Provenance Types"
- **How do I choose the right function?** → See lessons doc, "Decision Tree"
- **What should I test?** → See test plan, section "Test Implementation Details"
- **How do I implement Phase 1?** → See implementation plan, section "Phase 1"
- **Is this backward compatible?** → Yes; see "Risk Mitigation", design principle "Backward Compatibility"

---

## Conclusion

This epic adds **semantic clarity** to ID construction across all tagid-based projects. By encoding provenance intent in function names, we make code more self-documenting, easier to review, and less error-prone.

The four phases are sequential but can overlap, totaling ~5-8 days of work. Phase 1 is blocking for the others; Phase 3 (testing) is independent.

All documentation is reusable across projects (Turtle, downstream, all AI assistants).

**Ready to proceed?** Start with Phase 1 (tid-6n4.1).

