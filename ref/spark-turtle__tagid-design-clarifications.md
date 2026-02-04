---
Source: spark-turtle/history/PHASE3-DESIGN-CLARIFICATIONS.md
Commit: (from update session 2026-02-03)
Date: 2026-02-03
Notes: Documenting critical design clarifications for LabelPolicy and Opaque External IDs
---

# tagid Design Clarifications: LabelPolicy & Opaque IDs

This document clarifies two critical areas of the tagid-rs Phase 3 enhancement: the scope of `LabelPolicy` and the handling of `External` IDs.

## 1. LabelPolicy Scope

**Clarification**: `LabelPolicy` controls **only** the default behavior of the human-facing `.labeled()` method.

### Key Rules
- **Lives on**: `Provenance` trait as a `const LABEL_POLICY: LabelPolicy`.
- **Variants**:
  - `Opaque`: Caller must specify mode.
  - `EntityNameDefault`: Default to `Short` mode (`Entity::value`).
  - `ExternalKeyDefault`: Default to `Full` mode (`Entity@provenance::value`).
  - `OpaqueByDefault`: Default to `None` mode (`value`).
- **Effect**: Applied **only** when `.labeled()` is called with no arguments.
- **Invariants**: Does **NOT** affect:
  - `Display` / `to_string()` (always canonical)
  - `Serialize` / `serde` (always canonical)
  - `sqlx` encoding/decoding (always canonical)
  - ID construction or equality/hashing.

### Example
```rust
// External<Stripe> uses LabelPolicy::ExternalKeyDefault
let id = StripeCustomerId::new("cus_123");

println!("{}", id.labeled()); 
// Output: Customer@external/stripe::cus_123

println!("{}", id); 
// Output: cus_123 (Canonical)
```

## 2. External ID Opaqueness

**Clarification**: External IDs are strictly opaque and must be preserved exactly as received from the source system.

### Principles
- **No Stripping**: Do not parse or strip prefixes like `"cus_"` from Stripe IDs or `"app-"` from Spark IDs.
- **Canonical = Opaque**: The entire value is the canonical ID.
- **No Boundary Helpers**: Avoid creating source-specific helpers like `from_stripe_id` that might hide parsing logic. Pass the raw string directly into `from_source()` or `new()`.

### Example (Stripe)
- **Received**: `"cus_L3H8Z6K9j2"`
- **Stored**: `"cus_L3H8Z6K9j2"`
- **Canonical String**: `"cus_L3H8Z6K9j2"`
- **API Call**: Use `"cus_L3H8Z6K9j2"`

This ensures that tagid-rs remains a transparent wrapper for external identifiers, preventing subtle bugs caused by mismatched assumptions about ID formats between systems.

---

## Impact on Implementation

- `src/id/provenance.rs`: Add `LabelPolicy` enum and `LABEL_POLICY` constant to `Provenance` trait.
- `src/id/labeled.rs`: Implement `.labeled()` to consult `S::LABEL_POLICY`.
- `src/id/sourced.rs`: Ensure `SourceLabeler` and `Sourced` do not attempt to parse or "clean" ID values.
