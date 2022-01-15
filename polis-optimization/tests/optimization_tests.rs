use polis_optimization::{
    CacheManager, CompressionManager, CpuProfiler, LruCacheWrapper, MemoryOptimizer,
    MemoryProfiler, MultiLevelCache, OptimizationAction, OptimizationCondition,
    OptimizationManager, OptimizationRule, PerformanceOptimizer, Profiler, TtlCache,
};
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_memory_optimizer() {
    let mut optimizer = MemoryOptimizer::new().unwrap();

    // Test garbage collection threshold
    optimizer.set_gc_threshold(1024);
    assert!(!optimizer.should_garbage_collect());

    // Test compression
    optimizer.enable_compression(true);
    assert!(optimizer.compression_enabled);

    // Test optimization
    let result = optimizer.optimize_memory();
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_performance_optimizer() {
    let mut optimizer = PerformanceOptimizer::new();

    // Add optimization rule
    let rule = OptimizationRule {
        name: "test_rule".to_string(),
        condition: OptimizationCondition::CpuUsageAbove(80.0),
        action: OptimizationAction::ReduceConcurrency,
        enabled: true,
        priority: 1,
    };
    optimizer.add_optimization_rule(rule);

    // Check optimizations
    let result = optimizer.check_and_optimize().await.unwrap();
    assert!(result.is_empty() || result.contains(&"test_rule".to_string()));
}

#[tokio::test]
async fn test_cache_manager() {
    let manager = CacheManager::new();

    // Test container caching
    let container = polis_core::types::Container {
        id: polis_core::types::ContainerId::new(),
        name: "test".to_string(),
        image: polis_core::types::ImageId::new("alpine", "latest"),
        status: polis_core::types::ContainerStatus::Running,
        created_at: chrono::Utc::now(),
        started_at: Some(chrono::Utc::now()),
        finished_at: None,
        exit_code: None,
        command: vec!["echo".to_string(), "hello".to_string()],
        working_dir: std::path::PathBuf::from("/"),
        environment: HashMap::new(),
        labels: HashMap::new(),
        resource_limits: polis_core::types::ResourceLimits::default(),
        network_mode: polis_core::types::NetworkMode::Bridge,
        ports: vec![],
        volumes: vec![],
    };

    manager
        .set_container("test-id".to_string(), container.clone())
        .await;
    let cached = manager.get_container("test-id").await;
    assert!(cached.is_some());

    // Test cleanup
    manager.cleanup_all().await;

    // Test stats
    let stats = manager.get_all_stats().await;
    assert!(!stats.is_empty());
}

#[tokio::test]
async fn test_compression_manager() {
    let mut manager = CompressionManager::new(Default::default());

    // Test data compression
    let data = b"Hello, World! This is a test string for compression.".repeat(100);
    let compressed = manager.compress_data(&data).unwrap();

    assert!(compressed.compressed_size <= data.len());
    assert!(compressed.compression_ratio > 0.0);

    // Test decompression
    let decompressed = manager.decompress_data(&compressed).unwrap();
    assert_eq!(decompressed, data);

    // Test serializable compression
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct TestData {
        name: String,
        value: i32,
    }

    let test_data = TestData {
        name: "test".to_string(),
        value: 42,
    };

    let compressed = manager.compress_serializable(&test_data).unwrap();
    let decompressed: TestData = manager.decompress_deserializable(&compressed).unwrap();
    assert_eq!(test_data, decompressed);
}

#[tokio::test]
async fn test_profiler() {
    let profiler = Profiler::new();

    // Test profiling
    profiler
        .profile("test_function", || {
            std::thread::sleep(Duration::from_millis(10));
        })
        .await;

    let profile = profiler.get_profile("test_function").await;
    assert!(profile.is_some());
    assert_eq!(profile.unwrap().call_count, 1);

    // Test async profiling
    profiler
        .profile_async("test_async_function", async {
            tokio::time::sleep(Duration::from_millis(10)).await;
        })
        .await;

    let profile = profiler.get_profile("test_async_function").await;
    assert!(profile.is_some());
    assert_eq!(profile.unwrap().call_count, 1);

    // Test report generation
    let report = profiler.generate_report().await;
    assert_eq!(report.total_calls, 2);
    assert!(!report.profiles.is_empty());
}

#[tokio::test]
async fn test_memory_profiler() {
    let profiler = MemoryProfiler::new();

    // Test allocation recording
    profiler.record_allocation("test_function", 1024).await;
    profiler.record_allocation("test_function", 2048).await;
    profiler.record_deallocation("test_function", 512).await;

    let stats = profiler.get_memory_stats("test_function").await;
    assert!(stats.is_some());
    let stats = stats.unwrap();
    assert_eq!(stats.total_allocated, 3072);
    assert_eq!(stats.total_deallocated, 512);
    assert_eq!(stats.current_usage, 2560);
}

#[tokio::test]
async fn test_cpu_profiler() {
    let mut profiler = CpuProfiler::new(Duration::from_millis(100));

    profiler.start();
    profiler.sample("test_function").await;
    profiler.stop();

    let usage = profiler.get_cpu_usage("test_function").await;
    assert!(usage >= 0.0);
}

#[tokio::test]
async fn test_lru_cache() {
    let mut cache = LruCacheWrapper::new(2);

    cache.insert("key1", "value1");
    cache.insert("key2", "value2");
    cache.insert("key3", "value3"); // Should evict key1

    assert_eq!(cache.get(&"key1"), None);
    assert_eq!(cache.get(&"key2"), Some("value2"));
    assert_eq!(cache.get(&"key3"), Some("value3"));

    assert_eq!(cache.len(), 2);
    assert!(!cache.is_empty());
}

#[tokio::test]
async fn test_ttl_cache() {
    let mut cache = TtlCache::new(Duration::from_millis(100));

    cache.insert("key1", "value1");
    assert_eq!(cache.get(&"key1"), Some("value1"));

    std::thread::sleep(Duration::from_millis(150));
    assert_eq!(cache.get(&"key1"), None);

    // Test cleanup
    cache.cleanup_expired();
    assert!(cache.is_empty());
}

#[tokio::test]
async fn test_multi_level_cache() {
    let cache = MultiLevelCache::new(2, Duration::from_secs(1));

    cache.insert("key1", "value1").await;
    assert_eq!(cache.get(&"key1").await, Some("value1"));

    cache.remove(&"key1").await;
    assert_eq!(cache.get(&"key1").await, None);

    // Test cleanup
    cache.cleanup().await;

    // Test stats
    let stats = cache.stats().await;
    assert!(stats.total_entries >= 0);
}

#[tokio::test]
async fn test_optimization_manager() {
    let mut manager = OptimizationManager::new().unwrap();

    // Add optimization rules
    let memory_rule = OptimizationRule {
        name: "memory_rule".to_string(),
        condition: OptimizationCondition::MemoryUsageAbove(1000),
        action: OptimizationAction::ForceGarbageCollection,
        enabled: true,
        priority: 1,
    };
    manager.add_optimization_rule(memory_rule);

    let cpu_rule = OptimizationRule {
        name: "cpu_rule".to_string(),
        condition: OptimizationCondition::CpuUsageAbove(80.0),
        action: OptimizationAction::ReduceConcurrency,
        enabled: true,
        priority: 2,
    };
    manager.add_optimization_rule(cpu_rule);

    // Run optimization cycle
    let result = manager.run_optimization_cycle().await.unwrap();
    assert!(result.applied_optimizations.is_empty() || result.applied_optimizations.len() <= 2);

    // Get optimization status
    let status = manager.get_optimization_status().await;
    assert!(status.memory_usage >= 0);
    assert!(status.cpu_usage >= 0.0);
    assert!(status.cache_hit_rate >= 0.0);
    assert!(status.cache_hit_rate <= 1.0);
}

#[tokio::test]
async fn test_compression_algorithms() {
    use polis_optimization::CompressionAlgorithm;

    let algorithms = [
        CompressionAlgorithm::None,
        CompressionAlgorithm::Gzip,
        CompressionAlgorithm::Lz4,
        CompressionAlgorithm::Zstd,
        CompressionAlgorithm::Brotli,
    ];

    let data = b"Hello, World! This is a test string for compression.".repeat(100);

    for algorithm in algorithms {
        let config = polis_optimization::CompressionConfig {
            algorithm,
            level: 3,
            threshold: 0,
            enabled: true,
        };

        let mut manager = CompressionManager::new(config);
        let compressed = manager.compress_data(&data).unwrap();
        let decompressed = manager.decompress_data(&compressed).unwrap();

        assert_eq!(decompressed, data);
        assert!(compressed.compression_ratio > 0.0);
    }
}

#[tokio::test]
async fn test_benchmark_runner() {
    let mut runner = polis_optimization::BenchmarkRunner::new(10, 2);

    let result = runner
        .run_benchmark("test_benchmark", || {
            std::thread::sleep(Duration::from_millis(1));
            Ok(())
        })
        .await
        .unwrap();

    assert_eq!(result.iterations, 10);
    assert!(result.avg_duration >= Duration::from_millis(1));
    assert!(result.operations_per_second > 0.0);
    assert_eq!(result.error_count, 0);
}

#[tokio::test]
async fn test_string_interner() {
    use polis_optimization::StringInterner;

    let mut interner = StringInterner::new();

    let id1 = interner.intern("hello");
    let id2 = interner.intern("world");
    let id3 = interner.intern("hello");

    assert_eq!(id1, id3);
    assert_ne!(id1, id2);
    assert_eq!(interner.get(id1), Some("hello"));
    assert_eq!(interner.get(id2), Some("world"));

    assert!(interner.memory_usage() > 0);
}

#[tokio::test]
async fn test_small_vec() {
    use polis_optimization::SmallVec;

    let mut vec = SmallVec::<i32, 4>::new();

    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);

    vec.push(1).unwrap();
    vec.push(2).unwrap();

    assert_eq!(vec.len(), 2);
    assert!(!vec.is_empty());

    let items: Vec<i32> = vec.iter().copied().collect();
    assert_eq!(items, vec![1, 2]);

    assert_eq!(vec.pop(), Some(2));
    assert_eq!(vec.pop(), Some(1));
    assert_eq!(vec.pop(), None);
}

#[tokio::test]
async fn test_memory_pool() {
    use polis_optimization::MemoryPool;

    let mut pool = MemoryPool::new(1024, 10);

    let block1 = pool.allocate().unwrap();
    assert_eq!(block1.len(), 1024);

    let block2 = pool.allocate().unwrap();
    assert_eq!(block2.len(), 1024);

    pool.deallocate(block1);
    let block3 = pool.allocate().unwrap();
    assert_eq!(block3.len(), 1024);
}

#[tokio::test]
async fn test_performance_metrics() {
    use polis_optimization::PerformanceMetrics;

    let mut metrics = PerformanceMetrics::new();

    metrics.record_operation("test_op", Duration::from_millis(100), true);
    metrics.record_operation("test_op", Duration::from_millis(200), false);

    let op_metrics = metrics.get_operation_metrics("test_op");
    assert!(op_metrics.is_some());
    let op_metrics = op_metrics.unwrap();
    assert_eq!(op_metrics.count, 2);
    assert_eq!(op_metrics.error_count, 1);
    assert_eq!(op_metrics.avg_duration, Duration::from_millis(150));
}

#[tokio::test]
async fn test_compression_strategy() {
    use polis_optimization::CompressionStrategy;

    let strategy = CompressionStrategy::new();

    let algorithm = strategy.select_for_data_type("json");
    assert_eq!(algorithm, polis_optimization::CompressionAlgorithm::Zstd);

    let algorithm = strategy.select_for_data_type("text");
    assert_eq!(algorithm, polis_optimization::CompressionAlgorithm::Gzip);

    let algorithm = strategy.select_algorithm(1024, 0.5);
    assert!(algorithm != polis_optimization::CompressionAlgorithm::None);
}

#[tokio::test]
async fn test_optimization_recommender() {
    use polis_optimization::{OptimizationContext, OptimizationRecommender, RecommendationRule};

    let mut recommender = OptimizationRecommender::new();

    let rule = RecommendationRule {
        name: "test_recommendation".to_string(),
        condition: Box::new(|_| true),
        recommendation: "Test recommendation".to_string(),
        priority: 1,
    };

    recommender.add_rule(rule);

    let context = OptimizationContext {
        memory_stats: polis_optimization::MemoryStats::new(),
        performance_report: polis_optimization::PerformanceReport {
            uptime: Duration::ZERO,
            total_operations: 0,
            cpu_usage: 0.0,
            memory_usage: 0,
            operations: HashMap::new(),
        },
        cache_stats: HashMap::new(),
        compression_stats: polis_optimization::CompressionStats::default(),
        profiling_report: polis_optimization::ProfilingReport {
            profiles: HashMap::new(),
            total_calls: 0,
            total_time: Duration::ZERO,
            generated_at: chrono::Utc::now(),
        },
        system_metrics: polis_optimization::SystemMetrics {
            cpu_usage: 0.0,
            memory_usage: 0,
            disk_usage: 0,
            network_usage: 0,
            uptime: Duration::ZERO,
        },
    };

    let recommendations = recommender.get_recommendations(&context);
    assert!(!recommendations.is_empty());
    assert_eq!(recommendations[0].name, "test_recommendation");
}
