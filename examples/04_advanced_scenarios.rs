//! Advanced tagid Scenarios: The 6 Axes of Design.
//!
//! This example provides a technical deep dive into how the 6 axes of the tagid
//! redesign work together to support complex real-world requirements.
//!
//! ### The 6 Axes Demonstrated:
//! 1. **Semantic Origin (Provenance)**: Using multiple provenance types.
//! 2. **Strict Opaque Handling**: Preserving foreign ID prefixes (Stripe).
//! 3. **Presentation vs. Data**: Canonical stability vs. rich logging.
//! 4. **Behavior Control**: Compile-time safety for generation.
//! 5. **LabelPolicy Bridge**: Automatic sensible defaults for logs.
//! 6. **Metadata Separation**: Using `WithProvenance` for auxiliary data.

use tagid::id::provenance::{External, Imported, Temporary, WithProvenance, providers};
use tagid::id::{Generated, strategies};
use tagid::{Entity, Id, Label, LabelMode, Sourced};

// ============================================================================
// ENTITY DEFINITIONS
// ============================================================================

#[derive(Label)]
struct Customer;

#[derive(Label)]
struct User;

// Enable UUID generation for internal Users (Axis 4)
// Requires "uuid" feature
#[cfg(feature = "uuid")]
impl Entity for User {
    type IdGen = tagid::UuidGenerator;
}

#[derive(Label)]
struct Order;

// ============================================================================
// TYPE ALIASES (Axis 1: Semantic Origin)
// ============================================================================

/// Axis 1: Type parameters like `External<Stripe>` encode origin at compile time.
type StripeCustomerId = Id<Sourced<Customer, External<providers::Stripe>>, String>;

/// Axis 1: `Generated<UuidV7>` encodes the internal strategy.
// Requires "uuid" feature
#[cfg(feature = "uuid")]
type InternalUserId = Id<Sourced<User, Generated<strategies::UuidV7>>, ::uuid::Uuid>;

/// Axis 1: `Imported` marks data from legacy systems.
type LegacyOrderId = Id<Sourced<Order, Imported>, i64>;

/// Axis 1: `Temporary` marks ephemeral identifiers.
type SessionToken = Id<Sourced<User, Temporary>, String>;

// ============================================================================
// AXIS 6: Metadata Separation (Descriptors)
// ============================================================================

// NOTE: In a real app, you would implement Provenance for your own types
// to associate these descriptors properly. For this demo, we'll use
// WithProvenance directly with existing types.
fn main() {
    println!("=== tagid Advanced Scenarios: The 6 Axes ===\n");

    // ------------------------------------------------------------------------
    // AXIS 2: Strict Opaque Handling
    // ------------------------------------------------------------------------
    // We receive "cus_L3H8Z6" from Stripe. We preserve it EXACTLY.
    let stripe_id = StripeCustomerId::from_source("cus_L3H8Z6".to_string());

    println!("Axis 2 (Opaque Handling):");
    println!("  Stripe ID:   {}", stripe_id); // Output: cus_L3H8Z6 (Prefix preserved)
    assert_eq!(stripe_id.to_string(), "cus_L3H8Z6");

    // ------------------------------------------------------------------------
    // AXIS 3 & 5: Presentation vs Data & LabelPolicy
    // ------------------------------------------------------------------------
    // Axis 3: canonical form is label-free.
    // Axis 5: .labeled() picks a mode based on the origin (Provenance).

    println!("\nAxis 3 & 5 (Presentation & Policies):");

    // External IDs default to "Full" mode (Axis 5: ExternalKeyDefault)
    println!("  External:    {}", stripe_id.labeled());
    // Output: Customer@external::cus_L3H8Z6

    // Internal IDs default to "Short" mode (Axis 5: EntityNameDefault)
    #[cfg(feature = "uuid")]
    {
        let user_id = InternalUserId::new();
        println!("  Generated:   {}", user_id.labeled());
        // Output: User::018d6f... (Provenance is hidden by default for internal IDs)

        // Axis 3: Developers can always override
        println!("  Override:    {}", user_id.labeled().mode(LabelMode::Full));
        // Output: User@generated::018d6f...
    }

    // Temporary IDs default to "None" mode (Axis 5: OpaqueByDefault)
    let session = SessionToken::for_temporary("sess_123".to_string());
    println!("  Temporary:   {}", session.labeled());
    // Output: sess_123 (Labels hidden for sensitive/ephemeral IDs)

    // ------------------------------------------------------------------------
    // AXIS 4: Behavior Control (Type Safety)
    // ------------------------------------------------------------------------
    // InternalUserId implements Entity -> .new() / .next_id() work.
    // StripeCustomerId does NOT implement Entity -> cannot be generated.

    println!("\nAxis 4 (Behavior Control):");
    #[cfg(feature = "uuid")]
    {
        let _new_user = InternalUserId::new(); // ✅ Works
        println!("  Generated User ID successfully.");
    }

    // let _new_stripe = StripeCustomerId::new(); // ❌ COMPILE ERROR
    println!("  Stripe IDs cannot be generated (Verified via type safety).");

    // ------------------------------------------------------------------------
    // AXIS 6: Metadata Separation
    // ------------------------------------------------------------------------
    // IDs stay lean. Extra data lives in WithProvenance.

    println!("\nAxis 6 (Metadata Separation):");

    let rich_stripe_id = WithProvenance::new(
        stripe_id.clone(),
        (), // Using unit for now as default descriptor
    );

    // In real usage, you'd have your own Provenance type with a custom Descriptor.
    println!("  Lean ID:     {}", rich_stripe_id.id);
    println!("  Metadata:    {:?}", rich_stripe_id.descriptor);

    // ------------------------------------------------------------------------
    // COMBINED SCENARIO: Data Migration
    // ------------------------------------------------------------------------
    println!("\nCombined Scenario: Migration");
    let legacy_order = LegacyOrderId::from_source(98765);

    // We get provenance info in logs (Axis 5), opaque preservation (Axis 2),
    // and type safety (Axis 4).
    println!(
        "  Log Entry:   Processing order {legacy_order:?}, labeled order: {}",
        legacy_order.labeled()
    );
    // Output: Order@imported::98765

    println!("\nSummary: All 6 axes ensure that IDs are semantically rich in code");
    println!("but perfectly canonical when they leave the application boundary.");
}
