//! Provenance trait and core provenance types.
//!
//! # Overview
//!
//! Provenance defines the **origin** or **source** of an identifier. This module provides:
//!
//! - [`Provenance`] trait: Defines where an ID comes from and associated metadata
//! - [`LabelPolicy`] enum: Controls how IDs are displayed to humans
//! - 8 core provenance types covering all common scenarios
//!
//! For detailed guidance on choosing the right construction function based on
//! provenance, see [Provenance-Aware Construction Patterns](https://github.com/dmrolfs/tagid-rs/blob/main/ref/lessons/provenance-construction-patterns.md).
//!
//! # Philosophy
//!
//! Provenance is **semantic metadata about ID origin**, separate from:
//! - **Construction** (how you create the ID)
//! - **Storage** (how it's persisted)
//! - **Labeling** (human-readable context)
//!
//! The provenance is purely **type-level** and affects:
//! - Type safety (can't call `generate()` on External)
//! - Labeling defaults (what context to show humans)
//! - Semantic clarity (understanding where IDs come from)
//!
//! # Example
//!
//! ```ignore
//! use tagid::id::provenance::*;
//!
//! // Stripe customer ID (external, opaque)
//! pub struct Stripe;
//! pub type StripeCustomerId = TagId<Customer, External<Stripe>>;
//!
//! let id = StripeCustomerId::new("cus_L3H8Z6K9j2");
//! // Canonical (used everywhere: DB, serde, API)
//! assert_eq!(id.to_string(), "cus_L3H8Z6K9j2");
//!
//! // Labeled (human context, explicit opt-in)
//! assert_eq!(id.labeled(LabelMode::Full).to_string(),
//!            "Customer@ext/stripe::cus_L3H8Z6K9j2");
//! ```

#![allow(dead_code)]

use std::marker::PhantomData;

/// Defines the provenance (origin) of an identifier.
///
/// Provenance encodes *where an identifier comes from* and what metadata
/// might be associated with it.
///
/// # Implementers
///
/// This trait should be implemented by provenance types to define:
/// - Where the ID originates (external system, generated internally, etc.)
/// - What labeling behavior is preferred for human output
/// - What structured metadata might be associated
///
/// # Key Design
///
/// - **Provenance is type-level**: The provenance is determined at compile time via type parameters
/// - **Canonical ID is label-free**: Display, serde, storage never include labels
/// - **Labeling is explicit**: Use `.labeled()` for human-readable context
/// - **External IDs are opaque**: Preserve external values exactly (no parsing or transformation)
pub trait Provenance: 'static + Send + Sync + Default + Clone {
    /// Human-readable name for this provenance.
    ///
    /// Used in labeled output (via `.labeled()`) to show context.
    /// **Not** part of the canonical ID.
    ///
    /// Examples:
    /// - "external/stripe"
    /// - "generated"
    /// - "imported/spark"
    const NAME: &'static str;

    /// Canonical slug for wire format and display.
    ///
    /// Used in labeled output to show provenance compactly.
    /// Examples:
    /// - `"ext"` for External
    /// - `"gen"` for Generated
    /// - `"imp"` for Imported
    /// - `"der"` for Derived
    const SLUG: &'static str;

    /// Optional vendor/provider/strategy name for additional context.
    ///
    /// Used in full labeled output (e.g., `"ext/stripe"` instead of just `"ext"`).
    /// Examples:
    /// - `Some("stripe")` for External<Stripe>
    /// - `Some("spark")` for External<Spark>
    /// - `None` for Generated (no vendor)
    const VENDOR: Option<&'static str> = None;

    /// Preferred labeling behavior for this provenance.
    ///
    /// Controls the default output of `.labeled()` when called with no arguments.
    /// **Not** used for Display, serde, storage, or construction.
    ///
    /// Examples:
    /// - `ExternalKeyDefault` for External sources (show provenance in logs)
    /// - `EntityNameDefault` for Generated (show entity name in logs)
    /// - `OpaqueByDefault` for Temporary (hide by default)
    const LABEL_POLICY: LabelPolicy = LabelPolicy::Opaque;

    /// Optional provenance-specific metadata.
    ///
    /// Each provenance can define its own descriptor type for additional context.
    /// Kept separate from the canonical ID for clean separation of concerns.
    ///
    /// Examples:
    /// - `External<Stripe>` might have `StripeDescriptor` with API version
    /// - `Generated<Snowflake>` might have generation timestamp
    /// - `Imported<LegacyDb>` might have migration date
    type Descriptor: Default + Clone + Send + Sync + 'static;
}

/// A container combining a sourced ID with its provenance-specific metadata.
///
/// This implements **Axis 6 (Metadata Separation)** of the tagid design.
/// It keeps the core `Id` lean while allowing auxiliary data (like generation
/// timestamps or provider versions) to be associated with the identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WithProvenance<E: ?Sized, P: Provenance, ID> {
    /// The sourced identifier.
    pub id: crate::Id<crate::id::sourced::Sourced<E, P>, ID>,

    /// Provenance-specific metadata (e.g. generation time, source version).
    pub descriptor: P::Descriptor,
}

impl<E: ?Sized, P: Provenance, ID> WithProvenance<E, P, ID> {
    /// Creates a new container with the given ID and descriptor.
    pub fn new(
        id: crate::Id<crate::id::sourced::Sourced<E, P>, ID>,
        descriptor: P::Descriptor,
    ) -> Self {
        Self { id, descriptor }
    }
}

pub use crate::LabelPolicy;

/// External provenance: ID provided by an external system.
///
/// # Type Parameter
///
/// `Provider`: Optional marker for the source system (e.g., Stripe, Github, Spark).
/// When omitted, defaults to unit `()`.
///
/// # Opaqueness
///
/// External IDs are **completely opaque**. The entire value (including any prefix)
/// is the canonical ID and should be preserved exactly as received:
///
/// ```ignore
/// // Stripe provides "cus_L3H8Z6K9j2"
/// let id = StripeCustomerId::new("cus_L3H8Z6K9j2");
///
/// // Preserve exactly (no stripping of "cus_" prefix)
/// assert_eq!(id.to_string(), "cus_L3H8Z6K9j2");
/// db.insert(id.as_str());  // stores "cus_L3H8Z6K9j2"
/// ```
///
/// # Examples
///
/// ```ignore
/// pub struct Stripe;
/// pub struct Github;
/// pub struct Spark;
///
/// pub type StripeCustomerId = TagId<Customer, External<Stripe>>;
/// pub type GithubUserId = TagId<User, External<Github>>;
/// pub type SparkAppId = TagId<App, External<Spark>>;
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct External<Provider = ()>(PhantomData<Provider>);

// Generic implementation for External without vendor (catches External<()> and unknown types)
impl Provenance for External<()> {
    const NAME: &'static str = "external";
    const SLUG: &'static str = "ext";
    const VENDOR: Option<&'static str> = None;
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

// Specialized implementations for providers with vendor information
impl Provenance for External<providers::Stripe> {
    const NAME: &'static str = "external";
    const SLUG: &'static str = "ext";
    const VENDOR: Option<&'static str> = Some("stripe");
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

impl Provenance for External<providers::Github> {
    const NAME: &'static str = "external";
    const SLUG: &'static str = "ext";
    const VENDOR: Option<&'static str> = Some("github");
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

impl Provenance for External<providers::Spark> {
    const NAME: &'static str = "external";
    const SLUG: &'static str = "ext";
    const VENDOR: Option<&'static str> = Some("spark");
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

impl Provenance for External<providers::Okta> {
    const NAME: &'static str = "external";
    const SLUG: &'static str = "ext";
    const VENDOR: Option<&'static str> = Some("okta");
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

impl Provenance for External<providers::AwsS3> {
    const NAME: &'static str = "external";
    const SLUG: &'static str = "ext";
    const VENDOR: Option<&'static str> = Some("aws-s3");
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

impl Provenance for External<providers::GoogleCloud> {
    const NAME: &'static str = "external";
    const SLUG: &'static str = "ext";
    const VENDOR: Option<&'static str> = Some("google-cloud");
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

impl Provenance for External<providers::Iceberg> {
    const NAME: &'static str = "external";
    const SLUG: &'static str = "ext";
    const VENDOR: Option<&'static str> = Some("iceberg");
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

impl Provenance for External<providers::Nessie> {
    const NAME: &'static str = "external";
    const SLUG: &'static str = "ext";
    const VENDOR: Option<&'static str> = Some("nessie");
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

/// Generated provenance: ID created internally.
///
/// # Type Parameter
///
/// `Strategy`: Optional marker for generation strategy (e.g., UuidV7, CUID, Snowflake).
/// When omitted, defaults to unit `()`.
///
/// # Generation
///
/// Use `.generate()` to create new IDs with this provenance. Only `Generated`
/// sources can be generated (External, Imported, etc. are not generatable).
///
/// # Examples
///
/// ```ignore
/// pub struct UuidV7;
/// pub struct Snowflake;
///
/// pub type UserId = TagId<User, Generated<UuidV7>>;
/// pub type TenantId = TagId<Tenant, Generated<Snowflake>>;
///
/// let user_id = UserId::generate();
/// let tenant_id = TenantId::generate();
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Generated<Strategy = ()>(PhantomData<Strategy>);

impl<S> Provenance for Generated<S>
where
    S: 'static + Send + Sync + Default + Clone,
{
    const NAME: &'static str = "generated";
    const SLUG: &'static str = "gen";
    const VENDOR: Option<&'static str> = None;
    const LABEL_POLICY: LabelPolicy = LabelPolicy::EntityNameDefault;
    type Descriptor = ();
}

/// Imported provenance: ID brought in during migration or backfill.
///
/// # Type Parameter
///
/// `From`: Marker indicating the source of the import (e.g., LegacyDatabase, Spark).
///
/// # Use Case
///
/// When migrating data from an external system, imported IDs preserve the original
/// values while marking them as migration data for traceability.
///
/// # Examples
///
/// ```ignore
/// pub struct LegacyDatabase;
/// pub struct SparkMigration;
///
/// pub type LegacyUserId = TagId<User, Imported<LegacyDatabase>>;
/// pub type MigratedJobId = TagId<Job, Imported<SparkMigration>>;
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Imported<From = ()>(PhantomData<From>);

impl<F> Provenance for Imported<F>
where
    F: 'static + Send + Sync + Default + Clone,
{
    const NAME: &'static str = "imported";
    const SLUG: &'static str = "imp";
    const VENDOR: Option<&'static str> = None;
    const LABEL_POLICY: LabelPolicy = LabelPolicy::ExternalKeyDefault;
    type Descriptor = ();
}

/// Derived provenance: ID computed from other data.
///
/// # Type Parameter
///
/// `Method`: Marker indicating the derivation method (e.g., Slugify, Hash, Timestamp).
///
/// # Use Case
///
/// When an ID is computed from other fields (slugified names, content hashes, etc.),
/// derived provenance marks the ID as deterministically generated from source data.
///
/// # Examples
///
/// ```ignore
/// pub struct Slugify;
/// pub struct ContentHash;
///
/// pub type SlugId = TagId<Page, Derived<Slugify>>;
/// pub type ContentId = TagId<Content, Derived<ContentHash>>;
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Derived<Method = ()>(PhantomData<Method>);

impl<M> Provenance for Derived<M>
where
    M: 'static + Send + Sync + Default + Clone,
{
    const NAME: &'static str = "derived";
    const SLUG: &'static str = "der";
    const VENDOR: Option<&'static str> = None;
    const LABEL_POLICY: LabelPolicy = LabelPolicy::EntityNameDefault;
    type Descriptor = ();
}

/// Scoped provenance: Uniqueness depends on context.
///
/// # Type Parameters
///
/// - `Scope`: The scoping dimension (e.g., TenantId, OrganizationId)
/// - `Inner`: The inner provenance (e.g., Generated, External)
///
/// # Use Case
///
/// When IDs are only unique within a context (e.g., tenant IDs are unique per tenant,
/// not globally), scoped provenance marks this constraint.
///
/// # Examples
///
/// ```ignore
/// pub type TenantId = TagId<Tenant, Generated<Snowflake>>;
/// pub type TenantResourceId<R> =
///     TagId<R, Scoped<TenantId, Generated<UuidV7>>>;
///
/// // Each tenant has its own resource namespace
/// let resource_id = TenantResourceId::<File>::generate();
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Scoped<Scope = (), Inner = ()>(PhantomData<(Scope, Inner)>);

impl<Scope, Inner> Provenance for Scoped<Scope, Inner>
where
    Scope: 'static + Send + Sync + Default + Clone,
    Inner: Provenance,
{
    const NAME: &'static str = "scoped";
    const SLUG: &'static str = "sco";
    const VENDOR: Option<&'static str> = None;
    const LABEL_POLICY: LabelPolicy = Inner::LABEL_POLICY;
    type Descriptor = ();
}

/// Temporary provenance: Valid only short-term.
///
/// # Use Case
///
/// For ephemeral IDs that don't require persistence (optimistic IDs, session tokens,
/// temporary request IDs). Not suitable for long-term storage.
///
/// # Examples
///
/// ```ignore
/// pub type OptimisticId = TagId<Item, Temporary>;
/// pub type SessionToken = TagId<Session, Temporary>;
///
/// let optimistic = OptimisticId::new(uuid::Uuid::new_v4().to_string());
/// ```
#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Temporary;

impl Provenance for Temporary {
    const NAME: &'static str = "temporary";
    const SLUG: &'static str = "tmp";
    const VENDOR: Option<&'static str> = None;
    const LABEL_POLICY: LabelPolicy = LabelPolicy::OpaqueByDefault;
    type Descriptor = ();
}

/// ClientProvided provenance: User or client supplies the ID.
///
/// # Use Case
///
/// When clients provide their own IDs (e.g., idempotency keys in APIs,
/// user-supplied identifiers, bring-your-own-ID scenarios).
///
/// # Examples
///
/// ```ignore
/// pub type IdempotencyKey = TagId<Request, ClientProvided>;
/// pub type UserDefinedId = TagId<Resource, ClientProvided>;
///
/// let key = IdempotencyKey::new(client_provided_key);
/// ```
#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ClientProvided;

impl Provenance for ClientProvided {
    const NAME: &'static str = "client-provided";
    const SLUG: &'static str = "cli";
    const VENDOR: Option<&'static str> = None;
    const LABEL_POLICY: LabelPolicy = LabelPolicy::OpaqueByDefault;
    type Descriptor = ();
}

/// AliasOf provenance: Secondary identifier for the same entity.
///
/// # Type Parameter
///
/// `Canonical`: The canonical ID type this is an alias for.
///
/// # Use Case
///
/// When an entity has multiple identifiers (e.g., user email as alias for user_id,
/// URL slug as alias for page_id), alias provenance marks the secondary nature.
///
/// # Examples
///
/// ```ignore
/// pub type UserId = TagId<User, Generated<UuidV7>>;
/// pub type UserEmailAlias = TagId<User, AliasOf<UserId>>;
///
/// // Both refer to the same user, but email is secondary
/// let user_id = UserId::generate();
/// let email_alias = UserEmailAlias::new("user@example.com");
/// ```
#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AliasOf<Canonical = ()>(PhantomData<Canonical>);

impl<C> Provenance for AliasOf<C>
where
    C: 'static + Send + Sync + Default + Clone,
{
    const NAME: &'static str = "alias";
    const SLUG: &'static str = "als";
    const VENDOR: Option<&'static str> = None;
    const LABEL_POLICY: LabelPolicy = LabelPolicy::EntityNameDefault;
    type Descriptor = ();
}

// Marker traits for common providers and strategies
// These are zero-sized markers used to specialize External<> and Generated<>

/// Trait for vendor/provider markers to expose their vendor name.
///
/// Allows formatting slug/vendor display without runtime overhead.
/// Used primarily with `External<Provider>` to show vendor context in labeled output.
///
/// # Example
///
/// ```ignore
/// pub struct Stripe;
/// impl Vendor for Stripe {
///     const VENDOR: &'static str = "stripe";
/// }
///
/// // In labeled output:
/// // External<Stripe>::VENDOR = Some("stripe")
/// // Display: "ext/stripe" instead of just "ext"
/// ```
pub trait Vendor {
    /// The vendor/provider/strategy name (e.g., "stripe", "spark", "uuid-v7").
    const VENDOR: &'static str;
}

/// Provider markers for External<Provider>
pub mod providers {
    /// Stripe payment platform
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Stripe;

    /// GitHub
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Github;

    /// Apache Spark
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Spark;

    /// Okta identity management
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Okta;

    /// AWS S3
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct AwsS3;

    /// Google Cloud
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct GoogleCloud;

    /// Apache Iceberg table format
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Iceberg;

    /// Apache Nessie versioning
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Nessie;

    // Vendor trait implementations for providers
    impl super::Vendor for Stripe {
        const VENDOR: &'static str = "stripe";
    }

    impl super::Vendor for Github {
        const VENDOR: &'static str = "github";
    }

    impl super::Vendor for Spark {
        const VENDOR: &'static str = "spark";
    }

    impl super::Vendor for Okta {
        const VENDOR: &'static str = "okta";
    }

    impl super::Vendor for AwsS3 {
        const VENDOR: &'static str = "aws-s3";
    }

    impl super::Vendor for GoogleCloud {
        const VENDOR: &'static str = "google-cloud";
    }

    impl super::Vendor for Iceberg {
        const VENDOR: &'static str = "iceberg";
    }

    impl super::Vendor for Nessie {
        const VENDOR: &'static str = "nessie";
    }
}

/// Strategy markers for Generated<Strategy>
pub mod strategies {
    /// UUID version 4 (random)
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct UuidV4;

    /// UUID version 7 (time-based, sortable)
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct UuidV7;

    /// CUID unique identifier
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Cuid;

    /// CUID version 2
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Cuid2;

    /// Snowflake distributed ID algorithm
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Snowflake;

    /// Nanoid
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Nanoid;

    /// Hashids
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Hashids;

    /// ULID (Universally Unique Lexicographically Sortable Identifier)
    #[allow(dead_code)]
    #[derive(Debug, Default, Clone, Copy)]
    pub struct Ulid;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::sourced::Sourced;
    use crate::labeling::Labeling;
    use crate::{Entity, Id, IdGenerator, Label, MakeLabeling};
    use pretty_assertions::assert_eq;

    // --- Mock Entities for Extended Tests ---

    struct MockGenerator;
    impl IdGenerator for MockGenerator {
        type IdType = String;
        fn next_id_rep() -> Self::IdType {
            "mock-id".to_string()
        }
    }

    #[derive(Debug, PartialEq)]
    struct User;
    impl Label for User {
        type Labeler = MakeLabeling<Self>;
        fn labeler() -> Self::Labeler {
            MakeLabeling::default()
        }
    }
    impl Entity for User {
        type IdGen = MockGenerator;
    }

    struct Customer;
    impl Label for Customer {
        type Labeler = MakeLabeling<Self>;
        fn labeler() -> Self::Labeler {
            MakeLabeling::default()
        }
    }

    // Custom provenance for metadata testing
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    struct MetaProv;
    impl Provenance for MetaProv {
        const NAME: &'static str = "meta";
        const SLUG: &'static str = "met";
        type Descriptor = u32;
    }

    // --- Type Aliases for Extended Testing ---

    type UserId = Id<Sourced<User, Generated<strategies::UuidV7>>, String>;
    type StripeId = Id<Sourced<Customer, External<providers::Stripe>>, String>;
    type TempId = Id<Sourced<User, Temporary>, String>;

    #[test]
    fn test_provenance_trait_implemented() {
        // Verify all 8 core types implement Provenance
        fn assert_provenance<P: Provenance>() {}

        assert_provenance::<External>();
        assert_provenance::<Generated>();
        assert_provenance::<Imported>();
        assert_provenance::<Derived>();
        // Scoped requires inner to be Provenance, so test with Generated
        assert_provenance::<Scoped<(), Generated<()>>>();
        assert_provenance::<Temporary>();
        assert_provenance::<ClientProvided>();
        assert_provenance::<AliasOf>();
    }

    #[test]
    fn test_provenance_names() {
        assert_eq!(External::<()>::NAME, "external");
        assert_eq!(Generated::<()>::NAME, "generated");
        assert_eq!(Imported::<()>::NAME, "imported");
        assert_eq!(Derived::<()>::NAME, "derived");
        assert_eq!(Scoped::<(), Generated<()>>::NAME, "scoped");
        assert_eq!(Temporary::NAME, "temporary");
        assert_eq!(ClientProvided::NAME, "client-provided");
        assert_eq!(AliasOf::<()>::NAME, "alias");
    }

    #[test]
    fn test_all_core_provenance_slugs() {
        assert_eq!(External::<()>::SLUG, "ext");
        assert_eq!(Generated::<()>::SLUG, "gen");
        assert_eq!(Imported::<()>::SLUG, "imp");
        assert_eq!(Derived::<()>::SLUG, "der");
        assert_eq!(Scoped::<(), Generated<()>>::SLUG, "sco");
        assert_eq!(Temporary::SLUG, "tmp");
        assert_eq!(ClientProvided::SLUG, "cli");
        assert_eq!(AliasOf::<()>::SLUG, "als");
    }

    #[test]
    fn test_vendor_providers_expose_constants() {
        // Verify Vendor trait provides VENDOR constants
        assert_eq!(providers::Stripe::VENDOR, "stripe");
        assert_eq!(providers::Github::VENDOR, "github");
        assert_eq!(providers::Spark::VENDOR, "spark");
        assert_eq!(providers::Okta::VENDOR, "okta");
    }

    #[test]
    fn test_with_provenance_stores_both_fields() {
        let labeler = User::labeler();
        let id =
            Id::<Sourced<User, MetaProv>, String>::direct(labeler.label(), "test-id".to_string());
        let desc = 42u32;
        let with_prov = WithProvenance::new(id.clone(), desc);

        assert_eq!(with_prov.id, id);
        assert_eq!(with_prov.descriptor, desc);
    }

    #[test]
    fn test_with_provenance_equality() {
        let labeler = User::labeler();
        let id = Id::<Sourced<User, MetaProv>, String>::direct(labeler.label(), "id".to_string());
        let a = WithProvenance::new(id.clone(), 1u32);
        let b = WithProvenance::new(id.clone(), 1u32);
        let c = WithProvenance::new(id, 2u32);

        assert_eq!(a, b);
        assert_ne!(a, c); // Different descriptor
    }

    #[test]
    fn test_label_policies() {
        assert_eq!(
            External::<()>::LABEL_POLICY,
            LabelPolicy::ExternalKeyDefault
        );
        assert_eq!(
            Generated::<()>::LABEL_POLICY,
            LabelPolicy::EntityNameDefault
        );
        assert_eq!(
            Imported::<()>::LABEL_POLICY,
            LabelPolicy::ExternalKeyDefault
        );
        assert_eq!(Derived::<()>::LABEL_POLICY, LabelPolicy::EntityNameDefault);
        assert_eq!(Temporary::LABEL_POLICY, LabelPolicy::OpaqueByDefault);
        assert_eq!(ClientProvided::LABEL_POLICY, LabelPolicy::OpaqueByDefault);
    }

    #[test]
    fn test_canonical_id_is_opaque() {
        // Stripe ID example from design docs
        let id_str = "cus_L3H8Z6K9j2";
        let id = StripeId::from_canonical(id_str.to_string());

        // Canonical form must be exactly the input string
        assert_eq!(id.to_string(), id_str);
        assert_eq!(id.as_str(), id_str);
        assert_eq!(format!("{}", id), id_str);
    }

    #[test]
    fn test_labeling_output_by_policy() {
        // 1. External (ExternalKeyDefault -> Full)
        let stripe_id = StripeId::from_canonical("cus_123".to_string());
        // Default .labeled() should be Full: Entity@slug/vendor::value
        // Stripe provider includes vendor in slug display
        assert_eq!(
            stripe_id.labeled().to_string(),
            "Customer@ext/stripe::cus_123"
        );

        // 2. Generated (EntityNameDefault -> Short)
        let user_id = UserId::direct("User", "user-123".to_string());
        // Default .labeled() should be Short: Entity::value
        assert_eq!(user_id.labeled().to_string(), "User::user-123");

        // 3. Temporary (OpaqueByDefault -> None)
        let temp_id = TempId::from_canonical("temp-123".to_string());
        // Default .labeled() should be None: value
        assert_eq!(temp_id.labeled().to_string(), "temp-123");
    }

    #[test]
    fn test_explicit_labeling_override() {
        use crate::id::labeled::LabelMode;

        let stripe_id = StripeId::from_canonical("cus_123".to_string());

        // Force None
        assert_eq!(
            stripe_id.labeled().mode(LabelMode::None).to_string(),
            "cus_123"
        );

        // Force Short
        assert_eq!(
            stripe_id.labeled().mode(LabelMode::Short).to_string(),
            "Customer::cus_123"
        );
    }

    #[test]
    fn test_serialization_is_canonical() {
        let id = StripeId::from_canonical("cus_123".to_string());

        // Serialize
        let json = serde_json::to_string(&id).unwrap();
        assert_eq!(json, "\"cus_123\""); // Strictly the string value

        // Deserialize
        let deserialized: StripeId = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.to_string(), "cus_123");
    }

    #[test]
    fn test_sourced_entity_impl() {
        // Generated should implement Entity
        fn assert_entity<E: Entity>() {}
        assert_entity::<Sourced<User, Generated<strategies::UuidV7>>>();
    }

    #[test]
    fn test_providers_are_cloneable() {
        let stripe = providers::Stripe;
        let _stripe_clone = stripe;

        let github = providers::Github;
        let _github_clone = github;

        let spark = providers::Spark;
        let _spark_clone = spark;
    }

    #[test]
    fn test_strategies_are_cloneable() {
        let uuid_v7 = strategies::UuidV7;
        let _uuid_v7_clone = uuid_v7;

        let snowflake = strategies::Snowflake;
        let _snowflake_clone = snowflake;
    }
}
