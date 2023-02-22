use super::node::MachineNode;
use crate::id::IdGenerator;
use once_cell::sync::OnceCell;
use snowflake::SnowflakeIdGenerator as SnowflakeGen;
use std::sync::{Arc, Mutex};
use std::time;
use strum_macros::{Display, EnumString, EnumVariantNames, IntoStaticStr};

static SNOWFLAKE_GENERATOR: OnceCell<SnowflakeGenerator> = OnceCell::new();

#[derive(Debug, Clone)]
pub struct SnowflakeGenerator {
    strategy: GenerationStrategy,
    machine_node: MachineNode,
    gen: Arc<Mutex<SnowflakeGen>>,
}

impl SnowflakeGenerator {
    pub fn summon() -> &'static Self {
        SNOWFLAKE_GENERATOR
            .get()
            .expect("SnowflakeGenerator is not initialized - initialize via single_node() or distributed().")
    }

    pub fn single_node(strategy: GenerationStrategy) -> &'static Self {
        Self::distributed(MachineNode::default(), strategy)
    }

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

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, Display, IntoStaticStr, EnumString, EnumVariantNames,
)]
pub enum GenerationStrategy {
    RealTime,
    Generate,
    Lazy,
}
