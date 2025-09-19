use polis_auth::AuthManager;
use polis_core::{PolisError, Result};
use polis_image::ImageManager;
use polis_runtime::{ContainerRuntime, PolisRuntime};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RestServer {
    runtime: Arc<PolisRuntime>,
    image_manager: Arc<ImageManager>,
    auth_manager: Arc<RwLock<AuthManager>>,
}

impl RestServer {
    pub fn new(
        runtime: Arc<PolisRuntime>,
        image_manager: Arc<ImageManager>,
        auth_manager: Arc<RwLock<AuthManager>>,
    ) -> Self {
        Self {
            runtime,
            image_manager,
            auth_manager,
        }
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        println!("� API REST iniciada em http://0.0.0.0:{}", port);
        println!(" Implementação simplificada - funcionalidades completas em desenvolvimento");
        Ok(())
    }
}