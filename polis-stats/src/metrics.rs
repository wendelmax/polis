use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Container resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerMetrics {
    /// Container ID
    pub container_id: String,
    /// Timestamp when metrics were collected
    pub timestamp: SystemTime,
    /// CPU usage metrics
    pub cpu: CpuMetrics,
    /// Memory usage metrics
    pub memory: MemoryMetrics,
    /// Network I/O metrics
    pub network: NetworkMetrics,
    /// Disk I/O metrics
    pub disk: DiskMetrics,
    /// Process metrics
    pub processes: ProcessMetrics,
}

/// CPU usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuMetrics {
    /// CPU usage percentage (0.0 - 100.0)
    pub usage_percent: f64,
    /// Number of CPU cores
    pub cores: usize,
    /// CPU time in user mode (nanoseconds)
    pub user_time: u64,
    /// CPU time in system mode (nanoseconds)
    pub system_time: u64,
    /// Total CPU time (nanoseconds)
    pub total_time: u64,
    /// CPU throttling count
    pub throttled_count: u64,
    /// CPU throttling time (nanoseconds)
    pub throttled_time: u64,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Memory usage in bytes
    pub usage: u64,
    /// Memory limit in bytes (0 = unlimited)
    pub limit: u64,
    /// Memory usage percentage (0.0 - 100.0)
    pub usage_percent: f64,
    /// Peak memory usage in bytes
    pub peak_usage: u64,
    /// Memory cache in bytes
    pub cache: u64,
    /// Memory RSS (Resident Set Size) in bytes
    pub rss: u64,
    /// Memory swap usage in bytes
    pub swap: u64,
    /// Memory swap limit in bytes
    pub swap_limit: u64,
    /// Memory OOM kill count
    pub oom_kills: u64,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    /// Bytes received
    pub rx_bytes: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Packets received
    pub rx_packets: u64,
    /// Packets transmitted
    pub tx_packets: u64,
    /// Receive errors
    pub rx_errors: u64,
    /// Transmit errors
    pub tx_errors: u64,
    /// Receive dropped packets
    pub rx_dropped: u64,
    /// Transmit dropped packets
    pub tx_dropped: u64,
}

/// Disk I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetrics {
    /// Bytes read
    pub read_bytes: u64,
    /// Bytes written
    pub write_bytes: u64,
    /// Read operations
    pub read_ops: u64,
    /// Write operations
    pub write_ops: u64,
    /// Read time (nanoseconds)
    pub read_time: u64,
    /// Write time (nanoseconds)
    pub write_time: u64,
}

/// Process metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMetrics {
    /// Number of processes
    pub process_count: u32,
    /// Number of threads
    pub thread_count: u32,
    /// Number of file descriptors
    pub fd_count: u32,
    /// Number of open files
    pub open_files: u32,
    /// Process state
    pub state: String,
}

impl Default for ContainerMetrics {
    fn default() -> Self {
        Self {
            container_id: String::new(),
            timestamp: SystemTime::now(),
            cpu: CpuMetrics::default(),
            memory: MemoryMetrics::default(),
            network: NetworkMetrics::default(),
            disk: DiskMetrics::default(),
            processes: ProcessMetrics::default(),
        }
    }
}

impl Default for CpuMetrics {
    fn default() -> Self {
        Self {
            usage_percent: 0.0,
            cores: 1,
            user_time: 0,
            system_time: 0,
            total_time: 0,
            throttled_count: 0,
            throttled_time: 0,
        }
    }
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        Self {
            usage: 0,
            limit: 0,
            usage_percent: 0.0,
            peak_usage: 0,
            cache: 0,
            rss: 0,
            swap: 0,
            swap_limit: 0,
            oom_kills: 0,
        }
    }
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            rx_bytes: 0,
            tx_bytes: 0,
            rx_packets: 0,
            tx_packets: 0,
            rx_errors: 0,
            tx_errors: 0,
            rx_dropped: 0,
            tx_dropped: 0,
        }
    }
}

impl Default for DiskMetrics {
    fn default() -> Self {
        Self {
            read_bytes: 0,
            write_bytes: 0,
            read_ops: 0,
            write_ops: 0,
            read_time: 0,
            write_time: 0,
        }
    }
}

impl Default for ProcessMetrics {
    fn default() -> Self {
        Self {
            process_count: 0,
            thread_count: 0,
            fd_count: 0,
            open_files: 0,
            state: "unknown".to_string(),
        }
    }
}
