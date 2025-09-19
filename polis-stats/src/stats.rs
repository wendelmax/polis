use crate::{ContainerMetrics, Result, StatsError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Container statistics manager
#[derive(Debug)]
pub struct StatsManager {
    /// Container metrics cache
    metrics: Arc<RwLock<HashMap<String, ContainerMetrics>>>,
    /// Statistics collection interval (seconds)
    collection_interval: u64,
}

impl StatsManager {
    /// Create a new stats manager
    pub fn new(collection_interval: u64) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            collection_interval,
        }
    }

    /// Get metrics for a specific container
    pub async fn get_metrics(&self, container_id: &str) -> Result<Option<ContainerMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.get(container_id).cloned())
    }

    /// Get metrics for all containers
    pub async fn get_all_metrics(&self) -> Result<Vec<ContainerMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.values().cloned().collect())
    }

    /// Update metrics for a container
    pub async fn update_metrics(&self, container_id: &str, metrics: ContainerMetrics) -> Result<()> {
        let mut container_metrics = self.metrics.write().await;
        container_metrics.insert(container_id.to_string(), metrics);
        Ok(())
    }

    /// Remove metrics for a container
    pub async fn remove_metrics(&self, container_id: &str) -> Result<()> {
        let mut container_metrics = self.metrics.write().await;
        container_metrics.remove(container_id);
        Ok(())
    }

    /// Get container statistics summary
    pub async fn get_summary(&self) -> Result<StatsSummary> {
        let metrics = self.metrics.read().await;
        let mut summary = StatsSummary::default();
        
        for container_metrics in metrics.values() {
            summary.total_containers += 1;
            summary.total_cpu_usage += container_metrics.cpu.usage_percent;
            summary.total_memory_usage += container_metrics.memory.usage;
            summary.total_network_rx += container_metrics.network.rx_bytes;
            summary.total_network_tx += container_metrics.network.tx_bytes;
            summary.total_disk_read += container_metrics.disk.read_bytes;
            summary.total_disk_write += container_metrics.disk.write_bytes;
            summary.total_processes += container_metrics.processes.process_count;
        }
        
        if summary.total_containers > 0 {
            summary.avg_cpu_usage = summary.total_cpu_usage / summary.total_containers as f64;
        }
        
        Ok(summary)
    }

    /// Start continuous monitoring for a container
    pub async fn start_monitoring(&self, container_id: &str) -> Result<()> {
        // This would typically start a background task
        // For now, we'll just log the start
        tracing::info!("Started monitoring container: {}", container_id);
        Ok(())
    }

    /// Stop continuous monitoring for a container
    pub async fn stop_monitoring(&self, container_id: &str) -> Result<()> {
        // This would typically stop a background task
        // For now, we'll just log the stop
        tracing::info!("Stopped monitoring container: {}", container_id);
        Ok(())
    }
}

/// Statistics summary
#[derive(Debug, Clone, Default)]
pub struct StatsSummary {
    /// Total number of containers
    pub total_containers: u32,
    /// Total CPU usage percentage
    pub total_cpu_usage: f64,
    /// Average CPU usage percentage
    pub avg_cpu_usage: f64,
    /// Total memory usage in bytes
    pub total_memory_usage: u64,
    /// Total network RX bytes
    pub total_network_rx: u64,
    /// Total network TX bytes
    pub total_network_tx: u64,
    /// Total disk read bytes
    pub total_disk_read: u64,
    /// Total disk write bytes
    pub total_disk_write: u64,
    /// Total number of processes
    pub total_processes: u32,
}

impl StatsManager {
    /// Create a new stats manager with default settings
    pub fn default() -> Self {
        Self::new(5) // 5 second collection interval
    }
}
