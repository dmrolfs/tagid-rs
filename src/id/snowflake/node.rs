use crate::DELIMITER;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use validator::{Validate, ValidationErrors};

/// Used to supplement the sectionalization attribute of the Snowflake algorithm in a distributed
/// environment. The machine_id and node_id are combined to form a unique worker_id used by the
/// Snowflake algorithm. This worker_id must be unique for a target identifier space (e.g.,
/// identifier for a type of entity), otherwise identifier collisions can easily occur even in a
/// light concurrent environment.
#[derive(Debug, Copy, Clone, Validate, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MachineNode {
    /// For a target identifier space, the machine_id represents the largest granularity of
    /// uniqueness; e.g., a physical machine or a cluster identifier.
    #[validate(range(min = 0, max = 31))]
    pub machine_id: i32,

    /// For a target identifier space, the node_id represents
    #[validate(range(min = 0, max = 31))]
    pub node_id: i32,
}

impl fmt::Display for MachineNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}{DELIMITER}{})", self.machine_id, self.node_id)
    }
}

impl Default for MachineNode {
    fn default() -> Self {
        Self {
            machine_id: 1,
            node_id: 1,
        }
    }
}

impl MachineNode {
    pub fn new(machine_id: i32, node_id: i32) -> Result<Self, ValidationErrors> {
        let result = Self {
            machine_id,
            node_id,
        };
        result.validate()?;
        Ok(result)
    }
}

impl Ord for MachineNode {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.machine_id.cmp(&other.machine_id) {
            Ordering::Equal => self.node_id.cmp(&other.node_id),
            o => o,
        }
    }
}

impl PartialOrd for MachineNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
