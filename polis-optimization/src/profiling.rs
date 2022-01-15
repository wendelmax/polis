use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Profiling data for a function or operation
#[derive(Debug, Clone)]
pub struct ProfileData {
    pub name: String,
    pub call_count: u64,
    pub total_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub avg_time: Duration,
    pub p95_time: Duration,
    pub p99_time: Duration,
    pub memory_usage: usize,
    pub allocations: u64,
    pub deallocations: u64,
}

impl ProfileData {
    pub fn new(name: String) -> Self {
        Self {
            name,
            call_count: 0,
            total_time: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
            avg_time: Duration::ZERO,
            p95_time: Duration::ZERO,
            p99_time: Duration::ZERO,
            memory_usage: 0,
            allocations: 0,
            deallocations: 0,
        }
    }

    pub fn add_call(&mut self, duration: Duration, memory_delta: isize) {
        self.call_count += 1;
        self.total_time += duration;
        self.min_time = self.min_time.min(duration);
        self.max_time = self.max_time.max(duration);
        self.avg_time = self.total_time / self.call_count as u32;

        if memory_delta > 0 {
            self.allocations += memory_delta as u64;
            self.memory_usage += memory_delta as usize;
        } else {
            self.deallocations += (-memory_delta) as u64;
            self.memory_usage = self.memory_usage.saturating_sub((-memory_delta) as usize);
        }
    }

    pub fn calculate_percentiles(&mut self, times: &[Duration]) {
        if times.is_empty() {
            return;
        }

        let mut sorted_times = times.to_vec();
        sorted_times.sort();

        let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
        let p99_index = (sorted_times.len() as f64 * 0.99) as usize;

        self.p95_time = sorted_times[p95_index.min(sorted_times.len() - 1)];
        self.p99_time = sorted_times[p99_index.min(sorted_times.len() - 1)];
    }
}

/// Profiler for tracking function performance
pub struct Profiler {
    profiles: Arc<RwLock<HashMap<String, ProfileData>>>,
    call_times: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
    enabled: bool,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            call_times: Arc::new(RwLock::new(HashMap::new())),
            enabled: true,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub async fn profile<F, R>(&self, name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if !self.enabled {
            return f();
        }

        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        self.record_call(name, duration, 0).await;
        result
    }

    pub async fn profile_async<F, Fut, R>(&self, name: &str, f: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        if !self.enabled {
            return f().await;
        }

        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();

        self.record_call(name, duration, 0).await;
        result
    }

    async fn record_call(&self, name: &str, duration: Duration, memory_delta: isize) {
        let mut profiles = self.profiles.write().await;
        let profile = profiles
            .entry(name.to_string())
            .or_insert_with(|| ProfileData::new(name.to_string()));
        profile.add_call(duration, memory_delta);

        let mut call_times = self.call_times.write().await;
        call_times
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(duration);
    }

    pub async fn get_profile(&self, name: &str) -> Option<ProfileData> {
        let profiles = self.profiles.read().await;
        profiles.get(name).cloned()
    }

    pub async fn get_all_profiles(&self) -> HashMap<String, ProfileData> {
        let profiles = self.profiles.read().await;
        profiles.clone()
    }

    pub async fn calculate_percentiles(&self) {
        let call_times = self.call_times.read().await;
        let mut profiles = self.profiles.write().await;

        for (name, times) in call_times.iter() {
            if let Some(profile) = profiles.get_mut(name) {
                profile.calculate_percentiles(times);
            }
        }
    }

    pub async fn reset(&self) {
        let mut profiles = self.profiles.write().await;
        profiles.clear();

        let mut call_times = self.call_times.write().await;
        call_times.clear();
    }

    pub async fn generate_report(&self) -> ProfilingReport {
        self.calculate_percentiles().await;

        let profiles = self.get_all_profiles().await;
        let total_calls: u64 = profiles.values().map(|p| p.call_count).sum();
        let total_time: Duration = profiles.values().map(|p| p.total_time).sum();

        ProfilingReport {
            profiles,
            total_calls,
            total_time,
            generated_at: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfilingReport {
    pub profiles: HashMap<String, ProfileData>,
    pub total_calls: u64,
    pub total_time: Duration,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

impl ProfilingReport {
    pub fn top_functions_by_time(&self, limit: usize) -> Vec<&ProfileData> {
        let mut functions: Vec<&ProfileData> = self.profiles.values().collect();
        functions.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        functions.truncate(limit);
        functions
    }

    pub fn top_functions_by_calls(&self, limit: usize) -> Vec<&ProfileData> {
        let mut functions: Vec<&ProfileData> = self.profiles.values().collect();
        functions.sort_by(|a, b| b.call_count.cmp(&a.call_count));
        functions.truncate(limit);
        functions
    }

    pub fn functions_with_high_memory_usage(&self, threshold: usize) -> Vec<&ProfileData> {
        self.profiles
            .values()
            .filter(|p| p.memory_usage > threshold)
            .collect()
    }

    pub fn functions_with_slow_calls(&self, threshold: Duration) -> Vec<&ProfileData> {
        self.profiles
            .values()
            .filter(|p| p.avg_time > threshold)
            .collect()
    }
}

/// Memory profiler for tracking allocations
pub struct MemoryProfiler {
    allocations: Arc<RwLock<HashMap<String, usize>>>,
    deallocations: Arc<RwLock<HashMap<String, usize>>>,
    peak_usage: Arc<RwLock<HashMap<String, usize>>>,
    current_usage: Arc<RwLock<HashMap<String, usize>>>,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            deallocations: Arc::new(RwLock::new(HashMap::new())),
            peak_usage: Arc::new(RwLock::new(HashMap::new())),
            current_usage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_allocation(&self, function: &str, size: usize) {
        let mut allocations = self.allocations.write().await;
        *allocations.entry(function.to_string()).or_insert(0) += size;

        let mut current = self.current_usage.write().await;
        let current_size = *current.get(function).unwrap_or(&0);
        *current.entry(function.to_string()).or_insert(0) = current_size + size;

        let mut peak = self.peak_usage.write().await;
        let peak_size = *peak.get(function).unwrap_or(&0);
        if current_size + size > peak_size {
            *peak.entry(function.to_string()).or_insert(0) = current_size + size;
        }
    }

    pub async fn record_deallocation(&self, function: &str, size: usize) {
        let mut deallocations = self.deallocations.write().await;
        *deallocations.entry(function.to_string()).or_insert(0) += size;

        let mut current = self.current_usage.write().await;
        if let Some(current_size) = current.get_mut(function) {
            *current_size = current_size.saturating_sub(size);
        }
    }

    pub async fn get_memory_stats(&self, function: &str) -> Option<MemoryStats> {
        let allocations = self.allocations.read().await;
        let deallocations = self.deallocations.read().await;
        let peak = self.peak_usage.read().await;
        let current = self.current_usage.read().await;

        Some(MemoryStats {
            function: function.to_string(),
            total_allocated: *allocations.get(function).unwrap_or(&0),
            total_deallocated: *deallocations.get(function).unwrap_or(&0),
            peak_usage: *peak.get(function).unwrap_or(&0),
            current_usage: *current.get(function).unwrap_or(&0),
        })
    }

    pub async fn get_all_memory_stats(&self) -> HashMap<String, MemoryStats> {
        let mut stats = HashMap::new();
        let allocations = self.allocations.read().await;
        let deallocations = self.deallocations.read().await;
        let peak = self.peak_usage.read().await;
        let current = self.current_usage.read().await;

        for function in allocations.keys() {
            stats.insert(
                function.clone(),
                MemoryStats {
                    function: function.clone(),
                    total_allocated: *allocations.get(function).unwrap_or(&0),
                    total_deallocated: *deallocations.get(function).unwrap_or(&0),
                    peak_usage: *peak.get(function).unwrap_or(&0),
                    current_usage: *current.get(function).unwrap_or(&0),
                },
            );
        }

        stats
    }
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub function: String,
    pub total_allocated: usize,
    pub total_deallocated: usize,
    pub peak_usage: usize,
    pub current_usage: usize,
}

/// CPU profiler for tracking CPU usage
pub struct CpuProfiler {
    samples: Arc<RwLock<Vec<CpuSample>>>,
    sampling_interval: Duration,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct CpuSample {
    pub timestamp: Instant,
    pub cpu_usage: f64,
    pub function: String,
}

impl CpuProfiler {
    pub fn new(sampling_interval: Duration) -> Self {
        Self {
            samples: Arc::new(RwLock::new(Vec::new())),
            sampling_interval,
            enabled: false,
        }
    }

    pub fn start(&mut self) {
        self.enabled = true;
    }

    pub fn stop(&mut self) {
        self.enabled = false;
    }

    pub async fn sample(&self, function: &str) {
        if !self.enabled {
            return;
        }

        // This is a simplified CPU usage calculation
        // In practice, you'd read from /proc/stat or similar
        let cpu_usage = 0.0; // Placeholder

        let sample = CpuSample {
            timestamp: Instant::now(),
            cpu_usage,
            function: function.to_string(),
        };

        let mut samples = self.samples.write().await;
        samples.push(sample);
    }

    pub async fn get_cpu_usage(&self, function: &str) -> f64 {
        let samples = self.samples.read().await;
        let function_samples: Vec<&CpuSample> =
            samples.iter().filter(|s| s.function == function).collect();

        if function_samples.is_empty() {
            return 0.0;
        }

        function_samples.iter().map(|s| s.cpu_usage).sum::<f64>() / function_samples.len() as f64
    }

    pub async fn get_peak_cpu_usage(&self, function: &str) -> f64 {
        let samples = self.samples.read().await;
        let function_samples: Vec<&CpuSample> =
            samples.iter().filter(|s| s.function == function).collect();

        function_samples
            .iter()
            .map(|s| s.cpu_usage)
            .fold(0.0, f64::max)
    }
}

/// Flame graph generator
pub struct FlameGraphGenerator {
    profiler: Arc<Profiler>,
    memory_profiler: Arc<MemoryProfiler>,
}

impl FlameGraphGenerator {
    pub fn new(profiler: Arc<Profiler>, memory_profiler: Arc<MemoryProfiler>) -> Self {
        Self {
            profiler,
            memory_profiler,
        }
    }

    pub async fn generate_flame_graph(&self) -> Result<String> {
        let report = self.profiler.generate_report().await;
        let memory_stats = self.memory_profiler.get_all_memory_stats().await;

        let mut flame_graph = String::new();
        flame_graph.push_str("flamegraph {\n");

        for (name, profile) in report.profiles {
            let memory_stat = memory_stats.get(&name);
            flame_graph.push_str(&format!(
                "  {} [time={}ms, calls={}, memory={}]\n",
                name,
                profile.total_time.as_millis(),
                profile.call_count,
                memory_stat.map(|m| m.current_usage).unwrap_or(0)
            ));
        }

        flame_graph.push_str("}\n");
        Ok(flame_graph)
    }
}

/// Profiling macro for easy function profiling
#[macro_export]
macro_rules! profile {
    ($profiler:expr, $name:expr, $code:block) => {
        $profiler.profile($name, || $code).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[tokio::test]
    async fn test_profiler() {
        let profiler = Profiler::new();

        profiler
            .profile("test_function", || {
                thread::sleep(Duration::from_millis(10));
            })
            .await;

        let profile = profiler.get_profile("test_function").await;
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().call_count, 1);
    }

    #[tokio::test]
    async fn test_memory_profiler() {
        let memory_profiler = MemoryProfiler::new();

        memory_profiler
            .record_allocation("test_function", 1024)
            .await;
        memory_profiler
            .record_deallocation("test_function", 512)
            .await;

        let stats = memory_profiler.get_memory_stats("test_function").await;
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.total_allocated, 1024);
        assert_eq!(stats.total_deallocated, 512);
        assert_eq!(stats.current_usage, 512);
    }

    #[tokio::test]
    async fn test_cpu_profiler() {
        let mut cpu_profiler = CpuProfiler::new(Duration::from_millis(100));
        cpu_profiler.start();

        cpu_profiler.sample("test_function").await;
        cpu_profiler.stop();

        let usage = cpu_profiler.get_cpu_usage("test_function").await;
        assert!(usage >= 0.0);
    }

    #[tokio::test]
    async fn test_profiling_report() {
        let profiler = Profiler::new();

        for _ in 0..10 {
            profiler
                .profile("test_function", || {
                    thread::sleep(Duration::from_millis(1));
                })
                .await;
        }

        let report = profiler.generate_report().await;
        assert_eq!(report.total_calls, 10);
        assert!(!report.profiles.is_empty());
    }
}
