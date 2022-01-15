use polis_core::{PolisError, Result};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: DnsRecordType,
    pub value: String,
    pub ttl: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DnsRecordType {
    A,     // IPv4 address
    AAAA,  // IPv6 address
    CNAME, // Canonical name
    MX,    // Mail exchange
    TXT,   // Text record
    NS,    // Name server
}

#[derive(Debug, Clone)]
pub struct DnsZone {
    pub name: String,
    pub records: HashMap<String, Vec<DnsRecord>>,
    pub ttl: u32,
}

pub struct DnsManager {
    zones: HashMap<String, DnsZone>,
    default_zone: String,
    upstream_servers: Vec<IpAddr>,
}

impl DnsManager {
    pub fn new() -> Self {
        let mut manager = Self {
            zones: HashMap::new(),
            default_zone: "polis.local".to_string(),
            upstream_servers: vec![
                IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), // Google DNS
                IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), // Cloudflare DNS
            ],
        };

        // Create default zones (synchronous initialization)
        manager.zones.insert(
            "polis.local".to_string(),
            DnsZone {
                name: "polis.local".to_string(),
                records: HashMap::new(),
                ttl: 300,
            },
        );
        manager.zones.insert(
            "container.local".to_string(),
            DnsZone {
                name: "container.local".to_string(),
                records: HashMap::new(),
                ttl: 300,
            },
        );

        manager
    }

    pub async fn create_zone(&mut self, name: &str, ttl: u32) -> Result<()> {
        let zone = DnsZone {
            name: name.to_string(),
            records: HashMap::new(),
            ttl,
        };

        self.zones.insert(name.to_string(), zone);
        println!("🌐 Zona DNS '{}' criada (TTL: {})", name, ttl);
        Ok(())
    }

    pub async fn add_record(&mut self, zone_name: &str, record: DnsRecord) -> Result<()> {
        let zone = self
            .zones
            .get_mut(zone_name)
            .ok_or_else(|| PolisError::Network(format!("Zona '{}' não encontrada", zone_name)))?;

        let key = format!("{}.{}", record.name, zone_name);
        zone.records
            .entry(key)
            .or_insert_with(Vec::new)
            .push(record.clone());

        println!(
            "🌐 Registro DNS adicionado: {} {:?} -> {}",
            record.name, record.record_type, record.value
        );
        Ok(())
    }

    pub async fn remove_record(
        &mut self,
        zone_name: &str,
        name: &str,
        record_type: DnsRecordType,
    ) -> Result<()> {
        let zone = self
            .zones
            .get_mut(zone_name)
            .ok_or_else(|| PolisError::Network(format!("Zona '{}' não encontrada", zone_name)))?;

        let key = format!("{}.{}", name, zone_name);
        if let Some(records) = zone.records.get_mut(&key) {
            records.retain(|r| r.record_type != record_type);
            if records.is_empty() {
                zone.records.remove(&key);
            }
        }

        println!("🌐 Registro DNS removido: {} {:?}", name, record_type);
        Ok(())
    }

    pub async fn resolve(&self, name: &str, record_type: DnsRecordType) -> Result<Vec<DnsRecord>> {
        // First try to resolve in local zones
        for zone in self.zones.values() {
            let key = format!("{}.{}", name, zone.name);
            if let Some(records) = zone.records.get(&key) {
                let matching_records: Vec<DnsRecord> = records
                    .iter()
                    .filter(|r| r.record_type == record_type)
                    .cloned()
                    .collect();

                if !matching_records.is_empty() {
                    return Ok(matching_records);
                }
            }
        }

        // If not found locally, simulate upstream resolution
        self.resolve_upstream(name, record_type).await
    }

    pub async fn create_container_record(&mut self, container_id: &str, ip: IpAddr) -> Result<()> {
        let record = DnsRecord {
            name: container_id.to_string(),
            record_type: DnsRecordType::A,
            value: ip.to_string(),
            ttl: 300,
        };

        self.add_record("container.local", record).await?;
        Ok(())
    }

    pub async fn create_alias_record(
        &mut self,
        alias: &str,
        target: &str,
        zone: &str,
    ) -> Result<()> {
        let record = DnsRecord {
            name: alias.to_string(),
            record_type: DnsRecordType::CNAME,
            value: target.to_string(),
            ttl: 300,
        };

        self.add_record(zone, record).await?;
        Ok(())
    }

    pub async fn list_records(&self, zone_name: Option<&str>) -> Result<Vec<DnsRecord>> {
        let zone_name = zone_name.unwrap_or(&self.default_zone);
        let zone = self
            .zones
            .get(zone_name)
            .ok_or_else(|| PolisError::Network(format!("Zona '{}' não encontrada", zone_name)))?;

        let mut all_records = Vec::new();
        for records in zone.records.values() {
            all_records.extend(records.clone());
        }

        Ok(all_records)
    }

    pub async fn get_zone_stats(&self, zone_name: Option<&str>) -> Result<ZoneStats> {
        let zone_name = zone_name.unwrap_or(&self.default_zone);
        let zone = self
            .zones
            .get(zone_name)
            .ok_or_else(|| PolisError::Network(format!("Zona '{}' não encontrada", zone_name)))?;

        let mut record_counts = HashMap::new();
        for records in zone.records.values() {
            for record in records {
                *record_counts.entry(record.record_type.clone()).or_insert(0) += 1;
            }
        }

        Ok(ZoneStats {
            name: zone_name.to_string(),
            total_records: zone.records.values().map(|v| v.len()).sum(),
            record_counts,
            ttl: zone.ttl,
        })
    }

    pub async fn add_upstream_server(&mut self, server: IpAddr) -> Result<()> {
        self.upstream_servers.push(server);
        println!("🌐 Servidor DNS upstream adicionado: {}", server);
        Ok(())
    }

    pub async fn list_upstream_servers(&self) -> Result<Vec<IpAddr>> {
        Ok(self.upstream_servers.clone())
    }

    async fn resolve_upstream(
        &self,
        name: &str,
        record_type: DnsRecordType,
    ) -> Result<Vec<DnsRecord>> {
        // Simulate upstream DNS resolution
        // In a real implementation, this would make actual DNS queries

        match record_type {
            DnsRecordType::A => {
                // Simulate A record resolution
                if name == "google.com" {
                    Ok(vec![DnsRecord {
                        name: name.to_string(),
                        record_type: DnsRecordType::A,
                        value: "142.250.191.14".to_string(),
                        ttl: 300,
                    }])
                } else {
                    Err(PolisError::Network("Nome não encontrado".to_string()))
                }
            }
            _ => Err(PolisError::Network(
                "Tipo de registro não suportado".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZoneStats {
    pub name: String,
    pub total_records: usize,
    pub record_counts: HashMap<DnsRecordType, usize>,
    pub ttl: u32,
}

impl Default for DnsManager {
    fn default() -> Self {
        Self::new()
    }
}
