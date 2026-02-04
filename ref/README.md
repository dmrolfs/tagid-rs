# tagid-rs Reference Materials

This folder contains reference documentation copied from other projects using tagid-rs.

## Purpose

These documents provide context, specification, and examples for how tagid-rs enhancements are used by downstream projects (e.g., spark-turtle, icehouse). They are **NOT** authoritative API documentation — that lives in `../README.md` and rustdoc.

Use these for:
- Understanding design rationale for new features
- Seeing real-world usage patterns
- Verifying compatibility across projects
- Troubleshooting integration issues

## Contents

### tagid Redesign (Phase 3)

The definitive specification for the Provenance and Sourced enhancement effort:

- **`tagid-redesign.md`**  
  Consolidated specification of Provenance trait, Sourced wrapper, type parameters, labeling, and canonical ID rules. Includes the Feb 3 design clarifications regarding LabelPolicy scope and Opaque ID handling.

### spark-turtle Integration Supporting Docs

Reference documents ported from spark-turtle project:

- **`spark-turtle__tagid-implementation-checklist.md`**  
  Step-by-step guide with expected outputs for implementing all features.

- **`spark-turtle__tagid-quick-reference.md`**  
  Cheat sheet with copy-paste code patterns, aliases, and examples.

- **`spark-turtle__tagid-examples.md`** (optional)  
  Real-world examples: spark IDs (External<Spark>), turtle-generated IDs (Generated<UuidV7>), icehouse multi-tenant patterns.

### Architecture & Design History

- **`spark-turtle__adr-tagid-provenance.md`** (optional)  
  Architecture Decision Record: why "Provenance" terminology, why type parameters, why optional features.

## How to Use These Docs

### For Implementers

1. Start with `tagid-redesign.md` (understand the "why")
2. Use `spark-turtle__tagid-implementation-checklist.md` (step-by-step)
3. Reference `spark-turtle__tagid-quick-reference.md` while coding

### For Integration Teams

1. Read the relevant example for your use case
2. Copy patterns from `spark-turtle__tagid-quick-reference.md`
3. Verify your use case matches documented contracts

## Metadata

Each document includes a header block with traceability information.

---

**Last Updated**: 2026-02-03  
**Curator**: Cross-project documentation team