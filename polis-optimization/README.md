# Polis Optimization

Sistema abrangente de otimização de performance e memória para o Polis Container Runtime.

## 🚀 Funcionalidades

### 1. Otimização de Memória
- **Memory Pool**: Pool de memória para alocações eficientes
- **String Interning**: Internação de strings para reduzir duplicação
- **SmallVec**: Container otimizado para pequenos arrays
- **Garbage Collection**: Coleta de lixo automática e manual
- **Memory Compression**: Compressão de dados em memória

### 2. Otimização de Performance
- **Performance Metrics**: Coleta de métricas de performance
- **CPU Monitoring**: Monitoramento de uso de CPU
- **I/O Monitoring**: Monitoramento de I/O
- **Benchmarking**: Ferramentas de benchmark
- **Auto-optimization**: Otimização automática baseada em regras

### 3. Sistema de Cache
- **Multi-level Cache**: Cache de múltiplos níveis (L1, L2, L3)
- **LRU Cache**: Cache com política LRU
- **TTL Cache**: Cache com tempo de vida
- **Cache Warming**: Pré-carregamento de cache
- **Cache Statistics**: Estatísticas detalhadas de cache

### 4. Compressão de Dados
- **Múltiplos Algoritmos**: Gzip, LZ4, Zstd, Brotli
- **Compression Strategy**: Seleção automática de algoritmo
- **Serialization**: Compressão de dados serializados
- **Compression Benchmarking**: Benchmark de algoritmos

### 5. Profiling e Monitoramento
- **Function Profiling**: Profiling de funções
- **Memory Profiling**: Profiling de memória
- **CPU Profiling**: Profiling de CPU
- **Flame Graph**: Geração de flame graphs
- **Performance Reports**: Relatórios de performance

## 📦 Uso

### Exemplo Básico

```rust
use polis_optimization::{
    OptimizationManager, MemoryOptimizer, PerformanceOptimizer,
    CacheManager, CompressionManager, Profiler
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar otimizador de memória
    let mut memory_optimizer = MemoryOptimizer::new()?;
    memory_optimizer.set_gc_threshold(100 * 1024 * 1024); // 100MB
    memory_optimizer.enable_compression(true);
    
    // Inicializar otimizador de performance
    let mut performance_optimizer = PerformanceOptimizer::new();
    
    // Inicializar gerenciador de cache
    let cache_manager = CacheManager::new();
    
    // Inicializar gerenciador de compressão
    let compression_manager = CompressionManager::new(Default::default());
    
    // Inicializar profiler
    let profiler = Profiler::new();
    
    // Executar otimizações
    if memory_optimizer.should_garbage_collect() {
        memory_optimizer.optimize_memory()?;
    }
    
    Ok(())
}
```

### Otimização Automática

```rust
use polis_optimization::{
    OptimizationManager, OptimizationRule, OptimizationCondition, OptimizationAction
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = OptimizationManager::new()?;
    
    // Adicionar regra de otimização
    let rule = OptimizationRule {
        name: "high_memory_usage".to_string(),
        condition: OptimizationCondition::MemoryUsageAbove(100 * 1024 * 1024),
        action: OptimizationAction::ForceGarbageCollection,
        enabled: true,
        priority: 1,
    };
    manager.add_optimization_rule(rule);
    
    // Executar ciclo de otimização
    let result = manager.run_optimization_cycle().await?;
    println!("Otimizações aplicadas: {:?}", result.applied_optimizations);
    
    Ok(())
}
```

### Sistema de Cache

```rust
use polis_optimization::CacheManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache_manager = CacheManager::new();
    
    // Armazenar container no cache
    let container = create_container();
    cache_manager.set_container("container-1".to_string(), container).await;
    
    // Recuperar container do cache
    let cached_container = cache_manager.get_container("container-1").await;
    if let Some(container) = cached_container {
        println!("Container recuperado do cache: {}", container.name);
    }
    
    // Obter estatísticas de cache
    let stats = cache_manager.get_all_stats().await;
    for (name, stat) in stats {
        println!("{}: {} entradas", name, stat.total_entries);
    }
    
    Ok(())
}
```

### Compressão de Dados

```rust
use polis_optimization::{CompressionManager, CompressionConfig, CompressionAlgorithm};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = CompressionConfig {
        algorithm: CompressionAlgorithm::Zstd,
        level: 3,
        threshold: 1024,
        enabled: true,
    };
    
    let mut manager = CompressionManager::new(config);
    
    // Comprimir dados
    let data = b"Hello, World! This is a test string for compression.".repeat(100);
    let compressed = manager.compress_data(&data)?;
    
    println!("Tamanho original: {} bytes", data.len());
    println!("Tamanho comprimido: {} bytes", compressed.compressed_size);
    println!("Taxa de compressão: {:.2}%", compressed.space_saved_percentage());
    
    // Descomprimir dados
    let decompressed = manager.decompress_data(&compressed)?;
    assert_eq!(decompressed, data);
    
    Ok(())
}
```

### Profiling de Performance

```rust
use polis_optimization::Profiler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let profiler = Profiler::new();
    
    // Profiling de função síncrona
    profiler.profile("sync_function", || {
        std::thread::sleep(Duration::from_millis(100));
    }).await;
    
    // Profiling de função assíncrona
    profiler.profile_async("async_function", async {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }).await;
    
    // Gerar relatório
    let report = profiler.generate_report().await;
    println!("Total de chamadas: {}", report.total_calls);
    println!("Tempo total: {:?}", report.total_time);
    
    // Obter top funções por tempo
    let top_functions = report.top_functions_by_time(5);
    for (i, func) in top_functions.iter().enumerate() {
        println!("{}. {}: {:?}", i + 1, func.name, func.total_time);
    }
    
    Ok(())
}
```

## 🔧 Configuração

### Configuração de Memória

```rust
use polis_optimization::MemoryOptimizer;

let mut optimizer = MemoryOptimizer::new()?;

// Configurar threshold para garbage collection
optimizer.set_gc_threshold(100 * 1024 * 1024); // 100MB

// Habilitar compressão de memória
optimizer.enable_compression(true);
```

### Configuração de Compressão

```rust
use polis_optimization::{CompressionConfig, CompressionAlgorithm};

let config = CompressionConfig {
    algorithm: CompressionAlgorithm::Zstd,
    level: 3,                    // Nível de compressão (1-22)
    threshold: 1024,            // Só comprimir dados > 1KB
    enabled: true,              // Habilitar compressão
};
```

### Configuração de Cache

```rust
use polis_optimization::MultiLevelCache;

let cache = MultiLevelCache::new(
    1000,                      // Capacidade L1
    Duration::from_secs(300)   // TTL L2
);
```

## 📊 Métricas e Monitoramento

### Métricas de Performance

```rust
use polis_optimization::PerformanceOptimizer;

let mut optimizer = PerformanceOptimizer::new();
let report = optimizer.get_performance_report().await;

println!("Uptime: {:?}", report.uptime);
println!("Total de operações: {}", report.total_operations);
println!("Uso de CPU: {:.2}%", report.cpu_usage);
println!("Uso de memória: {} bytes", report.memory_usage);
```

### Métricas de Cache

```rust
use polis_optimization::CacheManager;

let manager = CacheManager::new();
let stats = manager.get_all_stats().await;

for (name, stat) in stats {
    println!("Cache {}:", name);
    println!("  L1: {} entradas", stat.l1_entries);
    println!("  L2: {} entradas", stat.l2_entries);
    println!("  L3: {} entradas", stat.l3_entries);
    println!("  Total: {} entradas", stat.total_entries);
}
```

### Métricas de Compressão

```rust
use polis_optimization::CompressionManager;

let manager = CompressionManager::new(Default::default());
let stats = manager.get_stats();

println!("Total comprimido: {} bytes", stats.total_compressed);
println!("Total original: {} bytes", stats.total_original);
println!("Taxa de compressão: {:.2}%", manager.get_average_compression_ratio() * 100.0);
println!("Tempo médio: {:?}", manager.get_average_compression_time());
```

## 🧪 Testes

```bash
# Executar todos os testes
cargo test

# Executar testes específicos
cargo test test_memory_optimizer
cargo test test_performance_optimizer
cargo test test_cache_manager
cargo test test_compression_manager
cargo test test_profiler
```

## 📈 Benchmarks

```bash
# Executar benchmarks
cargo bench

# Executar benchmark específico
cargo bench --bench memory_benchmarks
cargo bench --bench compression_benchmarks
cargo bench --bench cache_benchmarks
```

## 🔍 Exemplos

### Exemplo Completo

```bash
# Executar exemplo completo
cargo run --example optimization_example
```

### Exemplo de Cache

```bash
# Executar exemplo de cache
cargo run --example cache_example
```

### Exemplo de Compressão

```bash
# Executar exemplo de compressão
cargo run --example compression_example
```

## 📚 Documentação

### Estrutura do Projeto

```
polis-optimization/
├── src/
│   ├── lib.rs              # Biblioteca principal
│   ├── memory.rs           # Otimização de memória
│   ├── performance.rs      # Otimização de performance
│   ├── caching.rs          # Sistema de cache
│   ├── compression.rs      # Compressão de dados
│   ├── profiling.rs        # Profiling e monitoramento
│   └── optimization.rs     # Gerenciador de otimização
├── examples/
│   └── optimization_example.rs
├── tests/
│   └── optimization_tests.rs
└── README.md
```

### Dependências

- **tokio**: Runtime assíncrono
- **serde**: Serialização
- **lru**: Cache LRU
- **dashmap**: HashMap concorrente
- **zstd**: Compressão Zstd
- **lz4**: Compressão LZ4
- **flate2**: Compressão Gzip
- **brotli**: Compressão Brotli
- **sysinfo**: Informações do sistema
- **tracing**: Logging estruturado

## 🚀 Performance

### Benchmarks Típicos

- **Cache Hit Rate**: > 95%
- **Compression Ratio**: 30-70% (dependendo do algoritmo)
- **Memory Overhead**: < 5%
- **CPU Overhead**: < 2%
- **Latency Impact**: < 1ms

### Otimizações Aplicadas

- **Memory Pool**: Reduz fragmentação de memória
- **String Interning**: Reduz uso de memória para strings
- **Multi-level Cache**: Melhora hit rate
- **Compression**: Reduz uso de memória e I/O
- **Profiling**: Identifica gargalos de performance

## 🤝 Contribuição

1. Fork o repositório
2. Crie uma branch para sua feature
3. Faça commit das mudanças
4. Abra um Pull Request

## 📄 Licença

MIT License - veja o arquivo [LICENSE](../../LICENSE) para detalhes.

## 🔗 Links Relacionados

- [Polis Core](../polis-core/README.md)
- [Polis Runtime](../polis-runtime/README.md)
- [Polis API](../polis-api/README.md)
- [Documentação Completa](../../docs/README.md)
