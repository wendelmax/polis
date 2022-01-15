use crate::{ContainerManager, ProcessManager};
use async_trait::async_trait;
use chrono::Utc;
use polis_core::{
    log_container_created, log_container_removed, log_container_started, log_container_stopped,
    Container, ContainerId, ContainerStatus, ImageId, NetworkMode, PolisConfig, PolisError,
    ResourceLimits, Result,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait ContainerRuntime {
    async fn create_container(
        &self,
        name: String,
        image: String,
        command: Vec<String>,
    ) -> Result<ContainerId>;
    async fn start_container(&self, id: ContainerId) -> Result<()>;
    async fn stop_container(&self, id: ContainerId) -> Result<()>;
    async fn remove_container(&self, id: ContainerId) -> Result<()>;
    async fn list_containers(&self) -> Result<Vec<Container>>;
    async fn get_container(&self, id: ContainerId) -> Result<Container>;
    async fn pause_container(&self, id: ContainerId) -> Result<()>;
    async fn unpause_container(&self, id: ContainerId) -> Result<()>;
}

pub struct PolisRuntime {
    config: PolisConfig,
    containers: Arc<RwLock<HashMap<ContainerId, Container>>>,
    #[allow(dead_code)]
    container_manager: ContainerManager,
    process_manager: ProcessManager,
}

impl PolisRuntime {
    pub fn new(config: PolisConfig) -> Self {
        let containers = Arc::new(RwLock::new(HashMap::new()));
        let container_manager = ContainerManager::new(containers.clone());
        let process_manager = ProcessManager::new();

        Self {
            config,
            containers,
            container_manager,
            process_manager,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        // Criar diretórios necessários
        tokio::fs::create_dir_all(&self.config.runtime.root_dir).await?;
        tokio::fs::create_dir_all(&self.config.storage.root_dir).await?;

        // Inicializar logger (apenas se não estiver já inicializado)
        let _ = polis_core::Logger::new(
            self.config.runtime.log_level.clone(),
            Some(self.config.runtime.root_dir.join("logs")),
        )
        .init();

        Ok(())
    }
}

#[async_trait]
impl ContainerRuntime for PolisRuntime {
    async fn create_container(
        &self,
        name: String,
        image: String,
        command: Vec<String>,
    ) -> Result<ContainerId> {
        let container_id = ContainerId::new();
        let image_id = ImageId::from_string(&image);

        let container = Container {
            id: container_id.clone(),
            name: name.clone(),
            image: image_id,
            status: ContainerStatus::Created,
            created_at: Utc::now(),
            started_at: None,
            finished_at: None,
            exit_code: None,
            command,
            working_dir: PathBuf::from("/"),
            environment: HashMap::new(),
            labels: HashMap::new(),
            resource_limits: ResourceLimits::default(),
            network_mode: NetworkMode::default(),
            ports: Vec::new(),
            volumes: Vec::new(),
        };

        // Armazenar container
        {
            let mut containers = self.containers.write().await;
            containers.insert(container_id.clone(), container);
        }

        log_container_created(&container_id.0.to_string(), &name);
        Ok(container_id)
    }

    async fn start_container(&self, id: ContainerId) -> Result<()> {
        let mut container = {
            let mut containers = self.containers.write().await;
            containers
                .get_mut(&id)
                .ok_or_else(|| PolisError::Container("Container não encontrado".to_string()))?
                .clone()
        };

        if container.status != ContainerStatus::Created {
            return Err(PolisError::Container(
                "Container não está no estado correto para iniciar".to_string(),
            ));
        }

        // Atualizar status
        container.status = ContainerStatus::Running;
        container.started_at = Some(Utc::now());

        // Simular execução do processo
        let _process_id = self
            .process_manager
            .spawn(container.command.clone(), container.environment.clone())
            .await?;

        // Atualizar container no storage
        let container_name = container.name.clone();
        {
            let mut containers = self.containers.write().await;
            containers.insert(id.clone(), container);
        }

        log_container_started(&id.0.to_string(), &container_name);
        Ok(())
    }

    async fn stop_container(&self, id: ContainerId) -> Result<()> {
        let mut container = {
            let mut containers = self.containers.write().await;
            containers
                .get_mut(&id)
                .ok_or_else(|| PolisError::Container("Container não encontrado".to_string()))?
                .clone()
        };

        if container.status != ContainerStatus::Running {
            return Err(PolisError::Container(
                "Container não está rodando".to_string(),
            ));
        }

        // Parar processo
        self.process_manager.kill(123).await?; // Simular PID

        // Atualizar status
        container.status = ContainerStatus::Stopped;
        container.finished_at = Some(Utc::now());
        container.exit_code = Some(0);

        // Atualizar container no storage
        let container_name = container.name.clone();
        {
            let mut containers = self.containers.write().await;
            containers.insert(id.clone(), container);
        }

        log_container_stopped(&id.0.to_string(), &container_name, Some(0));
        Ok(())
    }

    async fn remove_container(&self, id: ContainerId) -> Result<()> {
        let container = {
            let mut containers = self.containers.write().await;
            containers
                .remove(&id)
                .ok_or_else(|| PolisError::Container("Container não encontrado".to_string()))?
        };

        if container.status == ContainerStatus::Running {
            return Err(PolisError::Container(
                "Container deve ser parado antes de ser removido".to_string(),
            ));
        }

        log_container_removed(&id.0.to_string(), &container.name);
        Ok(())
    }

    async fn list_containers(&self) -> Result<Vec<Container>> {
        let containers = self.containers.read().await;
        Ok(containers.values().cloned().collect())
    }

    async fn get_container(&self, id: ContainerId) -> Result<Container> {
        let containers = self.containers.read().await;
        containers
            .get(&id)
            .ok_or_else(|| PolisError::Container("Container não encontrado".to_string()))
            .cloned()
    }

    async fn pause_container(&self, id: ContainerId) -> Result<()> {
        let mut container = {
            let mut containers = self.containers.write().await;
            containers
                .get_mut(&id)
                .ok_or_else(|| PolisError::Container("Container não encontrado".to_string()))?
                .clone()
        };

        if container.status != ContainerStatus::Running {
            return Err(PolisError::Container(
                "Container não está rodando".to_string(),
            ));
        }

        container.status = ContainerStatus::Paused;

        {
            let mut containers = self.containers.write().await;
            containers.insert(id, container);
        }

        Ok(())
    }

    async fn unpause_container(&self, id: ContainerId) -> Result<()> {
        let mut container = {
            let mut containers = self.containers.write().await;
            containers
                .get_mut(&id)
                .ok_or_else(|| PolisError::Container("Container não encontrado".to_string()))?
                .clone()
        };

        if container.status != ContainerStatus::Paused {
            return Err(PolisError::Container(
                "Container não está pausado".to_string(),
            ));
        }

        container.status = ContainerStatus::Running;

        {
            let mut containers = self.containers.write().await;
            containers.insert(id, container);
        }

        Ok(())
    }
}
