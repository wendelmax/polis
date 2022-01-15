use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Performance metrics collector
pub struct PerformanceMetrics {
    start_time: Instant,
    operations: HashMap<String, OperationMetrics>,
    system_metrics: SystemMetrics,
}

#[derive(Debug, Clone)]
pub struct OperationMetrics {
    pub count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
    pub error_count: u64,
}

#[derive(Debug, Clone)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: usize,
    pub disk_io: u64,
    pub network_io: u64,
    pub context_switches: u64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            operations: HashMap::new(),
            system_metrics: SystemMetrics::default(),
        }
    }

    pub fn record_operation(&mut self, name: &str, duration: Duration, success: bool) {
        let metrics = self
            .operations
            .entry(name.to_string())
            .or_insert_with(|| OperationMetrics {
                count: 0,
                total_duration: Duration::ZERO,
                min_duration: Duration::MAX,
                max_duration: Duration::ZERO,
                avg_duration: Duration::ZERO,
                p95_duration: Duration::ZERO,
                p99_duration: Duration::ZERO,
                error_count: 0,
            });

        metrics.count += 1;
        metrics.total_duration += duration;
        metrics.min_duration = metrics.min_duration.min(duration);
        metrics.max_duration = metrics.max_duration.max(duration);
        metrics.avg_duration = metrics.total_duration / metrics.count as u32;

        if !success {
            metrics.error_count += 1;
        }
    }

    pub fn get_operation_metrics(&self, name: &str) -> Option<&OperationMetrics> {
        self.operations.get(name)
    }

    pub fn get_all_metrics(&self) -> &HashMap<String, OperationMetrics> {
        &self.operations
    }

    pub fn update_system_metrics(&mut self, metrics: SystemMetrics) {
        self.system_metrics = metrics;
    }

    pub fn get_system_metrics(&self) -> &SystemMetrics {
        &self.system_metrics
    }

    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn operations_per_second(&self, name: &str) -> f64 {
        if let Some(metrics) = self.operations.get(name) {
            let uptime_seconds = self.uptime().as_secs_f64();
            if uptime_seconds > 0.0 {
                metrics.count as f64 / uptime_seconds
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    pub fn error_rate(&self, name: &str) -> f64 {
        if let Some(metrics) = self.operations.get(name) {
            if metrics.count > 0 {
                metrics.error_count as f64 / metrics.count as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            disk_io: 0,
            network_io: 0,
            context_switches: 0,
        }
    }
}

/// Performance profiler for timing operations
pub struct Profiler {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    active_timers: HashMap<String, Instant>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::new())),
            active_timers: HashMap::new(),
        }
    }

    pub async fn start_timer(&mut self, operation: &str) {
        self.active_timers
            .insert(operation.to_string(), Instant::now());
    }

    pub async fn end_timer(&mut self, operation: &str, success: bool) -> Duration {
        if let Some(start_time) = self.active_timers.remove(operation) {
            let duration = start_time.elapsed();
            let mut metrics = self.metrics.write().await;
            metrics.record_operation(operation, duration, success);
            duration
        } else {
            Duration::ZERO
        }
    }

    pub async fn time_operation<F, R>(&mut self, operation: &str, f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        self.start_timer(operation).await;
        let result = f();
        let success = result.is_ok();
        self.end_timer(operation, success).await;
        result
    }

    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    pub async fn get_operation_metrics(&self, name: &str) -> Option<OperationMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get_operation_metrics(name).cloned()
    }

    pub async fn get_system_metrics(&self) -> SystemMetrics {
        let metrics = self.metrics.read().await;
        metrics.get_system_metrics().clone()
    }
}

/// CPU usage monitor
pub struct CpuMonitor {
    last_cpu_time: u64,
    last_timestamp: Instant,
}

impl CpuMonitor {
    pub fn new() -> Self {
        Self {
            last_cpu_time: 0,
            last_timestamp: Instant::now(),
        }
    }

    pub fn get_cpu_usage(&mut self) -> f64 {
        // This is a simplified implementation
        // In practice, you'd read from /proc/stat on Linux
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_timestamp);

        // Simulate CPU usage calculation
        let cpu_time = self.last_cpu_time + elapsed.as_millis() as u64;
        let usage = if elapsed.as_secs_f64() > 0.0 {
            (cpu_time - self.last_cpu_time) as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        self.last_cpu_time = cpu_time;
        self.last_timestamp = now;

        usage.min(100.0)
    }
}

/// I/O performance monitor
pub struct IoMonitor {
    read_bytes: u64,
    written_bytes: u64,
    read_ops: u64,
    write_ops: u64,
    last_timestamp: Instant,
}

impl IoMonitor {
    pub fn new() -> Self {
        Self {
            read_bytes: 0,
            written_bytes: 0,
            read_ops: 0,
            write_ops: 0,
            last_timestamp: Instant::now(),
        }
    }

    pub fn record_read(&mut self, bytes: u64) {
        self.read_bytes += bytes;
        self.read_ops += 1;
    }

    pub fn record_write(&mut self, bytes: u64) {
        self.written_bytes += bytes;
        self.write_ops += 1;
    }

    pub fn get_read_throughput(&self) -> f64 {
        let elapsed = self.last_timestamp.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.read_bytes as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn get_write_throughput(&self) -> f64 {
        let elapsed = self.last_timestamp.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.written_bytes as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn get_read_iops(&self) -> f64 {
        let elapsed = self.last_timestamp.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.read_ops as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn get_write_iops(&self) -> f64 {
        let elapsed = self.last_timestamp.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.write_ops as f64 / elapsed
        } else {
            0.0
        }
    }
}

/// Performance optimizer
pub struct PerformanceOptimizer {
    profiler: Profiler,
    cpu_monitor: CpuMonitor,
    io_monitor: IoMonitor,
    optimization_rules: Vec<OptimizationRule>,
}

#[derive(Debug, Clone)]
pub struct OptimizationRule {
    pub name: String,
    pub condition: OptimizationCondition,
    pub action: OptimizationAction,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum OptimizationCondition {
    CpuUsageAbove(f64),
    MemoryUsageAbove(usize),
    ErrorRateAbove(f64),
    LatencyAbove(Duration),
    ThroughputBelow(f64),
}

#[derive(Debug, Clone)]
pub enum OptimizationAction {
    EnableCaching,
    IncreasePoolSize,
    ReduceConcurrency,
    EnableCompression,
    ForceGarbageCollection,
    ScaleUp,
    ScaleDown,
}

impl PerformanceOptimizer {
    pub fn new() -> Self {
        Self {
            profiler: Profiler::new(),
            cpu_monitor: CpuMonitor::new(),
            io_monitor: IoMonitor::new(),
            optimization_rules: Vec::new(),
        }
    }

    pub fn add_optimization_rule(&mut self, rule: OptimizationRule) {
        self.optimization_rules.push(rule);
    }

    pub async fn check_and_optimize(&mut self) -> Result<Vec<String>> {
        let mut applied_optimizations = Vec::new();

        for rule in &self.optimization_rules {
            if !rule.enabled {
                continue;
            }

            if self.evaluate_condition(&rule.condition).await? {
                self.apply_action(&rule.action).await?;
                applied_optimizations.push(rule.name.clone());
                info!("Applied optimization: {}", rule.name);
            }
        }

        Ok(applied_optimizations)
    }

    async fn evaluate_condition(&mut self, condition: &OptimizationCondition) -> Result<bool> {
        match condition {
            OptimizationCondition::CpuUsageAbove(threshold) => {
                let cpu_usage = self.cpu_monitor.get_cpu_usage();
                Ok(cpu_usage > *threshold)
            }
            OptimizationCondition::MemoryUsageAbove(threshold) => {
                // This would check actual memory usage
                Ok(false) // Placeholder
            }
            OptimizationCondition::ErrorRateAbove(threshold) => {
                let metrics = self.profiler.get_metrics().await;
                let error_rate = metrics.error_rate("default");
                Ok(error_rate > *threshold)
            }
            OptimizationCondition::LatencyAbove(threshold) => {
                let metrics = self.profiler.get_metrics().await;
                if let Some(op_metrics) = metrics.get_operation_metrics("default") {
                    Ok(op_metrics.avg_duration > *threshold)
                } else {
                    Ok(false)
                }
            }
            OptimizationCondition::ThroughputBelow(threshold) => {
                let metrics = self.profiler.get_metrics().await;
                let throughput = metrics.operations_per_second("default");
                Ok(throughput < *threshold)
            }
        }
    }

    async fn apply_action(&mut self, action: &OptimizationAction) -> Result<()> {
        match action {
            OptimizationAction::EnableCaching => {
                // Enable caching mechanisms
                info!("Enabling caching");
            }
            OptimizationAction::IncreasePoolSize => {
                // Increase connection pool size
                info!("Increasing pool size");
            }
            OptimizationAction::ReduceConcurrency => {
                // Reduce concurrency level
                info!("Reducing concurrency");
            }
            OptimizationAction::EnableCompression => {
                // Enable compression
                info!("Enabling compression");
            }
            OptimizationAction::ForceGarbageCollection => {
                // Force garbage collection
                info!("Forcing garbage collection");
            }
            OptimizationAction::ScaleUp => {
                // Scale up resources
                info!("Scaling up");
            }
            OptimizationAction::ScaleDown => {
                // Scale down resources
                info!("Scaling down");
            }
        }
        Ok(())
    }

    pub async fn get_performance_report(&self) -> PerformanceReport {
        let metrics = self.profiler.get_metrics().await;
        let system_metrics = self.profiler.get_system_metrics().await;

        PerformanceReport {
            uptime: metrics.uptime(),
            total_operations: metrics.get_all_metrics().values().map(|m| m.count).sum(),
            cpu_usage: system_metrics.cpu_usage,
            memory_usage: system_metrics.memory_usage,
            operations: metrics.get_all_metrics().clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub uptime: Duration,
    pub total_operations: u64,
    pub cpu_usage: f64,
    pub memory_usage: usize,
    pub operations: HashMap<String, OperationMetrics>,
}

/// Performance benchmarking utilities
pub struct BenchmarkRunner {
    profiler: Profiler,
    iterations: u32,
    warmup_iterations: u32,
}

impl BenchmarkRunner {
    pub fn new(iterations: u32, warmup_iterations: u32) -> Self {
        Self {
            profiler: Profiler::new(),
            iterations,
            warmup_iterations,
        }
    }

    pub async fn run_benchmark<F, R>(&mut self, name: &str, f: F) -> Result<BenchmarkResult>
    where
        F: Fn() -> Result<R>,
    {
        // Warmup
        for _ in 0..self.warmup_iterations {
            let _ = f();
        }

        // Actual benchmark
        let mut durations = Vec::new();
        let mut errors = 0;

        for _ in 0..self.iterations {
            let start = Instant::now();
            let result = f();
            let duration = start.elapsed();

            durations.push(duration);
            if result.is_err() {
                errors += 1;
            }
        }

        durations.sort();

        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / durations.len() as u32;
        let min_duration = durations[0];
        let max_duration = durations[durations.len() - 1];
        let p95_index = (durations.len() as f64 * 0.95) as usize;
        let p99_index = (durations.len() as f64 * 0.99) as usize;
        let p95_duration = durations[p95_index.min(durations.len() - 1)];
        let p99_duration = durations[p99_index.min(durations.len() - 1)];

        Ok(BenchmarkResult {
            name: name.to_string(),
            iterations: self.iterations,
            total_duration,
            avg_duration,
            min_duration,
            max_duration,
            p95_duration,
            p99_duration,
            error_count: errors,
            operations_per_second: self.iterations as f64 / total_duration.as_secs_f64(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: u32,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
    pub error_count: u32,
    pub operations_per_second: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn test_profiler() {
        let mut profiler = Profiler::new();

        profiler.start_timer("test_operation").await;
        thread::sleep(Duration::from_millis(10));
        let duration = profiler.end_timer("test_operation", true).await;

        assert!(duration >= Duration::from_millis(10));

        let metrics = profiler.get_operation_metrics("test_operation").await;
        assert!(metrics.is_some());
        assert_eq!(metrics.unwrap().count, 1);
    }

    #[tokio::test]
    async fn test_benchmark_runner() {
        let mut runner = BenchmarkRunner::new(10, 2);

        let result = runner
            .run_benchmark("test", || {
                thread::sleep(Duration::from_millis(1));
                Ok(())
            })
            .await
            .unwrap();

        assert_eq!(result.iterations, 10);
        assert!(result.avg_duration >= Duration::from_millis(1));
        assert!(result.operations_per_second > 0.0);
    }

    #[test]
    fn test_cpu_monitor() {
        let mut monitor = CpuMonitor::new();

        let usage = monitor.get_cpu_usage();
        assert!(usage >= 0.0 && usage <= 100.0);
    }

    #[test]
    fn test_io_monitor() {
        let mut monitor = IoMonitor::new();

        monitor.record_read(1024);
        monitor.record_write(2048);

        assert_eq!(monitor.read_bytes, 1024);
        assert_eq!(monitor.written_bytes, 2048);
        assert_eq!(monitor.read_ops, 1);
        assert_eq!(monitor.write_ops, 1);
    }
}
