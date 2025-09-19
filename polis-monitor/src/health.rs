use polis_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

type ContainerHealthCheck = Box<dyn Fn(&str) -> Result<ContainerHealth> + Send + Sync>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: HealthStatus,
    pub checks: Vec<HealthCheck>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerHealth {
    pub container_id: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: u64,
    pub uptime_seconds: u64,
}

pub struct HealthChecker {
    checks: HashMap<String, Box<dyn Fn() -> Result<HealthCheck> + Send + Sync>>,
    container_checks: HashMap<String, ContainerHealthCheck>,
}

impl HealthChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            checks: HashMap::new(),
            container_checks: HashMap::new(),
        };

        // Register default system health checks
        checker.register_system_check("cpu", Box::new(Self::check_cpu_health));
        checker.register_system_check("memory", Box::new(Self::check_memory_health));
        checker.register_system_check("disk", Box::new(Self::check_disk_health));
        checker.register_system_check("network", Box::new(Self::check_network_health));

        // Register default container health checks
        checker.register_container_check("basic", Box::new(Self::check_container_basic));
        checker.register_container_check("resources", Box::new(Self::check_container_resources));

        checker
    }

    pub fn register_system_check<F>(&mut self, name: &str, check: Box<F>)
    where
        F: Fn() -> Result<HealthCheck> + Send + Sync + 'static,
    {
        self.checks.insert(name.to_string(), check);
    }

    pub fn register_container_check<F>(&mut self, name: &str, check: Box<F>)
    where
        F: Fn(&str) -> Result<ContainerHealth> + Send + Sync + 'static,
    {
        self.container_checks.insert(name.to_string(), check);
    }

    pub async fn check_system_health(&self) -> Result<SystemHealth> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut checks = Vec::new();
        let mut overall_status = HealthStatus::Healthy;

        for (name, check_fn) in &self.checks {
            let start_time = SystemTime::now();
            let check_result = check_fn();
            let duration = SystemTime::now()
                .duration_since(start_time)
                .unwrap()
                .as_millis() as u64;

            match check_result {
                Ok(mut health_check) => {
                    health_check.duration_ms = duration;
                    if health_check.status == HealthStatus::Unhealthy {
                        overall_status = HealthStatus::Unhealthy;
                    } else if health_check.status == HealthStatus::Degraded
                        && overall_status != HealthStatus::Unhealthy
                    {
                        overall_status = HealthStatus::Degraded;
                    }
                    checks.push(health_check);
                }
                Err(e) => {
                    let error_check = HealthCheck {
                        name: name.clone(),
                        status: HealthStatus::Unhealthy,
                        message: format!("Health check failed: {}", e),
                        timestamp,
                        duration_ms: duration,
                    };
                    overall_status = HealthStatus::Unhealthy;
                    checks.push(error_check);
                }
            }
        }

        let system_health = SystemHealth {
            overall_status,
            checks,
            timestamp,
        };

        println!(
            " Health check concluído: {:?} ({} checks)",
            system_health.overall_status,
            system_health.checks.len()
        );

        Ok(system_health)
    }

    pub async fn check_container_health(&self, container_id: &str) -> Result<Vec<ContainerHealth>> {
        let mut container_healths = Vec::new();

        for (name, check_fn) in &self.container_checks {
            match check_fn(container_id) {
                Ok(health) => {
                    println!(
                        " Container {} health check '{}': {:?}",
                        container_id, name, health.status
                    );
                    container_healths.push(health);
                }
                Err(e) => {
                    let error_health = ContainerHealth {
                        container_id: container_id.to_string(),
                        status: HealthStatus::Unhealthy,
                        message: format!("Health check '{}' failed: {}", name, e),
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        uptime_seconds: 0,
                    };
                    container_healths.push(error_health);
                }
            }
        }

        Ok(container_healths)
    }

    pub async fn get_system_health_summary(&self) -> Result<HealthSummary> {
        let system_health = self.check_system_health().await?;

        let healthy_checks = system_health
            .checks
            .iter()
            .filter(|c| c.status == HealthStatus::Healthy)
            .count();
        let unhealthy_checks = system_health
            .checks
            .iter()
            .filter(|c| c.status == HealthStatus::Unhealthy)
            .count();
        let degraded_checks = system_health
            .checks
            .iter()
            .filter(|c| c.status == HealthStatus::Degraded)
            .count();

        Ok(HealthSummary {
            overall_status: system_health.overall_status,
            total_checks: system_health.checks.len(),
            healthy_checks,
            unhealthy_checks,
            degraded_checks,
            last_check: system_health.timestamp,
        })
    }

    // Default system health check implementations
    fn check_cpu_health() -> Result<HealthCheck> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Simulate CPU health check
        let cpu_usage = 25.5; // Simulated CPU usage

        let (status, message) = if cpu_usage > 90.0 {
            (
                HealthStatus::Unhealthy,
                format!("CPU usage too high: {:.1}%", cpu_usage),
            )
        } else if cpu_usage > 70.0 {
            (
                HealthStatus::Degraded,
                format!("CPU usage elevated: {:.1}%", cpu_usage),
            )
        } else {
            (
                HealthStatus::Healthy,
                format!("CPU usage normal: {:.1}%", cpu_usage),
            )
        };

        Ok(HealthCheck {
            name: "cpu".to_string(),
            status,
            message,
            timestamp,
            duration_ms: 0, // Will be set by caller
        })
    }

    fn check_memory_health() -> Result<HealthCheck> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Simulate memory health check
        let memory_usage = 50.0; // Simulated memory usage percentage

        let (status, message) = if memory_usage > 95.0 {
            (
                HealthStatus::Unhealthy,
                format!("Memory usage critical: {:.1}%", memory_usage),
            )
        } else if memory_usage > 85.0 {
            (
                HealthStatus::Degraded,
                format!("Memory usage high: {:.1}%", memory_usage),
            )
        } else {
            (
                HealthStatus::Healthy,
                format!("Memory usage normal: {:.1}%", memory_usage),
            )
        };

        Ok(HealthCheck {
            name: "memory".to_string(),
            status,
            message,
            timestamp,
            duration_ms: 0,
        })
    }

    fn check_disk_health() -> Result<HealthCheck> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Simulate disk health check
        let disk_usage = 40.0; // Simulated disk usage percentage

        let (status, message) = if disk_usage > 95.0 {
            (
                HealthStatus::Unhealthy,
                format!("Disk space critical: {:.1}%", disk_usage),
            )
        } else if disk_usage > 85.0 {
            (
                HealthStatus::Degraded,
                format!("Disk space low: {:.1}%", disk_usage),
            )
        } else {
            (
                HealthStatus::Healthy,
                format!("Disk space normal: {:.1}%", disk_usage),
            )
        };

        Ok(HealthCheck {
            name: "disk".to_string(),
            status,
            message,
            timestamp,
            duration_ms: 0,
        })
    }

    fn check_network_health() -> Result<HealthCheck> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Simulate network health check
        let network_errors = 0; // Simulated network errors

        let (status, message) = if network_errors > 100 {
            (
                HealthStatus::Unhealthy,
                format!("Network errors high: {}", network_errors),
            )
        } else if network_errors > 10 {
            (
                HealthStatus::Degraded,
                format!("Network errors detected: {}", network_errors),
            )
        } else {
            (
                HealthStatus::Healthy,
                format!("Network healthy: {} errors", network_errors),
            )
        };

        Ok(HealthCheck {
            name: "network".to_string(),
            status,
            message,
            timestamp,
            duration_ms: 0,
        })
    }

    // Default container health check implementations
    fn check_container_basic(container_id: &str) -> Result<ContainerHealth> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Simulate basic container health check
        let is_running = true; // Simulated container running status

        let (status, message) = if is_running {
            (HealthStatus::Healthy, "Container is running".to_string())
        } else {
            (
                HealthStatus::Unhealthy,
                "Container is not running".to_string(),
            )
        };

        Ok(ContainerHealth {
            container_id: container_id.to_string(),
            status,
            message,
            timestamp,
            uptime_seconds: 3600, // Simulated uptime
        })
    }

    fn check_container_resources(container_id: &str) -> Result<ContainerHealth> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Simulate container resource health check
        let memory_usage = 25.0; // Simulated memory usage percentage
        let cpu_usage = 15.0; // Simulated CPU usage percentage

        let (status, message) = if memory_usage > 90.0 || cpu_usage > 90.0 {
            (
                HealthStatus::Unhealthy,
                format!(
                    "Resource usage critical: CPU {:.1}%, Memory {:.1}%",
                    cpu_usage, memory_usage
                ),
            )
        } else if memory_usage > 70.0 || cpu_usage > 70.0 {
            (
                HealthStatus::Degraded,
                format!(
                    "Resource usage high: CPU {:.1}%, Memory {:.1}%",
                    cpu_usage, memory_usage
                ),
            )
        } else {
            (
                HealthStatus::Healthy,
                format!(
                    "Resource usage normal: CPU {:.1}%, Memory {:.1}%",
                    cpu_usage, memory_usage
                ),
            )
        };

        Ok(ContainerHealth {
            container_id: container_id.to_string(),
            status,
            message,
            timestamp,
            uptime_seconds: 3600,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub overall_status: HealthStatus,
    pub total_checks: usize,
    pub healthy_checks: usize,
    pub unhealthy_checks: usize,
    pub degraded_checks: usize,
    pub last_check: u64,
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}
