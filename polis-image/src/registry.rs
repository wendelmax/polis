use polis_core::{ImageId, PolisError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciManifest {
    pub schema_version: u32,
    pub media_type: String,
    pub config: OciDescriptor,
    pub layers: Vec<OciDescriptor>,
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciDescriptor {
    pub media_type: String,
    pub size: u64,
    pub digest: String,
    pub urls: Option<Vec<String>>,
    pub annotations: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciConfig {
    pub architecture: String,
    pub os: String,
    pub config: OciImageConfig,
    pub rootfs: OciRootFs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciImageConfig {
    pub user: Option<String>,
    pub exposed_ports: Option<HashMap<String, serde_json::Value>>,
    pub env: Option<Vec<String>>,
    pub entrypoint: Option<Vec<String>>,
    pub cmd: Option<Vec<String>>,
    pub volumes: Option<HashMap<String, serde_json::Value>>,
    pub working_dir: Option<String>,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciRootFs {
    pub r#type: String,
    pub diff_ids: Vec<String>,
}

pub struct RegistryClient {
    client: Client,
    base_url: String,
    cache_dir: PathBuf,
}

impl RegistryClient {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            client: Client::new(),
            base_url: "https://registry-1.docker.io/v2".to_string(),
            cache_dir,
        }
    }

    pub async fn pull_image(&self, name: &str) -> Result<ImageId> {
        let image_id = ImageId::from_string(name);

        // Parse image name (e.g., "alpine:latest" -> "alpine", "latest")
        let (repo, tag) = if let Some(colon_pos) = name.rfind(':') {
            (
                name[..colon_pos].to_string(),
                name[colon_pos + 1..].to_string(),
            )
        } else {
            (name.to_string(), "latest".to_string())
        };

        // Create cache directory for this image
        let image_cache_dir = self.cache_dir.join(&repo).join(&tag);
        fs::create_dir_all(&image_cache_dir).await?;

        // Get manifest
        let manifest = self.fetch_manifest(&repo, &tag).await?;

        // Save manifest
        let manifest_path = image_cache_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, manifest_json).await?;

        // Download config
        let config = self.fetch_config(&repo, &manifest.config.digest).await?;
        let config_path = image_cache_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, config_json).await?;

        // Download layers
        for (i, layer) in manifest.layers.iter().enumerate() {
            let layer_path = image_cache_dir.join(format!("layer_{}.tar.gz", i));
            self.download_layer(&repo, &layer.digest, &layer_path)
                .await?;
        }

        println!("✅ Imagem '{}' baixada com sucesso", name);
        Ok(image_id)
    }

    async fn fetch_manifest(&self, repo: &str, tag: &str) -> Result<OciManifest> {
        let url = format!("{}/{}/manifests/{}", self.base_url, repo, tag);

        let response = self
            .client
            .get(&url)
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()
            .await
            .map_err(|e| PolisError::Image(format!("Erro ao buscar manifest: {}", e)))?;

        if !response.status().is_success() {
            return Err(PolisError::Image(format!(
                "Erro HTTP ao buscar manifest: {}",
                response.status()
            )));
        }

        let manifest: OciManifest = response
            .json()
            .await
            .map_err(|e| PolisError::Image(format!("Erro ao parsear manifest: {}", e)))?;

        Ok(manifest)
    }

    async fn fetch_config(&self, repo: &str, digest: &str) -> Result<OciConfig> {
        let url = format!("{}/{}/blobs/{}", self.base_url, repo, digest);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| PolisError::Image(format!("Erro ao buscar config: {}", e)))?;

        if !response.status().is_success() {
            return Err(PolisError::Image(format!(
                "Erro HTTP ao buscar config: {}",
                response.status()
            )));
        }

        let config: OciConfig = response
            .json()
            .await
            .map_err(|e| PolisError::Image(format!("Erro ao parsear config: {}", e)))?;

        Ok(config)
    }

    async fn download_layer(&self, repo: &str, digest: &str, path: &PathBuf) -> Result<()> {
        let url = format!("{}/{}/blobs/{}", self.base_url, repo, digest);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| PolisError::Image(format!("Erro ao baixar layer: {}", e)))?;

        if !response.status().is_success() {
            return Err(PolisError::Image(format!(
                "Erro HTTP ao baixar layer: {}",
                response.status()
            )));
        }

        let mut file = fs::File::create(path).await?;
        let bytes = response
            .bytes()
            .await
            .map_err(|e| PolisError::Image(format!("Erro ao baixar bytes: {}", e)))?;
        file.write_all(&bytes).await?;

        Ok(())
    }

    pub async fn push_image(&self, _name: &str) -> Result<()> {
        // TODO: Implement image pushing to registry
        Err(PolisError::Image("Push não implementado ainda".to_string()))
    }

    pub async fn list_images(&self) -> Result<Vec<ImageId>> {
        let mut images = Vec::new();

        if self.cache_dir.exists() {
            let mut entries = fs::read_dir(&self.cache_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_dir() {
                    let repo = entry.file_name().to_string_lossy().to_string();
                    let repo_path = entry.path();

                    if let Ok(mut tag_entries) = fs::read_dir(&repo_path).await {
                        while let Some(tag_entry) = tag_entries.next_entry().await? {
                            if tag_entry.file_type().await?.is_dir() {
                                let tag = tag_entry.file_name().to_string_lossy().to_string();
                                let image_name = format!("{}:{}", repo, tag);
                                images.push(ImageId::from_string(&image_name));
                            }
                        }
                    }
                }
            }
        }

        Ok(images)
    }
}
