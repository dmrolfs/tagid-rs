//! External Opaque IDs with Provenance.
//!
//! This example demonstrates how to handle IDs from external systems (like Stripe or Spark)
//! as opaque strings, preserving their prefixes and metadata.

use tagid::id::provenance::{External, providers};
use tagid::{Id, Label, MakeLabeling, Sourced};

// Define an entity type for a Stripe customer.
struct Customer;

impl Label for Customer {
    type Labeler = MakeLabeling<Self>;
    fn labeler() -> Self::Labeler {
        MakeLabeling::default()
    }
}

// Define a type alias for a Sourced ID with External provenance.
type StripeCustomerId = Id<Sourced<Customer, External<providers::Stripe>>, String>;

fn main() {
    // Received from Stripe API: "cus_L3H8Z6K9j2"
    let raw_id = "cus_L3H8Z6K9j2";

    // Create a typed ID from the external source.
    let id = StripeCustomerId::from_source(raw_id.to_string());

    // 1. PRINCIPLE: OPAQUENESS
    // We preserve the prefix "cus_" exactly as received.
    assert_eq!(id.to_string(), "cus_L3H8Z6K9j2");
    println!("Canonical (Opaque): {}", id);

    // 2. PRINCIPLE: LABEL POLICY
    // External sources default to Full labeling: "Entity@provenance::value"
    println!("Labeled (default):  {}", id.labeled());

    // 3. API USAGE
    // When sending back to Stripe, use .as_str() to get the canonical form.
    println!("Sending to API:     {}", id.as_str());
}
