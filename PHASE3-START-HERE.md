# Phase 3: START HERE (Architecture & Implementation Spec)

**Status**: ✅ Ready for Phase 3a coding  
**Date**: 2026-02-03  
**Scope**: Source trait design + LabelPolicy + Canonical slug table + Examples folder  
**Effort**: Phase 3a 3.5h + Phase 3b 1.75h + Phase 3c 2.75h + Phase 3d 1h + Phase 3e (parallel) 4h = **13.25h total**

---

## Quick Navigation

**Just starting?** → Read "Core Design" section (15 min)  
**Implementation ready?** → Jump to "Source Trait Spec" (30 min reference)  
**Need the slug table?** → See "Canonical Slug Table" (quick lookup)  
**Creating examples?** → See "Examples Folder Design" (specification)

---

## Core Design (15 min read)

### The 3-Layer Architecture

Phase 3 adds **Source trait + LabelPolicy** to separate concerns:

**Layer 1: Source Trait**
- Defines **what** provenance (where did this ID come from?)
- Provides SLUG, VENDOR constants
- Implements parse_from_spec / format_to_spec
- No labeling logic (keeps it simple)

**Layer 2: Labeling Trait**
- Defines **how** to validate/normalize labels
- Independent of source type
- Can evolve separately

**Layer 3: LabelPolicy Bridge**
- Defines **why** a source makes labeling choices
- Source expresses preference (Opaque, EntityNameDefault, etc.)
- Labeling logic respects preference

### Wire Format (Canonical)

```
<entity_label>@<source_slug>[/<vendor>[/<extra>]]

Examples:
  app_123@gen                    (Generated)
  customer_abc@ext/stripe        (External, Stripe provider)
  job_5@imp/spark                (Imported, Spark source)
  user_slug@der/slug             (Derived, slug method)
  resource@sco/tenant-acme       (Scoped, tenant context)
  session@tmp                    (Temporary)
  idempotency@usr                (User-provided)
  email@als/email                (Alias, email type)
```

### Generated Default Behavior

```rust
// Default: entity-name labeling
let stage_id = StageId::generate();  // label = "Stage"
println!("{}", stage_id);  // Output: Stage@gen

// Override: custom label
let id = StageId::generate_with_label("fast-path");
println!("{}", id);  // Output: fast-path@gen
```

---

## Canonical Slug Table

**Reference: Use this table when implementing source types**

| Source Type | Slug | VENDOR | LABEL_POLICY | Usage |
|------------|------|--------|--------------|-------|
| **Generated** | `gen` | (none) | EntityNameDefault | Internal UUID generation |
| **External** | `ext` | yes (stripe, spark, github) | Opaque | External system IDs |
| **Imported** | `imp` | yes (spark, legacy-db) | ExternalKey | Migration/backfill IDs |
| **Derived** | `der` | yes (slug, hash) | Derived | Computed labels |
| **Scoped** | `sco` | yes (tenant-id, org-xyz) | EntityNameDefault | Scope-dependent IDs |
| **Temporary** | `tmp` | (none) | Opaque | Ephemeral IDs |
| **ClientProvided** | `usr` | (none) | Opaque | User-supplied IDs |
| **AliasOf** | `als` | yes (email, slug) | ExternalKey | Secondary identifiers |

**Key Rules**:
- Slug is always lowercase, 3 chars (gen, ext, imp, der, sco, tmp, usr, als)
- VENDOR is vendor/system name (lowercase): `stripe`, `spark`, `github`, `tenant-123`
- Wire format: `slug` or `slug/vendor` or `slug/vendor/extra`

---

## Source Trait Specification

### Trait Definition

```rust
pub trait Source: Default + Clone + Debug {
    /// Canonical slug for this source type (3 letters, lowercase)
    /// Examples: "gen", "ext", "imp", "der", "sco", "tmp", "usr", "als"
    const SLUG: &'static str;
    
    /// Optional vendor/system name (if parameterized)
    /// Examples: "stripe", "spark", "github", "tenant-123"
    /// Default: None (for non-parameterized sources like Generated, Temporary)
    const VENDOR: Option<&'static str> = None;
    
    /// What labeling preference does this source have?
    /// Tells Labeling layer what strategy to use
    const LABEL_POLICY: LabelPolicy;
    
    /// Parse source specification from string
    /// E.g., "ext/stripe" → External<Stripe>
    /// Default: just verify SLUG matches, reject if parameterized
    fn parse_from_spec(spec: &str) -> Result<Self, ParseError> {
        // Default implementation checks slug only
        let parts: Vec<&str> = spec.split('/').collect();
        if parts[0] != Self::SLUG {
            return Err(ParseError::InvalidSlug);
        }
        if parts.len() > 1 && Self::VENDOR.is_none() {
            return Err(ParseError::UnexpectedVendor);
        }
        Ok(Self::default())
    }
    
    /// Format source back to specification string
    /// E.g., External<Stripe> → "ext/stripe"
    /// Default: just return SLUG; override if VENDOR present
    fn format_to_spec(&self) -> String {
        match Self::VENDOR {
            Some(v) => format!("{}/{}", Self::SLUG, v),
            None => Self::SLUG.to_string(),
        }
    }
}
```

### LabelPolicy Enum

```rust
pub enum LabelPolicy {
    /// Source has no opinion; caller must provide label explicitly
    /// Used by: External, Temporary, ClientProvided
    /// Example: customer_abc@ext/stripe (label must be provided)
    Opaque,
    
    /// Default label is Entity type name (if caller doesn't override)
    /// Used by: Generated, Scoped (when wrapping Generated)
    /// Example: StageId::generate() → label = "Stage"
    EntityNameDefault,
    
    /// Label must come from external source (caller provides key)
    /// Used by: Imported, AliasOf (for alias-type entities)
    /// Example: job_5@imp/spark (label is external key)
    ExternalKey,
    
    /// Label is derived/computed automatically
    /// Optional prefix for consistency
    /// Used by: Derived
    /// Example: user_john-smith@der/slug
    Derived { 
        prefix: Option<&'static str> 
    },
}
```

---

## Implementing the 8 Source Types

### Template for Each Type

```rust
#[derive(Debug, Clone, Copy, Default)]
pub struct Generated;

impl Source for Generated {
    const SLUG: &'static str = "gen";
    const VENDOR: Option<&'static str> = None;
    const LABEL_POLICY: LabelPolicy = LabelPolicy::EntityNameDefault;
    
    fn parse_from_spec(spec: &str) -> Result<Self, ParseError> {
        if spec != "gen" {
            Err(ParseError::InvalidSlug)
        } else {
            Ok(Generated)
        }
    }
    
    fn format_to_spec(&self) -> String {
        "gen".to_string()
    }
}
```

### Quick Reference: All 8 Types

```rust
// 1. Generated (Internal)
#[derive(Debug, Clone, Copy, Default)]
pub struct Generated;
// SLUG: gen, VENDOR: None, POLICY: EntityNameDefault

// 2. External<Provider> (Parameterized)
#[derive(Debug, Clone, Copy, Default)]
pub struct External;
// SLUG: ext, VENDOR: Some("stripe"), POLICY: Opaque
// VENDOR: Some("spark"), Some("github"), etc.

// 3. Imported<From> (Parameterized)
#[derive(Debug, Clone, Copy, Default)]
pub struct Imported;
// SLUG: imp, VENDOR: Some("spark"), POLICY: ExternalKey

// 4. Derived<Method> (Parameterized)
#[derive(Debug, Clone, Copy, Default)]
pub struct Derived;
// SLUG: der, VENDOR: Some("slug"), POLICY: Derived { prefix: None }

// 5. Scoped<Scope, Inner>
#[derive(Debug, Clone, Copy, Default)]
pub struct Scoped;
// SLUG: sco, VENDOR: Some("tenant-123"), POLICY: EntityNameDefault

// 6. Temporary (Ephemeral)
#[derive(Debug, Clone, Copy, Default)]
pub struct Temporary;
// SLUG: tmp, VENDOR: None, POLICY: Opaque

// 7. ClientProvided (User-supplied)
#[derive(Debug, Clone, Copy, Default)]
pub struct ClientProvided;
// SLUG: usr, VENDOR: None, POLICY: Opaque

// 8. AliasOf<Canonical> (Secondary ID)
#[derive(Debug, Clone, Copy, Default)]
pub struct AliasOf;
// SLUG: als, VENDOR: Some("email"), POLICY: ExternalKey
```

---

## Examples Folder Design

### Structure

```
tagid-rs/examples/
├── README.md                           (30min to write)
├── 00_parse_and_format.rs              (basic wire format)
├── 01_generated_default_label.rs       (Generated defaults)
├── 02_generated_override_label.rs      (Generated override)
├── 03_external_stripe.rs               (External<Stripe> pattern)
├── 04_imported_spark.rs                (Imported<Spark>)
├── 05_derived_slug.rs                  (Derived<Slugify>)
├── 06_scoped_tenant.rs                 (Scoped<TenantId, Gen>)
├── 07_custom_source.rs                 (Define custom Source)
├── 08_roundtrip_and_validation.rs      (Parse→format→parse)
├── 09_serde_json.rs                    (JSON serialization)
└── 10_backward_compat.rs               (Legacy ID parsing)
```

### What Each Example Shows

| File | Shows | Learning Outcome |
|------|-------|------------------|
| 00 | Parsing `app@gen` format | Wire format rules, basic API |
| 01 | `StageId::generate()` → label="Stage" | Generated default labeling |
| 02 | `StageId::generate_with_label("fast")` | Generated override |
| 03 | `External<Stripe>` with real provider | Parameterized sources, VENDOR |
| 04 | `Imported<Spark>` for migration IDs | ExternalKey policy |
| 05 | `Derived<Slug>` computed labels | Derived policy, custom labels |
| 06 | `Scoped<TenantId, Generated>` | Scope composition with inner source |
| 07 | Define custom Source type | Extensibility, implementing trait |
| 08 | `parse("x@gen") → format → parse` | Roundtrip validation |
| 09 | `serde_json` with tagids | Serialization format |
| 10 | Parse `app-123` (legacy, no @) | Migration path, backward compat |

### Example: 03_external_stripe.rs (Template)

```rust
//! Demonstrates External<Stripe> provider pattern
//! 
//! Shows:
//! - Creating External IDs from Stripe
//! - Serialization format
//! - LabelPolicy::Opaque (requires explicit label)

use tagid::prelude::*;

// Define label type
#[derive(Debug, Clone, Copy)]
pub struct StripeCustomer;

impl Label for StripeCustomer {
    const PREFIX: &'static str = "stripe_customer";
}

fn main() {
    // Create from Stripe API response
    let customer_id = "cus_L3H8Z6K9j2";
    
    // Using External<Stripe> (parameterized source)
    let id = TagId::with_label(
        customer_id.to_string(),
        Sourced::<StripeCustomer, External<Stripe>>::default(),
    );
    
    // Display (canonical, no labels in wire format)
    println!("ID: {}", id);
    // Output: cus_L3H8Z6K9j2
    
    // Full labeled display (for logs)
    println!("Full: {}", id.labeled(LabelMode::Full));
    // Output: StripeCustomer@ext/stripe::cus_L3H8Z6K9j2
    
    // Serialization (JSON)
    let json = serde_json::to_string(&id).unwrap();
    println!("JSON: {}", json);
    // Output: "cus_L3H8Z6K9j2"
    
    // Roundtrip
    let parsed: TagId<...> = serde_json::from_str(&json).unwrap();
    assert_eq!(id, parsed);
}
```

---

## Phase 3a Detailed Checklist

### Step 1: Create src/id/source.rs (1 hour)

**Deliverables**:
- [ ] Define `TAGID_DELIMITER` constant (`@`)
- [ ] Define `TAGID_SOURCE_SEP` constant (`/`)
- [ ] Implement `Source` trait with all methods
- [ ] Define `LabelPolicy` enum (Opaque, EntityNameDefault, ExternalKey, Derived)
- [ ] Implement 8 source types (Generated, External, Imported, Derived, Scoped, Temporary, ClientProvided, AliasOf)
- [ ] Each with correct SLUG, VENDOR, LABEL_POLICY
- [ ] Doc comments with examples for each

**Verification**:
```bash
cargo check --all
# Should compile with no errors
```

### Steps 2-5: Continue as per PHASE3-IMPLEMENTATION-CHECKLIST.md

---

## Constants to Define

```rust
// In tagid-rs/src/lib.rs or src/id/source.rs
pub const TAGID_DELIMITER: char = '@';      // Between entity and source
pub const TAGID_SOURCE_SEP: char = '/';     // Between source/vendor/extra
```

---

## Key Implementation Notes

### Parsing Rules

- Split on `@` first: `entity` and `source_spec`
- Parse source_spec as `slug ( "/" segment )*`
- Each segment: `[a-z0-9][a-z0-9_-]*` (lowercase, digits, dash, underscore)
- Forbid `@` inside segments

### Backward Compatibility

If pre-Phase-3 IDs exist:
- Parser detects absence of `@` → interpret as legacy
- Example: `app-123` (old) vs `app-123@gen` (new)
- Tests verify both formats parse correctly

### Customization

**Don't do**: per-source delimiter customization  
**Do**: keep delimiter global (`@`) and stable

---

## Questions Before Coding?

Review:
1. **Slug table**: Canonical slug table above (gen, ext, imp, etc.)
2. **Wire format**: entity@source[/vendor] examples
3. **LabelPolicy**: What each policy means
4. **Generated behavior**: Default label is entity name
5. **Examples**: 10 files covering all source types

---

## Next: Start Coding

1. Open `tagid-rs/src/id/source.rs` (will create)
2. Reference this document for trait spec + slug table
3. Implement per step-by-step checklist above
4. Run `cargo check --all` after Step 1
5. After all steps: `cargo test --lib id::source`

---

**Status**: ✅ Architecture Final, Ready to Implement  
**Document**: [Full spec in PHASE3-ORACLE-DESIGN-ANSWERS.md](../spark-turtle/history/PHASE3-ORACLE-DESIGN-ANSWERS.md)  
**Questions**: Reference "Quick Navigation" at top of this file
