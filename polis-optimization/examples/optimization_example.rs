use polis_core::PolisConfig;
use polis_optimization::{
    CacheManager, CompressionManager, CpuProfiler, MemoryOptimizer, MemoryProfiler,
    OptimizationAction, OptimizationCondition, OptimizationManager, OptimizationRule,
    PerformanceOptimizer, Profiler,
};
use std::collections::HashMap;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!(" Exemplo de Otimização do Polis");
    println!("=====================================");

    // 1. Memory Optimization
    println!("\n1. � Otimização de Memória");
    println!("----------------------------");

    let mut memory_optimizer = MemoryOptimizer::new()?;
    println!("    Otimizador de memória inicializado");

    // Simular uso de memória
    memory_optimizer.set_gc_threshold(50 * 1024 * 1024); // 50MB
    memory_optimizer.enable_compression(true);
    println!("    Configurações de memória definidas");

    if memory_optimizer.should_garbage_collect() {
        memory_optimizer.optimize_memory()?;
        println!("    Garbage collection executado");
    }

    // 2. Performance Optimization
    println!("\n2.  Otimização de Performance");
    println!("-------------------------------");

    let mut performance_optimizer = PerformanceOptimizer::new();
    println!("    Otimizador de performance inicializado");

    // Adicionar regras de otimização
    let rule = OptimizationRule {
        name: "high_cpu_usage".to_string(),
        condition: OptimizationCondition::CpuUsageAbove(80.0),
        action: OptimizationAction::ReduceConcurrency,
        enabled: true,
        priority: 1,
    };
    performance_optimizer.add_optimization_rule(rule);
    println!("    Regra de otimização adicionada");

    // 3. Caching System
    println!("\n3. � Sistema de Cache");
    println!("----------------------");

    let cache_manager = CacheManager::new();
    println!("    Gerenciador de cache inicializado");

    // Simular dados de cache
    let container = polis_core::types::Container {
        id: polis_core::types::ContainerId::new(),
        name: "test-container".to_string(),
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

    cache_manager
        .set_container("container-1".to_string(), container)
        .await;
    println!("    Container adicionado ao cache");

    let cached_container = cache_manager.get_container("container-1").await;
    if cached_container.is_some() {
        println!("    Container recuperado do cache");
    }

    // 4. Compression System
    println!("\n4.  Sistema de Compressão");
    println!("---------------------------");

    let compression_manager = CompressionManager::new(Default::default());
    println!("    Gerenciador de compressão inicializado");

    // Comprimir dados de exemplo
    let test_data = b"Hello, World! This is a test string for compression.".repeat(100);
    let compressed = compression_manager.compress_data(&test_data)?;
    println!("    Dados comprimidos");
    println!("   - Tamanho original: {} bytes", test_data.len());
    println!(
        "   - Tamanho comprimido: {} bytes",
        compressed.compressed_size
    );
    println!(
        "   - Taxa de compressão: {:.2}%",
        compressed.space_saved_percentage()
    );

    let decompressed = compression_manager.decompress_data(&compressed)?;
    println!("    Dados descomprimidos");
    println!("   - Dados idênticos: {}", test_data == decompressed);

    // 5. Profiling System
    println!("\n5.  Sistema de Profiling");
    println!("--------------------------");

    let profiler = Profiler::new();
    println!("    Profiler inicializado");

    // Simular operações para profiling
    for i in 0..10 {
        profiler
            .profile("test_operation", || {
                std::thread::sleep(Duration::from_millis(10));
                println!("   - Operação {} executada", i + 1);
            })
            .await;
    }

    let profile = profiler.get_profile("test_operation").await;
    if let Some(profile) = profile {
        println!("    Perfil de operação obtido");
        println!("   - Chamadas: {}", profile.call_count);
        println!("   - Tempo total: {:?}", profile.total_time);
        println!("   - Tempo médio: {:?}", profile.avg_time);
    }

    // 6. Memory Profiler
    println!("\n6. � Memory Profiler");
    println!("---------------------");

    let memory_profiler = MemoryProfiler::new();
    println!("    Memory profiler inicializado");

    // Simular alocações
    memory_profiler
        .record_allocation("test_function", 1024)
        .await;
    memory_profiler
        .record_allocation("test_function", 2048)
        .await;
    memory_profiler
        .record_deallocation("test_function", 512)
        .await;
    println!("    Alocações registradas");

    let stats = memory_profiler.get_memory_stats("test_function").await;
    if let Some(stats) = stats {
        println!("    Estatísticas de memória obtidas");
        println!("   - Total alocado: {} bytes", stats.total_allocated);
        println!("   - Total desalocado: {} bytes", stats.total_deallocated);
        println!("   - Uso atual: {} bytes", stats.current_usage);
        println!("   - Pico de uso: {} bytes", stats.peak_usage);
    }

    // 7. CPU Profiler
    println!("\n7. � CPU Profiler");
    println!("------------------");

    let mut cpu_profiler = CpuProfiler::new(Duration::from_millis(100));
    cpu_profiler.start();
    println!("    CPU profiler iniciado");

    // Simular amostras de CPU
    for i in 0..5 {
        cpu_profiler.sample("test_function").await;
        std::thread::sleep(Duration::from_millis(50));
        println!("   - Amostra {} coletada", i + 1);
    }

    cpu_profiler.stop();
    println!("    CPU profiler parado");

    let cpu_usage = cpu_profiler.get_cpu_usage("test_function").await;
    println!("   - Uso de CPU: {:.2}%", cpu_usage);

    // 8. Optimization Manager
    println!("\n8.  Optimization Manager");
    println!("-------------------------");

    let mut optimization_manager = OptimizationManager::new()?;
    println!("    Optimization manager inicializado");

    // Adicionar regras de otimização
    let memory_rule = OptimizationRule {
        name: "memory_optimization".to_string(),
        condition: OptimizationCondition::MemoryUsageAbove(100 * 1024 * 1024), // 100MB
        action: OptimizationAction::ForceGarbageCollection,
        enabled: true,
        priority: 1,
    };
    optimization_manager.add_optimization_rule(memory_rule);

    let cpu_rule = OptimizationRule {
        name: "cpu_optimization".to_string(),
        condition: OptimizationCondition::CpuUsageAbove(70.0),
        action: OptimizationAction::ReduceConcurrency,
        enabled: true,
        priority: 2,
    };
    optimization_manager.add_optimization_rule(cpu_rule);
    println!("    Regras de otimização adicionadas");

    // Executar ciclo de otimização
    let result = optimization_manager.run_optimization_cycle().await?;
    println!("    Ciclo de otimização executado");
    println!(
        "   - Otimizações aplicadas: {:?}",
        result.applied_optimizations
    );
    println!("   - Erros: {}", result.errors.len());
    println!("   - Duração: {:?}", result.duration);

    // Obter status de otimização
    let status = optimization_manager.get_optimization_status().await;
    println!("    Status de otimização obtido");
    println!("   - Uso de memória: {} bytes", status.memory_usage);
    println!("   - Uso de CPU: {:.2}%", status.cpu_usage);
    println!(
        "   - Taxa de hit do cache: {:.2}%",
        status.cache_hit_rate * 100.0
    );
    println!(
        "   - Taxa de compressão: {:.2}%",
        status.compression_ratio * 100.0
    );

    // 9. Benchmark de Performance
    println!("\n9.  Benchmark de Performance");
    println!("------------------------------");

    let mut benchmark = polis_optimization::BenchmarkRunner::new(100, 10);
    println!("    Benchmark runner inicializado");

    let result = benchmark
        .run_benchmark("test_benchmark", || {
            std::thread::sleep(Duration::from_millis(1));
            Ok(())
        })
        .await?;

    println!("    Benchmark executado");
    println!("   - Iterações: {}", result.iterations);
    println!("   - Tempo total: {:?}", result.total_duration);
    println!("   - Tempo médio: {:?}", result.avg_duration);
    println!("   - Tempo mínimo: {:?}", result.min_duration);
    println!("   - Tempo máximo: {:?}", result.max_duration);
    println!("   - P95: {:?}", result.p95_duration);
    println!("   - P99: {:?}", result.p99_duration);
    println!("   - Ops/segundo: {:.2}", result.operations_per_second);

    // 10. Relatório Final
    println!("\n10. � Relatório Final");
    println!("----------------------");

    let report = profiler.generate_report().await;
    println!("    Relatório de profiling gerado");
    println!("   - Total de chamadas: {}", report.total_calls);
    println!("   - Tempo total: {:?}", report.total_time);
    println!("   - Funções perfiladas: {}", report.profiles.len());

    let top_functions = report.top_functions_by_time(3);
    println!("   - Top 3 funções por tempo:");
    for (i, func) in top_functions.iter().enumerate() {
        println!("     {}. {}: {:?}", i + 1, func.name, func.total_time);
    }

    println!("\n Exemplo de otimização concluído com sucesso!");
    println!("================================================");

    Ok(())
}
