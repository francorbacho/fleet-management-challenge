use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub(crate) fn random_id() -> u64 {
    rand::random::<u64>() % 1_000_000_000_000
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct AgentId(u64);

impl AgentId {
    pub fn new() -> Self {
        Self(random_id())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl FromStr for AgentId {
    type Err = std::num::ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(value.parse()?))
    }
}

pub struct AgentIdDisplay(AgentId);

impl fmt::Display for AgentIdDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a#{}", self.0.0)
    }
}

pub struct CommandIdDisplay(u64);

impl fmt::Display for CommandIdDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "c#{}", self.0)
    }
}

pub struct JobIdDisplay(u64);

impl fmt::Display for JobIdDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "j#{}", self.0)
    }
}

pub fn display_agent_id(id: AgentId) -> AgentIdDisplay {
    AgentIdDisplay(id)
}

pub fn display_command_id(id: u64) -> CommandIdDisplay {
    CommandIdDisplay(id)
}

pub fn display_job_id(id: u64) -> JobIdDisplay {
    JobIdDisplay(id)
}
