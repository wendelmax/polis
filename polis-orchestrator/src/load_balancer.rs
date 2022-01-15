use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::service_discovery::{LoadBalancingAlgorithm, Protocol, ServiceEndpoint};

/// Load balancer for distributing traffic across service endpoints
pub struct LoadBalancer {
    algorithm: LoadBalancingAlgorithm,
    endpoints: Arc<RwLock<Vec<ServiceEndpoint>>>,
    current_index: Arc<RwLock<usize>>,
    connection_counts: Arc<RwLock<HashMap<String, u32>>>,
    last_used: Arc<RwLock<HashMap<String, Instant>>>,
    sticky_sessions: Arc<RwLock<HashMap<String, String>>>, // session_id -> endpoint_id
    health_checker: Arc<HealthChecker>,
}

/// Load balancer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub endpoint_stats: HashMap<String, EndpointStats>,
}

/// Endpoint statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointStats {
    pub endpoint_id: String,
    pub requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub active_connections: u32,
    #[serde(skip)]
    pub last_used: Option<Instant>,
}

/// Load balancer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerRequest {
    pub client_ip: Option<IpAddr>,
    pub session_id: Option<String>,
    pub headers: HashMap<String, String>,
    pub path: String,
    pub method: String,
}

/// Load balancer response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerResponse {
    pub endpoint: ServiceEndpoint,
    pub status_code: u16,
    pub response_time: Duration,
    pub error: Option<String>,
}

/// Health checker for load balancer
pub struct HealthChecker {
    client: reqwest::Client,
    timeout: Duration,
}

/// Consistent hash ring
pub struct ConsistentHashRing {
    ring: Vec<RingNode>,
    virtual_nodes: u32,
}

/// Ring node
#[derive(Debug, Clone)]
pub struct RingNode {
    pub hash: u64,
    pub endpoint_id: String,
    pub endpoint: ServiceEndpoint,
}

impl LoadBalancer {
    pub fn new(algorithm: LoadBalancingAlgorithm) -> Self {
        Self {
            algorithm,
            endpoints: Arc::new(RwLock::new(Vec::new())),
            current_index: Arc::new(RwLock::new(0)),
            connection_counts: Arc::new(RwLock::new(HashMap::new())),
            last_used: Arc::new(RwLock::new(HashMap::new())),
            sticky_sessions: Arc::new(RwLock::new(HashMap::new())),
            health_checker: Arc::new(HealthChecker::new()),
        }
    }

    pub async fn add_endpoint(&self, endpoint: ServiceEndpoint) {
        let mut endpoints = self.endpoints.write().await;
        endpoints.push(endpoint);
    }

    pub async fn remove_endpoint(&self, endpoint_id: &str) {
        let mut endpoints = self.endpoints.write().await;
        endpoints.retain(|ep| ep.id != endpoint_id);
    }

    pub async fn update_endpoints(&self, new_endpoints: Vec<ServiceEndpoint>) {
        let mut endpoints = self.endpoints.write().await;
        *endpoints = new_endpoints;
    }

    pub async fn select_endpoint(
        &self,
        request: &LoadBalancerRequest,
    ) -> Result<Option<ServiceEndpoint>> {
        let endpoints = self.endpoints.read().await;
        let healthy_endpoints: Vec<&ServiceEndpoint> = endpoints
            .iter()
            .filter(|ep| ep.health_status == crate::service_discovery::HealthStatus::Healthy)
            .collect();

        if healthy_endpoints.is_empty() {
            return Ok(None);
        }

        let selected = match self.algorithm {
            LoadBalancingAlgorithm::RoundRobin => self.select_round_robin(&healthy_endpoints).await,
            LoadBalancingAlgorithm::WeightedRoundRobin => {
                self.select_weighted_round_robin(&healthy_endpoints).await
            }
            LoadBalancingAlgorithm::LeastConnections => {
                self.select_least_connections(&healthy_endpoints).await
            }
            LoadBalancingAlgorithm::Random => self.select_random(&healthy_endpoints),
            LoadBalancingAlgorithm::IpHash => {
                self.select_ip_hash(&healthy_endpoints, request.client_ip)
            }
            LoadBalancingAlgorithm::ConsistentHash => {
                self.select_consistent_hash(&healthy_endpoints, &request.path)
                    .await
            }
        };

        Ok(selected.cloned())
    }

    async fn select_round_robin<'a>(
        &self,
        endpoints: &[&'a ServiceEndpoint],
    ) -> Option<&'a ServiceEndpoint> {
        if endpoints.is_empty() {
            return None;
        }

        let mut current_index = self.current_index.write().await;
        let selected = endpoints[*current_index % endpoints.len()];
        *current_index = (*current_index + 1) % endpoints.len();
        Some(selected)
    }

    async fn select_weighted_round_robin<'a>(
        &self,
        endpoints: &[&'a ServiceEndpoint],
    ) -> Option<&'a ServiceEndpoint> {
        if endpoints.is_empty() {
            return None;
        }

        let total_weight: u32 = endpoints.iter().map(|ep| ep.weight).sum();
        if total_weight == 0 {
            return self.select_round_robin(endpoints).await;
        }

        let mut current_index = self.current_index.write().await;
        let mut current_weight = 0u32;
        let mut selected_index = 0;

        for (i, endpoint) in endpoints.iter().enumerate() {
            current_weight += endpoint.weight;
            if *current_index < current_weight as usize {
                selected_index = i;
                break;
            }
        }

        *current_index = (*current_index + 1) % total_weight as usize;
        Some(endpoints[selected_index])
    }

    async fn select_least_connections<'a>(
        &self,
        endpoints: &[&'a ServiceEndpoint],
    ) -> Option<&'a ServiceEndpoint> {
        if endpoints.is_empty() {
            return None;
        }

        let connection_counts = self.connection_counts.read().await;
        let mut min_connections = u32::MAX;
        let mut selected = None;

        for endpoint in endpoints {
            let connections = connection_counts.get(&endpoint.id).unwrap_or(&0);
            if *connections < min_connections {
                min_connections = *connections;
                selected = Some(*endpoint);
            }
        }

        selected
    }

    fn select_random<'a>(&self, endpoints: &[&'a ServiceEndpoint]) -> Option<&'a ServiceEndpoint> {
        if endpoints.is_empty() {
            return None;
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..endpoints.len());
        Some(endpoints[index])
    }

    fn select_ip_hash<'a>(
        &self,
        endpoints: &[&'a ServiceEndpoint],
        client_ip: Option<IpAddr>,
    ) -> Option<&'a ServiceEndpoint> {
        if endpoints.is_empty() {
            return None;
        }

        let client_ip =
            client_ip.unwrap_or_else(|| std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)));
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        client_ip.hash(&mut hasher);
        let hash = hasher.finish();
        let index = (hash as usize) % endpoints.len();
        Some(endpoints[index])
    }

    async fn select_consistent_hash<'a>(
        &self,
        endpoints: &[&'a ServiceEndpoint],
        key: &str,
    ) -> Option<&'a ServiceEndpoint> {
        if endpoints.is_empty() {
            return None;
        }

        // Simplified consistent hash implementation
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let index = (hash as usize) % endpoints.len();
        Some(endpoints[index])
    }

    pub async fn handle_request(
        &self,
        request: LoadBalancerRequest,
    ) -> Result<LoadBalancerResponse> {
        let start_time = Instant::now();

        // Check for sticky session
        if let Some(session_id) = &request.session_id {
            let sticky_sessions = self.sticky_sessions.read().await;
            if let Some(endpoint_id) = sticky_sessions.get(session_id) {
                let endpoints = self.endpoints.read().await;
                if let Some(endpoint) = endpoints.iter().find(|ep| ep.id == *endpoint_id) {
                    return self.forward_request(endpoint, request, start_time).await;
                }
            }
        }

        // Select endpoint
        let endpoint = match self.select_endpoint(&request).await? {
            Some(ep) => ep,
            None => {
                return Ok(LoadBalancerResponse {
                    endpoint: ServiceEndpoint::new("".to_string(), 0, Protocol::Http),
                    status_code: 503,
                    response_time: start_time.elapsed(),
                    error: Some("No healthy endpoints available".to_string()),
                });
            }
        };

        // Store sticky session
        if let Some(session_id) = &request.session_id {
            let mut sticky_sessions = self.sticky_sessions.write().await;
            sticky_sessions.insert(session_id.clone(), endpoint.id.clone());
        }

        // Update connection count
        {
            let mut connection_counts = self.connection_counts.write().await;
            *connection_counts.entry(endpoint.id.clone()).or_insert(0) += 1;
        }

        // Update last used
        {
            let mut last_used = self.last_used.write().await;
            last_used.insert(endpoint.id.clone(), Instant::now());
        }

        self.forward_request(&endpoint, request, start_time).await
    }

    async fn forward_request(
        &self,
        endpoint: &ServiceEndpoint,
        request: LoadBalancerRequest,
        start_time: Instant,
    ) -> Result<LoadBalancerResponse> {
        let url = format!(
            "{}://{}:{}{}",
            match endpoint.protocol {
                Protocol::Http => "http",
                Protocol::Https => "https",
                Protocol::Grpc => "http", // gRPC over HTTP
                _ => "http",
            },
            endpoint.address,
            endpoint.port,
            request.path
        );

        let client = reqwest::Client::new();
        let response = client
            .request(
                match request.method.as_str() {
                    "GET" => reqwest::Method::GET,
                    "POST" => reqwest::Method::POST,
                    "PUT" => reqwest::Method::PUT,
                    "DELETE" => reqwest::Method::DELETE,
                    _ => reqwest::Method::GET,
                },
                &url,
            )
            .headers({
                let mut headers = reqwest::header::HeaderMap::new();
                for (key, value) in &request.headers {
                    if let (Ok(header_name), Ok(header_value)) = (
                        key.parse::<reqwest::header::HeaderName>(),
                        value.parse::<reqwest::header::HeaderValue>(),
                    ) {
                        headers.insert(header_name, header_value);
                    }
                }
                headers
            })
            .timeout(Duration::from_secs(30))
            .send()
            .await;

        let response_time = start_time.elapsed();
        let status_code = response
            .as_ref()
            .map(|r| r.status().as_u16())
            .unwrap_or(500);
        let error = if response.is_err() {
            Some(response.err().unwrap().to_string())
        } else {
            None
        };

        // Update connection count
        {
            let mut connection_counts = self.connection_counts.write().await;
            if let Some(count) = connection_counts.get_mut(&endpoint.id) {
                *count = count.saturating_sub(1);
            }
        }

        Ok(LoadBalancerResponse {
            endpoint: endpoint.clone(),
            status_code,
            response_time,
            error,
        })
    }

    pub async fn get_stats(&self) -> LoadBalancerStats {
        let endpoints = self.endpoints.read().await;
        let connection_counts = self.connection_counts.read().await;
        let last_used = self.last_used.read().await;

        let mut endpoint_stats = HashMap::new();
        let total_requests = 0u64;
        let successful_requests = 0u64;
        let failed_requests = 0u64;
        let total_response_time = Duration::ZERO;

        for endpoint in endpoints.iter() {
            let connections = connection_counts.get(&endpoint.id).unwrap_or(&0);
            let last_used_time = last_used.get(&endpoint.id).cloned();

            let stats = EndpointStats {
                endpoint_id: endpoint.id.clone(),
                requests: 0, // This would be tracked in a real implementation
                successful_requests: 0,
                failed_requests: 0,
                average_response_time: Duration::ZERO,
                active_connections: *connections,
                last_used: last_used_time,
            };

            endpoint_stats.insert(endpoint.id.clone(), stats);
        }

        LoadBalancerStats {
            total_requests,
            successful_requests,
            failed_requests,
            average_response_time: if total_requests > 0 {
                total_response_time / total_requests as u32
            } else {
                Duration::ZERO
            },
            endpoint_stats,
        }
    }

    pub async fn health_check_endpoints(&self) {
        let endpoints = self.endpoints.read().await;
        let mut unhealthy_endpoints = Vec::new();

        for endpoint in endpoints.iter() {
            let is_healthy = self.health_checker.check_endpoint(endpoint).await;
            if !is_healthy {
                unhealthy_endpoints.push(endpoint.id.clone());
            }
        }

        drop(endpoints);

        // Remove unhealthy endpoints
        if !unhealthy_endpoints.is_empty() {
            let mut endpoints = self.endpoints.write().await;
            endpoints.retain(|ep| !unhealthy_endpoints.contains(&ep.id));
        }
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            timeout: Duration::from_secs(5),
        }
    }

    pub async fn check_endpoint(&self, endpoint: &ServiceEndpoint) -> bool {
        let url = format!(
            "{}://{}:{}",
            match endpoint.protocol {
                Protocol::Http => "http",
                Protocol::Https => "https",
                _ => "http",
            },
            endpoint.address,
            endpoint.port
        );

        match self.client.get(&url).timeout(self.timeout).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
}

impl ConsistentHashRing {
    pub fn new(endpoints: &[&ServiceEndpoint], virtual_nodes: u32) -> Self {
        let mut ring = Vec::new();

        for endpoint in endpoints {
            for i in 0..virtual_nodes {
                let key = format!("{}:{}", endpoint.id, i);
                let hash = Self::hash(&key);
                ring.push(RingNode {
                    hash,
                    endpoint_id: endpoint.id.clone(),
                    endpoint: (*endpoint).clone(),
                });
            }
        }

        ring.sort_by_key(|node| node.hash);

        Self {
            ring,
            virtual_nodes,
        }
    }

    pub fn get_endpoint(&self, key: &str) -> Option<&ServiceEndpoint> {
        if self.ring.is_empty() {
            return None;
        }

        let hash = Self::hash(key);
        let index = self
            .ring
            .binary_search_by_key(&hash, |node| node.hash)
            .unwrap_or_else(|index| index % self.ring.len());

        Some(&self.ring[index].endpoint)
    }

    fn hash(key: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service_discovery::{HealthStatus, Protocol, ServiceEndpoint};

    fn create_test_endpoint(id: &str, address: &str, port: u16) -> ServiceEndpoint {
        ServiceEndpoint {
            id: id.to_string(),
            address: address.to_string(),
            port,
            protocol: Protocol::Http,
            weight: 1,
            priority: 0,
            health_status: HealthStatus::Healthy,
            last_health_check: None,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_round_robin() {
        let lb = LoadBalancer::new(LoadBalancingAlgorithm::RoundRobin);

        let endpoint1 = create_test_endpoint("1", "127.0.0.1", 8080);
        let endpoint2 = create_test_endpoint("2", "127.0.0.1", 8081);

        lb.add_endpoint(endpoint1).await;
        lb.add_endpoint(endpoint2).await;

        let request = LoadBalancerRequest {
            client_ip: None,
            session_id: None,
            headers: HashMap::new(),
            path: "/".to_string(),
            method: "GET".to_string(),
        };

        let selected1 = lb.select_endpoint(&request).await.unwrap();
        let selected2 = lb.select_endpoint(&request).await.unwrap();

        // Should alternate between endpoints
        assert_ne!(selected1.id, selected2.id);
    }

    #[tokio::test]
    async fn test_weighted_round_robin() {
        let lb = LoadBalancer::new(LoadBalancingAlgorithm::WeightedRoundRobin);

        let mut endpoint1 = create_test_endpoint("1", "127.0.0.1", 8080);
        endpoint1.weight = 3;

        let mut endpoint2 = create_test_endpoint("2", "127.0.0.1", 8081);
        endpoint2.weight = 1;

        lb.add_endpoint(endpoint1).await;
        lb.add_endpoint(endpoint2).await;

        let request = LoadBalancerRequest {
            client_ip: None,
            session_id: None,
            headers: HashMap::new(),
            path: "/".to_string(),
            method: "GET".to_string(),
        };

        // Should select endpoint1 more often due to higher weight
        let mut endpoint1_count = 0;
        for _ in 0..100 {
            let selected = lb.select_endpoint(&request).await.unwrap();
            if selected.id == "1" {
                endpoint1_count += 1;
            }
        }

        assert!(endpoint1_count > 50); // Should be more than 50% due to weight 3:1
    }

    #[tokio::test]
    async fn test_consistent_hash() {
        let lb = LoadBalancer::new(LoadBalancingAlgorithm::ConsistentHash);

        let endpoint1 = create_test_endpoint("1", "127.0.0.1", 8080);
        let endpoint2 = create_test_endpoint("2", "127.0.0.1", 8081);

        lb.add_endpoint(endpoint1).await;
        lb.add_endpoint(endpoint2).await;

        let request = LoadBalancerRequest {
            client_ip: None,
            session_id: None,
            headers: HashMap::new(),
            path: "/test".to_string(),
            method: "GET".to_string(),
        };

        // Same key should always select the same endpoint
        let selected1 = lb.select_endpoint(&request).await.unwrap();
        let selected2 = lb.select_endpoint(&request).await.unwrap();

        assert_eq!(selected1.id, selected2.id);
    }

    #[tokio::test]
    async fn test_ip_hash() {
        let lb = LoadBalancer::new(LoadBalancingAlgorithm::IpHash);

        let endpoint1 = create_test_endpoint("1", "127.0.0.1", 8080);
        let endpoint2 = create_test_endpoint("2", "127.0.0.1", 8081);

        lb.add_endpoint(endpoint1).await;
        lb.add_endpoint(endpoint2).await;

        let request = LoadBalancerRequest {
            client_ip: Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(
                192, 168, 1, 1,
            ))),
            session_id: None,
            headers: HashMap::new(),
            path: "/".to_string(),
            method: "GET".to_string(),
        };

        // Same IP should always select the same endpoint
        let selected1 = lb.select_endpoint(&request).await.unwrap();
        let selected2 = lb.select_endpoint(&request).await.unwrap();

        assert_eq!(selected1.id, selected2.id);
    }
}
