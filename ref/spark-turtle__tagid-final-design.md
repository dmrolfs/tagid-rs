---
Source: spark-turtle/history/phase3-ids-final-design-v2.md
Commit: (from design session 2026-02-03)
Date: 2026-02-03
Notes: Core spec extracted for tagid-rs context; removed spark-turtle-specific examples; kept generalized patterns for icehouse/future projects
---

# tagid Enhancement Specification: Provenance & Sourced Semantics

**Status**: Final Design ✅ | Ready for Implementation  
**Scope**: tagid-rs core library (no breaking changes to existing APIs)  
**Effort**: 7-9 hours total (5h core, 2-4h docs/integration)

---

## Executive Summary

Enhance tagid-rs with **Provenance trait** and **Sourced<E, S> wrapper** to enable rich semantic information about where IDs originate. This allows projects like spark-turtle, icehouse, and future systems to:

- ✅ Distinguish External IDs (from other systems) from Generated IDs (internal)
- ✅ Add optional type parameters (External<Stripe>, Generated<UuidV7>) without complexity
- ✅ Format IDs with optional labels (canonical, short, full) for logging
- ✅ Maintain strict serialization rules (canonical only, no labels in JSON/DB)
- ✅ Stay backward compatible (new features opt-in)

---

## Core Design Decisions

### 1. Terminology: Provenance (not Source)

**Term**: "Provenance" = where an ID originates + potential metadata

**Why**: 
- "Provenance" correctly describes origin/lineage
- Familiar in databases, art, supply chain
- Covers external, generated, imported, derived, scoped, temporary, client-provided, alias cases
- More precise than "source"

**Backward Compat**: `pub use Provenance as Source;` allows old code to work

### 2. Type Parameters: Optional, Zero-Cost

**Design**:
```rust
pub struct External<Provider = ()>;      // Default: unit type
pub struct Generated<Strategy = ()>;     // Default: unit type
```

**Examples**:
```rust
pub type AppId = Id<Sourced<AppLabel, External<Spark>>, String>;
pub type StageId = Id<Sourced<StageLabel, Generated<UuidV7>>, String>;
pub type GenericId = Id<Sourced<Entity, External>, String>;  // Works with defaults
```

**Why**:
- Optional (defaults to unit `()`)
- Zero-cost (PhantomData, no runtime overhead)
- Enables rich semantics without forcing complexity
- Projects can specialize later (External → External<Stripe>)

### 3. Canonical ID: Always Label-Free

**Rules**:
- `Serialize` (JSON) → canonical only (no labels)
- `Deserialize` (JSON) → expects canonical format
- `Display` → canonical only
- `Debug` → can include labels (not stable)
- `.as_str()` → canonical only
- `.labeled(mode)` → opt-in labeling

**Why**:
- Prevents subtle bugs (serde surprises, failed DB queries)
- Canonical ID = stable value
- Labeling = human presentation only
- Clear rules remove confusion

### 4. Labeling: Explicit & Opt-In

**Design**:
```rust
pub enum LabelMode {
    None,   // canonical only: "value-123"
    Short,  // entity + canonical: "AppLabel::value-123"
    Full,   // entity + provenance + canonical: "AppLabel@external/spark::value-123"
}

// Usage
println!("{}", id.labeled(LabelMode::Full));
tracing::info!("Processed {}", app_id.labeled(LabelMode::Short));
```

**Why**:
- Explicit (intentional, no surprises)
- Flexible (choose richness per context)
- Composable (works in logging macros)
- Prevents accidental label leakage

### 5. Descriptors: Separate from Canonical ID

**Design**:
```rust
pub struct WithProvenance<E, P: Provenance> {
    pub id: Id<Sourced<E, P>, String>,
    pub descriptor: P::Descriptor,  // Provenance-specific metadata
}
```

**Why**:
- Keeps `Id` lean (canonical + phantom types)
- Each provenance defines its own `Descriptor`
- Metadata lives separately (optional, persisted separately)
- No monolithic enum that bloats

---

## 8 Core Provenance Types

### 1. External<Provider = ()>
ID provided by external system. Cannot be generated; must be from_source.

**Examples**:
- `External<Spark>` - Spark application/job IDs
- `External<Stripe>` - Stripe customer/payment IDs
- `External<Github>` - Github user/repo IDs
- `External` (bare) - generic external ID

### 2. Generated<Strategy = ()>
ID created internally. Can only be generated via generate().

**Examples**:
- `Generated<UuidV7>` - UUID v7 strategy
- `Generated<Snowflake>` - Snowflake ID strategy
- `Generated` (bare) - generic generated ID

### 3. Imported<From>
ID brought in from migration/backfill. Has source system.

**Examples**:
- `Imported<LegacyDatabase>`
- `Imported<Csv>`

### 4. Derived<Method>
ID computed/derived from other data.

**Examples**:
- `Derived<Slugify>` - slug from name
- `Derived<Hash>` - hash of content

### 5. Scoped<Scope, Inner: Provenance>
Uniqueness depends on context.

**Examples**:
- `Scoped<TenantId, Generated<UuidV7>>` - tenant-scoped UUID
- `Scoped<OrganizationId, External<Stripe>>` - org-scoped Stripe ID

### 6. Temporary
Valid only short-term (optimistic IDs, session tokens).

### 7. ClientProvided
User/client supplies the ID (idempotency keys, BYO primary key).

### 8. AliasOf<Canonical>
Secondary identifier (email as alias for user).

---

## Provider Markers (ZST)

Marker types to parameterize External<Provider>:

```rust
pub struct Spark;        // Spark system
pub struct Stripe;       // Stripe payments
pub struct Github;       // Github platform
pub struct Okta;         // Okta identity
pub struct AwsS3;        // AWS S3
pub struct GoogleCloud;  // Google Cloud
// ... add as needed
```

These are zero-sized types (PhantomData), purely for compile-time type safety.

---

## Strategy Markers (ZST)

Marker types to parameterize Generated<Strategy>:

```rust
pub struct UuidV4;       // UUID version 4
pub struct UuidV7;       // UUID version 7 (time-based)
pub struct Cuid;         // CUID generation
pub struct Cuid2;        // CUID v2
pub struct Snowflake;    // Snowflake ID
pub struct Nanoid;       // Nano ID
pub struct Hashids;      // Hash-based IDs
// ... add as needed
```

---

## Provenance Trait

```rust
pub trait Provenance: Default + Clone {
    /// Name of this provenance (for display/logging)
    const NAME: &'static str;
    
    /// Optional descriptor type for this provenance
    type Descriptor: Default + Clone;
}

impl Provenance for External<P> {
    const NAME: &'static str = "external";
    type Descriptor = ();  // No metadata by default
}

impl Provenance for Generated<S> {
    const NAME: &'static str = "generated";
    type Descriptor = ();  // No metadata by default
}

// And so on for other 6 types...
```

---

## Sourced<E, S> Wrapper

```rust
pub struct Sourced<E: Label, S: Provenance> {
    entity: PhantomData<E>,
    source: PhantomData<S>,
}

impl<E: Label, S: Provenance> Label for Sourced<E, S> {
    const PREFIX: &'static str = "sourced";
    
    // Decorate label with provenance info
    fn decorate_label(&self) -> String {
        format!("{}@{}", E::PREFIX, S::NAME)
    }
}

// Critical: Entity impl ONLY for Generated sources
impl<E: Label + Entity> Entity for Sourced<E, Generated> {
    type IdGen = E::IdGen;
}

// NO impl for Sourced<E, External> - type-safe!
```

---

## Real-World Usage: spark-turtle

### Pattern 1: External IDs from Spark

```rust
// spark-turtle/src/domain/ids.rs
pub type AppId = Id<Sourced<AppLabel, External<Spark>>, String>;
pub type JobId = Id<Sourced<JobLabel, External<Spark>>, String>;

// In mcp_client.rs, parsing from MCP response
let app_id = AppId::from_source("app-20231215-001");

// Serialization (canonical only, no labels)
let json = serde_json::to_string(&app_id)?;  // "app-20231215-001"

// Logging with labels
tracing::info!("Processing {}", app_id.labeled(LabelMode::Full));
// Output: Processing AppLabel@external/spark::app-20231215-001
```

### Pattern 2: Turtle-Generated IDs

```rust
pub type StageId = Id<Sourced<StageLabel, Generated<UuidV7>>, String>;
pub type TaskId = Id<Sourced<TaskLabel, Generated<UuidV7>>, String>;

// Generation (type-safe, can't do StageId::generate() if using External)
let stage_id = StageId::generate();  // UuidV7 under the hood

// Type-safe semantics
let external_app: Id<Sourced<AppLabel, External<Spark>>, String> = ...;
// external_app.generate()  // COMPILE ERROR - can't generate External IDs!
```

---

## Real-World Usage: icehouse (future)

```rust
// Multi-provider SaaS
pub type StripeCustomerId = Id<Sourced<Customer, External<Stripe>>, String>;
pub type GithubUserId = Id<Sourced<User, External<Github>>, String>;
pub type OktaUserId = Id<Sourced<User, External<Okta>>, String>;

// Internal generation
pub type InternalUserId = Id<Sourced<User, Generated<UuidV7>>, String>;

// Tenant-scoped resources
pub type TenantResourceId<R> = Id<Sourced<R, Scoped<TenantId, Generated<UuidV7>>>, String>;
```

---

## Serialization Contract

**CRITICAL: Canonical format only, no labels**

```rust
// Old code (still works)
let json = r#""app-123""#;
let id: AppId = serde_json::from_str(json)?;
assert_eq!(id.to_string(), "app-123");

// New code (same format)
let new_json = serde_json::to_string(&id)?;  // "app-123"

// Labeling is NOT in JSON
println!("{}", id);  // "app-123"
println!("{}", id.labeled(LabelMode::Full));  // "AppLabel@external/spark::app-123"
```

**Database integration**:
```rust
// sqlx encode/decode uses canonical only
let query = "SELECT * FROM apps WHERE id = ?";
sqlx::query(query)
    .bind(app_id.as_str())  // Always canonical
    .execute(db)
    .await?;
```

---

## Type Safety Benefits

**Compile-time protection**:

```rust
pub type ExternalAppId = Id<Sourced<AppLabel, External<Spark>>, String>;
pub type GeneratedStageId = Id<Sourced<StageLabel, Generated<UuidV7>>, String>;

// This works ✅
let stage_id = GeneratedStageId::generate();

// This FAILS at compile time ❌
let app_id: ExternalAppId = ExternalAppId::generate();  // ERROR: no Entity impl

// This FAILS at compile time ❌
let wrong: ExternalAppId = GeneratedStageId::generate();  // ERROR: type mismatch
```

---

## Backward Compatibility

**Existing code continues to work**:

```rust
// Old style (still works)
pub type AppId = Id<Entity, String>;
let id = AppId::from_string("app-123");

// New style (opt-in)
pub type AppId = Id<Sourced<AppLabel, External<Spark>>, String>;
let id = AppId::from_source("app-123");

// Aliases for transition
pub use Provenance as Source;
pub type Sourced<E, S> = Provenanced<E, S>;  // Old name still works
```

**Serialization format unchanged**:
- Old JSON: `"app-123"`
- New JSON: `"app-123"` (same!)
- No migration needed

---

## Effort Breakdown

| Task | Duration | Notes |
|------|----------|-------|
| 1. Provenance trait + 8 types | 1h | Core foundation |
| 2. Sourced<E, S> wrapper | 0.5h | Label impl + Entity (Generated only) |
| 3. Type parameters | 0.5h | External<P>, Generated<S> with defaults |
| 4. Labeled wrapper + LabelMode | 1h | Display/Debug rules |
| 5. Canonical ID rules (serde/sqlx) | 1h | Verification + tests |
| 6. Test suite | 1h | Unit + integration tests |
| 7. Port docs to ref/ | 0.75h | This document + others |
| 8. Update README + examples | 1h | Usage guide |
| 9. Release preparation | 0.5h | Version, tag, publish |
| **Total** | **7.75h** | |

---

## Success Criteria

✅ All code compiles with zero warnings  
✅ All tests pass (no regressions)  
✅ Backward compatible (old APIs still work)  
✅ Type-safe (External can't generate, Generated can't be from_source)  
✅ Serialization verified (canonical only)  
✅ Documentation complete (README + examples + rustdoc)  
✅ spark-turtle can integrate cleanly  
✅ icehouse patterns enabled (but not blocking)

---

## FAQ

**Q: Why not call it Source instead of Provenance?**  
A: "Provenance" is more accurate (covers 8 types, not just "source"), and matches database terminology.

**Q: What if I want simple External IDs without type parameters?**  
A: Use `External` (bare) with unit default. Zero overhead, same pattern.

**Q: Will serialization break existing code?**  
A: No. Format unchanged (canonical only). Old and new code can interoperate.

**Q: Can I mix External and Generated in the same collection?**  
A: Not without a wrapper. This is intentional (type safety). Use enum if needed:
```rust
pub enum AnyId {
    App(ExternalAppId),
    Stage(GeneratedStageId),
}
```

**Q: What about database constraints (unique, primary key)?**  
A: Use `id.as_str()` (canonical) for all DB operations. Labeling is only for logging.

---

## References

- **spark-turtle DEPENDENCIES.md**: Cross-project dependency tracking
- **Beads**: Implementation tasks (tid-abl epic in tagid-rs)
- **Code**: `src/id/source.rs`, `src/id/sourced.rs`, `src/id/mod.rs`
