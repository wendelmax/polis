use polis_core::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<u32, ProcessInfo>>>,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub environment: HashMap<String, String>,
    pub status: ProcessStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessStatus {
    Running,
    Stopped,
    Exited,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn spawn(
        &self,
        command: Vec<String>,
        environment: HashMap<String, String>,
    ) -> Result<u32> {
        if command.is_empty() {
            return Err(polis_core::PolisError::Runtime("Comando vazio".to_string()));
        }

        // Para testes, simular spawn em vez de executar comandos reais
        let pid = std::process::id() + command.len() as u32;

        // Armazenar informações do processo
        let process_info = ProcessInfo {
            pid,
            command: command[0].clone(),
            args: command[1..].to_vec(),
            working_dir: PathBuf::from("/"),
            environment,
            status: ProcessStatus::Running,
        };

        {
            let mut processes = self.processes.write().await;
            processes.insert(pid, process_info);
        }

        Ok(pid)
    }

    pub async fn kill(&self, pid: u32) -> Result<()> {
        // Simular kill do processo
        {
            let mut processes = self.processes.write().await;
            if let Some(process_info) = processes.get_mut(&pid) {
                process_info.status = ProcessStatus::Stopped;
            }
        }

        // Em um sistema real, aqui faríamos o kill do processo
        // Por enquanto, apenas simulamos
        Ok(())
    }

    pub async fn get_process(&self, pid: u32) -> Result<ProcessInfo> {
        let processes = self.processes.read().await;
        processes
            .get(&pid)
            .ok_or_else(|| polis_core::PolisError::Runtime("Processo não encontrado".to_string()))
            .cloned()
    }

    pub async fn list_processes(&self) -> Result<Vec<ProcessInfo>> {
        let processes = self.processes.read().await;
        Ok(processes.values().cloned().collect())
    }

    pub async fn wait_for_process(&self, _pid: u32) -> Result<i32> {
        // Simular wait do processo
        // Em um sistema real, aqui faríamos o wait do processo
        Ok(0)
    }
}
