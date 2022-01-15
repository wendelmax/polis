use crate::error::{PolisError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolisConfig {
    pub runtime: RuntimeConfig,
    pub storage: StorageConfig,
    pub network: NetworkConfig,
    pub security: SecurityConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub root_dir: PathBuf,
    pub log_level: LogLevel,
    pub debug: bool,
    pub max_containers: u32,
    pub container_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub driver: StorageDriver,
    pub root_dir: PathBuf,
    pub max_size: Option<u64>,
    pub cleanup_on_exit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub driver: NetworkDriver,
    pub bridge_name: String,
    pub subnet: Option<String>,
    pub gateway: Option<String>,
    pub dns_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub seccomp_profile: String,
    pub apparmor_profile: String,
    pub no_new_privileges: bool,
    pub drop_capabilities: Vec<String>,
    pub read_only_rootfs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub rest_port: u16,
    pub grpc_port: u16,
    pub host: String,
    pub enable_cors: bool,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageDriver {
    Overlay2,
    Btrfs,
    Zfs,
    Dir,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkDriver {
    Bridge,
    Host,
    None,
    Macvlan,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            root_dir: PathBuf::from("/var/lib/polis"),
            log_level: LogLevel::Info,
            debug: false,
            max_containers: 100,
            container_timeout: 30,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            driver: StorageDriver::Overlay2,
            root_dir: PathBuf::from("/var/lib/polis/storage"),
            max_size: None,
            cleanup_on_exit: true,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            driver: NetworkDriver::Bridge,
            bridge_name: "polis0".to_string(),
            subnet: Some("172.17.0.0/16".to_string()),
            gateway: Some("172.17.0.1".to_string()),
            dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            seccomp_profile: "default".to_string(),
            apparmor_profile: "docker-default".to_string(),
            no_new_privileges: true,
            drop_capabilities: vec!["ALL".to_string()],
            read_only_rootfs: false,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            rest_port: 8080,
            grpc_port: 9090,
            host: "0.0.0.0".to_string(),
            enable_cors: true,
            timeout_seconds: 30,
        }
    }
}

impl PolisConfig {
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(PolisError::Io)?;

        let config = if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&content)
                .map_err(|e| PolisError::Runtime(format!("Erro ao parsear YAML: {}", e)))?
        } else if path.ends_with(".toml") {
            toml::from_str(&content)
                .map_err(|e| PolisError::Runtime(format!("Erro ao parsear TOML: {}", e)))?
        } else if path.ends_with(".json") {
            serde_json::from_str(&content).map_err(PolisError::Serialization)?
        } else {
            return Err(PolisError::Runtime(format!(
                "Formato de arquivo não suportado: {}",
                path
            )));
        };

        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::to_string(self)
                .map_err(|e| PolisError::Runtime(format!("Erro ao serializar YAML: {}", e)))?
        } else if path.ends_with(".toml") {
            toml::to_string(self)
                .map_err(|e| PolisError::Runtime(format!("Erro ao serializar TOML: {}", e)))?
        } else if path.ends_with(".json") {
            serde_json::to_string_pretty(self).map_err(PolisError::Serialization)?
        } else {
            return Err(PolisError::Runtime(format!(
                "Formato de arquivo não suportado: {}",
                path
            )));
        };

        std::fs::write(path, content).map_err(PolisError::Io)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        if self.runtime.max_containers == 0 {
            return Err(PolisError::Runtime(
                "max_containers deve ser maior que 0".to_string(),
            ));
        }

        if self.runtime.container_timeout == 0 {
            return Err(PolisError::Runtime(
                "container_timeout deve ser maior que 0".to_string(),
            ));
        }

        if self.api.rest_port == 0 || self.api.grpc_port == 0 {
            return Err(PolisError::Runtime(
                "Portas da API devem ser maiores que 0".to_string(),
            ));
        }

        if self.api.rest_port == self.api.grpc_port {
            return Err(PolisError::Runtime(
                "Portas REST e gRPC devem ser diferentes".to_string(),
            ));
        }

        Ok(())
    }
}
