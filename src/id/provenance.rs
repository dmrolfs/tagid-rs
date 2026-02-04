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

/// Controls how IDs are displayed to humans (via `.labeled()`).
///
/// # Scope
///
/// This enum affects **only** human-facing output. It does **NOT** affect:
/// - Display/to_string() → always canonical
/// - Serialize/serde → always canonical
/// - Database storage → always canonical
/// - ID construction → no complexity added
/// - Equality/hashing → based on canonical ID only
///
/// # Usage
///
/// Specified on the `Provenance` trait as `LABEL_POLICY` constant.
/// Controls the default behavior of `.labeled()` when called with no arguments.
///
/// # Examples
///
/// ```ignore
/// // External<Stripe> has LabelPolicy::ExternalKeyDefault
/// let stripe_id = StripeCustomerId::new("cus_L3H8Z6K9j2");
/// stripe_id.labeled()  // Shows provenance by default
/// // Output: Customer@ext/stripe::cus_L3H8Z6K9j2
///
/// // Generated<UuidV7> has LabelPolicy::EntityNameDefault
/// let user_id = UserId::generate();
/// user_id.labeled()  // Shows entity name by default
/// // Output: User::550e8400-...
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelPolicy {
    /// No preference; caller controls labeling via `.labeled(mode)`.
    ///
    /// Used when the provenance doesn't have a strong preference.
    /// Caller must explicitly specify mode or use `.labeled(LabelMode::Full)`.
    Opaque,

    /// Default to showing entity type name in human output.
    ///
    /// Used for internal origins (Generated, Derived) where the entity type
    /// provides sufficient context without showing provenance.
    ///
    /// Default output: `Entity::value`
    /// With explicit Full: `Entity@provenance::value`
    EntityNameDefault,

    /// Default to showing entity and provenance in human output.
    ///
    /// Used for external origins (External, Imported) where knowing the source
    /// is important for understanding the ID.
    ///
    /// Default output: `Entity@provenance::value`
    ExternalKeyDefault,

    /// Hide by default; only show on explicit request.
    ///
    /// Used for sensitive sources (Temporary, ClientProvided) where the ID
    /// itself might be sensitive and context shouldn't leak in logs.
    ///
    /// Default output: `value` (canonical only)
    /// With explicit Full: `Entity@provenance::value`
    OpaqueByDefault,
}

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
#[derive(Debug, Default, Clone, Copy)]
pub struct External<Provider = ()>(PhantomData<Provider>);

impl<P> Provenance for External<P>
where
    P: 'static + Send + Sync + Default + Clone,
{
    const NAME: &'static str = "external";
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
#[derive(Debug, Default, Clone, Copy)]
pub struct Generated<Strategy = ()>(PhantomData<Strategy>);

impl<S> Provenance for Generated<S>
where
    S: 'static + Send + Sync + Default + Clone,
{
    const NAME: &'static str = "generated";
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
#[derive(Debug, Default, Clone, Copy)]
pub struct Imported<From = ()>(PhantomData<From>);

impl<F> Provenance for Imported<F>
where
    F: 'static + Send + Sync + Default + Clone,
{
    const NAME: &'static str = "imported";
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
#[derive(Debug, Default, Clone, Copy)]
pub struct Derived<Method = ()>(PhantomData<Method>);

impl<M> Provenance for Derived<M>
where
    M: 'static + Send + Sync + Default + Clone,
{
    const NAME: &'static str = "derived";
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
#[derive(Debug, Default, Clone, Copy)]
pub struct Scoped<Scope = (), Inner = ()>(PhantomData<(Scope, Inner)>);

impl<Scope, Inner> Provenance for Scoped<Scope, Inner>
where
    Scope: 'static + Send + Sync + Default + Clone,
    Inner: Provenance,
{
    const NAME: &'static str = "scoped";
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
#[derive(Debug, Default, Clone, Copy)]
pub struct AliasOf<Canonical = ()>(PhantomData<Canonical>);

impl<C> Provenance for AliasOf<C>
where
    C: 'static + Send + Sync + Default + Clone,
{
    const NAME: &'static str = "alias";
    const LABEL_POLICY: LabelPolicy = LabelPolicy::EntityNameDefault;
    type Descriptor = ();
}

// Marker traits for common providers and strategies
// These are zero-sized markers used to specialize External<> and Generated<>

/// Provider markers for External<Provider>
pub mod providers {
    //! Common provider markers for use with External<Provider>.

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
}

/// Strategy markers for Generated<Strategy>
pub mod strategies {
    //! Common strategy markers for use with Generated<Strategy>.

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
    use pretty_assertions::assert_eq;

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
    fn test_label_policies() {
        assert_eq!(External::<()>::LABEL_POLICY, LabelPolicy::ExternalKeyDefault);
        assert_eq!(Generated::<()>::LABEL_POLICY, LabelPolicy::EntityNameDefault);
        assert_eq!(Imported::<()>::LABEL_POLICY, LabelPolicy::ExternalKeyDefault);
        assert_eq!(Derived::<()>::LABEL_POLICY, LabelPolicy::EntityNameDefault);
        assert_eq!(Temporary::LABEL_POLICY, LabelPolicy::OpaqueByDefault);
        assert_eq!(
            ClientProvided::LABEL_POLICY,
            LabelPolicy::OpaqueByDefault
        );
    }

    #[test]
    fn test_providers_are_cloneable() {
        let stripe = providers::Stripe;
        let _stripe_clone = stripe.clone();

        let github = providers::Github;
        let _github_clone = github.clone();

        let spark = providers::Spark;
        let _spark_clone = spark.clone();
    }

    #[test]
    fn test_strategies_are_cloneable() {
        let uuid_v7 = strategies::UuidV7;
        let _uuid_v7_clone = uuid_v7.clone();

        let snowflake = strategies::Snowflake;
        let _snowflake_clone = snowflake.clone();
    }
}
