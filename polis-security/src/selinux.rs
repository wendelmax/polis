use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SELinuxContext {
    pub user: String,
    pub role: String,
    pub r#type: String,
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SELinuxPolicy {
    pub name: String,
    pub rules: Vec<String>,
    pub context: SELinuxContext,
}

#[derive(Debug, Clone)]
pub struct SELinuxManager {
    pub policies: std::collections::HashMap<String, SELinuxPolicy>,
}

impl SELinuxManager {
    pub fn new() -> Self {
        Self {
            policies: std::collections::HashMap::new(),
        }
    }

    pub async fn is_available(&self) -> bool {
        // Verificar se o SELinux está disponível e habilitado
        Command::new("getenforce")
            .output()
            .map(|output| {
                let status = String::from_utf8_lossy(&output.stdout);
                status.trim() == "Enforcing" || status.trim() == "Permissive"
            })
            .unwrap_or(false)
    }

    pub async fn get_status(&self) -> Result<SELinuxStatus> {
        if !self.is_available().await {
            return Ok(SELinuxStatus {
                available: false,
                mode: "disabled".to_string(),
                context: None,
            });
        }

        let output = Command::new("getenforce")
            .output()
            .map_err(|e| PolisError::Security(format!("Erro ao obter status do SELinux: {}", e)))?;

        let mode = String::from_utf8_lossy(&output.stdout).trim().to_string();

        let context = self.get_current_context().await?;

        Ok(SELinuxStatus {
            available: true,
            mode,
            context: Some(context),
        })
    }

    pub async fn get_current_context(&self) -> Result<SELinuxContext> {
        let output = Command::new("id")
            .arg("-Z")
            .output()
            .map_err(|e| PolisError::Security(format!("Erro ao obter contexto SELinux: {}", e)))?;

        if !output.status.success() {
            return Err(PolisError::Security(
                "Erro ao obter contexto SELinux".to_string(),
            ));
        }

        let context_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        self.parse_context(&context_str)
    }

    pub async fn create_policy(
        &mut self,
        name: String,
        rules: Vec<String>,
        context: SELinuxContext,
    ) -> Result<SELinuxPolicy> {
        if !self.is_available().await {
            return Err(PolisError::Security(
                "SELinux não está disponível no sistema".to_string(),
            ));
        }

        let policy = SELinuxPolicy {
            name: name.clone(),
            rules,
            context,
        };

        self.policies.insert(name, policy.clone());
        Ok(policy)
    }

    pub async fn apply_context_to_file(
        &self,
        file_path: &str,
        context: &SELinuxContext,
    ) -> Result<()> {
        if !self.is_available().await {
            return Err(PolisError::Security(
                "SELinux não está disponível no sistema".to_string(),
            ));
        }

        let _context_str = format!(
            "{}:{}:{}:{}",
            context.user, context.role, context.r#type, context.level
        );

        let output = Command::new("chcon")
            .arg("-t")
            .arg(&context.r#type)
            .arg(file_path)
            .output()
            .map_err(|e| {
                PolisError::Security(format!("Erro ao aplicar contexto SELinux: {}", e))
            })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PolisError::Security(format!(
                "Erro ao aplicar contexto: {}",
                error
            )));
        }

        Ok(())
    }

    pub async fn apply_context_to_process(
        &self,
        _pid: u32,
        context: &SELinuxContext,
    ) -> Result<()> {
        if !self.is_available().await {
            return Err(PolisError::Security(
                "SELinux não está disponível no sistema".to_string(),
            ));
        }

        let _context_str = format!(
            "{}:{}:{}:{}",
            context.user, context.role, context.r#type, context.level
        );

        let output = Command::new("runcon")
            .arg("-t")
            .arg(&context.r#type)
            .arg("-r")
            .arg(&context.role)
            .arg("--")
            .arg("sleep")
            .arg("1")
            .output()
            .map_err(|e| {
                PolisError::Security(format!("Erro ao aplicar contexto ao processo: {}", e))
            })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PolisError::Security(format!(
                "Erro ao aplicar contexto: {}",
                error
            )));
        }

        Ok(())
    }

    pub async fn create_container_context(&self, container_id: &str) -> Result<SELinuxContext> {
        Ok(SELinuxContext {
            user: "system_u".to_string(),
            role: "system_r".to_string(),
            r#type: format!("polis_container_t_{}", container_id),
            level: "s0".to_string(),
        })
    }

    pub async fn create_container_policy(&self, container_id: &str) -> Result<SELinuxPolicy> {
        let context = self.create_container_context(container_id).await?;

        let rules = vec![
            format!("allow polis_container_t_{} self:capability {{ setuid setgid dac_override dac_read_search fowner fsetid kill setpcap net_bind_service net_raw sys_chroot mknod audit_write setfcap }};", container_id),
            format!("allow polis_container_t_{} self:process {{ transition signal_perms getattr getcap setcap setexec setfscreate setkeycreate setrlimit setpgid setsession setuid setgid setgroups }};", container_id),
            format!("allow polis_container_t_{} self:file {{ read write create getattr setattr lock append unlink link rename execute execute_no_trans entrypoint open }};", container_id),
            format!("allow polis_container_t_{} self:dir {{ read write create getattr setattr lock add_name remove_name reparent search rmdir open }};", container_id),
            format!("allow polis_container_t_{} self:lnk_file {{ read write create getattr setattr lock append unlink link rename }};", container_id),
            format!("allow polis_container_t_{} self:fifo_file {{ read write create getattr setattr lock append unlink link rename }};", container_id),
            format!("allow polis_container_t_{} self:sock_file {{ read write create getattr setattr lock append unlink link rename }};", container_id),
            format!("allow polis_container_t_{} self:unix_stream_socket {{ create connect write read getattr setattr lock append bind listen accept getopt setopt shutdown }};", container_id),
            format!("allow polis_container_t_{} self:unix_dgram_socket {{ create connect write read getattr setattr lock append bind getopt setopt shutdown }};", container_id),
            format!("allow polis_container_t_{} self:tcp_socket {{ create connect write read getattr setattr lock append bind listen accept getopt setopt shutdown }};", container_id),
            format!("allow polis_container_t_{} self:udp_socket {{ create connect write read getattr setattr lock append bind getopt setopt shutdown }};", container_id),
            format!("allow polis_container_t_{} self:rawip_socket {{ create connect write read getattr setattr lock append bind getopt setopt shutdown }};", container_id),
            format!("allow polis_container_t_{} self:packet_socket {{ create connect write read getattr setattr lock append bind getopt setopt shutdown }};", container_id),
            format!("allow polis_container_t_{} self:key {{ search }};", container_id),
            format!("allow polis_container_t_{} self:shm {{ create read write destroy getattr setattr lock associate unlink lock }};", container_id),
            format!("allow polis_container_t_{} self:sem {{ create read write destroy getattr setattr lock associate unlink lock }};", container_id),
            format!("allow polis_container_t_{} self:msg {{ create read write destroy getattr setattr lock associate unlink lock }};", container_id),
            format!("allow polis_container_t_{} self:msgq {{ create read write destroy getattr setattr lock associate unlink lock }};", container_id),
            format!("allow polis_container_t_{} self:msg {{{{ create read write destroy getattr setattr lock associate unlink lock }}}};", container_id),
            format!("allow polis_container_t_{} self:msgq {{{{ create read write destroy getattr setattr lock associate unlink lock }}}};", container_id),
        ];

        Ok(SELinuxPolicy {
            name: format!("polis-container-{}", container_id),
            rules,
            context,
        })
    }

    pub async fn set_boolean(&self, name: &str, value: bool) -> Result<()> {
        if !self.is_available().await {
            return Err(PolisError::Security(
                "SELinux não está disponível no sistema".to_string(),
            ));
        }

        let value_str = if value { "1" } else { "0" };

        let output = Command::new("setsebool")
            .arg("-P")
            .arg(name)
            .arg(value_str)
            .output()
            .map_err(|e| PolisError::Security(format!("Erro ao definir boolean SELinux: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PolisError::Security(format!(
                "Erro ao definir boolean: {}",
                error
            )));
        }

        Ok(())
    }

    pub async fn get_boolean(&self, name: &str) -> Result<bool> {
        if !self.is_available().await {
            return Err(PolisError::Security(
                "SELinux não está disponível no sistema".to_string(),
            ));
        }

        let output = Command::new("getsebool")
            .arg(name)
            .output()
            .map_err(|e| PolisError::Security(format!("Erro ao obter boolean SELinux: {}", e)))?;

        if !output.status.success() {
            return Err(PolisError::Security(
                "Erro ao obter boolean SELinux".to_string(),
            ));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(output_str.contains("on"))
    }

    pub fn parse_context(&self, context_str: &str) -> Result<SELinuxContext> {
        let parts: Vec<&str> = context_str.split(':').collect();
        if parts.len() != 4 {
            return Err(PolisError::Security(
                "Formato de contexto SELinux inválido".to_string(),
            ));
        }

        Ok(SELinuxContext {
            user: parts[0].to_string(),
            role: parts[1].to_string(),
            r#type: parts[2].to_string(),
            level: parts[3].to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SELinuxStatus {
    pub available: bool,
    pub mode: String,
    pub context: Option<SELinuxContext>,
}
