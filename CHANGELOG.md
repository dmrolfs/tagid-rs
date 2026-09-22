# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.1] - 2026-09-22

### Fixed

- **`Ulid`'s own `sqlx::Type`/`Encode`/`Decode` impls disagreed with each other.** `Type::type_info()`
  declared the column type as Postgres `uuid` (via `<Uuid as Type<DB>>::type_info()`), but
  `Encode`/`Decode` actually transmitted the ULID's own base32 canonical text
  (`.to_string()`/`Self::from_string()`) -- a real mismatch, not a documentation gap: binding a
  `Ulid`/`UlidId<T>` value against a genuine Postgres column failed no matter which type the
  column actually was, because `Type` and `Encode`/`Decode` disagreed with each other about what
  they were sending. Found downstream (HERE AI Hub LLM Gateway, task `lgw-3d6`) while binding a
  `UlidId` against a real column and confirmed by reading this crate's own source directly. That
  project's own schema now declares these columns `TEXT` and binds via `.to_string()` explicitly,
  as a workaround for exactly this bug -- a workaround this fix may let it eventually remove.

  **Fixed by making `Type`, `Encode`, and `Decode` agree on `text`, not `uuid`.** A `uuid`-backed
  fix was drafted and considered first -- reusing Postgres's fixed-width 16-byte `uuid` slot is
  lossless and order-preserving for a ULID's own 128 bits -- but was rejected before ever being
  published: the result is not a real RFC 4122 UUID (the version/variant bits are never set, it
  is simply the ULID's raw bits reinterpreted), and it discards the human-readable, lexically
  sortable canonical base32 string that is this type's entire reason to exist over a plain random
  UUID -- `psql`/tooling would show a hex-dashed UUID rendering instead of the ULID's own
  timestamp-prefixed text. `Type`, `Encode`, and `Decode` all route through `String` instead: the
  column type is `text`, and the wire value is `Ulid`'s own `Display`/`FromStr` canonical form.
  This matches how `CuidId<T>` (a plain `Id<T, String>`) already works, and needs no bespoke
  conversion type -- `Ulid`'s own `Display`/`Self::from_string` already existed and are exercised
  directly.

  **Breaking, for the same reason the underlying bug was never usable in the first place**: any
  consumer relying on `Ulid`/`UlidId<T>`'s own `sqlx` impls to bind against a genuine `uuid`
  column will need to widen that column to `text`/`varchar`. A consumer already binding against a
  `text`/`varchar` column, or sourcing the ULID's own text form explicitly at the call site
  (`.to_string()`/`.parse()`, sidestepping these impls entirely -- the only way anyone could have
  used this type successfully before this fix, since `Type` and `Encode`/`Decode` never agreed on
  anything), is unaffected.

  New tests: `Ulid -> String -> Ulid` round-trips losslessly (a random and a fixed value), the
  canonical base32 string's own lexical order preserves `Ulid`'s own chronological `Ord`
  (sortability is this type's whole reason to exist over a plain random UUID), and
  `<Ulid as Type<Postgres>>::type_info()` now equals `<String as Type<Postgres>>::type_info()` --
  the property that was violated before this fix.

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
