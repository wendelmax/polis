use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppArmorProfile {
    pub name: String,
    pub rules: Vec<String>,
    pub mode: AppArmorMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppArmorMode {
    Enforce,
    Complain,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct AppArmorManager {
    pub profiles: std::collections::HashMap<String, AppArmorProfile>,
}

impl AppArmorManager {
    pub fn new() -> Self {
        Self {
            profiles: std::collections::HashMap::new(),
        }
    }

    pub async fn is_available(&self) -> bool {
        // Verificar se o AppArmor está disponível no sistema
        Command::new("aa-status")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub async fn create_profile(
        &mut self,
        name: String,
        rules: Vec<String>,
    ) -> Result<AppArmorProfile> {
        if !self.is_available().await {
            return Err(PolisError::Security(
                "AppArmor não está disponível no sistema".to_string(),
            ));
        }

        let profile = AppArmorProfile {
            name: name.clone(),
            rules,
            mode: AppArmorMode::Complain, // Modo seguro por padrão
        };

        self.profiles.insert(name.clone(), profile.clone());
        Ok(profile)
    }

    pub async fn load_profile(&self, profile: &AppArmorProfile) -> Result<()> {
        if !self.is_available().await {
            return Err(PolisError::Security(
                "AppArmor não está disponível no sistema".to_string(),
            ));
        }

        // Criar arquivo de perfil temporário
        let temp_file = format!("/tmp/polis-{}.conf", profile.name);
        let profile_content = self.generate_profile_content(profile);

        std::fs::write(&temp_file, profile_content)
            .map_err(|e| PolisError::Security(format!("Erro ao escrever perfil: {}", e)))?;

        // Carregar perfil no AppArmor
        let output = Command::new("apparmor_parser")
            .arg("-r")
            .arg(&temp_file)
            .output()
            .map_err(|e| {
                PolisError::Security(format!("Erro ao executar apparmor_parser: {}", e))
            })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PolisError::Security(format!(
                "Erro ao carregar perfil: {}",
                error
            )));
        }

        // Limpar arquivo temporário
        let _ = std::fs::remove_file(&temp_file);

        Ok(())
    }

    pub async fn unload_profile(&self, profile_name: &str) -> Result<()> {
        if !self.is_available().await {
            return Err(PolisError::Security(
                "AppArmor não está disponível no sistema".to_string(),
            ));
        }

        let output = Command::new("apparmor_parser")
            .arg("-R")
            .arg(&format!("polis-{}", profile_name))
            .output()
            .map_err(|e| {
                PolisError::Security(format!("Erro ao executar apparmor_parser: {}", e))
            })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PolisError::Security(format!(
                "Erro ao descarregar perfil: {}",
                error
            )));
        }

        Ok(())
    }

    pub async fn set_profile_mode(&self, profile_name: &str, mode: AppArmorMode) -> Result<()> {
        if !self.is_available().await {
            return Err(PolisError::Security(
                "AppArmor não está disponível no sistema".to_string(),
            ));
        }

        let mode_str = match mode {
            AppArmorMode::Enforce => "enforce",
            AppArmorMode::Complain => "complain",
            AppArmorMode::Disabled => "disable",
        };

        let output = Command::new("aa-enforce")
            .arg("-d")
            .arg(&format!("polis-{}", profile_name))
            .arg(mode_str)
            .output()
            .map_err(|e| PolisError::Security(format!("Erro ao alterar modo do perfil: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PolisError::Security(format!(
                "Erro ao alterar modo: {}",
                error
            )));
        }

        Ok(())
    }

    pub async fn apply_to_process(&self, _pid: u32, profile_name: &str) -> Result<()> {
        if !self.is_available().await {
            return Err(PolisError::Security(
                "AppArmor não está disponível no sistema".to_string(),
            ));
        }

        let output = Command::new("aa-exec")
            .arg("-p")
            .arg(&format!("polis-{}", profile_name))
            .arg("--")
            .arg("sleep")
            .arg("1")
            .output()
            .map_err(|e| {
                PolisError::Security(format!("Erro ao aplicar perfil ao processo: {}", e))
            })?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(PolisError::Security(format!(
                "Erro ao aplicar perfil: {}",
                error
            )));
        }

        Ok(())
    }

    pub async fn get_status(&self) -> Result<AppArmorStatus> {
        if !self.is_available().await {
            return Ok(AppArmorStatus {
                available: false,
                profiles: Vec::new(),
                mode: "disabled".to_string(),
            });
        }

        let output = Command::new("aa-status").output().map_err(|e| {
            PolisError::Security(format!("Erro ao obter status do AppArmor: {}", e))
        })?;

        if !output.status.success() {
            return Err(PolisError::Security(
                "Erro ao obter status do AppArmor".to_string(),
            ));
        }

        let status_output = String::from_utf8_lossy(&output.stdout);
        let profiles = self.parse_status_output(&status_output);

        Ok(AppArmorStatus {
            available: true,
            profiles,
            mode: "enforce".to_string(), // Simplificado
        })
    }

    fn generate_profile_content(&self, profile: &AppArmorProfile) -> String {
        let mut content = format!("#include <tunables/global>\n\n");
        content.push_str(&format!("polis-{} {{\n", profile.name));

        for rule in &profile.rules {
            content.push_str(&format!("  {}\n", rule));
        }

        content.push_str("}\n");
        content
    }

    fn parse_status_output(&self, output: &str) -> Vec<String> {
        output
            .lines()
            .filter(|line| line.contains("polis-"))
            .map(|line| line.trim().to_string())
            .collect()
    }

    pub async fn create_container_profile(&self, container_id: &str) -> Result<AppArmorProfile> {
        let rules = vec![
            "capability,".to_string(),
            "network,".to_string(),
            "mount,".to_string(),
            "umount,".to_string(),
            "signal,".to_string(),
            "ptrace,".to_string(),
            "file,".to_string(),
            "deny /proc/sys/kernel/core_pattern w,".to_string(),
            "deny /proc/sysrq-trigger w,".to_string(),
            "deny /proc/sys/kernel/hostname w,".to_string(),
            "deny /proc/sys/kernel/domainname w,".to_string(),
            "deny /proc/sys/kernel/modprobe w,".to_string(),
            "deny /proc/sys/kernel/modules_disabled w,".to_string(),
            "deny /proc/sys/kernel/tainted w,".to_string(),
            "deny /proc/sys/kernel/randomize_va_space w,".to_string(),
            "deny /proc/sys/kernel/kptr_restrict w,".to_string(),
            "deny /proc/sys/kernel/kexec_load_disabled w,".to_string(),
            "deny /proc/sys/kernel/perf_event_paranoid w,".to_string(),
            "deny /proc/sys/kernel/unprivileged_bpf_disabled w,".to_string(),
            "deny /proc/sys/kernel/sysrq w,".to_string(),
            "deny /proc/sys/kernel/printk w,".to_string(),
            "deny /proc/sys/kernel/dmesg_restrict w,".to_string(),
            "deny /proc/sys/kernel/panic_on_oops w,".to_string(),
            "deny /proc/sys/kernel/panic w,".to_string(),
            "deny /proc/sys/kernel/panic_on_warn w,".to_string(),
            "deny /proc/sys/kernel/panic_on_rcu_stall w,".to_string(),
            "deny /proc/sys/kernel/panic_on_io_nmi w,".to_string(),
            "deny /proc/sys/kernel/panic_on_unrecovered_nmi w,".to_string(),
            "deny /proc/sys/kernel/panic_on_stackoverflow w,".to_string(),
            "deny /proc/sys/kernel/panic_on_warn w,".to_string(),
            "deny /proc/sys/kernel/panic_on_rcu_stall w,".to_string(),
            "deny /proc/sys/kernel/panic_on_io_nmi w,".to_string(),
            "deny /proc/sys/kernel/panic_on_unrecovered_nmi w,".to_string(),
            "deny /proc/sys/kernel/panic_on_stackoverflow w,".to_string(),
        ];

        Ok(AppArmorProfile {
            name: format!("container-{}", container_id),
            rules,
            mode: AppArmorMode::Enforce,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppArmorStatus {
    pub available: bool,
    pub profiles: Vec<String>,
    pub mode: String,
}
