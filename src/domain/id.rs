use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(crate) fn random_id() -> u64 {
    rand::random::<u64>() % 0xFFFF_FFFF_FFFF
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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
        write!(formatter, "{:x}", self.0)
    }
}

impl FromStr for AgentId {
    type Err = std::num::ParseIntError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self(u64::from_str_radix(value, 16)?))
    }
}

impl Serialize for AgentId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&format!("{:x}", self.0))
    }
}

impl<'de> Deserialize<'de> for AgentId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        u64::from_str_radix(&s, 16)
            .map(Self)
            .map_err(serde::de::Error::custom)
    }
}

pub type JobId = u64;

pub fn format_job_id(id: JobId) -> String {
    format!("{:x}", id)
}

pub fn parse_job_id(s: &str) -> Result<JobId, std::num::ParseIntError> {
    u64::from_str_radix(s, 16)
}

pub struct AgentIdDisplay(AgentId);

impl fmt::Display for AgentIdDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "a#{:x}", self.0.0)
    }
}

pub struct JobIdDisplay(u64);

impl fmt::Display for JobIdDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "j#{:x}", self.0)
    }
}

pub fn display_agent_id(id: AgentId) -> AgentIdDisplay {
    AgentIdDisplay(id)
}

pub fn display_job_id(id: u64) -> JobIdDisplay {
    JobIdDisplay(id)
}
