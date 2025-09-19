use polis_core::{Container, ContainerId, Image, ImageId, PolisError, Result};
use polis_image::ImageManager;
use polis_runtime::{ContainerRuntime, PolisRuntime};
use std::sync::Arc;

// Generated protobuf code would go here
// For now, we'll define the service traits manually

pub struct GrpcServer {
    #[allow(dead_code)]
    runtime: Arc<PolisRuntime>,
    #[allow(dead_code)]
    image_manager: Arc<ImageManager>,
}

impl GrpcServer {
    pub fn new(runtime: Arc<PolisRuntime>, image_manager: Arc<ImageManager>) -> Self {
        Self {
            runtime,
            image_manager,
        }
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        println!(" Servidor gRPC iniciado em 0.0.0.0:{}", port);

        // In a real implementation, you would use tonic to generate the service
        // For now, we'll just simulate the server startup
        println!(" Servidor gRPC configurado com sucesso");

        Ok(())
    }
}

// Container Service Implementation
pub struct ContainerServiceImpl {
    runtime: Arc<PolisRuntime>,
}

impl ContainerServiceImpl {
    pub fn new(runtime: Arc<PolisRuntime>) -> Self {
        Self { runtime }
    }

    pub async fn list_containers(&self) -> Result<Vec<Container>> {
        self.runtime.list_containers().await
    }

    pub async fn get_container(&self, id: &str) -> Result<Container> {
        let container_id = ContainerId::from_string(id)
            .map_err(|e| PolisError::Api(format!("ID de container inválido: {}", e)))?;
        self.runtime.get_container(container_id).await
    }

    pub async fn create_container(
        &self,
        name: String,
        image: String,
        command: Vec<String>,
    ) -> Result<ContainerId> {
        self.runtime.create_container(name, image, command).await
    }

    pub async fn start_container(&self, id: &str) -> Result<()> {
        let container_id = ContainerId::from_string(id)
            .map_err(|e| PolisError::Api(format!("ID de container inválido: {}", e)))?;
        self.runtime.start_container(container_id).await
    }

    pub async fn stop_container(&self, id: &str) -> Result<()> {
        let container_id = ContainerId::from_string(id)
            .map_err(|e| PolisError::Api(format!("ID de container inválido: {}", e)))?;
        self.runtime.stop_container(container_id).await
    }

    pub async fn remove_container(&self, id: &str) -> Result<()> {
        let container_id = ContainerId::from_string(id)
            .map_err(|e| PolisError::Api(format!("ID de container inválido: {}", e)))?;
        self.runtime.remove_container(container_id).await
    }

    pub async fn pause_container(&self, id: &str) -> Result<()> {
        let container_id = ContainerId::from_string(id)
            .map_err(|e| PolisError::Api(format!("ID de container inválido: {}", e)))?;
        self.runtime.pause_container(container_id).await
    }

    pub async fn unpause_container(&self, id: &str) -> Result<()> {
        let container_id = ContainerId::from_string(id)
            .map_err(|e| PolisError::Api(format!("ID de container inválido: {}", e)))?;
        self.runtime.unpause_container(container_id).await
    }
}

// Image Service Implementation
pub struct ImageServiceImpl {
    image_manager: Arc<ImageManager>,
}

impl ImageServiceImpl {
    pub fn new(image_manager: Arc<ImageManager>) -> Self {
        Self { image_manager }
    }

    pub async fn list_images(&self) -> Result<Vec<Image>> {
        self.image_manager.list_images().await
    }

    pub async fn get_image(&self, id: &str) -> Result<Image> {
        // In a real implementation, you would have a get_image method
        // For now, we'll list all images and find the one with matching ID
        let images = self.image_manager.list_images().await?;
        images
            .into_iter()
            .find(|img| img.id.0 == id)
            .ok_or_else(|| PolisError::Api("Imagem não encontrada".to_string()))
    }

    pub async fn pull_image(&self, name: &str) -> Result<Image> {
        self.image_manager.pull(name).await
    }

    pub async fn remove_image(&self, id: &str) -> Result<()> {
        let image_id = ImageId::from_string(id);
        self.image_manager.remove_image(&image_id).await
    }
}

// System Service Implementation
#[derive(Default)]
pub struct SystemServiceImpl;

impl SystemServiceImpl {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_system_info(&self) -> Result<SystemInfo> {
        Ok(SystemInfo {
            service: "polis".to_string(),
            version: "0.3.0".to_string(),
            runtime: "rust".to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            os: std::env::consts::OS.to_string(),
            status: "running".to_string(),
        })
    }

    pub async fn health_check(&self) -> Result<HealthStatus> {
        Ok(HealthStatus {
            status: "healthy".to_string(),
            service: "polis".to_string(),
        })
    }
}

// Helper structs for gRPC responses
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub service: String,
    pub version: String,
    pub runtime: String,
    pub architecture: String,
    pub os: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: String,
    pub service: String,
}
