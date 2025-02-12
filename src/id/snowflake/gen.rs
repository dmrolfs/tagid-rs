//! Snowflake ID generation module.
//!
//! This module provides a `SnowflakeGenerator` for distributed unique ID generation
//! based on Twitter's Snowflake algorithm. It supports different generation strategies
//! and can be initialized as a singleton for either a single node or a distributed system.

use super::node::MachineNode;
use crate::id::IdGenerator;
use once_cell::sync::OnceCell;
use snowflake::SnowflakeIdGenerator as SnowflakeGen;
use std::sync::{Arc, Mutex};
use std::time;
use strum_macros::{Display, EnumString, IntoStaticStr, VariantNames};

/// A globally initialized `SnowflakeGenerator` instance.
static SNOWFLAKE_GENERATOR: OnceCell<SnowflakeGenerator> = OnceCell::new();

/// A generator for unique Snowflake IDs.
///
/// The `SnowflakeGenerator` uses a `SnowflakeIdGenerator` internally and supports
/// different `GenerationStrategy` options. It ensures a singleton instance for ID generation.
///
/// # Examples
///
/// ```rust
/// use tagid::IdGenerator;
/// use tagid::snowflake::{SnowflakeGenerator, GenerationStrategy};
///
/// // Initialize as a single node generator
/// let generator = SnowflakeGenerator::single_node(GenerationStrategy::RealTime);
///
/// // Generate a new ID
/// let id = SnowflakeGenerator::next_id_rep();
/// ```
#[derive(Debug, Clone)]
pub struct SnowflakeGenerator {
    /// The strategy used for ID generation.
    strategy: GenerationStrategy,

    /// The machine node configuration.
    machine_node: MachineNode,

    /// The underlying Snowflake ID generator.
    gen: Arc<Mutex<SnowflakeGen>>,
}

impl SnowflakeGenerator {
    /// Retrieves the globally initialized `SnowflakeGenerator` instance.
    ///
    /// # Panics
    ///
    /// Panics if the generator has not been initialized using `single_node` or `distributed`.
    pub fn summon() -> &'static Self {
        SNOWFLAKE_GENERATOR
            .get()
            .expect("SnowflakeGenerator is not initialized - initialize via single_node() or distributed().")
    }

    /// Initializes the `SnowflakeGenerator` as a single-node instance.
    ///
    /// # Parameters
    /// - `strategy`: The ID generation strategy to use.
    ///
    /// # Returns
    ///
    /// A reference to the globally initialized `SnowflakeGenerator`.
    pub fn single_node(strategy: GenerationStrategy) -> &'static Self {
        Self::distributed(MachineNode::default(), strategy)
    }

    /// Initializes the `SnowflakeGenerator` for a distributed system.
    ///
    /// # Parameters
    /// - `machine_node`: The machine and node identifier.
    /// - `strategy`: The ID generation strategy to use.
    ///
    /// # Returns
    ///
    /// A reference to the globally initialized `SnowflakeGenerator`.
    pub fn distributed(machine_node: MachineNode, strategy: GenerationStrategy) -> &'static Self {
        let gen = SnowflakeGen::with_epoch(
            machine_node.machine_id,
            machine_node.node_id,
            time::UNIX_EPOCH,
        );
        SNOWFLAKE_GENERATOR.get_or_init(|| Self {
            machine_node,
            strategy,
            gen: Arc::new(Mutex::new(gen)),
        })
    }
}

impl IdGenerator for SnowflakeGenerator {
    type IdType = i64;

    /// Generates the next unique ID.
    ///
    /// Uses the configured `GenerationStrategy` to determine how the ID is created.
    ///
    /// # Returns
    ///
    /// A unique `i64` ID.
    fn next_id_rep() -> Self::IdType {
        let generator = Self::summon();
        let mut gen = generator.gen.lock().unwrap();
        match generator.strategy {
            GenerationStrategy::RealTime => gen.real_time_generate(),
            GenerationStrategy::Generate => gen.generate(),
            GenerationStrategy::Lazy => gen.lazy_generate(),
        }
    }
}

impl PartialEq for SnowflakeGenerator {
    fn eq(&self, other: &Self) -> bool {
        self.strategy == other.strategy && self.machine_node == other.machine_node
    }
}

impl Eq for SnowflakeGenerator {}

/// The strategy used for ID generation.
///
/// Determines how the `SnowflakeGenerator` produces new IDs.
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, Display, IntoStaticStr, EnumString, VariantNames,
)]
pub enum GenerationStrategy {
    RealTime,
    Generate,
    Lazy,
}
