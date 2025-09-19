use polis_core::Result;
use crate::volume::{VolumeDriver, VolumeDriverTrait, Volume, VolumeStats, MountOptions};
use std::collections::HashMap;
use std::path::PathBuf;

/// Driver manager for handling different volume drivers
pub struct DriverManager {
    drivers: HashMap<VolumeDriver, Box<dyn VolumeDriverTrait + Send + Sync>>,
}

impl DriverManager {
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
        }
    }

    pub async fn register_driver(
        &mut self,
        driver_type: VolumeDriver,
        driver: Box<dyn VolumeDriverTrait + Send + Sync>,
    ) -> Result<()> {
        self.drivers.insert(driver_type, driver);
        Ok(())
    }

    pub fn get_driver(&self, driver_type: &VolumeDriver) -> Option<&(dyn VolumeDriverTrait + Send + Sync)> {
        self.drivers.get(driver_type).map(|d| d.as_ref())
    }

    pub fn list_drivers(&self) -> Vec<VolumeDriver> {
        self.drivers.keys().cloned().collect()
    }
}

impl Default for DriverManager {
    fn default() -> Self {
        Self::new()
    }
}
