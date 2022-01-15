use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

// use polis_core::{PolisError, Result as PolisResult};

/// Auto-scaling manager
pub struct AutoScaler {
    policies: Arc<RwLock<HashMap<String, ScalingPolicy>>>,
    deployments: Arc<RwLock<HashMap<String, Deployment>>>,
    metrics_collector: Arc<MetricsCollector>,
    scaling_engine: Arc<ScalingEngine>,
    event_sender: Arc<tokio::sync::mpsc::UnboundedSender<ScalingEvent>>,
}

/// Scaling policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    pub id: String,
    pub name: String,
    pub deployment_id: String,
    pub min_replicas: u32,
    pub max_replicas: u32,
    pub target_cpu_utilization: f64,
    pub target_memory_utilization: f64,
    pub target_requests_per_second: f64,
    pub scale_up_cooldown: Duration,
    pub scale_down_cooldown: Duration,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub image: String,
    pub replicas: u32,
    pub desired_replicas: u32,
    pub ready_replicas: u32,
    pub available_replicas: u32,
    pub unavailable_replicas: u32,
    pub status: DeploymentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub resource_limits: ResourceLimits,
    pub resource_requests: ResourceRequests,
}

/// Deployment status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentStatus {
    Pending,
    Running,
    Failed,
    Scaling,
    Paused,
}

/// Resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub storage: Option<String>,
}

/// Resource requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequests {
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub storage: Option<String>,
}

/// Scaling metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingMetrics {
    pub deployment_id: String,
    pub timestamp: DateTime<Utc>,
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub requests_per_second: f64,
    pub response_time: Duration,
    pub error_rate: f64,
    pub active_connections: u32,
}

/// Scaling event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingEvent {
    ScaleUp {
        deployment_id: String,
        from: u32,
        to: u32,
        reason: String,
    },
    ScaleDown {
        deployment_id: String,
        from: u32,
        to: u32,
        reason: String,
    },
    ScalingBlocked {
        deployment_id: String,
        reason: String,
    },
    PolicyUpdated {
        policy_id: String,
    },
    DeploymentUpdated {
        deployment_id: String,
    },
}

/// Metrics collector
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, Vec<ScalingMetrics>>>>,
    collection_interval: Duration,
}

/// Scaling engine
pub struct ScalingEngine {
    auto_scaler: Arc<AutoScaler>,
    scaling_history: Arc<RwLock<Vec<ScalingAction>>>,
}

/// Scaling action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingAction {
    pub deployment_id: String,
    pub action_type: ScalingActionType,
    pub from_replicas: u32,
    pub to_replicas: u32,
    pub reason: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
}

/// Scaling action type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScalingActionType {
    ScaleUp,
    ScaleDown,
    NoAction,
}

impl AutoScaler {
    pub fn new() -> Self {
        let (event_sender, _event_receiver) = tokio::sync::mpsc::unbounded_channel();

        let auto_scaler = Arc::new(AutoScaler {
            policies: Arc::new(RwLock::new(HashMap::new())),
            deployments: Arc::new(RwLock::new(HashMap::new())),
            metrics_collector: Arc::new(MetricsCollector::new()),
            scaling_engine: Arc::new(ScalingEngine::new()),
            event_sender: Arc::new(event_sender),
        });

        let scaling_engine = Arc::new(ScalingEngine::new());
        scaling_engine.set_auto_scaler(Arc::clone(&auto_scaler));

        Self {
            policies: auto_scaler.policies.clone(),
            deployments: auto_scaler.deployments.clone(),
            metrics_collector: auto_scaler.metrics_collector.clone(),
            scaling_engine: auto_scaler.scaling_engine.clone(),
            event_sender: auto_scaler.event_sender.clone(),
        }
    }

    pub async fn create_scaling_policy(&self, policy: ScalingPolicy) -> Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy.clone());
        drop(policies);

        // Send event
        let _ = self.event_sender.send(ScalingEvent::PolicyUpdated {
            policy_id: policy.id.clone(),
        });

        Ok(())
    }

    pub async fn update_scaling_policy(&self, policy: ScalingPolicy) -> Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.id.clone(), policy.clone());
        drop(policies);

        // Send event
        let _ = self.event_sender.send(ScalingEvent::PolicyUpdated {
            policy_id: policy.id.clone(),
        });

        Ok(())
    }

    pub async fn delete_scaling_policy(&self, policy_id: &str) -> Result<()> {
        let mut policies = self.policies.write().await;
        policies.remove(policy_id);
        Ok(())
    }

    pub async fn get_scaling_policy(&self, policy_id: &str) -> Option<ScalingPolicy> {
        let policies = self.policies.read().await;
        policies.get(policy_id).cloned()
    }

    pub async fn list_scaling_policies(&self) -> Vec<ScalingPolicy> {
        let policies = self.policies.read().await;
        policies.values().cloned().collect()
    }

    pub async fn create_deployment(&self, deployment: Deployment) -> Result<()> {
        let mut deployments = self.deployments.write().await;
        deployments.insert(deployment.id.clone(), deployment.clone());
        drop(deployments);

        // Send event
        let _ = self.event_sender.send(ScalingEvent::DeploymentUpdated {
            deployment_id: deployment.id.clone(),
        });

        Ok(())
    }

    pub async fn update_deployment(&self, deployment: Deployment) -> Result<()> {
        let mut deployments = self.deployments.write().await;
        deployments.insert(deployment.id.clone(), deployment.clone());
        drop(deployments);

        // Send event
        let _ = self.event_sender.send(ScalingEvent::DeploymentUpdated {
            deployment_id: deployment.id.clone(),
        });

        Ok(())
    }

    pub async fn get_deployment(&self, deployment_id: &str) -> Option<Deployment> {
        let deployments = self.deployments.read().await;
        deployments.get(deployment_id).cloned()
    }

    pub async fn list_deployments(&self) -> Vec<Deployment> {
        let deployments = self.deployments.read().await;
        deployments.values().cloned().collect()
    }

    pub async fn collect_metrics(
        &self,
        deployment_id: &str,
        metrics: ScalingMetrics,
    ) -> Result<()> {
        self.metrics_collector
            .collect_metrics(deployment_id, metrics)
            .await
    }

    pub async fn evaluate_scaling(&self, deployment_id: &str) -> Result<ScalingAction> {
        let policy = self.get_scaling_policy_for_deployment(deployment_id).await;
        let deployment = self.get_deployment(deployment_id).await;
        let metrics = self
            .metrics_collector
            .get_latest_metrics(deployment_id)
            .await;

        if policy.is_none() || deployment.is_none() || metrics.is_none() {
            return Ok(ScalingAction {
                deployment_id: deployment_id.to_string(),
                action_type: ScalingActionType::NoAction,
                from_replicas: 0,
                to_replicas: 0,
                reason: "Missing policy, deployment, or metrics".to_string(),
                timestamp: Utc::now(),
                success: false,
            });
        }

        let policy = policy.unwrap();
        let deployment = deployment.unwrap();
        let metrics = metrics.unwrap();

        if !policy.enabled {
            return Ok(ScalingAction {
                deployment_id: deployment_id.to_string(),
                action_type: ScalingActionType::NoAction,
                from_replicas: deployment.replicas,
                to_replicas: deployment.replicas,
                reason: "Scaling policy disabled".to_string(),
                timestamp: Utc::now(),
                success: true,
            });
        }

        let current_replicas = deployment.replicas;
        let mut desired_replicas = current_replicas;
        let mut reason = String::new();

        // Check if we need to scale up
        if metrics.cpu_utilization > policy.target_cpu_utilization
            || metrics.memory_utilization > policy.target_memory_utilization
            || metrics.requests_per_second > policy.target_requests_per_second
        {
            if current_replicas < policy.max_replicas {
                desired_replicas = (current_replicas * 2).min(policy.max_replicas);
                reason = format!(
                    "High utilization: CPU={:.1}%, Memory={:.1}%, RPS={:.1}",
                    metrics.cpu_utilization,
                    metrics.memory_utilization,
                    metrics.requests_per_second
                );
            }
        }
        // Check if we need to scale down
        else if metrics.cpu_utilization < policy.target_cpu_utilization * 0.5
            && metrics.memory_utilization < policy.target_memory_utilization * 0.5
            && metrics.requests_per_second < policy.target_requests_per_second * 0.5
        {
            if current_replicas > policy.min_replicas {
                desired_replicas = (current_replicas / 2).max(policy.min_replicas);
                reason = format!(
                    "Low utilization: CPU={:.1}%, Memory={:.1}%, RPS={:.1}",
                    metrics.cpu_utilization,
                    metrics.memory_utilization,
                    metrics.requests_per_second
                );
            }
        }

        let action_type = if desired_replicas > current_replicas {
            ScalingActionType::ScaleUp
        } else if desired_replicas < current_replicas {
            ScalingActionType::ScaleDown
        } else {
            ScalingActionType::NoAction
        };

        let action = ScalingAction {
            deployment_id: deployment_id.to_string(),
            action_type,
            from_replicas: current_replicas,
            to_replicas: desired_replicas,
            reason,
            timestamp: Utc::now(),
            success: true,
        };

        // Apply scaling action
        if action.action_type != ScalingActionType::NoAction {
            self.apply_scaling_action(&action).await?;
        }

        Ok(action)
    }

    async fn get_scaling_policy_for_deployment(
        &self,
        deployment_id: &str,
    ) -> Option<ScalingPolicy> {
        let policies = self.policies.read().await;
        policies
            .values()
            .find(|policy| policy.deployment_id == deployment_id)
            .cloned()
    }

    async fn apply_scaling_action(&self, action: &ScalingAction) -> Result<()> {
        let mut deployments = self.deployments.write().await;
        if let Some(deployment) = deployments.get_mut(&action.deployment_id) {
            deployment.replicas = action.to_replicas;
            deployment.desired_replicas = action.to_replicas;
            deployment.updated_at = Utc::now();

            if action.action_type == ScalingActionType::ScaleUp {
                deployment.status = DeploymentStatus::Scaling;
            }
        }
        drop(deployments);

        // Send scaling event
        let event = match action.action_type {
            ScalingActionType::ScaleUp => ScalingEvent::ScaleUp {
                deployment_id: action.deployment_id.clone(),
                from: action.from_replicas,
                to: action.to_replicas,
                reason: action.reason.clone(),
            },
            ScalingActionType::ScaleDown => ScalingEvent::ScaleDown {
                deployment_id: action.deployment_id.clone(),
                from: action.from_replicas,
                to: action.to_replicas,
                reason: action.reason.clone(),
            },
            ScalingActionType::NoAction => return Ok(()),
        };

        let _ = self.event_sender.send(event);

        Ok(())
    }

    pub async fn start_scaling_loop(&self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            let deployments = self.list_deployments().await;
            for deployment in deployments {
                if let Err(e) = self.evaluate_scaling(&deployment.id).await {
                    eprintln!(
                        "Error evaluating scaling for deployment {}: {}",
                        deployment.id, e
                    );
                }
            }
        }
    }

    pub async fn get_scaling_events(&self) -> tokio::sync::mpsc::UnboundedReceiver<ScalingEvent> {
        let (_sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        // This would need to be implemented to forward events
        receiver
    }

    pub async fn get_scaling_history(&self, deployment_id: &str) -> Vec<ScalingAction> {
        self.scaling_engine.get_scaling_history(deployment_id).await
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            collection_interval: Duration::from_secs(30),
        }
    }

    pub async fn collect_metrics(
        &self,
        deployment_id: &str,
        metrics: ScalingMetrics,
    ) -> Result<()> {
        let mut metrics_map = self.metrics.write().await;
        let deployment_metrics = metrics_map
            .entry(deployment_id.to_string())
            .or_insert_with(Vec::new);
        deployment_metrics.push(metrics);

        // Keep only last 100 metrics per deployment
        if deployment_metrics.len() > 100 {
            deployment_metrics.drain(0..deployment_metrics.len() - 100);
        }

        Ok(())
    }

    pub async fn get_latest_metrics(&self, deployment_id: &str) -> Option<ScalingMetrics> {
        let metrics_map = self.metrics.read().await;
        metrics_map.get(deployment_id)?.last().cloned()
    }

    pub async fn get_metrics_history(
        &self,
        deployment_id: &str,
        limit: usize,
    ) -> Vec<ScalingMetrics> {
        let metrics_map = self.metrics.read().await;
        if let Some(metrics) = metrics_map.get(deployment_id) {
            metrics.iter().rev().take(limit).cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub async fn get_average_metrics(
        &self,
        deployment_id: &str,
        duration: Duration,
    ) -> Option<ScalingMetrics> {
        let metrics_map = self.metrics.read().await;
        let metrics = metrics_map.get(deployment_id)?;

        let cutoff_time = Utc::now() - chrono::Duration::from_std(duration).unwrap_or_default();
        let recent_metrics: Vec<&ScalingMetrics> = metrics
            .iter()
            .filter(|m| m.timestamp >= cutoff_time)
            .collect();

        if recent_metrics.is_empty() {
            return None;
        }

        let avg_cpu = recent_metrics
            .iter()
            .map(|m| m.cpu_utilization)
            .sum::<f64>()
            / recent_metrics.len() as f64;
        let avg_memory = recent_metrics
            .iter()
            .map(|m| m.memory_utilization)
            .sum::<f64>()
            / recent_metrics.len() as f64;
        let avg_rps = recent_metrics
            .iter()
            .map(|m| m.requests_per_second)
            .sum::<f64>()
            / recent_metrics.len() as f64;
        let avg_response_time = recent_metrics
            .iter()
            .map(|m| m.response_time)
            .sum::<Duration>()
            / recent_metrics.len() as u32;
        let avg_error_rate =
            recent_metrics.iter().map(|m| m.error_rate).sum::<f64>() / recent_metrics.len() as f64;
        let avg_connections = recent_metrics
            .iter()
            .map(|m| m.active_connections)
            .sum::<u32>()
            / recent_metrics.len() as u32;

        Some(ScalingMetrics {
            deployment_id: deployment_id.to_string(),
            timestamp: Utc::now(),
            cpu_utilization: avg_cpu,
            memory_utilization: avg_memory,
            requests_per_second: avg_rps,
            response_time: avg_response_time,
            error_rate: avg_error_rate,
            active_connections: avg_connections,
        })
    }
}

impl ScalingEngine {
    pub fn new() -> Self {
        Self {
            auto_scaler: Arc::new(AutoScaler::new()),
            scaling_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn set_auto_scaler(&self, _auto_scaler: Arc<AutoScaler>) {
        // This would need to be implemented properly
    }

    pub async fn get_scaling_history(&self, deployment_id: &str) -> Vec<ScalingAction> {
        let history = self.scaling_history.read().await;
        history
            .iter()
            .filter(|action| action.deployment_id == deployment_id)
            .cloned()
            .collect()
    }

    pub async fn add_scaling_action(&self, action: ScalingAction) {
        let mut history = self.scaling_history.write().await;
        history.push(action);

        // Keep only last 1000 actions
        if history.len() > 1000 {
            let keep_count = history.len() - 1000;
            history.drain(0..keep_count);
        }
    }
}

impl ScalingPolicy {
    pub fn new(
        id: String,
        name: String,
        deployment_id: String,
        min_replicas: u32,
        max_replicas: u32,
    ) -> Self {
        Self {
            id,
            name,
            deployment_id,
            min_replicas,
            max_replicas,
            target_cpu_utilization: 70.0,
            target_memory_utilization: 80.0,
            target_requests_per_second: 100.0,
            scale_up_cooldown: Duration::from_secs(300), // 5 minutes
            scale_down_cooldown: Duration::from_secs(600), // 10 minutes
            enabled: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn with_target_cpu_utilization(mut self, target: f64) -> Self {
        self.target_cpu_utilization = target;
        self
    }

    pub fn with_target_memory_utilization(mut self, target: f64) -> Self {
        self.target_memory_utilization = target;
        self
    }

    pub fn with_target_requests_per_second(mut self, target: f64) -> Self {
        self.target_requests_per_second = target;
        self
    }

    pub fn with_scale_up_cooldown(mut self, cooldown: Duration) -> Self {
        self.scale_up_cooldown = cooldown;
        self
    }

    pub fn with_scale_down_cooldown(mut self, cooldown: Duration) -> Self {
        self.scale_down_cooldown = cooldown;
        self
    }
}

impl Deployment {
    pub fn new(id: String, name: String, namespace: String, image: String) -> Self {
        Self {
            id,
            name,
            namespace,
            image,
            replicas: 1,
            desired_replicas: 1,
            ready_replicas: 0,
            available_replicas: 0,
            unavailable_replicas: 0,
            status: DeploymentStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
            resource_limits: ResourceLimits {
                cpu: None,
                memory: None,
                storage: None,
            },
            resource_requests: ResourceRequests {
                cpu: None,
                memory: None,
                storage: None,
            },
        }
    }

    pub fn with_replicas(mut self, replicas: u32) -> Self {
        self.replicas = replicas;
        self.desired_replicas = replicas;
        self
    }

    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    pub fn with_resource_requests(mut self, requests: ResourceRequests) -> Self {
        self.resource_requests = requests;
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
    async fn test_auto_scaler() {
        let auto_scaler = AutoScaler::new();

        let deployment = Deployment::new(
            "test-deployment".to_string(),
            "test".to_string(),
            "default".to_string(),
            "nginx:latest".to_string(),
        )
        .with_replicas(2);

        auto_scaler.create_deployment(deployment).await.unwrap();

        let policy = ScalingPolicy::new(
            "test-policy".to_string(),
            "test".to_string(),
            "test-deployment".to_string(),
            1,
            10,
        )
        .with_target_cpu_utilization(50.0);

        auto_scaler.create_scaling_policy(policy).await.unwrap();

        let metrics = ScalingMetrics {
            deployment_id: "test-deployment".to_string(),
            timestamp: Utc::now(),
            cpu_utilization: 80.0,
            memory_utilization: 60.0,
            requests_per_second: 150.0,
            response_time: Duration::from_millis(100),
            error_rate: 0.01,
            active_connections: 50,
        };

        auto_scaler
            .collect_metrics("test-deployment", metrics)
            .await
            .unwrap();

        let action = auto_scaler
            .evaluate_scaling("test-deployment")
            .await
            .unwrap();
        assert_eq!(action.action_type, ScalingActionType::ScaleUp);
        assert!(action.to_replicas > action.from_replicas);
    }

    #[tokio::test]
    async fn test_scaling_policy() {
        let policy = ScalingPolicy::new(
            "test-policy".to_string(),
            "test".to_string(),
            "test-deployment".to_string(),
            1,
            10,
        )
        .with_target_cpu_utilization(70.0)
        .with_target_memory_utilization(80.0)
        .with_scale_up_cooldown(Duration::from_secs(300));

        assert_eq!(policy.min_replicas, 1);
        assert_eq!(policy.max_replicas, 10);
        assert_eq!(policy.target_cpu_utilization, 70.0);
        assert_eq!(policy.target_memory_utilization, 80.0);
    }

    #[tokio::test]
    async fn test_deployment() {
        let deployment = Deployment::new(
            "test-deployment".to_string(),
            "test".to_string(),
            "default".to_string(),
            "nginx:latest".to_string(),
        )
        .with_replicas(3)
        .with_label("app".to_string(), "nginx".to_string());

        assert_eq!(deployment.replicas, 3);
        assert_eq!(deployment.desired_replicas, 3);
        assert_eq!(deployment.labels.get("app"), Some(&"nginx".to_string()));
    }

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();

        let metrics = ScalingMetrics {
            deployment_id: "test-deployment".to_string(),
            timestamp: Utc::now(),
            cpu_utilization: 50.0,
            memory_utilization: 60.0,
            requests_per_second: 100.0,
            response_time: Duration::from_millis(200),
            error_rate: 0.01,
            active_connections: 25,
        };

        collector
            .collect_metrics("test-deployment", metrics)
            .await
            .unwrap();

        let latest = collector.get_latest_metrics("test-deployment").await;
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().cpu_utilization, 50.0);
    }
}
