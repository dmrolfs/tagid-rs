---
Source: spark-turtle/history/phase3-ids-final-design-v2.md
Commit: (from update session 2026-02-03)
Date: 2026-02-03
Notes: Core spec extracted for tagid-rs context; removed spark-turtle-specific examples; kept generalized patterns for icehouse/future projects. Updated with Feb 3 Clarifications (LabelPolicy, Opaque IDs).
---

# tagid Enhancement Specification: Provenance & Sourced Semantics

**Status**: Final Design v3 ✅ | Ready for Implementation  
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

### 3. Canonical ID: Always Label-Free & Opaque

**Rules**:
- `Serialize` (JSON) → canonical only (no labels)
- `Deserialize` (JSON) → expects canonical format
- `Display` → canonical only
- `Debug` → can include labels (not stable)
- `.as_str()` → canonical only
- `.labeled(mode)` → opt-in labeling

**Clarification (Feb 3)**: **External IDs are OPAQUE**.
- If Stripe provides `"cus_L3H8Z6K9j2"`, the canonical ID is `"cus_L3H8Z6K9j2"`.
- Do **NOT** strip prefixes (e.g., `cus_`).
- Do **NOT** parse or transform the ID at the application boundary.
- The entire value provided by the external system is the canonical ID.

**Why**:
- Prevents subtle bugs (serde surprises, failed DB queries)
- Canonical ID = stable value
- Labeling = human presentation only
- Clear rules remove confusion

### 4. Labeling: Explicit & Opt-In (LabelPolicy)

**Design**:
```rust
pub enum LabelMode {
    None,   // canonical only: "value-123"
    Short,  // entity + canonical: "AppLabel::value-123"
    Full,   // entity + provenance + canonical: "AppLabel@external/spark::value-123"
}

// Usage
println!("{}", id.labeled(LabelMode::Full));
tracing::info!("Processed {}", app_id.labeled()); // Uses default from LabelPolicy
```

**Clarification (Feb 3): LabelPolicy Scope**
`LabelPolicy` is a constant on the `Provenance` trait that controls the **default** behavior of `.labeled()` when called without arguments.
- It does **NOT** affect Display, Serialize, or database storage.
- **Variants**:
    - `Opaque`: Caller must specify mode.
    - `EntityNameDefault`: Default to `Short`.
    - `ExternalKeyDefault`: Default to `Full`.
    - `OpaqueByDefault`: Default to `None`.

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
**Policy**: `ExternalKeyDefault` (Show provenance by default).
**Opaqueness**: Strictly opaque. No prefix stripping.

**Examples**:
- `External<Spark>` - Spark application/job IDs
- `External<Stripe>` - Stripe customer/payment IDs
- `External<Github>` - Github user/repo IDs
- `External` (bare) - generic external ID

### 2. Generated<Strategy = ()>
ID created internally. Can only be generated via generate().
**Policy**: `EntityNameDefault` (Show entity name by default).

**Examples**:
- `Generated<UuidV7>` - UUID v7 strategy
- `Generated<Snowflake>` - Snowflake ID strategy
- `Generated` (bare) - generic generated ID

### 3. Imported<From>
ID brought in from migration/backfill. Has source system.
**Policy**: `ExternalKeyDefault`.

**Examples**:
- `Imported<LegacyDatabase>`
- `Imported<Csv>`

### 4. Derived<Method>
ID computed/derived from other data.
**Policy**: `EntityNameDefault`.

**Examples**:
- `Derived<Slugify>` - slug from name
- `Derived<Hash>` - hash of content

### 5. Scoped<Scope, Inner: Provenance>
Uniqueness depends on context.
**Policy**: Inherits from `Inner`.

**Examples**:
- `Scoped<TenantId, Generated<UuidV7>>` - tenant-scoped UUID
- `Scoped<OrganizationId, External<Stripe>>` - org-scoped Stripe ID

### 6. Temporary
Valid only short-term (optimistic IDs, session tokens).
**Policy**: `OpaqueByDefault`.

### 7. ClientProvided
User/client supplies the ID (idempotency keys, BYO primary key).
**Policy**: `OpaqueByDefault`.

### 8. AliasOf<Canonical>
Secondary identifier (email as alias for user).
**Policy**: `EntityNameDefault`.

---

## Provenance Trait

```rust
pub trait Provenance: Default + Clone {
    /// Name of this provenance (for display/logging)
    const NAME: &'static str;
    
    /// Default labeling policy for human-facing output (.labeled())
    const LABEL_POLICY: LabelPolicy;

    /// Optional descriptor type for this provenance
    type Descriptor: Default + Clone;
}

impl Provenance for External<P> {
    const NAME: &'static str = "external";
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

impl Provenance for Generated<S> {
    const NAME: &'static str = "generated";
    const LABEL_POLICY: LabelPolicy = LabelPolicy::EntityNameDefault;
    type Descriptor = ();
}
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

---

## Feb 3 Design Clarifications Summary

This section documents the specific updates integrated into this design on 2026-02-03.

1.  **LabelPolicy Scope**: 
    - Clarified that `LabelPolicy` *only* affects the default behavior of `.labeled()` when called without arguments. 
    - It does not influence the canonical ID, serialization, or database storage.

2.  **External ID Opaqueness**: 
    - Clarified that IDs from external sources (e.g., Stripe, Spark) are treated as opaque strings.
    - No parsing, stripping, or transformation of prefixes (e.g., "cus_", "app-") is performed. 
    - The raw string IS the canonical ID.