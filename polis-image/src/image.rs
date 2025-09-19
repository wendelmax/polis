use chrono::{DateTime, Utc};
use polis_core::{Image, ImageId, PolisError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageMetadata {
    pub id: ImageId,
    pub name: String,
    pub tag: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
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

pub struct ImageManager {
    cache_dir: PathBuf,
    registry_client: Arc<Mutex<crate::registry::RegistryClient>>,
}

impl ImageManager {
    pub fn new(cache_dir: PathBuf) -> Self {
        let registry_client = Arc::new(Mutex::new(crate::registry::RegistryClient::new(cache_dir.clone())));
        Self {
            cache_dir,
            registry_client,
        }
    }

    pub fn with_docker_hub_token(cache_dir: PathBuf, token: String) -> Self {
        let registry_client = Arc::new(Mutex::new(
            crate::registry::RegistryClient::new(cache_dir.clone())
                .with_token(token)
        ));
        Self {
            cache_dir,
            registry_client,
        }
    }

    pub async fn pull(&self, name: &str) -> Result<Image> {
        // Pull image from registry
        let mut client = self.registry_client.lock().await;
        let image_id = client.pull_image(name).await?;

        // Try to load existing metadata, or create new one
        let metadata = match self.load_image_metadata(&image_id).await {
            Ok(metadata) => metadata,
            Err(_) => {
                // Create metadata from image name if not found
                let (repo, tag) = if let Some(colon_pos) = name.rfind(':') {
                    (
                        name[..colon_pos].to_string(),
                        name[colon_pos + 1..].to_string(),
                    )
                } else {
                    (name.to_string(), "latest".to_string())
                };

                ImageMetadata {
                    id: image_id.clone(),
                    name: repo,
                    tag,
                    size: 1024 * 1024, // 1MB default
                    created_at: chrono::Utc::now(),
                    architecture: "amd64".to_string(),
                    os: "linux".to_string(),
                    layers: vec!["sha256:local-layer".to_string()],
                    config: ImageConfig {
                        entrypoint: Some(vec!["/bin/sh".to_string()]),
                        cmd: Some(vec!["-c".to_string()]),
                        env: Some(vec!["PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string()]),
                        working_dir: Some("/".to_string()),
                        exposed_ports: None,
                        volumes: None,
                        labels: Some(std::collections::HashMap::new()),
                    },
                }
            }
        };

        // Create Image struct
        let image = Image {
            id: image_id.clone(),
            name: metadata.name.clone(),
            tag: metadata.tag.clone(),
            digest: "".to_string(), // TODO: Get from manifest
            size: metadata.size,
            created_at: metadata.created_at,
            architecture: metadata.architecture,
            os: metadata.os,
            layers: metadata.layers,
            config: polis_core::ImageConfig {
                entrypoint: metadata.config.entrypoint,
                cmd: metadata.config.cmd,
                env: metadata.config.env,
                working_dir: metadata.config.working_dir,
                exposed_ports: metadata.config.exposed_ports,
                volumes: metadata.config.volumes,
                labels: metadata.config.labels,
            },
        };

        // Save image metadata
        self.save_image_metadata(&image).await?;

        Ok(image)
    }

    pub async fn list_images(&self) -> Result<Vec<Image>> {
        let mut images = Vec::new();

        // List images from local cache
        let images_dir = self.cache_dir.join("images");
        if images_dir.exists() {
            let mut entries = fs::read_dir(&images_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir() {
                    let image_name = entry.file_name().to_string_lossy().to_string();
                    let image_id = ImageId::from_string(&image_name);
                    
                    if let Ok(metadata) = self.load_image_metadata(&image_id).await {
                        let image = Image {
                            id: image_id,
                            name: metadata.name,
                            tag: metadata.tag,
                            digest: "".to_string(), // TODO: Get from manifest
                            size: metadata.size,
                            created_at: metadata.created_at,
                            architecture: metadata.architecture,
                            os: metadata.os,
                            layers: metadata.layers,
                            config: polis_core::ImageConfig {
                                entrypoint: metadata.config.entrypoint,
                                cmd: metadata.config.cmd,
                                env: metadata.config.env,
                                working_dir: metadata.config.working_dir,
                                exposed_ports: metadata.config.exposed_ports,
                                volumes: metadata.config.volumes,
                                labels: metadata.config.labels,
                            },
                        };
                        images.push(image);
                    }
                }
            }
        }

        Ok(images)
    }

    pub async fn remove_image(&self, id: &ImageId) -> Result<()> {
        // Remove image directory
        let image_dir = self.get_image_dir(id);
        if image_dir.exists() {
            fs::remove_dir_all(&image_dir).await?;
            Ok(())
        } else {
            Err(PolisError::Image(format!(
                "Imagem não encontrada: {}",
                id.0
            )))
        }
    }

    async fn load_image_metadata(&self, image_id: &ImageId) -> Result<ImageMetadata> {
        let image_dir = self.get_image_dir(image_id);
        let metadata_path = image_dir.join("metadata.json");

        if !metadata_path.exists() {
            return Err(PolisError::Image("Metadata não encontrada".to_string()));
        }

        let content = fs::read_to_string(&metadata_path).await?;
        let metadata: ImageMetadata = serde_json::from_str(&content)?;

        Ok(metadata)
    }

    async fn save_image_metadata(&self, image: &Image) -> Result<()> {
        let image_dir = self.get_image_dir(&image.id);
        fs::create_dir_all(&image_dir).await?;

        let metadata = ImageMetadata {
            id: image.id.clone(),
            name: image.name.clone(),
            tag: image.tag.clone(),
            size: image.size,
            created_at: image.created_at,
            architecture: image.architecture.clone(),
            os: image.os.clone(),
            layers: image.layers.clone(),
            config: ImageConfig {
                entrypoint: image.config.entrypoint.clone(),
                cmd: image.config.cmd.clone(),
                env: image.config.env.clone(),
                working_dir: image.config.working_dir.clone(),
                exposed_ports: image.config.exposed_ports.clone(),
                volumes: image.config.volumes.clone(),
                labels: image.config.labels.clone(),
            },
        };

        let metadata_path = image_dir.join("metadata.json");
        let content = serde_json::to_string_pretty(&metadata)?;
        fs::write(&metadata_path, content).await?;

        Ok(())
    }

    fn get_image_dir(&self, image_id: &ImageId) -> PathBuf {
        self.cache_dir.join("images").join(&image_id.0)
    }
}
