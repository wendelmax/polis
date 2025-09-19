use polis_core::{PolisError, Result};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Bridge {
    pub name: String,
    pub ip: IpAddr,
    pub subnet: String,
    pub mtu: u16,
    pub interfaces: Vec<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct BridgeStats {
    pub name: String,
    pub ip: IpAddr,
    pub subnet: String,
    pub interface_count: usize,
    pub enabled: bool,
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

pub struct BridgeManager {
    bridges: HashMap<String, Bridge>,
    default_bridge: String,
}

impl BridgeManager {
    pub fn new() -> Self {
        Self {
            bridges: HashMap::new(),
            default_bridge: "polis0".to_string(),
        }
    }

    pub async fn create_bridge(
        &mut self,
        name: &str,
        ip: &str,
        subnet: &str,
        mtu: u16,
    ) -> Result<()> {
        let bridge_ip =
            IpAddr::from_str(ip).map_err(|e| PolisError::Network(format!("IP inválido: {}", e)))?;

        let bridge = Bridge {
            name: name.to_string(),
            ip: bridge_ip,
            subnet: subnet.to_string(),
            mtu,
            interfaces: Vec::new(),
            enabled: true,
        };

        self.bridges.insert(name.to_string(), bridge);
        println!("� Bridge '{}' criada: {} ({})", name, ip, subnet);
        Ok(())
    }

    pub async fn delete_bridge(&mut self, name: &str) -> Result<()> {
        if self.bridges.remove(name).is_some() {
            println!("� Bridge '{}' removida", name);
            Ok(())
        } else {
            Err(PolisError::Network(format!(
                "Bridge '{}' não encontrada",
                name
            )))
        }
    }

    pub async fn add_interface(&mut self, bridge_name: &str, interface_name: &str) -> Result<()> {
        let bridge = self.bridges.get_mut(bridge_name).ok_or_else(|| {
            PolisError::Network(format!("Bridge '{}' não encontrada", bridge_name))
        })?;

        if !bridge.interfaces.contains(&interface_name.to_string()) {
            bridge.interfaces.push(interface_name.to_string());
            println!(
                "� Interface '{}' adicionada à bridge '{}'",
                interface_name, bridge_name
            );
        }

        Ok(())
    }

    pub async fn remove_interface(
        &mut self,
        bridge_name: &str,
        interface_name: &str,
    ) -> Result<()> {
        let bridge = self.bridges.get_mut(bridge_name).ok_or_else(|| {
            PolisError::Network(format!("Bridge '{}' não encontrada", bridge_name))
        })?;

        bridge.interfaces.retain(|iface| iface != interface_name);
        println!(
            "� Interface '{}' removida da bridge '{}'",
            interface_name, bridge_name
        );
        Ok(())
    }

    pub async fn enable_bridge(&mut self, name: &str) -> Result<()> {
        if let Some(bridge) = self.bridges.get_mut(name) {
            bridge.enabled = true;
            println!("� Bridge '{}' habilitada", name);
            Ok(())
        } else {
            Err(PolisError::Network(format!(
                "Bridge '{}' não encontrada",
                name
            )))
        }
    }

    pub async fn disable_bridge(&mut self, name: &str) -> Result<()> {
        if let Some(bridge) = self.bridges.get_mut(name) {
            bridge.enabled = false;
            println!("� Bridge '{}' desabilitada", name);
            Ok(())
        } else {
            Err(PolisError::Network(format!(
                "Bridge '{}' não encontrada",
                name
            )))
        }
    }

    pub async fn get_bridge(&self, name: &str) -> Result<Option<Bridge>> {
        Ok(self.bridges.get(name).cloned())
    }

    pub async fn list_bridges(&self) -> Result<Vec<Bridge>> {
        Ok(self.bridges.values().cloned().collect())
    }

    pub async fn get_bridge_stats(&self, name: &str) -> Result<BridgeStats> {
        let bridge = self
            .bridges
            .get(name)
            .ok_or_else(|| PolisError::Network(format!("Bridge '{}' não encontrada", name)))?;

        // Simulate statistics
        Ok(BridgeStats {
            name: bridge.name.clone(),
            ip: bridge.ip,
            subnet: bridge.subnet.clone(),
            interface_count: bridge.interfaces.len(),
            enabled: bridge.enabled,
            rx_packets: 1000,
            tx_packets: 1000,
            rx_bytes: 1024000,
            tx_bytes: 1024000,
        })
    }

    pub async fn create_default_bridge(&mut self) -> Result<()> {
        self.create_bridge("polis0", "172.17.0.1", "172.17.0.0/16", 1500)
            .await?;
        Ok(())
    }

    pub async fn setup_container_network(
        &mut self,
        container_id: &str,
        container_ip: IpAddr,
    ) -> Result<()> {
        let interface_name = format!("veth-{}", container_id);
        let bridge_name = self.default_bridge.clone();

        // Add interface to bridge
        self.add_interface(&bridge_name, &interface_name).await?;

        println!(
            "� Rede do container configurada: {} -> {}",
            container_id, container_ip
        );
        Ok(())
    }

    pub async fn cleanup_container_network(&mut self, container_id: &str) -> Result<()> {
        let interface_name = format!("veth-{}", container_id);
        let bridge_name = self.default_bridge.clone();

        // Remove interface from bridge
        self.remove_interface(&bridge_name, &interface_name).await?;

        println!("� Rede do container limpa: {}", container_id);
        Ok(())
    }
}

impl Default for BridgeManager {
    fn default() -> Self {
        Self::new()
    }
}
