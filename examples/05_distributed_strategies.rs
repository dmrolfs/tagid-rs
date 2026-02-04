//! Distributed ID Generation Strategies.
//!
//! This example demonstrates how to use various distributed ID generation strategies
//! (CUID, Snowflake, ULID) within the tagid redesign framework.
//!
//! Each strategy is wrapped in `Sourced<E, Generated<Strategy>>` to maintain
//! semantic origin tracking while providing specific performance or ordering
//! characteristics.

use tagid::id::provenance::{Generated, strategies};
use tagid::{Entity, Id, Label, Sourced};

// ============================================================================
// ENTITY DEFINITIONS
// ============================================================================

#[derive(Label)]
struct Session;

#[allow(dead_code)]
#[derive(Label)]
struct LogEntry;

#[allow(dead_code)]
#[derive(Label)]
struct DistributedNode;

// ============================================================================
// STRATEGY 1: CUID (Optimized for Horizontal Scalability)
// ============================================================================
// Requires "cuid" feature.
#[cfg(feature = "cuid")]
impl Entity for Session {
    type IdGen = tagid::CuidGenerator;
}

#[cfg(feature = "cuid")]
type SessionId = Id<Sourced<Session, Generated<strategies::Cuid>>, String>;

// ============================================================================
// STRATEGY 2: Snowflake (Time-ordered, Distributed IDs)
// ============================================================================
// Requires "snowflake" feature.
#[cfg(feature = "snowflake")]
impl Entity for LogEntry {
    type IdGen = tagid::SnowflakeGenerator;
}

#[cfg(feature = "snowflake")]
type LogId = Id<Sourced<LogEntry, Generated<strategies::Snowflake>>, i64>;

// ============================================================================
// STRATEGY 3: ULID (Lexicographically Sortable, UUID Compatible)
// ============================================================================
// Requires "ulid" feature.
#[cfg(feature = "ulid")]
impl Entity for DistributedNode {
    type IdGen = tagid::UlidGenerator;
}

#[cfg(feature = "ulid")]
type NodeId = Id<Sourced<DistributedNode, Generated<strategies::Ulid>>, tagid::id::ulid::Ulid>;

fn main() {
    println!("=== tagid Redesign: Distributed ID Strategies ===\n");

    // ------------------------------------------------------------------------
    // CUID Implementation
    // ------------------------------------------------------------------------
    #[cfg(feature = "cuid")]
    {
        let session_id = SessionId::new();
        println!("Strategy: CUID");
        println!("  Canonical: {}", session_id);
        println!("  Human:     {}\n", session_id.labeled());
    }

    // ------------------------------------------------------------------------
    // Snowflake Implementation
    // ------------------------------------------------------------------------
    #[cfg(feature = "snowflake")]
    {
        // Note: Snowflake usually requires global node initialization.
        // We assume defaults here for demonstration.
        let log_id = LogId::new();
        println!("Strategy: Snowflake");
        println!("  Canonical: {}", log_id);
        println!("  Human:     {}\n", log_id.labeled());
    }

    // ------------------------------------------------------------------------
    // ULID Implementation
    // ------------------------------------------------------------------------
    #[cfg(feature = "ulid")]
    {
        let node_id = NodeId::new();
        println!("Strategy: ULID");
        println!("  Canonical: {}", node_id);
        println!("  Human:     {}", node_id.labeled().mode(LabelMode::Full));
        println!("  (ULIDs are lexicographically sortable strings/u128s)\n");
    }

    if cfg!(not(any(
        feature = "cuid",
        feature = "snowflake",
        feature = "ulid"
    ))) {
        println!("This example requires enabling 'cuid', 'snowflake', or 'ulid' features.");
    }

    println!("Summary: tagid allows swapping ID generation strategies per-entity");
    println!("without changing how identifiers are handled, logged, or stored.");
}
