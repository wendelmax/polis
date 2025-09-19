use crate::{ContainerMetrics, Result, StatsError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};
use tracing::{info, warn, error};

/// Container statistics collector with real-time monitoring
#[derive(Debug)]
pub struct ContainerStatsCollector {
    /// Container metrics cache
    metrics: Arc<RwLock<HashMap<String, ContainerMetrics>>>,
    /// Collection interval
    collection_interval: Duration,
    /// Running state
    running: Arc<RwLock<bool>>,
}

impl ContainerStatsCollector {
    /// Create a new container stats collector
    pub fn new(collection_interval: Duration) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            collection_interval,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start collecting statistics for a container
    pub async fn start_collecting(&self, container_id: &str) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if !metrics.contains_key(container_id) {
            metrics.insert(container_id.to_string(), ContainerMetrics::default());
            info!("Started collecting stats for container: {}", container_id);
        }
        Ok(())
    }

    /// Stop collecting statistics for a container
    pub async fn stop_collecting(&self, container_id: &str) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if metrics.remove(container_id).is_some() {
            info!("Stopped collecting stats for container: {}", container_id);
        }
        Ok(())
    }

    /// Get current metrics for a container
    pub async fn get_metrics(&self, container_id: &str) -> Result<Option<ContainerMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.get(container_id).cloned())
    }

    /// Get all container metrics
    pub async fn get_all_metrics(&self) -> Result<Vec<ContainerMetrics>> {
        let metrics = self.metrics.read().await;
        Ok(metrics.values().cloned().collect())
    }

    /// Update metrics for a container
    pub async fn update_metrics(&self, container_id: &str, new_metrics: ContainerMetrics) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        metrics.insert(container_id.to_string(), new_metrics);
        Ok(())
    }

    /// Get container statistics summary
    pub async fn get_summary(&self) -> Result<ContainerStatsSummary> {
        let metrics = self.metrics.read().await;
        let mut summary = ContainerStatsSummary::default();
        
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
            summary.avg_memory_usage = summary.total_memory_usage / summary.total_containers as u64;
        }
        
        Ok(summary)
    }

    /// Start continuous monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        let metrics = Arc::clone(&self.metrics);
        let collection_interval = self.collection_interval;
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(collection_interval);
            
            loop {
                interval.tick().await;
                
                let should_continue = {
                    let running_guard = running.read().await;
                    *running_guard
                };
                
                if !should_continue {
                    break;
                }

                // Update metrics for all containers
                let container_ids: Vec<String> = {
                    let metrics_guard = metrics.read().await;
                    metrics_guard.keys().cloned().collect()
                };

                for container_id in container_ids {
                    if let Err(e) = Self::update_container_metrics(&metrics, &container_id).await {
                        error!("Failed to update metrics for container {}: {}", container_id, e);
                    }
                }
            }
        });

        info!("Started continuous monitoring with interval: {:?}", self.collection_interval);
        Ok(())
    }

    /// Stop continuous monitoring
    pub async fn stop_monitoring(&self) -> Result<()> {
        let mut running = self.running.write().await;
        *running = false;
        info!("Stopped continuous monitoring");
        Ok(())
    }

    /// Update metrics for a specific container
    async fn update_container_metrics(
        metrics: &Arc<RwLock<HashMap<String, ContainerMetrics>>>,
        container_id: &str,
    ) -> Result<()> {
        // This would typically read from /proc/[pid]/stat, /proc/[pid]/status, etc.
        // For now, we'll simulate some metrics
        let mut new_metrics = ContainerMetrics::default();
        new_metrics.container_id = container_id.to_string();
        new_metrics.timestamp = std::time::SystemTime::now();
        
        // Simulate some CPU usage
        new_metrics.cpu.usage_percent = (rand::random::<f64>() * 100.0).min(100.0);
        new_metrics.cpu.cores = num_cpus::get();
        
        // Simulate some memory usage
        new_metrics.memory.usage = rand::random::<u64>() % (1024 * 1024 * 1024); // Up to 1GB
        new_metrics.memory.limit = 1024 * 1024 * 1024 * 2; // 2GB limit
        new_metrics.memory.usage_percent = (new_metrics.memory.usage as f64 / new_metrics.memory.limit as f64) * 100.0;
        
        // Simulate some network activity
        new_metrics.network.rx_bytes = rand::random::<u64>() % (100 * 1024 * 1024); // Up to 100MB
        new_metrics.network.tx_bytes = rand::random::<u64>() % (100 * 1024 * 1024); // Up to 100MB
        new_metrics.network.rx_packets = rand::random::<u64>() % 10000;
        new_metrics.network.tx_packets = rand::random::<u64>() % 10000;
        
        // Simulate some disk activity
        new_metrics.disk.read_bytes = rand::random::<u64>() % (50 * 1024 * 1024); // Up to 50MB
        new_metrics.disk.write_bytes = rand::random::<u64>() % (50 * 1024 * 1024); // Up to 50MB
        new_metrics.disk.read_ops = rand::random::<u64>() % 1000;
        new_metrics.disk.write_ops = rand::random::<u64>() % 1000;
        
        // Simulate some process activity
        new_metrics.processes.process_count = (rand::random::<u32>() % 50) + 1;
        new_metrics.processes.thread_count = new_metrics.processes.process_count * 2;
        new_metrics.processes.fd_count = rand::random::<u32>() % 100;
        new_metrics.processes.state = "running".to_string();

        let mut metrics_guard = metrics.write().await;
        metrics_guard.insert(container_id.to_string(), new_metrics);
        
        Ok(())
    }
}

/// Container statistics summary
#[derive(Debug, Clone, Default)]
pub struct ContainerStatsSummary {
    /// Total number of containers
    pub total_containers: u32,
    /// Total CPU usage percentage
    pub total_cpu_usage: f64,
    /// Average CPU usage percentage
    pub avg_cpu_usage: f64,
    /// Total memory usage in bytes
    pub total_memory_usage: u64,
    /// Average memory usage in bytes
    pub avg_memory_usage: u64,
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

impl Default for ContainerStatsCollector {
    fn default() -> Self {
        Self::new(Duration::from_secs(5))
    }
}
