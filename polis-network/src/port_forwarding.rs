use polis_core::{PolisError, Result};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
    Both,
}

#[derive(Debug, Clone)]
pub struct PortForwardingRule {
    pub id: String,
    pub host_ip: IpAddr,
    pub host_port: u16,
    pub container_ip: IpAddr,
    pub container_port: u16,
    pub protocol: Protocol,
    pub enabled: bool,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PortForwardingStats {
    pub total_rules: usize,
    pub active_rules: usize,
    pub inactive_rules: usize,
    pub tcp_rules: usize,
    pub udp_rules: usize,
}

pub struct PortForwardingManager {
    rules: HashMap<String, PortForwardingRule>,
    next_id: u32,
}

impl PortForwardingManager {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            next_id: 1,
        }
    }

    pub async fn add_rule(
        &mut self,
        host_ip: IpAddr,
        host_port: u16,
        container_ip: IpAddr,
        container_port: u16,
        protocol: Protocol,
        comment: Option<String>,
    ) -> Result<String> {
        let rule_id = format!("pf-{}", self.next_id);
        self.next_id += 1;

        let rule = PortForwardingRule {
            id: rule_id.clone(),
            host_ip,
            host_port,
            container_ip,
            container_port,
            protocol: protocol.clone(),
            enabled: true,
            comment,
        };

        // Check for conflicts
        if self
            .has_port_conflict(host_ip, host_port, &protocol)
            .await?
        {
            return Err(PolisError::Network(format!(
                "Conflito de porta: {}:{} já está em uso",
                host_ip, host_port
            )));
        }

        self.rules.insert(rule_id.clone(), rule.clone());
        println!(
            "🌐 Port forwarding criado: {}:{} -> {}:{} ({:?})",
            host_ip, host_port, container_ip, container_port, protocol
        );

        Ok(rule_id)
    }

    pub async fn remove_rule(&mut self, rule_id: &str) -> Result<()> {
        if self.rules.remove(rule_id).is_some() {
            println!("🌐 Port forwarding removido: {}", rule_id);
            Ok(())
        } else {
            Err(PolisError::Network(format!(
                "Regra '{}' não encontrada",
                rule_id
            )))
        }
    }

    pub async fn enable_rule(&mut self, rule_id: &str) -> Result<()> {
        if let Some(rule) = self.rules.get_mut(rule_id) {
            rule.enabled = true;
            println!("🌐 Port forwarding habilitado: {}", rule_id);
            Ok(())
        } else {
            Err(PolisError::Network(format!(
                "Regra '{}' não encontrada",
                rule_id
            )))
        }
    }

    pub async fn disable_rule(&mut self, rule_id: &str) -> Result<()> {
        if let Some(rule) = self.rules.get_mut(rule_id) {
            rule.enabled = false;
            println!("🌐 Port forwarding desabilitado: {}", rule_id);
            Ok(())
        } else {
            Err(PolisError::Network(format!(
                "Regra '{}' não encontrada",
                rule_id
            )))
        }
    }

    pub async fn get_rule(&self, rule_id: &str) -> Result<Option<PortForwardingRule>> {
        Ok(self.rules.get(rule_id).cloned())
    }

    pub async fn list_rules(&self) -> Result<Vec<PortForwardingRule>> {
        Ok(self.rules.values().cloned().collect())
    }

    pub async fn list_rules_for_container(
        &self,
        container_ip: IpAddr,
    ) -> Result<Vec<PortForwardingRule>> {
        let container_rules: Vec<PortForwardingRule> = self
            .rules
            .values()
            .filter(|rule| rule.container_ip == container_ip)
            .cloned()
            .collect();

        Ok(container_rules)
    }

    pub async fn create_container_forwarding(
        &mut self,
        container_ip: IpAddr,
        container_port: u16,
        host_port: Option<u16>,
        protocol: Protocol,
    ) -> Result<String> {
        let host_ip = Ipv4Addr::new(0, 0, 0, 0); // Listen on all interfaces
        let host_port = host_port.unwrap_or(container_port);

        let comment = Some(format!(
            "Container forwarding: {}:{}",
            container_ip, container_port
        ));

        self.add_rule(
            host_ip.into(),
            host_port,
            container_ip,
            container_port,
            protocol,
            comment,
        )
        .await
    }

    pub async fn create_range_forwarding(
        &mut self,
        host_ip: IpAddr,
        host_start_port: u16,
        host_end_port: u16,
        container_ip: IpAddr,
        container_start_port: u16,
        protocol: Protocol,
    ) -> Result<Vec<String>> {
        if host_start_port > host_end_port {
            return Err(PolisError::Network(
                "Porta inicial deve ser menor que a final".to_string(),
            ));
        }

        let mut rule_ids = Vec::new();
        let port_count = host_end_port - host_start_port + 1;

        for i in 0..port_count {
            let host_port = host_start_port + i;
            let container_port = container_start_port + i;

            let comment = Some(format!(
                "Range forwarding: {}:{} -> {}:{}",
                host_ip, host_port, container_ip, container_port
            ));

            let rule_id = self
                .add_rule(
                    host_ip,
                    host_port,
                    container_ip,
                    container_port,
                    protocol.clone(),
                    comment,
                )
                .await?;
            rule_ids.push(rule_id);
        }

        println!(
            "🌐 {} regras de port forwarding criadas para range {}:{}",
            rule_ids.len(),
            host_start_port,
            host_end_port
        );

        Ok(rule_ids)
    }

    pub async fn get_stats(&self) -> Result<PortForwardingStats> {
        let total_rules = self.rules.len();
        let active_rules = self.rules.values().filter(|r| r.enabled).count();
        let inactive_rules = total_rules - active_rules;
        let tcp_rules = self
            .rules
            .values()
            .filter(|r| r.protocol == Protocol::Tcp)
            .count();
        let udp_rules = self
            .rules
            .values()
            .filter(|r| r.protocol == Protocol::Udp)
            .count();

        Ok(PortForwardingStats {
            total_rules,
            active_rules,
            inactive_rules,
            tcp_rules,
            udp_rules,
        })
    }

    pub async fn clear_rules(&mut self) -> Result<()> {
        let count = self.rules.len();
        self.rules.clear();
        println!("🌐 {} regras de port forwarding removidas", count);
        Ok(())
    }

    pub async fn clear_container_rules(&mut self, container_ip: IpAddr) -> Result<()> {
        let mut to_remove = Vec::new();

        for (rule_id, rule) in &self.rules {
            if rule.container_ip == container_ip {
                to_remove.push(rule_id.clone());
            }
        }

        for rule_id in to_remove {
            self.rules.remove(&rule_id);
        }

        println!(
            "🌐 Regras de port forwarding removidas para container {}",
            container_ip
        );
        Ok(())
    }

    async fn has_port_conflict(
        &self,
        host_ip: IpAddr,
        host_port: u16,
        protocol: &Protocol,
    ) -> Result<bool> {
        for rule in self.rules.values() {
            if rule.host_ip == host_ip && rule.host_port == host_port && rule.enabled {
                match (&rule.protocol, &protocol) {
                    (Protocol::Tcp, Protocol::Tcp) | (Protocol::Udp, Protocol::Udp) => {
                        return Ok(true)
                    }
                    (Protocol::Both, _) | (_, Protocol::Both) => return Ok(true),
                    _ => {} // Different protocols, no conflict
                }
            }
        }
        Ok(false)
    }
}

impl Default for PortForwardingManager {
    fn default() -> Self {
        Self::new()
    }
}
