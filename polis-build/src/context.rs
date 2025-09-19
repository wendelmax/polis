use crate::{BuildError, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Build context for container image building
#[derive(Debug, Clone)]
pub struct BuildContext {
    pub path: PathBuf,
    pub files: HashMap<String, PathBuf>,
    pub dockerfile: Option<PathBuf>,
    pub dockerignore: Option<PathBuf>,
    pub size: u64,
}

impl BuildContext {
    /// Create a new build context from a directory
    pub fn new(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(BuildError::Context(format!("Path does not exist: {:?}", path)));
        }

        if !path.is_dir() {
            return Err(BuildError::Context(format!("Path is not a directory: {:?}", path)));
        }

        let mut context = Self {
            path: path.clone(),
            files: HashMap::new(),
            dockerfile: None,
            dockerignore: None,
            size: 0,
        };

        context.scan_directory()?;
        Ok(context)
    }

    /// Scan the directory for files
    fn scan_directory(&mut self) -> Result<()> {
        let ignore_patterns = self.load_dockerignore()?;

        for entry in WalkDir::new(&self.path) {
            let entry = entry.map_err(|e| BuildError::Io(e.into()))?;
            let path = entry.path();
            let relative_path = path.strip_prefix(&self.path)
                .map_err(|e| BuildError::Context(format!("Failed to strip prefix: {}", e)))?;

            // Skip if matches ignore patterns
            if self.should_ignore(relative_path, &ignore_patterns) {
                continue;
            }

            if path.is_file() {
                let relative_str = relative_path.to_string_lossy().to_string();
                self.files.insert(relative_str, path.to_path_buf());
                
                // Add file size
                if let Ok(metadata) = path.metadata() {
                    self.size += metadata.len();
                }
            }

            // Check for Dockerfile
            if path.file_name().and_then(|n| n.to_str()) == Some("Dockerfile") {
                self.dockerfile = Some(path.to_path_buf());
            }

            // Check for .dockerignore
            if path.file_name().and_then(|n| n.to_str()) == Some(".dockerignore") {
                self.dockerignore = Some(path.to_path_buf());
            }
        }

        Ok(())
    }

    /// Load .dockerignore patterns
    fn load_dockerignore(&self) -> Result<Vec<String>> {
        let mut patterns = Vec::new();
        
        if let Some(dockerignore_path) = &self.dockerignore {
            let content = std::fs::read_to_string(dockerignore_path)
                .map_err(|e| BuildError::Io(e))?;
            
            for line in content.lines() {
                let line = line.trim();
                if !line.is_empty() && !line.starts_with('#') {
                    patterns.push(line.to_string());
                }
            }
        }

        // Add default ignore patterns
        patterns.extend(vec![
            ".git".to_string(),
            ".gitignore".to_string(),
            ".dockerignore".to_string(),
            "target/".to_string(),
            "node_modules/".to_string(),
            ".DS_Store".to_string(),
            "Thumbs.db".to_string(),
        ]);

        Ok(patterns)
    }

    /// Check if a path should be ignored
    fn should_ignore(&self, path: &Path, patterns: &[String]) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in patterns {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }
        
        false
    }

    /// Check if a path matches a pattern
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Simple pattern matching (can be enhanced with proper glob matching)
        if pattern.ends_with('/') {
            // Directory pattern
            path.starts_with(pattern) || path.contains(&format!("{}/", pattern.trim_end_matches('/')))
        } else if pattern.starts_with('*') {
            // Wildcard pattern
            path.ends_with(pattern.trim_start_matches('*'))
        } else {
            // Exact match
            path == pattern || path.starts_with(&format!("{}/", pattern))
        }
    }

    /// Get the Dockerfile path
    pub fn get_dockerfile(&self) -> Option<&PathBuf> {
        self.dockerfile.as_ref()
    }

    /// Get all files in the context
    pub fn get_files(&self) -> &HashMap<String, PathBuf> {
        &self.files
    }

    /// Get the total size of the context
    pub fn get_size(&self) -> u64 {
        self.size
    }

    /// Create a tar archive of the build context
    pub fn create_tar_archive(&self) -> Result<Vec<u8>> {
        let mut tar_builder = tar::Builder::new(Vec::new());
        
        for (relative_path, absolute_path) in &self.files {
            tar_builder.append_path_with_name(absolute_path, relative_path)
                .map_err(|e| BuildError::Io(e))?;
        }
        
        tar_builder.finish().map_err(|e| BuildError::Io(e))?;
        Ok(tar_builder.into_inner().map_err(|e| BuildError::Io(e))?)
    }

    /// Get file count
    pub fn get_file_count(&self) -> usize {
        self.files.len()
    }

    /// Check if context is valid for building
    pub fn is_valid(&self) -> bool {
        self.dockerfile.is_some() && !self.files.is_empty()
    }
}
