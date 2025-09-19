use polis_core::{ImageId, Result, PolisError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

/// Image cleanup options
#[derive(Debug, Clone)]
pub struct CleanupOptions {
    pub force: bool,
    pub remove_untagged: bool,
    pub remove_dangling: bool,
    pub older_than: Option<chrono::Duration>,
    pub keep_latest: bool,
    pub dry_run: bool,
}

impl Default for CleanupOptions {
    fn default() -> Self {
        Self {
            force: false,
            remove_untagged: true,
            remove_dangling: true,
            older_than: None,
            keep_latest: true,
            dry_run: false,
        }
    }
}

/// Image cleanup statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupStats {
    pub images_removed: usize,
    pub space_freed: u64,
    pub layers_removed: usize,
    pub dangling_removed: usize,
    pub untagged_removed: usize,
}

/// Image cleanup manager
#[derive(Debug)]
pub struct ImageCleanupManager {
    pub image_dir: PathBuf,
    pub images: HashMap<ImageId, ImageInfo>,
}

#[derive(Debug, Clone)]
struct ImageInfo {
    pub id: ImageId,
    pub name: String,
    pub tag: String,
    pub size: u64,
    pub created_at: SystemTime,
    pub last_used: SystemTime,
    pub layers: Vec<String>,
    pub is_dangling: bool,
    pub is_untagged: bool,
}

impl ImageCleanupManager {
    /// Create a new image cleanup manager
    pub fn new(image_dir: PathBuf) -> Result<Self> {
        let mut manager = Self {
            image_dir,
            images: HashMap::new(),
        };

        manager.scan_images()?;
        Ok(manager)
    }

    /// Scan for existing images
    fn scan_images(&mut self) -> Result<()> {
        if !self.image_dir.exists() {
        std::fs::create_dir_all(&self.image_dir)
            .map_err(|e| PolisError::Io(e))?;
            return Ok(());
        }

        // Scan for image directories
        for entry in std::fs::read_dir(&self.image_dir)
            .map_err(|e| PolisError::Io(e))? {
            let entry = entry.map_err(|e| PolisError::Io(e))?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    let image_id = ImageId::from_string(dir_name);
                    let image_info = self.load_image_info(&path, image_id.clone())?;
                    self.images.insert(image_id, image_info);
                }
            }
        }

        Ok(())
    }

    /// Load image information from directory
    fn load_image_info(&self, path: &PathBuf, image_id: ImageId) -> Result<ImageInfo> {
        let metadata = path.metadata()
            .map_err(|e| PolisError::Io(e))?;

        let created_at = metadata.created()
            .map_err(|e| PolisError::Io(e))?;

        // Try to load image metadata
        let metadata_file = path.join("metadata.json");
        let (name, tag, layers) = if metadata_file.exists() {
            let content = std::fs::read_to_string(&metadata_file)
                .map_err(|e| PolisError::Io(e))?;
            
            let metadata: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| PolisError::Config(format!("Failed to parse metadata: {}", e)))?;
            
            let name = metadata.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            
            let tag = metadata.get("tag")
                .and_then(|v| v.as_str())
                .unwrap_or("latest")
                .to_string();
            
            let layers = metadata.get("layers")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect())
                .unwrap_or_default();
            
            (name, tag, layers)
        } else {
            ("unknown".to_string(), "latest".to_string(), Vec::new())
        };

        // Determine if image is dangling or untagged
        let is_dangling = name == "unknown" || name.is_empty();
        let is_untagged = tag == "latest" || tag.is_empty();

        Ok(ImageInfo {
            id: image_id,
            name,
            tag,
            size: self.calculate_image_size(path)?,
            created_at,
            last_used: created_at, // For now, use creation time
            layers,
            is_dangling,
            is_untagged,
        })
    }

    /// Calculate total size of an image
    fn calculate_image_size(&self, path: &PathBuf) -> Result<u64> {
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

    /// Clean up images based on options
    pub async fn cleanup_images(&mut self, options: CleanupOptions) -> Result<CleanupStats> {
        let mut stats = CleanupStats {
            images_removed: 0,
            space_freed: 0,
            layers_removed: 0,
            dangling_removed: 0,
            untagged_removed: 0,
        };

        let mut to_remove = Vec::new();

        // Find images to remove
        for (image_id, image_info) in &self.images {
            let should_remove = self.should_remove_image(image_info, &options);
            
            if should_remove {
                to_remove.push((image_id.clone(), image_info.clone()));
            }
        }

        // Remove images
        for (image_id, image_info) in &to_remove {
            if options.dry_run {
                println!("Would remove image: {} ({} bytes)", image_info.id.0, image_info.size);
                stats.images_removed += 1;
                stats.space_freed += image_info.size;
                stats.layers_removed += image_info.layers.len();
                
                if image_info.is_dangling {
                    stats.dangling_removed += 1;
                }
                if image_info.is_untagged {
                    stats.untagged_removed += 1;
                }
            } else {
                if let Err(e) = self.remove_image(image_id, image_info).await {
                    if !options.force {
                        return Err(e);
                    }
                    tracing::warn!("Failed to remove image {}: {}", image_id.0, e);
                } else {
                    stats.images_removed += 1;
                    stats.space_freed += image_info.size;
                    stats.layers_removed += image_info.layers.len();
                    
                    if image_info.is_dangling {
                        stats.dangling_removed += 1;
                    }
                    if image_info.is_untagged {
                        stats.untagged_removed += 1;
                    }
                }
            }
        }

        // Remove from in-memory cache
        if !options.dry_run {
            for (image_id, _) in to_remove {
                self.images.remove(&image_id);
            }
        }

        Ok(stats)
    }

    /// Determine if an image should be removed
    fn should_remove_image(&self, image_info: &ImageInfo, options: &CleanupOptions) -> bool {
        // Check if image is dangling
        if options.remove_dangling && image_info.is_dangling {
            return true;
        }

        // Check if image is untagged
        if options.remove_untagged && image_info.is_untagged {
            return true;
        }

        // Check age
        if let Some(older_than) = options.older_than {
            let age = SystemTime::now()
                .duration_since(image_info.created_at)
                .unwrap_or_default();
            
            if age > older_than.to_std().unwrap_or_default() {
                return true;
            }
        }

        // Keep latest images if requested
        if options.keep_latest && image_info.tag == "latest" {
            return false;
        }

        false
    }

    /// Remove a specific image
    async fn remove_image(&self, image_id: &ImageId, image_info: &ImageInfo) -> Result<()> {
        let image_path = self.image_dir.join(&image_id.0);
        
        if image_path.exists() {
            std::fs::remove_dir_all(&image_path)
                .map_err(|e| PolisError::Io(e))?;
        }

        // Remove associated layers
        for layer_id in &image_info.layers {
            let layer_path = self.image_dir.join(format!("{}.tar", layer_id));
            if layer_path.exists() {
                let _ = std::fs::remove_file(&layer_path);
            }
        }

        Ok(())
    }

    /// Get cleanup statistics
    pub fn get_cleanup_stats(&self) -> CleanupStats {
        let mut stats = CleanupStats {
            images_removed: 0,
            space_freed: 0,
            layers_removed: 0,
            dangling_removed: 0,
            untagged_removed: 0,
        };

        for image_info in self.images.values() {
            stats.space_freed += image_info.size;
            stats.layers_removed += image_info.layers.len();
            
            if image_info.is_dangling {
                stats.dangling_removed += 1;
            }
            if image_info.is_untagged {
                stats.untagged_removed += 1;
            }
        }

        stats
    }

    /// List images that would be removed
    pub fn list_cleanup_candidates(&self, options: &CleanupOptions) -> Vec<ImageInfo> {
        self.images.values()
            .filter(|image_info| self.should_remove_image(image_info, options))
            .cloned()
            .collect()
    }

    /// Get total space used by images
    pub fn get_total_space_used(&self) -> u64 {
        self.images.values().map(|info| info.size).sum()
    }

    /// Get image count
    pub fn get_image_count(&self) -> usize {
        self.images.len()
    }
}
