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

### spark-turtle Integration (Phase 3)

Documents ported from spark-turtle project to explain tagid-rs enhancements needed for Spark ID management:

- **`spark-turtle__tagid-final-design.md`**  
  Complete specification of Provenance trait, Sourced wrapper, type parameters, labeling, and canonical ID rules. Answer to "why these changes and how do they work?"

- **`spark-turtle__tagid-implementation-checklist.md`**  
  Step-by-step guide with expected outputs for implementing all features. Answer to "how do I actually code this?"

- **`spark-turtle__tagid-quick-reference.md`**  
  Cheat sheet with copy-paste code patterns, aliases, and examples. Answer to "show me the pattern"

- **`spark-turtle__tagid-examples.md`** (optional)  
  Real-world examples: spark IDs (External<Spark>), turtle-generated IDs (Generated<UuidV7>), icehouse multi-tenant patterns.

### Architecture & Design History

- **`spark-turtle__adr-tagid-provenance.md`** (optional)  
  Architecture Decision Record: why "Provenance" terminology, why type parameters, why optional features.

## How to Use These Docs

### For Implementers (coding tagid-rs features)

1. Start with `spark-turtle__tagid-final-design.md` (understand the "why")
2. Use `spark-turtle__tagid-implementation-checklist.md` (step-by-step)
3. Reference `spark-turtle__tagid-quick-reference.md` while coding
4. Check `spark-turtle__tagid-examples.md` for patterns

### For Integration Teams (using tagid-rs in your project)

1. Read the relevant spark-turtle example for your use case
2. Copy patterns from `spark-turtle__tagid-quick-reference.md`
3. Verify your use case matches documented contracts
4. Report incompatibilities via issue/PR

### For Reviewers

Check against:
- Contracts in `spark-turtle__tagid-final-design.md` (Serialization, Type Safety sections)
- Implementation steps in `spark-turtle__tagid-implementation-checklist.md`
- Example patterns in `spark-turtle__tagid-quick-reference.md`

## Metadata

Each document includes a header block with:
- **Source**: Original project and path
- **Commit**: Git commit hash it was copied from (for traceability)
- **Date**: When it was ported
- **Notes**: Any edits or adaptations made

Example:
```
---
Source: spark-turtle/history/phase3-ids-final-design-v2.md
Commit: abc123def456
Date: 2026-02-03
Notes: Removed spark-turtle-specific examples; kept core spec
---
```

## Maintenance

### When to Update

Update when:
- The corresponding document in spark-turtle changes significantly
- New projects adopt similar patterns
- Contracts/APIs change

### How to Update

1. Note the commit hash from the source project
2. Merge in changes (resolve conflicts as needed)
3. Update the source metadata at top of file
4. Consider adding a "Changes from source" section

### When to NOT Update

Don't update these documents for:
- Local optimization to tagid-rs
- Features not used by the documented use case
- typo/grammar fixes in rustdoc (update in README.md/code instead)

---

## Related

- **spark-turtle DEPENDENCIES.md**: Reciprocal tracking of tagid-rs dependency
- **spark-turtle/history/**: Full planning documents and decision logs
- **../README.md**: Authoritative API documentation
- **../src/**: Implementation source

---

**Last Updated**: 2026-02-03  
**Curator**: Cross-project documentation team
