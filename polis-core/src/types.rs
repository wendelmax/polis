use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct ContainerId(pub Uuid);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ImageId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NetworkId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VolumeId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContainerStatus {
    Created,
    Running,
    Paused,
    Stopped,
    Exited,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: ContainerId,
    pub name: String,
    pub image: ImageId,
    pub status: ContainerStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub exit_code: Option<i32>,
    pub command: Vec<String>,
    pub working_dir: PathBuf,
    pub environment: HashMap<String, String>,
    pub labels: HashMap<String, String>,
    pub resource_limits: ResourceLimits,
    pub network_mode: NetworkMode,
    pub ports: Vec<PortMapping>,
    pub volumes: Vec<VolumeMount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: ImageId,
    pub name: String,
    pub tag: String,
    pub digest: String,
    pub size: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub architecture: String,
    pub os: String,
    pub layers: Vec<String>,
    pub config: ImageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub entrypoint: Option<Vec<String>>,
    pub cmd: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub working_dir: Option<String>,
    pub exposed_ports: Option<HashMap<String, serde_json::Value>>,
    pub volumes: Option<HashMap<String, serde_json::Value>>,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub id: NetworkId,
    pub name: String,
    pub driver: String,
    pub subnet: Option<String>,
    pub gateway: Option<String>,
    pub containers: Vec<ContainerId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: VolumeId,
    pub name: String,
    pub driver: String,
    pub mountpoint: PathBuf,
    pub size: Option<u64>,
    pub labels: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResourceLimits {
    pub memory_limit: Option<u64>,
    pub memory_swap: Option<u64>,
    pub cpu_quota: Option<f64>,
    pub cpu_period: Option<u64>,
    pub disk_quota: Option<u64>,
    pub pids_limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMode {
    Bridge,
    Host,
    None,
    Container(ContainerId),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: Protocol,
    pub host_ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub source: String,
    pub destination: PathBuf,
    pub mode: VolumeMode,
    pub read_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VolumeMode {
    Bind,
    Volume,
    Tmpfs,
}

impl ContainerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl fmt::Display for ContainerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ImageId {
    pub fn new(name: &str, tag: &str) -> Self {
        Self(format!("{}:{}", name, tag))
    }

    pub fn from_string(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl NetworkId {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl VolumeId {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl Default for NetworkMode {
    fn default() -> Self {
        Self::Bridge
    }
}
