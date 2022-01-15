use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

// use polis_core::{PolisError, Result as PolisResult};

/// Health monitoring system
pub struct HealthMonitor {
    checks: Arc<RwLock<HashMap<String, HealthCheck>>>,
    results: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    event_sender: Arc<tokio::sync::mpsc::UnboundedSender<HealthEvent>>,
    checker: Arc<HealthChecker>,
}

/// Health check definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub id: String,
    pub name: String,
    pub target_type: TargetType,
    pub target_id: String,
    pub check_type: CheckType,
    pub interval: Duration,
    pub timeout: Duration,
    pub retries: u32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

/// Target type for health checks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TargetType {
    Container,
    Service,
    Deployment,
    Node,
    Custom,
}

/// Health check type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CheckType {
    Http { path: String, expected_status: u16 },
    Tcp { port: u16 },
    Udp { port: u16 },
    Grpc { service: String, method: String },
    Command { command: String, args: Vec<String> },
    File { path: String, exists: bool },
    Custom { script: String },
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub check_id: String,
    pub target_id: String,
    pub status: HealthStatus,
    pub message: String,
    pub response_time: Duration,
    pub timestamp: DateTime<Utc>,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub metadata: HashMap<String, String>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
    Unknown,
}

/// Health event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthEvent {
    CheckPassed {
        check_id: String,
        target_id: String,
        message: String,
    },
    CheckFailed {
        check_id: String,
        target_id: String,
        message: String,
    },
    CheckDegraded {
        check_id: String,
        target_id: String,
        message: String,
    },
    CheckRecovered {
        check_id: String,
        target_id: String,
        message: String,
    },
    CheckCreated {
        check_id: String,
        target_id: String,
    },
    CheckDeleted {
        check_id: String,
        target_id: String,
    },
}

/// Health checker
pub struct HealthChecker {
    client: reqwest::Client,
    command_executor: Arc<CommandExecutor>,
}

/// Command executor
pub struct CommandExecutor {
    timeout: Duration,
}

/// Health check statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStats {
    pub total_checks: u32,
    pub healthy_checks: u32,
    pub unhealthy_checks: u32,
    pub degraded_checks: u32,
    pub unknown_checks: u32,
    pub average_response_time: Duration,
    pub uptime_percentage: f64,
    pub last_check: Option<DateTime<Utc>>,
}

/// Health check summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    pub overall_status: HealthStatus,
    pub total_checks: u32,
    pub healthy_checks: u32,
    pub unhealthy_checks: u32,
    pub degraded_checks: u32,
    pub unknown_checks: u32,
    pub checks: Vec<HealthCheckResult>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        let (event_sender, _event_receiver) = tokio::sync::mpsc::unbounded_channel();

        Self {
            checks: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            event_sender: Arc::new(event_sender),
            checker: Arc::new(HealthChecker::new()),
        }
    }

    pub async fn create_health_check(&self, check: HealthCheck) -> Result<()> {
        let check_id = check.id.clone();
        let target_id = check.target_id.clone();

        let mut checks = self.checks.write().await;
        checks.insert(check_id.clone(), check);
        drop(checks);

        // Send event
        let _ = self.event_sender.send(HealthEvent::CheckCreated {
            check_id: check_id.clone(),
            target_id: target_id.clone(),
        });

        // Start health checking
        self.start_health_checking(&check_id).await?;

        Ok(())
    }

    pub async fn update_health_check(&self, check: HealthCheck) -> Result<()> {
        let mut checks = self.checks.write().await;
        checks.insert(check.id.clone(), check);
        Ok(())
    }

    pub async fn delete_health_check(&self, check_id: &str) -> Result<()> {
        let mut checks = self.checks.write().await;
        let check = checks.remove(check_id);
        drop(checks);

        if let Some(check) = check {
            // Send event
            let _ = self.event_sender.send(HealthEvent::CheckDeleted {
                check_id: check_id.to_string(),
                target_id: check.target_id,
            });
        }

        Ok(())
    }

    pub async fn get_health_check(&self, check_id: &str) -> Option<HealthCheck> {
        let checks = self.checks.read().await;
        checks.get(check_id).cloned()
    }

    pub async fn list_health_checks(&self) -> Vec<HealthCheck> {
        let checks = self.checks.read().await;
        checks.values().cloned().collect()
    }

    pub async fn get_health_check_result(&self, check_id: &str) -> Option<HealthCheckResult> {
        let results = self.results.read().await;
        results.get(check_id).cloned()
    }

    pub async fn get_health_check_results(
        &self,
        target_id: Option<&str>,
    ) -> Vec<HealthCheckResult> {
        let results = self.results.read().await;
        if let Some(target_id) = target_id {
            results
                .values()
                .filter(|result| result.target_id == target_id)
                .cloned()
                .collect()
        } else {
            results.values().cloned().collect()
        }
    }

    pub async fn get_health_summary(&self, target_id: Option<&str>) -> HealthSummary {
        let results = self.get_health_check_results(target_id).await;

        let total_checks = results.len() as u32;
        let healthy_checks = results
            .iter()
            .filter(|r| r.status == HealthStatus::Healthy)
            .count() as u32;
        let unhealthy_checks = results
            .iter()
            .filter(|r| r.status == HealthStatus::Unhealthy)
            .count() as u32;
        let degraded_checks = results
            .iter()
            .filter(|r| r.status == HealthStatus::Degraded)
            .count() as u32;
        let unknown_checks = results
            .iter()
            .filter(|r| r.status == HealthStatus::Unknown)
            .count() as u32;

        let overall_status = if unhealthy_checks > 0 {
            HealthStatus::Unhealthy
        } else if degraded_checks > 0 {
            HealthStatus::Degraded
        } else if healthy_checks > 0 {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };

        HealthSummary {
            overall_status,
            total_checks,
            healthy_checks,
            unhealthy_checks,
            degraded_checks,
            unknown_checks,
            checks: results,
        }
    }

    pub async fn get_health_stats(&self) -> HealthStats {
        let results = self.get_health_check_results(None).await;

        let total_checks = results.len() as u32;
        let healthy_checks = results
            .iter()
            .filter(|r| r.status == HealthStatus::Healthy)
            .count() as u32;
        let unhealthy_checks = results
            .iter()
            .filter(|r| r.status == HealthStatus::Unhealthy)
            .count() as u32;
        let degraded_checks = results
            .iter()
            .filter(|r| r.status == HealthStatus::Degraded)
            .count() as u32;
        let unknown_checks = results
            .iter()
            .filter(|r| r.status == HealthStatus::Unknown)
            .count() as u32;

        let average_response_time = if !results.is_empty() {
            let total_time: Duration = results.iter().map(|r| r.response_time).sum();
            total_time / results.len() as u32
        } else {
            Duration::ZERO
        };

        let uptime_percentage = if total_checks > 0 {
            (healthy_checks as f64 / total_checks as f64) * 100.0
        } else {
            0.0
        };

        let last_check = results.iter().map(|r| r.timestamp).max();

        HealthStats {
            total_checks,
            healthy_checks,
            unhealthy_checks,
            degraded_checks,
            unknown_checks,
            average_response_time,
            uptime_percentage,
            last_check,
        }
    }

    async fn start_health_checking(&self, check_id: &str) -> Result<()> {
        let check = {
            let checks = self.checks.read().await;
            checks.get(check_id).cloned()
        };

        if let Some(check) = check {
            if !check.enabled {
                return Ok(());
            }

            let check_id = check_id.to_string();
            let checker = Arc::clone(&self.checker);
            let results = Arc::clone(&self.results);
            let event_sender = Arc::clone(&self.event_sender);
            let checks = Arc::clone(&self.checks);

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(check.interval);

                loop {
                    interval.tick().await;

                    // Check if the health check still exists and is enabled
                    let should_continue = {
                        let checks = checks.read().await;
                        checks.get(&check_id).map_or(false, |c| c.enabled)
                    };

                    if !should_continue {
                        break;
                    }

                    // Perform health check
                    let result = checker.check_health(&check).await;

                    // Update results
                    {
                        let mut results = results.write().await;
                        results.insert(check_id.clone(), result.clone());
                    }

                    // Send events based on status change
                    let previous_result = {
                        let results = results.read().await;
                        results.get(&check_id).cloned()
                    };

                    if let Some(previous) = previous_result {
                        match (previous.status, result.status) {
                            (HealthStatus::Unhealthy, HealthStatus::Healthy) => {
                                let _ = event_sender.send(HealthEvent::CheckRecovered {
                                    check_id: check_id.clone(),
                                    target_id: check.target_id.clone(),
                                    message: result.message.clone(),
                                });
                            }
                            (HealthStatus::Healthy, HealthStatus::Unhealthy) => {
                                let _ = event_sender.send(HealthEvent::CheckFailed {
                                    check_id: check_id.clone(),
                                    target_id: check.target_id.clone(),
                                    message: result.message.clone(),
                                });
                            }
                            (_, HealthStatus::Degraded) => {
                                let _ = event_sender.send(HealthEvent::CheckDegraded {
                                    check_id: check_id.clone(),
                                    target_id: check.target_id.clone(),
                                    message: result.message.clone(),
                                });
                            }
                            (HealthStatus::Degraded, HealthStatus::Healthy) => {
                                let _ = event_sender.send(HealthEvent::CheckPassed {
                                    check_id: check_id.clone(),
                                    target_id: check.target_id.clone(),
                                    message: result.message.clone(),
                                });
                            }
                            _ => {}
                        }
                    }
                }
            });
        }

        Ok(())
    }

    pub async fn get_health_events(&self) -> tokio::sync::mpsc::UnboundedReceiver<HealthEvent> {
        let (_sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        // This would need to be implemented to forward events
        receiver
    }

    pub async fn run_health_check(&self, check_id: &str) -> Result<HealthCheckResult> {
        let check = {
            let checks = self.checks.read().await;
            checks.get(check_id).cloned()
        };

        match check {
            Some(check) => {
                let result = self.checker.check_health(&check).await;

                // Update results
                {
                    let mut results = self.results.write().await;
                    results.insert(check_id.to_string(), result.clone());
                }

                Ok(result)
            }
            None => Err(anyhow::anyhow!("Health check not found: {}", check_id)),
        }
    }
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            command_executor: Arc::new(CommandExecutor::new()),
        }
    }

    pub async fn check_health(&self, check: &HealthCheck) -> HealthCheckResult {
        let start_time = Instant::now();
        let mut result = HealthCheckResult {
            check_id: check.id.clone(),
            target_id: check.target_id.clone(),
            status: HealthStatus::Unknown,
            message: String::new(),
            response_time: Duration::ZERO,
            timestamp: Utc::now(),
            consecutive_failures: 0,
            consecutive_successes: 0,
            metadata: HashMap::new(),
        };

        for attempt in 0..check.retries {
            match self.perform_check(check).await {
                Ok(status) => {
                    result.status = status;
                    result.message = "Health check passed".to_string();
                    result.consecutive_successes += 1;
                    result.consecutive_failures = 0;
                    break;
                }
                Err(e) => {
                    result.message =
                        format!("Health check failed (attempt {}): {}", attempt + 1, e);
                    result.consecutive_failures += 1;
                    result.consecutive_successes = 0;

                    if attempt < check.retries - 1 {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }

        result.response_time = start_time.elapsed();
        result
    }

    async fn perform_check(&self, check: &HealthCheck) -> Result<HealthStatus> {
        match &check.check_type {
            CheckType::Http {
                path,
                expected_status,
            } => self.check_http(check, path, *expected_status).await,
            CheckType::Tcp { port } => self.check_tcp(check, *port).await,
            CheckType::Udp { port } => self.check_udp(check, *port).await,
            CheckType::Grpc { service, method } => self.check_grpc(check, service, method).await,
            CheckType::Command { command, args } => self.check_command(check, command, args).await,
            CheckType::File { path, exists } => self.check_file(check, path, *exists).await,
            CheckType::Custom { script } => self.check_custom(check, script).await,
        }
    }

    async fn check_http(
        &self,
        check: &HealthCheck,
        path: &str,
        expected_status: u16,
    ) -> Result<HealthStatus> {
        let url = format!("http://{}:{}", check.target_id, 8080); // Assuming port 8080
        let full_url = format!("{}{}", url, path);

        let response = self
            .client
            .get(&full_url)
            .timeout(check.timeout)
            .send()
            .await?;

        if response.status().as_u16() == expected_status {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy)
        }
    }

    async fn check_tcp(&self, check: &HealthCheck, port: u16) -> Result<HealthStatus> {
        use tokio::net::TcpStream;

        let address = format!("{}:{}", check.target_id, port);
        match TcpStream::connect(&address).await {
            Ok(_) => Ok(HealthStatus::Healthy),
            Err(_) => Ok(HealthStatus::Unhealthy),
        }
    }

    async fn check_udp(&self, check: &HealthCheck, port: u16) -> Result<HealthStatus> {
        use tokio::net::UdpSocket;

        let address = format!("{}:{}", check.target_id, port);
        match UdpSocket::bind("0.0.0.0:0").await {
            Ok(socket) => match socket.connect(&address).await {
                Ok(_) => Ok(HealthStatus::Healthy),
                Err(_) => Ok(HealthStatus::Unhealthy),
            },
            Err(_) => Ok(HealthStatus::Unhealthy),
        }
    }

    async fn check_grpc(
        &self,
        check: &HealthCheck,
        _service: &str,
        _method: &str,
    ) -> Result<HealthStatus> {
        // This is a simplified gRPC health check
        // In practice, you would use a proper gRPC client
        let url = format!("http://{}:{}", check.target_id, 8080);
        let response = self
            .client
            .post(&url)
            .header("content-type", "application/grpc")
            .timeout(check.timeout)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(HealthStatus::Healthy)
        } else {
            Ok(HealthStatus::Unhealthy)
        }
    }

    async fn check_command(
        &self,
        check: &HealthCheck,
        command: &str,
        args: &[String],
    ) -> Result<HealthStatus> {
        self.command_executor
            .execute(command, args, check.timeout)
            .await
    }

    async fn check_file(
        &self,
        check: &HealthCheck,
        path: &str,
        exists: bool,
    ) -> Result<HealthStatus> {
        use tokio::fs;

        let full_path = format!("{}/{}", check.target_id, path);
        match fs::metadata(&full_path).await {
            Ok(_) => {
                if exists {
                    Ok(HealthStatus::Healthy)
                } else {
                    Ok(HealthStatus::Unhealthy)
                }
            }
            Err(_) => {
                if exists {
                    Ok(HealthStatus::Unhealthy)
                } else {
                    Ok(HealthStatus::Healthy)
                }
            }
        }
    }

    async fn check_custom(&self, check: &HealthCheck, script: &str) -> Result<HealthStatus> {
        // Execute custom script
        self.command_executor
            .execute("sh", &["-c".to_string(), script.to_string()], check.timeout)
            .await
    }
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }

    pub async fn execute(
        &self,
        command: &str,
        args: &[String],
        timeout: Duration,
    ) -> Result<HealthStatus> {
        use tokio::process::Command;

        let mut cmd = Command::new(command);
        cmd.args(args);

        match tokio::time::timeout(timeout, cmd.output()).await {
            Ok(Ok(output)) => {
                if output.status.success() {
                    Ok(HealthStatus::Healthy)
                } else {
                    Ok(HealthStatus::Unhealthy)
                }
            }
            Ok(Err(_)) => Ok(HealthStatus::Unhealthy),
            Err(_) => Ok(HealthStatus::Unhealthy),
        }
    }
}

impl HealthCheck {
    pub fn new(
        id: String,
        name: String,
        target_type: TargetType,
        target_id: String,
        check_type: CheckType,
    ) -> Self {
        Self {
            id,
            name,
            target_type,
            target_id,
            check_type,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            retries: 3,
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    pub fn with_label(mut self, key: String, value: String) -> Self {
        self.labels.insert(key, value);
        self
    }

    pub fn with_annotation(mut self, key: String, value: String) -> Self {
        self.annotations.insert(key, value);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_health_monitor() {
        let monitor = HealthMonitor::new();

        let check = HealthCheck::new(
            "test-check".to_string(),
            "test".to_string(),
            TargetType::Container,
            "test-container".to_string(),
            CheckType::Http {
                path: "/health".to_string(),
                expected_status: 200,
            },
        )
        .with_interval(Duration::from_secs(10))
        .with_timeout(Duration::from_secs(5));

        monitor.create_health_check(check).await.unwrap();

        let checks = monitor.list_health_checks().await;
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].name, "test");
    }

    #[tokio::test]
    async fn test_health_check_types() {
        let monitor = HealthMonitor::new();

        // HTTP check
        let http_check = HealthCheck::new(
            "http-check".to_string(),
            "HTTP".to_string(),
            TargetType::Service,
            "test-service".to_string(),
            CheckType::Http {
                path: "/health".to_string(),
                expected_status: 200,
            },
        );

        // TCP check
        let tcp_check = HealthCheck::new(
            "tcp-check".to_string(),
            "TCP".to_string(),
            TargetType::Container,
            "test-container".to_string(),
            CheckType::Tcp { port: 8080 },
        );

        // Command check
        let cmd_check = HealthCheck::new(
            "cmd-check".to_string(),
            "Command".to_string(),
            TargetType::Container,
            "test-container".to_string(),
            CheckType::Command {
                command: "echo".to_string(),
                args: vec!["hello".to_string()],
            },
        );

        monitor.create_health_check(http_check).await.unwrap();
        monitor.create_health_check(tcp_check).await.unwrap();
        monitor.create_health_check(cmd_check).await.unwrap();

        let checks = monitor.list_health_checks().await;
        assert_eq!(checks.len(), 3);
    }

    #[tokio::test]
    async fn test_health_summary() {
        let monitor = HealthMonitor::new();

        let summary = monitor.get_health_summary(None).await;
        assert_eq!(summary.total_checks, 0);
        assert_eq!(summary.overall_status, HealthStatus::Unknown);
    }

    #[tokio::test]
    async fn test_health_stats() {
        let monitor = HealthMonitor::new();

        let stats = monitor.get_health_stats().await;
        assert_eq!(stats.total_checks, 0);
        assert_eq!(stats.uptime_percentage, 0.0);
    }
}
