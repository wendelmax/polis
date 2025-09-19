use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use polis_core::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub unqualified_search_registries: Vec<String>,
    pub registries: HashMap<String, RegistryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub location: String,
    pub mirror: Option<String>,
    pub insecure: Option<bool>,
    pub blocked: Option<bool>,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        let mut registries = HashMap::new();
        
        // Docker Hub com mirror
        registries.insert("docker.io".to_string(), RegistryEntry {
            location: "https://registry-1.docker.io".to_string(),
            mirror: Some("https://mirror.gcr.io".to_string()),
            insecure: Some(false),
            blocked: Some(false),
        });
        
        // Quay.io
        registries.insert("quay.io".to_string(), RegistryEntry {
            location: "https://quay.io".to_string(),
            mirror: None,
            insecure: Some(false),
            blocked: Some(false),
        });
        
        // Red Hat Registry
        registries.insert("registry.redhat.io".to_string(), RegistryEntry {
            location: "https://registry.redhat.io".to_string(),
            mirror: None,
            insecure: Some(false),
            blocked: Some(false),
        });
        
        // Google Container Registry
        registries.insert("gcr.io".to_string(), RegistryEntry {
            location: "https://gcr.io".to_string(),
            mirror: None,
            insecure: Some(false),
            blocked: Some(false),
        });
        
        Self {
            unqualified_search_registries: vec![
                "docker.io".to_string(),
                "quay.io".to_string(),
                "registry.redhat.io".to_string(),
            ],
            registries,
        }
    }
}

impl RegistryConfig {
    pub fn load() -> Result<Self> {
        // Try to load from user config first
        if let Some(user_config) = Self::load_from_path(&Self::user_config_path())? {
            return Ok(user_config);
        }
        
        // Try to load from system config
        if let Some(system_config) = Self::load_from_path(&Self::system_config_path())? {
            return Ok(system_config);
        }
        
        // Return default config
        Ok(Self::default())
    }
    
    fn load_from_path(path: &PathBuf) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(path)
            .map_err(|e| polis_core::PolisError::Config(format!("Erro ao ler arquivo de configuração: {}", e)))?;
        
        let config: RegistryConfig = toml::from_str(&content)
            .map_err(|e| polis_core::PolisError::Config(format!("Erro ao parsear configuração: {}", e)))?;
        
        Ok(Some(config))
    }
    
    pub fn user_config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| std::env::var("USERPROFILE").unwrap_or_default());
        PathBuf::from(home).join(".config").join("polis").join("registries.conf")
    }
    
    fn system_config_path() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from("C:\\ProgramData\\Polis\\registries.conf")
        } else {
            PathBuf::from("/etc/polis/registries.conf")
        }
    }
    
    pub fn save_user_config(&self) -> Result<()> {
        let user_config_path = RegistryConfig::user_config_path();
        let config_dir = user_config_path.parent().unwrap();
        fs::create_dir_all(config_dir)
            .map_err(|e| polis_core::PolisError::Config(format!("Erro ao criar diretório de configuração: {}", e)))?;
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| polis_core::PolisError::Config(format!("Erro ao serializar configuração: {}", e)))?;
        
        fs::write(&RegistryConfig::user_config_path(), content)
            .map_err(|e| polis_core::PolisError::Config(format!("Erro ao salvar configuração: {}", e)))?;
        
        Ok(())
    }
    
    pub fn get_registry_url(&self, registry: &str) -> Option<String> {
        self.registries.get(registry).map(|entry| {
            if let Some(mirror) = &entry.mirror {
                format!("{}/v2", mirror)
            } else {
                format!("{}/v2", entry.location)
            }
        })
    }
    
    pub fn get_fallback_url(&self, registry: &str) -> Option<String> {
        self.registries.get(registry).map(|entry| {
            format!("{}/v2", entry.location)
        })
    }
    
    pub fn is_registry_blocked(&self, registry: &str) -> bool {
        self.registries.get(registry)
            .and_then(|entry| entry.blocked)
            .unwrap_or(false)
    }
    
    pub fn is_registry_insecure(&self, registry: &str) -> bool {
        self.registries.get(registry)
            .and_then(|entry| entry.insecure)
            .unwrap_or(false)
    }
    
    pub fn get_search_registries(&self) -> &Vec<String> {
        &self.unqualified_search_registries
    }
}
