use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;
use tracing::{info, debug, warn, error};

/// Volume driver types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VolumeDriver {
    Local,
    Nfs,
    Cifs,
    Bind,
    Tmpfs,
}

/// Volume mount options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountOptions {
    pub read_only: bool,
    pub no_exec: bool,
    pub no_dev: bool,
    pub no_suid: bool,
    pub user: Option<String>,
    pub group: Option<String>,
    pub mode: Option<u32>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
}

impl Default for MountOptions {
    fn default() -> Self {
        Self {
            read_only: false,
            no_exec: false,
            no_dev: false,
            no_suid: false,
            user: None,
            group: None,
            mode: Some(0o755),
            uid: None,
            gid: None,
        }
    }
}

/// Volume information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub name: String,
    pub driver: VolumeDriver,
    pub mountpoint: PathBuf,
    pub created_at: SystemTime,
    pub labels: HashMap<String, String>,
    pub scope: String,
    pub options: HashMap<String, String>,
    pub size: Option<u64>,
    pub in_use: bool,
}

/// Volume statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeStats {
    pub name: String,
    pub size: u64,
    pub used: u64,
    pub available: u64,
    pub in_use: bool,
    pub mount_count: u32,
    pub last_mounted: Option<SystemTime>,
}

/// Volume driver interface
#[async_trait::async_trait]
pub trait VolumeDriverTrait {
    async fn create_volume(&self, name: &str, options: &HashMap<String, String>) -> Result<PathBuf>;
    async fn remove_volume(&self, name: &str) -> Result<()>;
    async fn mount_volume(&self, name: &str, target: &PathBuf, options: &MountOptions) -> Result<()>;
    async fn unmount_volume(&self, name: &str) -> Result<()>;
    async fn get_volume_stats(&self, name: &str) -> Result<VolumeStats>;
    async fn list_volumes(&self) -> Result<Vec<Volume>>;
}

/// Local volume driver implementation
pub struct LocalVolumeDriver {
    base_path: PathBuf,
}

impl LocalVolumeDriver {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

#[async_trait::async_trait]
impl VolumeDriverTrait for LocalVolumeDriver {
    async fn create_volume(&self, name: &str, _options: &HashMap<String, String>) -> Result<PathBuf> {
        let volume_path = self.base_path.join(name);
        
        if volume_path.exists() {
            return Err(PolisError::Storage(format!("Volume '{}' já existe", name)));
        }

        std::fs::create_dir_all(&volume_path)
            .map_err(|e| PolisError::Io(e))?;

        info!("Volume local '{}' criado em {:?}", name, volume_path);
        Ok(volume_path)
    }

    async fn remove_volume(&self, name: &str) -> Result<()> {
        let volume_path = self.base_path.join(name);
        
        if !volume_path.exists() {
            return Err(PolisError::Storage(format!("Volume '{}' não encontrado", name)));
        }

        std::fs::remove_dir_all(&volume_path)
            .map_err(|e| PolisError::Io(e))?;

        info!("Volume local '{}' removido", name);
        Ok(())
    }

    async fn mount_volume(&self, name: &str, target: &PathBuf, _options: &MountOptions) -> Result<()> {
        let volume_path = self.base_path.join(name);
        
        if !volume_path.exists() {
            return Err(PolisError::Storage(format!("Volume '{}' não encontrado", name)));
        }

        // Create target directory if it doesn't exist
        std::fs::create_dir_all(target)
            .map_err(|e| PolisError::Io(e))?;

        // In a real implementation, this would create a bind mount
        // For now, we'll just simulate it
        info!("Volume '{}' montado em {:?}", name, target);
        Ok(())
    }

    async fn unmount_volume(&self, name: &str) -> Result<()> {
        // In a real implementation, this would unmount the volume
        info!("Volume '{}' desmontado", name);
        Ok(())
    }

    async fn get_volume_stats(&self, name: &str) -> Result<VolumeStats> {
        let volume_path = self.base_path.join(name);
        
        if !volume_path.exists() {
            return Err(PolisError::Storage(format!("Volume '{}' não encontrado", name)));
        }

        let metadata = volume_path.metadata()
            .map_err(|e| PolisError::Io(e))?;

        let size = self.calculate_directory_size(&volume_path)?;

        Ok(VolumeStats {
            name: name.to_string(),
            size: size,
            used: size,
            available: 1000000000, // 1GB simulated
            in_use: false,
            mount_count: 0,
            last_mounted: None,
        })
    }

    async fn list_volumes(&self) -> Result<Vec<Volume>> {
        let mut volumes = Vec::new();

        if !self.base_path.exists() {
            return Ok(volumes);
        }

        for entry in std::fs::read_dir(&self.base_path)
            .map_err(|e| PolisError::Io(e))? {
            let entry = entry.map_err(|e| PolisError::Io(e))?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    let metadata = path.metadata()
                        .map_err(|e| PolisError::Io(e))?;

                    let volume = Volume {
                        name: name.to_string(),
                        driver: VolumeDriver::Local,
                        mountpoint: path.clone(),
                        created_at: metadata.created()
                            .map_err(|e| PolisError::Io(e))?,
                        labels: HashMap::new(),
                        scope: "local".to_string(),
                        options: HashMap::new(),
                        size: Some(self.calculate_directory_size(&path)?),
                        in_use: false,
                    };

                    volumes.push(volume);
                }
            }
        }

        Ok(volumes)
    }
}

impl LocalVolumeDriver {
    fn calculate_directory_size(&self, path: &PathBuf) -> Result<u64> {
        let mut total_size = 0;
        
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry.map_err(|e| PolisError::Io(e.into()))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Ok(metadata) = path.metadata() {
                    total_size += metadata.len();
                }
            }
        }
        
        Ok(total_size)
    }
}

/// Main volume manager
pub struct VolumeManager {
    drivers: HashMap<VolumeDriver, Box<dyn VolumeDriverTrait + Send + Sync>>,
    volumes: HashMap<String, Volume>,
    base_path: PathBuf,
}

impl VolumeManager {
    pub async fn new(base_path: PathBuf) -> Result<Self> {
        let mut manager = Self {
            drivers: HashMap::new(),
            volumes: HashMap::new(),
            base_path: base_path.clone(),
        };

        // Initialize local driver
        let local_driver = LocalVolumeDriver::new(base_path.join("local"));
        manager.drivers.insert(VolumeDriver::Local, Box::new(local_driver));

        // Load existing volumes
        manager.load_volumes().await?;

        Ok(manager)
    }

    async fn load_volumes(&mut self) -> Result<()> {
        if let Some(local_driver) = self.drivers.get(&VolumeDriver::Local) {
            let volumes = local_driver.list_volumes().await?;
            for volume in volumes {
                self.volumes.insert(volume.name.clone(), volume);
            }
        }
        Ok(())
    }

    pub async fn create_volume(
        &mut self,
        name: &str,
        driver: VolumeDriver,
        options: HashMap<String, String>,
        labels: HashMap<String, String>,
    ) -> Result<Volume> {
        if self.volumes.contains_key(name) {
            return Err(PolisError::Storage(format!("Volume '{}' já existe", name)));
        }

        let driver_ref = self.drivers.get(&driver)
            .ok_or_else(|| PolisError::Storage(format!("Driver '{:?}' não encontrado", driver)))?;

        let mountpoint = driver_ref.create_volume(name, &options).await?;

        let volume = Volume {
            name: name.to_string(),
            driver: driver.clone(),
            mountpoint,
            created_at: SystemTime::now(),
            labels,
            scope: "local".to_string(),
            options,
            size: None,
            in_use: false,
        };

        self.volumes.insert(name.to_string(), volume.clone());
        info!("Volume '{}' criado com driver {:?}", name, driver);
        Ok(volume)
    }

    pub async fn remove_volume(&mut self, name: &str, force: bool) -> Result<()> {
        let volume = self.volumes.get(name)
            .ok_or_else(|| PolisError::Storage(format!("Volume '{}' não encontrado", name)))?;

        if volume.in_use && !force {
            return Err(PolisError::Storage(format!(
                "Volume '{}' está em uso. Use --force para remover",
                name
            )));
        }

        let driver_ref = self.drivers.get(&volume.driver)
            .ok_or_else(|| PolisError::Storage(format!("Driver '{:?}' não encontrado", volume.driver)))?;

        driver_ref.remove_volume(name).await?;
        self.volumes.remove(name);

        info!("Volume '{}' removido", name);
        Ok(())
    }

    pub async fn mount_volume(
        &mut self,
        name: &str,
        target: &PathBuf,
        options: MountOptions,
    ) -> Result<()> {
        let volume = self.volumes.get(name)
            .ok_or_else(|| PolisError::Storage(format!("Volume '{}' não encontrado", name)))?;

        let driver_ref = self.drivers.get(&volume.driver)
            .ok_or_else(|| PolisError::Storage(format!("Driver '{:?}' não encontrado", volume.driver)))?;

        driver_ref.mount_volume(name, target, &options).await?;

        // Update volume as in use
        if let Some(volume) = self.volumes.get_mut(name) {
            volume.in_use = true;
        }

        info!("Volume '{}' montado em {:?}", name, target);
        Ok(())
    }

    pub async fn unmount_volume(&mut self, name: &str) -> Result<()> {
        let volume = self.volumes.get(name)
            .ok_or_else(|| PolisError::Storage(format!("Volume '{}' não encontrado", name)))?;

        let driver_ref = self.drivers.get(&volume.driver)
            .ok_or_else(|| PolisError::Storage(format!("Driver '{:?}' não encontrado", volume.driver)))?;

        driver_ref.unmount_volume(name).await?;

        // Update volume as not in use
        if let Some(volume) = self.volumes.get_mut(name) {
            volume.in_use = false;
        }

        info!("Volume '{}' desmontado", name);
        Ok(())
    }

    pub async fn get_volume(&self, name: &str) -> Result<Option<Volume>> {
        Ok(self.volumes.get(name).cloned())
    }

    pub async fn list_volumes(&self) -> Result<Vec<Volume>> {
        Ok(self.volumes.values().cloned().collect())
    }

    pub async fn get_volume_stats(&self, name: &str) -> Result<VolumeStats> {
        let volume = self.volumes.get(name)
            .ok_or_else(|| PolisError::Storage(format!("Volume '{}' não encontrado", name)))?;

        let driver_ref = self.drivers.get(&volume.driver)
            .ok_or_else(|| PolisError::Storage(format!("Driver '{:?}' não encontrado", volume.driver)))?;

        driver_ref.get_volume_stats(name).await
    }

    pub async fn prune_volumes(&mut self, force: bool) -> Result<PruneStats> {
        let mut to_remove = Vec::new();
        let mut space_freed = 0;

        for (name, volume) in &self.volumes {
            if !volume.in_use {
                to_remove.push(name.clone());
                if let Some(size) = volume.size {
                    space_freed += size;
                }
            }
        }

        if !force && !to_remove.is_empty() {
            println!("Seriam removidos {} volumes ({} bytes)", to_remove.len(), space_freed);
            return Ok(PruneStats {
                volumes_removed: 0,
                space_freed: 0,
            });
        }

        for name in &to_remove {
            if let Err(e) = self.remove_volume(name, true).await {
                warn!("Erro ao remover volume '{}': {}", name, e);
            }
        }

        Ok(PruneStats {
            volumes_removed: to_remove.len(),
            space_freed,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PruneStats {
    pub volumes_removed: usize,
    pub space_freed: u64,
}
