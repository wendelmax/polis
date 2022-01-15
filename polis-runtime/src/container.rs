use polis_core::{Container, ContainerId, ContainerStatus, PolisError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ContainerManager {
    containers: Arc<RwLock<HashMap<ContainerId, Container>>>,
}

impl ContainerManager {
    pub fn new(containers: Arc<RwLock<HashMap<ContainerId, Container>>>) -> Self {
        Self { containers }
    }

    pub async fn create(&self, _name: String, _image: String) -> Result<Container> {
        // TODO: Implement container creation logic
        Err(PolisError::Container("Not implemented".to_string()))
    }

    pub async fn start(&self, _id: ContainerId) -> Result<()> {
        // TODO: Implement container start logic
        Err(PolisError::Container("Not implemented".to_string()))
    }

    pub async fn stop(&self, _id: ContainerId) -> Result<()> {
        // TODO: Implement container stop logic
        Err(PolisError::Container("Not implemented".to_string()))
    }

    pub async fn get_container(&self, id: &ContainerId) -> Result<Container> {
        let containers = self.containers.read().await;
        containers
            .get(id)
            .ok_or_else(|| PolisError::Container("Container não encontrado".to_string()))
            .cloned()
    }

    pub async fn update_container_status(
        &self,
        id: &ContainerId,
        status: ContainerStatus,
    ) -> Result<()> {
        let mut containers = self.containers.write().await;
        if let Some(container) = containers.get_mut(id) {
            container.status = status;
            Ok(())
        } else {
            Err(PolisError::Container(
                "Container não encontrado".to_string(),
            ))
        }
    }

    pub async fn list_containers(&self) -> Result<Vec<Container>> {
        let containers = self.containers.read().await;
        Ok(containers.values().cloned().collect())
    }

    pub async fn remove_container(&self, id: &ContainerId) -> Result<Container> {
        let mut containers = self.containers.write().await;
        containers
            .remove(id)
            .ok_or_else(|| PolisError::Container("Container não encontrado".to_string()))
    }
}
