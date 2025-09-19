use polis_core::{PolisError, Result};
use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq)]
pub enum FirewallAction {
    Allow,
    Deny,
    Reject,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    All,
}

#[derive(Debug, Clone)]
pub struct FirewallRule {
    pub id: String,
    pub action: FirewallAction,
    pub protocol: Protocol,
    pub source_ip: Option<IpAddr>,
    pub source_port: Option<u16>,
    pub dest_ip: Option<IpAddr>,
    pub dest_port: Option<u16>,
    pub interface: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FirewallChain {
    pub name: String,
    pub rules: Vec<FirewallRule>,
    pub default_action: FirewallAction,
}

pub struct FirewallManager {
    chains: HashMap<String, FirewallChain>,
    default_chain: String,
}

impl FirewallManager {
    pub fn new() -> Self {
        let mut manager = Self {
            chains: HashMap::new(),
            default_chain: "POLIS-FILTER".to_string(),
        };

        // Create default chains (synchronous initialization)
        manager.chains.insert(
            "POLIS-FILTER".to_string(),
            FirewallChain {
                name: "POLIS-FILTER".to_string(),
                rules: Vec::new(),
                default_action: FirewallAction::Allow,
            },
        );
        manager.chains.insert(
            "POLIS-INPUT".to_string(),
            FirewallChain {
                name: "POLIS-INPUT".to_string(),
                rules: Vec::new(),
                default_action: FirewallAction::Allow,
            },
        );
        manager.chains.insert(
            "POLIS-FORWARD".to_string(),
            FirewallChain {
                name: "POLIS-FORWARD".to_string(),
                rules: Vec::new(),
                default_action: FirewallAction::Allow,
            },
        );
        manager.chains.insert(
            "POLIS-OUTPUT".to_string(),
            FirewallChain {
                name: "POLIS-OUTPUT".to_string(),
                rules: Vec::new(),
                default_action: FirewallAction::Allow,
            },
        );

        manager
    }

    pub async fn create_chain(&mut self, name: &str, default_action: FirewallAction) -> Result<()> {
        let chain = FirewallChain {
            name: name.to_string(),
            rules: Vec::new(),
            default_action: default_action.clone(),
        };

        self.chains.insert(name.to_string(), chain);
        println!(
            "� Chain '{}' criada com ação padrão: {:?}",
            name, default_action
        );
        Ok(())
    }

    pub async fn add_rule(&mut self, chain_name: &str, rule: FirewallRule) -> Result<()> {
        let chain = self
            .chains
            .get_mut(chain_name)
            .ok_or_else(|| PolisError::Network(format!("Chain '{}' não encontrada", chain_name)))?;

        chain.rules.push(rule.clone());
        println!(
            "� Regra adicionada à chain '{}': {:?}",
            chain_name, rule.action
        );
        Ok(())
    }

    pub async fn remove_rule(&mut self, chain_name: &str, rule_id: &str) -> Result<()> {
        let chain = self
            .chains
            .get_mut(chain_name)
            .ok_or_else(|| PolisError::Network(format!("Chain '{}' não encontrada", chain_name)))?;

        chain.rules.retain(|rule| rule.id != rule_id);
        println!("� Regra '{}' removida da chain '{}'", rule_id, chain_name);
        Ok(())
    }

    pub async fn list_rules(&self, chain_name: Option<&str>) -> Result<Vec<FirewallRule>> {
        let chain_name = chain_name.unwrap_or(&self.default_chain);
        let chain = self
            .chains
            .get(chain_name)
            .ok_or_else(|| PolisError::Network(format!("Chain '{}' não encontrada", chain_name)))?;

        Ok(chain.rules.clone())
    }

    pub async fn create_container_rule(
        &mut self,
        container_id: &str,
        action: FirewallAction,
    ) -> Result<String> {
        let rule_id = format!("container-{}", container_id);

        let rule = FirewallRule {
            id: rule_id.clone(),
            action,
            protocol: Protocol::All,
            source_ip: None,
            source_port: None,
            dest_ip: None,
            dest_port: None,
            interface: Some(format!("polis-{}", container_id)),
            comment: Some(format!("Regra para container {}", container_id)),
        };

        self.add_rule("POLIS-FILTER", rule).await?;
        Ok(rule_id)
    }

    pub async fn create_port_rule(
        &mut self,
        port: u16,
        protocol: Protocol,
        action: FirewallAction,
    ) -> Result<String> {
        let rule_id = format!("port-{}-{:?}", port, protocol);

        let rule = FirewallRule {
            id: rule_id.clone(),
            action,
            protocol: protocol.clone(),
            source_ip: None,
            source_port: None,
            dest_ip: None,
            dest_port: Some(port),
            interface: None,
            comment: Some(format!("Regra para porta {} {:?}", port, protocol)),
        };

        self.add_rule("POLIS-INPUT", rule).await?;
        Ok(rule_id)
    }

    pub async fn create_ip_rule(
        &mut self,
        source_ip: IpAddr,
        action: FirewallAction,
    ) -> Result<String> {
        let rule_id = format!("ip-{}", source_ip);

        let rule = FirewallRule {
            id: rule_id.clone(),
            action,
            protocol: Protocol::All,
            source_ip: Some(source_ip),
            source_port: None,
            dest_ip: None,
            dest_port: None,
            interface: None,
            comment: Some(format!("Regra para IP {}", source_ip)),
        };

        self.add_rule("POLIS-INPUT", rule).await?;
        Ok(rule_id)
    }

    pub async fn get_chain_stats(&self, chain_name: Option<&str>) -> Result<ChainStats> {
        let chain_name = chain_name.unwrap_or(&self.default_chain);
        let chain = self
            .chains
            .get(chain_name)
            .ok_or_else(|| PolisError::Network(format!("Chain '{}' não encontrada", chain_name)))?;

        let allow_count = chain
            .rules
            .iter()
            .filter(|r| r.action == FirewallAction::Allow)
            .count();
        let deny_count = chain
            .rules
            .iter()
            .filter(|r| r.action == FirewallAction::Deny)
            .count();
        let reject_count = chain
            .rules
            .iter()
            .filter(|r| r.action == FirewallAction::Reject)
            .count();

        Ok(ChainStats {
            name: chain_name.to_string(),
            total_rules: chain.rules.len(),
            allow_rules: allow_count,
            deny_rules: deny_count,
            reject_rules: reject_count,
            default_action: chain.default_action.clone(),
        })
    }

    pub async fn flush_chain(&mut self, chain_name: &str) -> Result<()> {
        let chain = self
            .chains
            .get_mut(chain_name)
            .ok_or_else(|| PolisError::Network(format!("Chain '{}' não encontrada", chain_name)))?;

        chain.rules.clear();
        println!("� Chain '{}' limpa", chain_name);
        Ok(())
    }

    pub async fn list_chains(&self) -> Result<Vec<String>> {
        Ok(self.chains.keys().cloned().collect())
    }
}

#[derive(Debug, Clone)]
pub struct ChainStats {
    pub name: String,
    pub total_rules: usize,
    pub allow_rules: usize,
    pub deny_rules: usize,
    pub reject_rules: usize,
    pub default_action: FirewallAction,
}

impl Default for FirewallManager {
    fn default() -> Self {
        Self::new()
    }
}
