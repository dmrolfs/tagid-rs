# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-02-04

### Added

- **Provenance-Aware Construction Functions**: Added semantic methods to `Id<T, ID>` that clarify ID origin based on provenance type:
  - `from_source()` — for External/Imported IDs from systems
  - `derived_from()` — for Derived IDs computed from data
  - `from_client()` — for ClientProvided IDs from users
  - `for_scope()` — for Scoped IDs with context
  - `alias_for()` — for AliasOf secondary IDs
  - `for_temporary()` — for Temporary ephemeral IDs
  - `for_test()` — for Generated IDs in tests

  These are **zero-cost aliases** to `from_canonical()` (formerly `for_labeled()`) and serve as **semantic guidance** for choosing the right construction method based on provenance type.
  All methods are **backward compatible**.

- **Renamed `for_labeled()` to `from_canonical()`**: The generic constructor was renamed to better reflect that it takes a canonical ID value as an argument. `for_labeled()` remains as a deprecated alias.

## [1.0.0] - 2026-02-04

### Added
- **Semantic ID System**: Completely redesigned ID system focusing on origin and meaning.
- **Provenance Trait**: Defined a formal `Provenance` trait for tracking the origin of identifiers.
- **Core Provenance Types**: Implemented 8 core types: `External`, `Generated`, `Imported`, `Derived`, `Scoped`, `Temporary`, `ClientProvided`, and `AliasOf`.
- **Sourced Wrapper**: Added `Sourced<E, S>` wrapper for associating entities with provenance at the type level with zero runtime overhead.
- **Labeling Policy**: Integrated `LabelPolicy` to control human-readable presentation modes.
- **Labeled Presentation**: Introduced `Labeled<T, ID>` wrapper with `LabelMode` (None, Short, Full) for explicit opt-in human-readable formatting.
- **Typed Provenance**: Support for specialized providers and strategies (e.g., `External<Stripe>`, `Generated<UuidV7>`).
- **Comprehensive Documentation**: New `ref/` folder with detailed design specs and `examples/` for common use cases.
- **Enhanced Test Suite**: significantly expanded test coverage for core semantics, serialization, and type-level invariants.

### Changed
- Refactored `Id` to support the new labeling and provenance system.
- Standardized feature flags (e.g., `with-uuid` -> `uuid`).
- Improved `Debug` and `Display` implementations for `Id`.

### Removed
- Deprecated legacy source types in favor of the new `Provenance` system.
