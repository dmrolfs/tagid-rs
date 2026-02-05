use super::labeled::Labeled;
use crate::{DELIMITER, Entity, IdGenerator, Label, Labeling};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use smol_str::SmolStr;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

#[cfg(feature = "disintegrate")]
use disintegrate::{IdentifierType, IdentifierValue, IntoIdentifierValue};

/// A struct representing an identifier for an entity, and supports id labeling in logs and other
/// output.
///
/// `Id<T, ID>` associates an entity type `T` with an ID value of type `ID`.
///
/// For detailed guidance on choosing the right construction function based on
/// provenance, see [Provenance-Aware Construction Patterns](https://github.com/dmrolfs/tagid-rs/blob/main/ref/lessons/provenance-construction-patterns.md).
///
/// # Example
///
/// ```rust,ignore
/// use tagid::{Id, Entity, Label, MakeLabeling};
///
/// struct User;
///
/// impl Label for User {
///     type Labeler = MakeLabeling<Self>;
///
///     fn labeler() -> Self::Labeler {
///         MakeLabeling::default()
///     }
/// }
///
/// impl Entity for User {
///     type IdGen = tagid::UuidGenerator;
/// }
///
/// let user_id = User::next_id();
/// println!("User ID: {}", user_id);
/// ```
pub struct Id<T: ?Sized, ID> {
    /// The label associated with the entity type.
    pub label: SmolStr,

    /// The unique identifier value.
    pub id: ID,

    marker: PhantomData<T>,
}

/// Safety: `Id<T, ID>` is Send only if ID is Send.
///
/// T is never stored at runtime—it's purely a type-level marker (PhantomData).
/// Rust's automatic Send/Sync derivation doesn't understand this, so we explicitly
/// state the actual requirement: only ID determines thread safety, not T.
/// This is safe because T is never dereferenced or moved at runtime.
#[allow(unsafe_code)]
unsafe impl<T: ?Sized, ID: Send> Send for Id<T, ID> {}

/// Safety: `Id<T, ID>` is Sync only if ID is Sync.
///
/// Same reasoning as Send impl: T is type-level only (PhantomData).
/// T is never actually stored or accessed at runtime, so its thread safety doesn't matter.
/// Only ID's thread safety determines whether the overall type is Sync.
#[allow(unsafe_code)]
unsafe impl<T: ?Sized, ID: Sync> Sync for Id<T, ID> {}

impl<T: ?Sized, ID> AsRef<ID> for Id<T, ID> {
    /// Returns the inner string representation of the `ID`.
    ///
    /// This method provides access to the underlying id value as a `&ID`.
    fn as_ref(&self) -> &ID {
        &self.id
    }
}

impl<T: ?Sized + Label, ID> From<ID> for Id<T, ID> {
    fn from(id: ID) -> Self {
        Self::from_canonical(id)
    }
}

impl<E> Id<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: ?Sized + Entity + Label,
{
    /// Generates a new `Id` using the entity's [`IdGenerator`].
    pub fn new() -> Self {
        let labeler = <E as Label>::labeler();
        Self {
            label: SmolStr::new(labeler.label()),
            id: E::IdGen::next_id_rep(),
            marker: PhantomData,
        }
    }
}

impl<E> Default for Id<E, <<E as Entity>::IdGen as IdGenerator>::IdType>
where
    E: Entity + Label + ?Sized,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, ID> Id<T, ID>
where
    T: Label + ?Sized,
{
    /// Create an `Id` from a canonical ID value.
    ///
    /// The label is automatically retrieved from the entity type `T`.
    pub fn from_canonical(id: ID) -> Self {
        let labeler = <T as Label>::labeler();
        Self {
            label: SmolStr::new(labeler.label()),
            id,
            marker: PhantomData,
        }
    }

    #[deprecated(since = "1.1.0", note = "use from_canonical instead")]
    pub fn for_labeled(id: ID) -> Self {
        Self::from_canonical(id)
    }

    // ========== EXTERNAL / IMPORTED ==========

    /// Create an ID from an external system or legacy source.
    ///
    /// Used for IDs that originate from external systems (APIs, databases,
    /// data imports, migrations) where the value is **opaque** and should
    /// be preserved exactly as received.
    ///
    /// # Semantics
    /// - The ID comes **FROM** an external source
    /// - The value is **opaque** — don't parse, transform, or validate it
    /// - Used with `External<Provider>` or `Imported<From>` provenance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::External;
    ///
    /// pub struct Stripe;
    /// pub type StripeCustomerId = Id<Sourced<Customer, External<Stripe>>, String>;
    ///
    /// let id = StripeCustomerId::from_source("cus_L3H8Z6K9j2");
    /// assert_eq!(id.to_string(), "cus_L3H8Z6K9j2");
    /// ```
    ///
    /// # See Also
    /// - `derived_from()` — for IDs computed from data
    /// - `from_client()` — for user-provided IDs
    /// - For `External<Provider>` provenance
    pub fn from_source(id: ID) -> Self {
        Self::from_canonical(id)
    }

    // ========== DERIVED ==========

    /// Create an ID derived deterministically from source data.
    ///
    /// Used for IDs that are **computed** from other fields where the same
    /// source always produces the same ID (unlike `generate()` which is random).
    ///
    /// # Semantics
    /// - The ID is **derived FROM** source data
    /// - Creation is **deterministic** — same input → same output
    /// - Different from `Generated` (which is random)
    /// - Different from `External` (which is opaque)
    /// - Used with `Derived<Method>` provenance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::Derived;
    ///
    /// pub struct Slugify;
    /// pub type PageSlugId = Id<Sourced<Page, Derived<Slugify>>, String>;
    ///
    /// let slug = PageSlugId::derived_from(slugify("My Page Title"));
    /// assert_eq!(slug.to_string(), "my-page-title");
    /// ```
    ///
    /// # See Also
    /// - `from_source()` — for external/legacy IDs
    /// - `generate()` — for random IDs
    /// - For `Derived<Method>` provenance
    pub fn derived_from(id: ID) -> Self {
        Self::from_canonical(id)
    }

    // ========== CLIENT PROVIDED ==========

    /// Create an ID provided by a user or client.
    ///
    /// Used for IDs that are **supplied by the client or user** rather than
    /// generated or imported by the system.
    ///
    /// # Semantics
    /// - The ID comes **FROM** the user/client
    /// - Different from `External` which comes from a **system** or **API**
    /// - Used with `ClientProvided` provenance
    /// - Often validated but not transformed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::ClientProvided;
    ///
    /// pub type IdempotencyKey = Id<Sourced<Request, ClientProvided>, String>;
    ///
    /// let key = IdempotencyKey::from_client(client_provided_key);
    /// assert_eq!(key.to_string(), client_provided_key);
    /// ```
    ///
    /// # See Also
    /// - `from_source()` — for external system IDs
    /// - For `ClientProvided` provenance
    pub fn from_client(id: ID) -> Self {
        Self::from_canonical(id)
    }

    // ========== SCOPED ==========

    /// Create a context-scoped ID (unique within a scope, not globally).
    ///
    /// Used for IDs that are **unique within a context** (tenant, organization,
    /// workspace) but not globally unique.
    ///
    /// # Semantics
    /// - The ID is **scoped TO** a context
    /// - Uniqueness is **context-relative**, not global
    /// - Used with `Scoped<Scope, Inner>` provenance
    /// - The inner provenance determines allowed operations
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::{Scoped, Generated};
    ///
    /// pub type TenantId = Id<Sourced<Tenant, Generated<UuidV7>>, String>;
    /// pub type TenantResourceId =
    ///     Id<Sourced<Resource, Scoped<TenantId, Generated<UuidV7>>>, String>;
    ///
    /// let resource = TenantResourceId::for_scope(tenant_id);
    /// // Or use the inner provenance's method:
    /// let resource = TenantResourceId::generate();  // Generates, still scoped
    /// ```
    ///
    /// # See Also
    /// - Inner provenance determines `.generate()`, `.from_source()`, etc.
    /// - For `Scoped<Scope, Inner>` provenance
    pub fn for_scope(id: ID) -> Self {
        Self::from_canonical(id)
    }

    // ========== ALIAS ==========

    /// Create a secondary identifier (alias) for an entity.
    ///
    /// Used for IDs that are **aliases** or **secondary identifiers** for the
    /// same entity where the primary ID is defined elsewhere.
    ///
    /// # Semantics
    /// - This is a **secondary ID**, not the primary/canonical ID
    /// - Used with `AliasOf<Canonical>` provenance
    /// - Often combined with `Derived` (e.g., email is derived)
    /// - Enables alternative lookup but primary ID is canonical
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::AliasOf;
    ///
    /// pub type UserId = Id<Sourced<User, Generated<UuidV7>>, String>;
    /// pub type UserEmailAlias = Id<Sourced<User, AliasOf<UserId>>, String>;
    ///
    /// let email_alias = UserEmailAlias::alias_for("user@example.com");
    /// assert_eq!(email_alias.to_string(), "user@example.com");
    /// ```
    ///
    /// # See Also
    /// - Often paired with `Derived` for computed aliases
    /// - For `AliasOf<Canonical>` provenance
    pub fn alias_for(id: ID) -> Self {
        Self::from_canonical(id)
    }

    // ========== TEMPORARY ==========

    /// Create a temporary ID (valid only short-term, not for persistence).
    ///
    /// Used for **ephemeral IDs** that should not be persisted to databases
    /// or caches.
    ///
    /// # Semantics
    /// - The ID is **FOR TEMPORARY use only**
    /// - Should **never be persisted** to databases or caches
    /// - Used with `Temporary` provenance
    /// - Examples: optimistic IDs, session tokens, request IDs
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::Temporary;
    ///
    /// pub type OptimisticId = Id<Sourced<Item, Temporary>, String>;
    ///
    /// let id = OptimisticId::for_temporary(uuid::Uuid::new_v4().to_string());
    /// // ⚠️  Important: never persist this to the database!
    /// ```
    ///
    /// # See Also
    /// - Different from `Generated` which **should** be persisted
    /// - For `Temporary` provenance
    pub fn for_temporary(id: ID) -> Self {
        Self::from_canonical(id)
    }

    // ========== GENERATED (Testing/Fixtures) ==========

    /// Create a Generated ID for testing purposes.
    ///
    /// Used in fixtures and tests where you need to construct IDs explicitly
    /// without actually generating them (which is done via `generate()` or
    /// the `Entity` trait in production).
    ///
    /// # Semantics
    /// - Used **only for tests and fixtures**, not production
    /// - For production ID generation, use `generate()` instead
    /// - Used with `Generated<Strategy>` provenance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use tagid::id::provenance::Generated;
    ///
    /// pub type UserId = Id<Sourced<User, Generated<UuidV7>>, String>;
    ///
    /// #[test]
    /// fn test_user_creation() {
    ///     let user_id = UserId::for_test("user-123".to_string());
    ///     assert_eq!(user_id.to_string(), "user-123");
    /// }
    ///
    /// // For production:
    /// let user_id = UserId::generate();  // ← use this instead
    /// ```
    ///
    /// # See Also
    /// - `generate()` — for actual ID generation (production)
    /// - For `Generated<Strategy>` provenance
    pub fn for_test(id: ID) -> Self {
        Self::from_canonical(id)
    }
}

impl<T: ?Sized, ID> Id<T, ID> {
    /// Creates an `Id` with a specific label and ID value.
    pub fn direct(label: impl AsRef<str>, id: ID) -> Self {
        Self {
            label: SmolStr::new(label.as_ref()),
            id,
            marker: PhantomData,
        }
    }

    /// Consumes the `Id<T, ID>`, returning the inner `ID` value.
    pub fn into_inner(self) -> ID {
        self.id
    }
}

impl<T: ?Sized, ID: AsRef<str>> Id<T, ID> {
    /// Returns the canonical ID value as a string slice.
    pub fn as_str(&self) -> &str {
        self.id.as_ref()
    }
}

impl<T: ?Sized, ID: Clone> Id<T, ID> {
    /// Converts the `Id` to another entity type while retaining the same ID value.
    pub fn relabel<B: Label>(&self) -> Id<B, ID> {
        let b_labeler = B::labeler();
        Id {
            label: SmolStr::new(b_labeler.label()),
            id: self.id.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized, ID: Clone> Clone for Id<T, ID> {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            id: self.id.clone(),
            marker: PhantomData,
        }
    }
}

impl<T: ?Sized, ID: fmt::Debug> fmt::Debug for Id<T, ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            f.debug_struct("Id")
                .field("label", &self.label)
                .field("id", &self.id)
                .finish()
        } else if self.label.is_empty() {
            write!(f, "{:?}", self.id)
        } else {
            write!(f, "{}{DELIMITER}{:?}", self.label, self.id)
        }
    }
}

impl<T: ?Sized, ID: fmt::Display> fmt::Display for Id<T, ID> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<T: ?Sized, ID: PartialEq> PartialEq for Id<T, ID> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: ?Sized, ID: Eq> Eq for Id<T, ID> {}

impl<T: ?Sized, ID: Ord> Ord for Id<T, ID> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<T: ?Sized, ID: PartialOrd> PartialOrd for Id<T, ID> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<T: ?Sized, ID: Hash> Hash for Id<T, ID> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl<T: ?Sized, ID: Serialize> Serialize for Id<T, ID> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id.serialize(serializer)
    }
}

impl<'de, T, ID> Deserialize<'de> for Id<T, ID>
where
    T: Label + ?Sized,
    ID: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rep = ID::deserialize(deserializer)?;
        let labeler = <T as Label>::labeler();
        Ok(Self::direct(labeler.label(), rep))
    }
}

impl<T: ?Sized + Label, ID> Id<T, ID> {
    /// Returns a wrapper that formats this ID for human-readable output.
    ///
    /// The default formatting mode is determined by `T::POLICY`.
    /// You can override the mode using the builder pattern:
    ///
    /// ```ignore
    /// println!("{}", id.labeled()); // Uses default policy
    /// println!("{}", id.labeled().mode(LabelMode::Full)); // Forces Full mode
    /// ```
    pub fn labeled(&self) -> Labeled<'_, T, ID> {
        Labeled::new(self, T::POLICY.into())
    }
}

#[cfg(feature = "sqlx")]
impl<'q, T, ID, DB> sqlx::Decode<'q, DB> for Id<T, ID>
where
    T: Label,
    ID: sqlx::Decode<'q, DB>,
    DB: sqlx::Database,
{
    fn decode(
        value: <DB as sqlx::Database>::ValueRef<'q>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <ID as sqlx::Decode<DB>>::decode(value)?;
        Ok(Self::from_canonical(value))
    }
}

#[cfg(feature = "sqlx")]
impl<'q, T, ID, DB> sqlx::Encode<'q, DB> for Id<T, ID>
where
    ID: sqlx::Encode<'q, DB>,
    DB: sqlx::Database,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <ID as sqlx::Encode<DB>>::encode_by_ref(&self.id, buf)
    }
}

#[cfg(feature = "sqlx")]
impl<T, ID, DB> sqlx::Type<DB> for Id<T, ID>
where
    ID: sqlx::Type<DB>,
    DB: sqlx::Database,
{
    fn type_info() -> DB::TypeInfo {
        <ID as sqlx::Type<DB>>::type_info()
    }
}

#[cfg(feature = "disintegrate")]
impl<T, ID: fmt::Display> IntoIdentifierValue for Id<T, ID> {
    const TYPE: IdentifierType = IdentifierType::String;

    fn into_identifier_value(self) -> IdentifierValue {
        IdentifierValue::String(self.id.to_string())
    }
}
