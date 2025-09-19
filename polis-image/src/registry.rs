use polis_core::{ImageId, PolisError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use base64;
use url::Url;
use crate::RegistryConfig;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerHubToken {
    pub token: String,
    pub access_token: String,
    pub expires_in: u64,
    pub issued_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerHubAuth {
    pub username: String,
    pub password: String,
}

    pub struct RegistryClient {
        client: Client,
        base_url: String,
        cache_dir: PathBuf,
        username: Option<String>,
        password: Option<String>,
        docker_hub_token: Option<String>,
        config: RegistryConfig,
    }

impl RegistryClient {
    pub fn new(cache_dir: PathBuf) -> Self {
        let config = RegistryConfig::load().unwrap_or_default();
        
        Self {
            client: Client::new(),
            base_url: "https://registry-1.docker.io/v2".to_string(),
            cache_dir,
            username: None,
            password: None,
            docker_hub_token: None,
            config,
        }
    }

    pub fn with_auth(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.docker_hub_token = Some(token);
        self
    }

    fn get_registry_info(&self, image_name: &str) -> (String, String, String) {
        // Parse image name to extract registry, repo, and tag
        let (registry, repo, tag) = if image_name.contains('/') {
            let parts: Vec<&str> = image_name.split('/').collect();
            if parts.len() >= 2 {
                let registry_part = parts[0];
                let repo_part = parts[1..].join("/");
                
                // Check if it's a known registry
                if registry_part.contains('.') || registry_part == "quay.io" {
                    (registry_part.to_string(), repo_part, "latest".to_string())
                } else {
                    // Default to docker.io
                    ("docker.io".to_string(), image_name.to_string(), "latest".to_string())
                }
            } else {
                ("docker.io".to_string(), image_name.to_string(), "latest".to_string())
            }
        } else {
            // No slash means it's a library image on docker.io
            ("docker.io".to_string(), format!("library/{}", image_name), "latest".to_string())
        };

        // Extract tag if present
        let (repo, tag) = if repo.contains(':') {
            let parts: Vec<&str> = repo.split(':').collect();
            if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                (repo, tag)
            }
        } else {
            (repo, tag)
        };

        (registry, repo, tag)
    }

    fn get_base_url(&self, registry: &str) -> String {
        self.config.get_registry_url(registry)
            .unwrap_or_else(|| format!("https://{}/v2", registry))
    }
    
    fn get_fallback_url(&self, registry: &str) -> String {
        self.config.get_fallback_url(registry)
            .unwrap_or_else(|| format!("https://{}/v2", registry))
    }

    async fn get_docker_hub_token(&mut self, repo: &str) -> Result<String> {
        // Try to get token for public images first (no auth required)
        let auth_url = format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:{}:pull", repo);
        
        println!(" Tentando obter token do Docker Hub para: {}", repo);
        
        let response = self
            .client
            .get(&auth_url)
            .send()
            .await
            .map_err(|e| PolisError::Image(format!("Erro ao obter token: {}", e)))?;

        println!(" Status da resposta do token: {}", response.status());

        if response.status().is_success() {
            let token_response: DockerHubToken = response
                .json()
                .await
                .map_err(|e| PolisError::Image(format!("Erro ao parsear token: {}", e)))?;
            
            println!(" Token obtido com sucesso!");
            self.docker_hub_token = Some(token_response.token.clone());
            return Ok(token_response.token);
        }

        // If public access fails, try with credentials if available
        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            let auth = base64::encode(format!("{}:{}", username, password));
            let response = self
                .client
                .get(&auth_url)
                .header("Authorization", format!("Basic {}", auth))
                .send()
                .await
                .map_err(|e| PolisError::Image(format!("Erro ao obter token com auth: {}", e)))?;

            if response.status().is_success() {
                let token_response: DockerHubToken = response
                    .json()
                    .await
                    .map_err(|e| PolisError::Image(format!("Erro ao parsear token: {}", e)))?;
                
                self.docker_hub_token = Some(token_response.token.clone());
                return Ok(token_response.token);
            }
        }

        Err(PolisError::Image("Não foi possível obter token de acesso".to_string()))
    }

    pub async fn pull_image(&mut self, name: &str) -> Result<ImageId> {
        let image_id = ImageId::from_string(name);

        // Parse image name to extract registry, repo, and tag
        let (registry, repo, tag) = self.get_registry_info(name);
        
        println!(" Registry: {}, Repo: {}, Tag: {}", registry, repo, tag);

        // Create cache directory for this image
        let image_cache_dir = self.cache_dir.join(&repo).join(&tag);
        fs::create_dir_all(&image_cache_dir).await?;

        // Get the appropriate base URL (with mirror support)
        let base_url = self.get_base_url(&registry);
        println!(" Usando registry: {}", base_url);

        // Use the provided Docker Hub token directly
        if let Some(token) = &self.docker_hub_token {
            println!(" Usando token fornecido: {}...", &token[..20]);
        } else {
            // Fallback: try to get token from Docker Hub API
            let token = self.get_docker_hub_token(&repo).await;
            if let Ok(token) = token {
                println!(" Usando token da API: {}...", &token[..20]);
            }
        }

        // Try to fetch from registry first
        match self.fetch_manifest_with_url(&base_url, &repo, &tag).await {
            Ok(manifest) => {
                // Save manifest
                let manifest_path = image_cache_dir.join("manifest.json");
                let manifest_json = serde_json::to_string_pretty(&manifest)?;
                fs::write(&manifest_path, manifest_json).await?;

                // Download config
                let config = self.fetch_config_with_url(&base_url, &repo, &manifest.config.digest).await?;
                let config_path = image_cache_dir.join("config.json");
                let config_json = serde_json::to_string_pretty(&config)?;
                fs::write(&config_path, config_json).await?;

                // Download layers
                for (i, layer) in manifest.layers.iter().enumerate() {
                    let layer_path = image_cache_dir.join(format!("layer_{}.tar.gz", i));
                    self.download_layer_with_url(&base_url, &repo, &layer.digest, &layer_path)
                        .await?;
                }

                println!(" Imagem '{}' baixada com sucesso do registry {}", name, registry);
            }
            Err(e) => {
                println!(" Aviso: Não foi possível baixar '{}' do registry {}: {}", name, registry, e);
                
                // Try fallback registry if available
                if let Some(fallback_url) = self.config.get_fallback_url(&registry) {
                    if fallback_url != base_url {
                        println!(" Tentando fallback para registry principal...");
                        match self.fetch_manifest_with_url(&fallback_url, &repo, &tag).await {
                            Ok(manifest) => {
                                println!(" Sucesso com registry principal!");
                                // Process manifest...
                                let manifest_path = image_cache_dir.join("manifest.json");
                                let manifest_json = serde_json::to_string_pretty(&manifest)?;
                                fs::write(&manifest_path, manifest_json).await?;
                                
                                // Download config and layers
                                let config = self.fetch_config_with_url(&fallback_url, &repo, &manifest.config.digest).await?;
                                let config_path = image_cache_dir.join("config.json");
                                let config_json = serde_json::to_string_pretty(&config)?;
                                fs::write(&config_path, config_json).await?;

                                for (i, layer) in manifest.layers.iter().enumerate() {
                                    let layer_path = image_cache_dir.join(format!("layer_{}.tar.gz", i));
                                    self.download_layer_with_url(&fallback_url, &repo, &layer.digest, &layer_path)
                                        .await?;
                                }
                                
                                println!(" Imagem '{}' baixada com sucesso do registry principal {}", name, registry);
                            }
                            Err(_) => {
                                println!(" Criando imagem local de exemplo...");
                                self.create_local_image(&repo, &tag, &image_cache_dir).await?;
                            }
                        }
                    } else {
                        println!(" Criando imagem local de exemplo...");
                        self.create_local_image(&repo, &tag, &image_cache_dir).await?;
                    }
                } else {
                    println!(" Criando imagem local de exemplo...");
                    self.create_local_image(&repo, &tag, &image_cache_dir).await?;
                }
            }
        }

        Ok(image_id)
    }

    async fn fetch_manifest(&self, repo: &str, tag: &str) -> Result<OciManifest> {
        self.fetch_manifest_with_url(&self.base_url, repo, tag).await
    }

    async fn fetch_manifest_with_url(&self, base_url: &str, repo: &str, tag: &str) -> Result<OciManifest> {
        let url = format!("{}/{}/manifests/{}", base_url, repo, tag);

        let mut request = self
            .client
            .get(&url)
            .header("User-Agent", "polis/0.1.0")
            .header("Accept", "application/vnd.docker.distribution.manifest.v2+json")
            .header("Accept", "application/vnd.oci.image.manifest.v1+json");

        // Add Docker Hub token if available
        if let Some(token) = &self.docker_hub_token {
            // Try Bearer token first
            request = request.header("Authorization", format!("Bearer {}", token));
        } else if let (Some(username), Some(password)) = (&self.username, &self.password) {
            // Fallback to basic auth
            let auth = base64::encode(format!("{}:{}", username, password));
            request = request.header("Authorization", format!("Basic {}", auth));
        }

        let response = request
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
        self.fetch_config_with_url(&self.base_url, repo, digest).await
    }

    async fn fetch_config_with_url(&self, base_url: &str, repo: &str, digest: &str) -> Result<OciConfig> {
        let url = format!("{}/{}/blobs/{}", base_url, repo, digest);

        let mut request = self
            .client
            .get(&url)
            .header("User-Agent", "polis/0.1.0");

        // Add authentication if available
        if let Some(token) = &self.docker_hub_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        } else if let (Some(username), Some(password)) = (&self.username, &self.password) {
            let auth = base64::encode(format!("{}:{}", username, password));
            request = request.header("Authorization", format!("Basic {}", auth));
        }

        let response = request
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
        self.download_layer_with_url(&self.base_url, repo, digest, path).await
    }

    async fn download_layer_with_url(&self, base_url: &str, repo: &str, digest: &str, path: &PathBuf) -> Result<()> {
        let url = format!("{}/{}/blobs/{}", base_url, repo, digest);

        let mut request = self
            .client
            .get(&url)
            .header("User-Agent", "polis/0.1.0");

        // Add authentication if available
        if let Some(token) = &self.docker_hub_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        } else if let (Some(username), Some(password)) = (&self.username, &self.password) {
            let auth = base64::encode(format!("{}:{}", username, password));
            request = request.header("Authorization", format!("Basic {}", auth));
        }

        let response = request
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

    async fn create_local_image(&self, repo: &str, tag: &str, image_cache_dir: &PathBuf) -> Result<()> {
        // Create a simple local manifest
        let manifest = OciManifest {
            schema_version: 2,
            media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
            config: OciDescriptor {
                media_type: "application/vnd.oci.image.config.v1+json".to_string(),
                size: 1024,
                digest: "sha256:local-example".to_string(),
                annotations: None,
                urls: None,
            },
            layers: vec![OciDescriptor {
                media_type: "application/vnd.oci.image.layer.v1.tar+gzip".to_string(),
                size: 2048,
                digest: "sha256:local-layer".to_string(),
                annotations: None,
                urls: None,
            }],
            annotations: None,
        };

        // Save manifest
        let manifest_path = image_cache_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, manifest_json).await?;

        // Create a simple config
        let config = OciConfig {
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
            config: OciImageConfig {
                user: Some("root".to_string()),
                exposed_ports: None,
                env: Some(vec!["PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string()]),
                entrypoint: Some(vec!["/bin/sh".to_string()]),
                cmd: Some(vec!["-c".to_string()]),
                volumes: None,
                working_dir: Some("/".to_string()),
                labels: Some(std::collections::HashMap::new()),
            },
            rootfs: OciRootFs {
                r#type: "layers".to_string(),
                diff_ids: vec!["sha256:local-layer".to_string()],
            },
        };

        // Save config
        let config_path = image_cache_dir.join("config.json");
        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, config_json).await?;

        // Create a dummy layer file
        let layer_path = image_cache_dir.join("layer_0.tar.gz");
        fs::write(&layer_path, b"dummy layer content").await?;

        println!(" Imagem local '{}:{}' criada com sucesso", repo, tag);
        Ok(())
    }
}
