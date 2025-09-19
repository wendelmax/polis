use polis_core::{PolisError, Result};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct IpPool {
    pub subnet: String,
    pub gateway: IpAddr,
    pub allocated_ips: HashMap<String, IpAddr>,
    pub available_ips: Vec<IpAddr>,
}

#[derive(Debug, Clone)]
pub struct IpAllocation {
    pub container_id: String,
    pub ip: IpAddr,
    pub subnet: String,
    pub gateway: IpAddr,
}

pub struct IpamManager {
    pools: HashMap<String, IpPool>,
    default_pool: String,
}

impl IpamManager {
    pub fn new() -> Self {
        Self {
            pools: HashMap::new(),
            default_pool: "default".to_string(),
        }
    }

    pub async fn create_pool(&mut self, name: &str, subnet: &str, gateway: &str) -> Result<()> {
        let gateway_ip = IpAddr::from_str(gateway)
            .map_err(|e| PolisError::Network(format!("Gateway inválido: {}", e)))?;

        let available_ips = self.generate_ip_range(subnet, gateway_ip)?;

        let pool = IpPool {
            subnet: subnet.to_string(),
            gateway: gateway_ip,
            allocated_ips: HashMap::new(),
            available_ips,
        };

        self.pools.insert(name.to_string(), pool);
        println!(
            "� Pool IP '{}' criado: {} (gateway: {})",
            name, subnet, gateway
        );
        Ok(())
    }

    pub async fn allocate_ip(
        &mut self,
        container_id: &str,
        pool_name: Option<&str>,
    ) -> Result<IpAllocation> {
        let pool_name = pool_name.unwrap_or(&self.default_pool);
        let pool = self
            .pools
            .get_mut(pool_name)
            .ok_or_else(|| PolisError::Network(format!("Pool '{}' não encontrado", pool_name)))?;

        if pool.available_ips.is_empty() {
            return Err(PolisError::Network(
                "Nenhum IP disponível no pool".to_string(),
            ));
        }

        let ip = pool.available_ips.pop().unwrap();
        pool.allocated_ips.insert(container_id.to_string(), ip);

        let allocation = IpAllocation {
            container_id: container_id.to_string(),
            ip,
            subnet: pool.subnet.clone(),
            gateway: pool.gateway,
        };

        println!("� IP {} alocado para container {}", ip, container_id);
        Ok(allocation)
    }

    pub async fn deallocate_ip(
        &mut self,
        container_id: &str,
        pool_name: Option<&str>,
    ) -> Result<()> {
        let pool_name = pool_name.unwrap_or(&self.default_pool);
        let pool = self
            .pools
            .get_mut(pool_name)
            .ok_or_else(|| PolisError::Network(format!("Pool '{}' não encontrado", pool_name)))?;

        if let Some(ip) = pool.allocated_ips.remove(container_id) {
            pool.available_ips.push(ip);
            println!("� IP {} desalocado do container {}", ip, container_id);
        }

        Ok(())
    }

    pub async fn get_allocation(
        &self,
        container_id: &str,
        pool_name: Option<&str>,
    ) -> Result<Option<IpAllocation>> {
        let pool_name = pool_name.unwrap_or(&self.default_pool);
        let pool = self
            .pools
            .get(pool_name)
            .ok_or_else(|| PolisError::Network(format!("Pool '{}' não encontrado", pool_name)))?;

        if let Some(&ip) = pool.allocated_ips.get(container_id) {
            Ok(Some(IpAllocation {
                container_id: container_id.to_string(),
                ip,
                subnet: pool.subnet.clone(),
                gateway: pool.gateway,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_allocations(&self, pool_name: Option<&str>) -> Result<Vec<IpAllocation>> {
        let pool_name = pool_name.unwrap_or(&self.default_pool);
        let pool = self
            .pools
            .get(pool_name)
            .ok_or_else(|| PolisError::Network(format!("Pool '{}' não encontrado", pool_name)))?;

        let allocations: Vec<IpAllocation> = pool
            .allocated_ips
            .iter()
            .map(|(container_id, &ip)| IpAllocation {
                container_id: container_id.clone(),
                ip,
                subnet: pool.subnet.clone(),
                gateway: pool.gateway,
            })
            .collect();

        Ok(allocations)
    }

    pub async fn get_pool_stats(&self, pool_name: Option<&str>) -> Result<PoolStats> {
        let pool_name = pool_name.unwrap_or(&self.default_pool);
        let pool = self
            .pools
            .get(pool_name)
            .ok_or_else(|| PolisError::Network(format!("Pool '{}' não encontrado", pool_name)))?;

        Ok(PoolStats {
            name: pool_name.to_string(),
            subnet: pool.subnet.clone(),
            gateway: pool.gateway,
            total_ips: pool.allocated_ips.len() + pool.available_ips.len(),
            allocated_ips: pool.allocated_ips.len(),
            available_ips: pool.available_ips.len(),
        })
    }

    fn generate_ip_range(&self, subnet: &str, gateway: IpAddr) -> Result<Vec<IpAddr>> {
        // Parse subnet (e.g., "172.17.0.0/16")
        let parts: Vec<&str> = subnet.split('/').collect();
        if parts.len() != 2 {
            return Err(PolisError::Network(
                "Formato de subnet inválido".to_string(),
            ));
        }

        let network = IpAddr::from_str(parts[0])
            .map_err(|e| PolisError::Network(format!("IP de rede inválido: {}", e)))?;
        let prefix_len: u8 = parts[1]
            .parse()
            .map_err(|e| PolisError::Network(format!("Prefixo inválido: {}", e)))?;

        let mut ips = Vec::new();

        match network {
            IpAddr::V4(network_v4) => {
                let host_bits = 32 - prefix_len;
                let host_count = 2_u32.pow(host_bits as u32) - 2; // -2 for network and broadcast

                for i in 1..=host_count {
                    let ip = Ipv4Addr::from(u32::from(network_v4) + i);
                    let ip_addr = IpAddr::V4(ip);
                    if ip_addr != gateway {
                        ips.push(ip_addr);
                    }
                }
            }
            IpAddr::V6(_) => {
                // IPv6 support would go here
                return Err(PolisError::Network("IPv6 não suportado ainda".to_string()));
            }
        }

        Ok(ips)
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub name: String,
    pub subnet: String,
    pub gateway: IpAddr,
    pub total_ips: usize,
    pub allocated_ips: usize,
    pub available_ips: usize,
}

impl Default for IpamManager {
    fn default() -> Self {
        Self::new()
    }
}
