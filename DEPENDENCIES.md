# Cross-Project Dependencies

## Phase 3: tagid-rs Enhancement for Turtle-Spark

This document tracks cross-project dependencies for Phase 3 (tagid-rs enhancement).

### Status

**Status**: ✅ Beads structure created, ready for implementation  
**Last Updated**: 2026-02-03  
**Primary Epic**: `tid-abl` (tagid-rs) enables `trtl-7d0` (spark-turtle)

---

## Dependency Map

### Upstream: tagid-rs (this project)

```
tagid-rs Work (tid-abl)
├── tid-gyr: Create Provenance trait + 8 types (1h)
├── tid-pn4: Create Sourced<E, S> wrapper (0.5h)
├── tid-k5r: Add type parameters (0.5h)
├── tid-sxw: Labeled wrapper + LabelMode (1h)
├── tid-80v: Canonical ID rules (1h)
├── tid-6zq: Test suite (1h)
├── tid-440: Port docs to ref/ (0.75h)
├── tid-ppy: Update README + examples (1h)
└── tid-io0: 🚀 Release tagid-rs vX.Y.Z (0.5h)
       ↓
spark-turtle Integration (trtl-7d0)
├── trtl-e5k: Update Cargo.toml
├── trtl-uoz: Refactor ids.rs
├── trtl-b6w: Update fixtures.rs
├── trtl-tpr: Update mcp_client.rs
├── trtl-hku: Update models.rs
├── trtl-6kq: Integration tests
├── trtl-7lp: Serialization compat
└── trtl-4le: Documentation
```

**Timeline**:
- **Phase A** (4-5 days): tid-gyr through tid-ppy (this project)
- **Phase B** (5-7 days): trtl-e5k through trtl-4le (spark-turtle, after tid-io0)
- **Phase C** (2 days): Release & integration complete

### Critical Dependencies

**Must Complete Before spark-turtle Integration**:
1. tid-gyr (Provenance trait) - TYPE FOUNDATION
2. tid-pn4 (Sourced<E, S>) - WRAPPER FRAMEWORK  
3. tid-k5r (Type parameters) - SEMANTIC RICHNESS
4. tid-sxw (Labeling) - HUMAN READABILITY
5. tid-80v (Canonical rules) - SERIALIZATION SAFETY
6. tid-io0 (Release) - CONSUMABLE VERSION

---

## API Contract (What spark-turtle expects)

### Types & Traits

```rust
// From Provenance trait
pub trait Provenance: Default + Clone {
    const NAME: &'static str;
    type Descriptor: Default + Clone;
}

// Concrete types
pub struct External<Provider = ()>;
pub struct Generated<Strategy = ()>;
pub struct Imported<From>;
pub struct Derived<Method>;
pub struct Scoped<Scope, Inner>;
pub struct Temporary;
pub struct ClientProvided;
pub struct AliasOf<Canonical>;

// Wrapper
pub struct Sourced<E: Label, S: Provenance>;

// Labeled formatting
pub enum LabelMode {
    None,
    Short,
    Full,
}
```

### Behavior Requirements

**ID Creation**:
- `AppId::from_source(s)` - External IDs from Spark
- `StageId::generate()` - Internal UUID generation
- Type-safe: can't call `generate()` on External types

**Serialization**:
- Serde (JSON): canonical format ONLY (no labels)
- Display: canonical format ONLY
- Debug: can include labels (not stable)
- `.labeled(mode)` - opt-in human-readable labeling

**Backward Compatibility**:
- Existing `Id<E, T>` still works
- Existing tests still compile
- Serde format understandable by old code

---

## Beads Task References

| ID | Title | Status | Effort | Notes |
|----|-------|--------|--------|-------|
| tid-abl | EPIC: tagid-rs Enhancement | OPEN | 7.75h | Foundation epic |
| tid-gyr | Provenance trait + 8 types | OPEN | 1h | **CRITICAL PATH** |
| tid-pn4 | Sourced<E, S> wrapper | OPEN | 0.5h | **CRITICAL PATH** |
| tid-k5r | Type parameters | OPEN | 0.5h | **CRITICAL PATH** |
| tid-sxw | Labeled wrapper | OPEN | 1h | **CRITICAL PATH** |
| tid-80v | Canonical ID rules | OPEN | 1h | **CRITICAL PATH** |
| tid-6zq | Test suite | OPEN | 1h | Verify all works |
| tid-440 | Port docs to ref/ | OPEN | 0.75h | Porting work |
| tid-ppy | README + examples | OPEN | 1h | Documentation |
| tid-io0 | Release tagid-rs | OPEN | 0.5h | **BLOCKER UNBLOCK** |

**View all**:
```bash
bd ls --parent tid-abl --json
bd ready --json  # Show unblocked work
```

---

## Verification Checklist

### Before Starting Implementation

- [ ] Read `ref/tagid-redesign.md` (full spec)
- [ ] Read `ref/spark-turtle__tagid-implementation-checklist.md` (step-by-step)
- [ ] Understand Oracle reasoning
- [ ] Review examples in `ref/` folder

### During Implementation

- [ ] After tid-gyr: `cargo test --lib id::source` passes
- [ ] After tid-pn4: `cargo test --lib id::sourced` passes
- [ ] After tid-80v: Serialization round-trip tests pass
- [ ] Before tid-io0: `cargo clippy --all` clean

### Before Release

- [ ] All tests passing: `cargo test --all`
- [ ] No Clippy warnings: `cargo clippy --all --deny warnings`
- [ ] Docs generated: `cargo doc --no-deps`
- [ ] CHANGELOG updated
- [ ] Version bumped in Cargo.toml

### After Release

- [ ] Git tag created: `git tag tagid-rs/vX.Y.Z`
- [ ] spark-turtle can pull new version
- [ ] Cross-project DEPENDENCIES.md updated

---

## Consumers

### Primary: spark-turtle

**spark-turtle EPIC**: `trtl-7d0`  
**Status**: Waiting for tid-io0 (release)  
**Will consume**: 
- New Provenance & Sourced types
- Type parameters on External/Generated
- Labeling mechanism
- Backward compat aliases

**Example use** (after this epic completes):
```rust
use tagid::id::{Sourced};
use tagid::id::source::{External, Generated};

pub type AppId = Id<Sourced<AppLabel, External<Spark>>, String>;
pub type StageId = Id<Sourced<StageLabel, Generated<UuidV7>>, String>;
```

### Secondary: icehouse (future)

**Status**: Preparatory (not started)  
**Timeline**: After spark-turtle integration succeeds  
**Will use**: Multi-provider semantics (Stripe, Github, Okta, etc.)

---

## Reference Materials

### In this Project (tagid-rs)

- **`ref/`** folder (created with tid-440):
  - `tagid-redesign.md`
  - `spark-turtle__tagid-implementation-checklist.md`
  - `spark-turtle__tagid-examples.md`
  - `spark-turtle__tagid-architecture.md`

- **`README.md`** (updated with tid-ppy):
  - Concepts: Canonical ID, Provenance, Type parameters
  - Examples for External, Generated
  - Real-world scenarios (turtle, icehouse)

### External References

- **spark-turtle project**:
  - `DEPENDENCIES.md` - Reciprocal tracking
  - `history/phase3-ids-final-design-v2.md` - Full specification
  - `history/PHASE3-IMPLEMENTATION-CHECKLIST.md` - Integration steps
  - `history/QUICK-REFERENCE-PHASE3.md` - Cheat sheet

---

## Parallel Work

### Phase 4: MCP Helper Refactor (spark-turtle)

**Epic**: `trtl-00l` (separate)  
**Status**: Designed, ready for implementation  
**Dependency**: None on tagid-rs  
**Timeline**: Can start immediately or after Phase 3b

---

## Success Criteria

✅ **DELIVERED BY THIS EPIC**:
- [x] Beads structure created (tid-abl + 9 children)
- [ ] Provenance trait implemented (tid-gyr)
- [ ] Sourced wrapper implemented (tid-pn4)
- [ ] Type parameters working (tid-k5r)
- [ ] Labeling functional (tid-sxw)
- [ ] Canonical rules enforced (tid-80v)
- [ ] Tests pass (tid-6zq)
- [ ] Docs ported (tid-440)
- [ ] README updated (tid-ppy)
- [ ] Release published (tid-io0)

✅ **QUALITY GATES**:
- [ ] 100% test coverage on source.rs, sourced.rs
- [ ] 0 Clippy warnings
- [ ] Backward compatible (no breaking changes to existing APIs)
- [ ] No performance regression

✅ **INTEGRATION READY**:
- [ ] spark-turtle can pull tagid-rs vX.Y.Z
- [ ] All contracts met (type, behavior, serialization)
- [ ] Documentation complete
- [ ] Examples working

---

## Communication & Handoff

### Status Updates

- **To spark-turtle team**: Message when tid-io0 (release) is ready
- **In DEPENDENCIES.md**: Update status after each major milestone
- **In commits**: Reference bead IDs (e.g., "tid-gyr: implement Provenance trait")

### Format for Commits

```
[tid-gyr] Implement Provenance trait + 8 core types

- Create source.rs with External, Generated, Imported, Derived, Scoped, Temporary, ClientProvided, AliasOf
- Define Provenance trait with NAME and Descriptor
- Add doc comments and examples

Implements: tid-abl epic
```

### Rollback Plan

If critical blocker:
```bash
git checkout HEAD -- src/
bd update tid-abl --status blocked --json
bd update trtl-7d0 --status blocked --json
# Document blocker in beads description
```

---

## Timeline Estimate

| Phase | Tasks | Duration | Gate |
|-------|-------|----------|------|
| A: Core | tid-gyr through tid-80v | 4-5 days | Tests passing |
| B: Docs & Testing | tid-6zq, tid-440, tid-ppy | 2-3 days | Clippy clean |
| C: Release | tid-io0 | 0.5 day | Version published |
| **Total** | | **7-9 days** | spark-turtle ready |

**Critical Path**: tid-gyr → tid-pn4 → tid-sxw → tid-80v → tid-io0 (4-5 days minimum)

---

## FAQ

**Q: Can I start tid-ppy (docs) before tid-80v?**  
A: Yes, doc writing can start in parallel. Just ensure tid-80v completes before final review.

**Q: What if spark-turtle has questions during our implementation?**  
A: Great - they might discover edge cases. Update tid-abl description with findings. We can iterate.

**Q: Do we need to support icehouse during this phase?**  
A: No. Design is extensible for icehouse. Just ensure nothing breaks multi-provider patterns.

**Q: What about backward compat with existing tagid users?**  
A: CRITICAL. Ensure old `Entity` + `Label` traits still work. New features are opt-in.

---

**Last Updated**: 2026-02-03  
**Created By**: Oracle-guided cross-project planning  
**Next Update**: When tid-abl moves to in_progress
