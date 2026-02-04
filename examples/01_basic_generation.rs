//! Basic ID Generation with Provenance.
//!
//! This example shows how to use `Generated` provenance for IDs created
//! internally by your application.

use tagid::{Label, MakeLabeling};

// Define an entity type.
#[allow(dead_code)]
struct User;

impl Label for User {
    type Labeler = MakeLabeling<Self>;
    fn labeler() -> Self::Labeler {
        MakeLabeling::default()
    }
}

// Implement Entity to enable ID generation.
// Note: Requires "uuid" feature for UuidGenerator.
#[cfg(feature = "uuid")]
impl Entity for User {
    type IdGen = tagid::UuidGenerator;
}

#[cfg(feature = "uuid")]
type UserId = Id<Sourced<User, Generated<strategies::UuidV7>>, ::uuid::Uuid>;

fn main() {
    #[cfg(feature = "uuid")]
    {
        // Generate a new ID.
        let user_id = UserId::new();

        // Canonical display (always label-free)
        println!("Canonical ID: {}", user_id);

        // Human-facing labeled display (respects LabelPolicy::EntityNameDefault)
        // For Generated sources, this defaults to Short mode: "Entity::value"
        println!("Labeled (default): {}", user_id.labeled());

        // Explicitly request Full mode: "Entity@provenance::value"
        println!(
            "Labeled (full):    {}",
            user_id.labeled().mode(tagid::LabelMode::Full)
        );
    }

    #[cfg(not(feature = "with-uuid"))]
    println!("This example requires the 'with-uuid' feature.");
}
