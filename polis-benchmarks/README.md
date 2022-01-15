# Benchmarks do Polis

Este diretório contém benchmarks de performance para o sistema Polis, permitindo medir e otimizar o desempenho de diferentes componentes.

## Estrutura

```
polis-benchmarks/
├── benches/
│   ├── container_benchmarks.rs    # Benchmarks de containers
│   ├── image_benchmarks.rs        # Benchmarks de imagens
│   ├── api_benchmarks.rs          # Benchmarks de APIs
│   ├── security_benchmarks.rs     # Benchmarks de segurança
│   ├── network_benchmarks.rs      # Benchmarks de rede
│   └── storage_benchmarks.rs      # Benchmarks de armazenamento
├── reports/                        # Relatórios gerados
├── run_benchmarks.sh              # Script de execução (Linux/Mac)
├── run_benchmarks.bat             # Script de execução (Windows)
├── criterion.toml                 # Configuração do Criterion
└── README.md                      # Este arquivo
```

## Como Executar

### Executar Todos os Benchmarks

```bash
# Linux/Mac
./run_benchmarks.sh

# Windows
run_benchmarks.bat
```

### Executar Benchmarks Específicos

```bash
# Containers
cargo bench --bench container_benchmarks

# Imagens
cargo bench --bench image_benchmarks

# APIs
cargo bench --bench api_benchmarks

# Segurança
cargo bench --bench security_benchmarks

# Rede
cargo bench --bench network_benchmarks

# Armazenamento
cargo bench --bench storage_benchmarks
```

### Executar com Configurações Personalizadas

```bash
# Com mais amostras
cargo bench -- --sample-size 1000

# Com mais tempo de medição
cargo bench -- --measurement-time 10s

# Com saída em JSON
cargo bench -- --output-format json
```

## Tipos de Benchmarks

### 1. Container Benchmarks

- **Criação de Containers**: Mede tempo para criar 1, 10, 50, 100 containers
- **Lifecycle**: Criação → Inicialização → Parada → Remoção
- **Listagem**: Tempo para listar e buscar containers
- **Operações Concorrentes**: Performance com múltiplas operações simultâneas

### 2. Image Benchmarks

- **Download**: Tempo para baixar imagens populares (alpine, ubuntu, nginx, redis, postgres)
- **Listagem**: Performance de listagem e busca de imagens
- **Parsing**: Velocidade de parsing de manifests OCI
- **Conversão**: Conversão de formatos Docker para OCI

### 3. API Benchmarks

- **REST API**: Endpoints de containers, imagens e sistema
- **gRPC API**: Serviços gRPC
- **Autenticação**: Criação de usuários, login, geração de tokens
- **Serialização**: Performance de serialização JSON

### 4. Security Benchmarks

- **AppArmor**: Criação de perfis e aplicação a containers
- **SELinux**: Criação de políticas e contextos
- **Security Manager**: Gerenciamento unificado de segurança
- **Operações Concorrentes**: Performance com múltiplas operações de segurança

### 5. Network Benchmarks

- **Bridges**: Criação e gerenciamento de bridges
- **IPAM**: Alocação de IPs e subnets
- **Firewall**: Criação e aplicação de regras
- **DNS**: Criação de registros e resolução
- **Port Forwarding**: Configuração de redirecionamento de portas

### 6. Storage Benchmarks

- **Volumes**: Criação, montagem e desmontagem
- **Snapshots**: Criação e restauração de snapshots
- **Backups**: Criação e restauração de backups
- **Operações de Arquivo**: Leitura, escrita e exclusão

## Métricas Coletadas

### Performance
- **Throughput**: Operações por segundo
- **Latência**: Tempo médio de resposta
- **P95/P99**: Percentis de latência
- **Variance**: Variação nos tempos de resposta

### Recursos
- **CPU**: Uso de processador
- **Memória**: Consumo de RAM
- **I/O**: Operações de disco
- **Rede**: Tráfego de rede

### Escalabilidade
- **Linear**: Performance com diferentes cargas
- **Concorrência**: Performance com múltiplas threads
- **Memória**: Uso de memória com diferentes volumes

## Interpretando os Resultados

### Relatórios HTML
- Gráficos interativos de performance
- Comparação entre diferentes configurações
- Análise de tendências e outliers
- Recomendações de otimização

### Relatórios JSON
- Dados brutos para análise programática
- Integração com ferramentas de CI/CD
- Comparação entre versões
- Alertas de regressão de performance

### Relatórios CSV
- Dados tabulares para análise em planilhas
- Exportação para ferramentas de BI
- Análise estatística detalhada

## Configuração

### criterion.toml
```toml
[global]
sample_size = 100          # Número de amostras
measurement_time = "5s"    # Tempo de medição
warm_up_time = "2s"        # Tempo de aquecimento

[report]
output = ["html", "json"]  # Formatos de saída
output_directory = "reports"  # Diretório de saída
```

### Variáveis de Ambiente
```bash
# Limite de memória (MB)
export POLIS_BENCHMARK_MEMORY_LIMIT=1024

# Limite de CPU (%)
export POLIS_BENCHMARK_CPU_LIMIT=80

# Número de threads
export POLIS_BENCHMARK_THREADS=4
```

## Integração com CI/CD

### GitHub Actions
```yaml
- name: Run Benchmarks
  run: |
    cd polis-benchmarks
    cargo bench -- --output-format json > benchmark_results.json
    
- name: Upload Results
  uses: actions/upload-artifact@v2
  with:
    name: benchmark-results
    path: polis-benchmarks/reports/
```

### Comparação de Versões
```bash
# Executar benchmarks na versão atual
cargo bench -- --output-format json > current.json

# Executar benchmarks na versão anterior
git checkout previous-version
cargo bench -- --output-format json > previous.json

# Comparar resultados
cargo bench -- --baseline current --baseline previous
```

## Troubleshooting

### Problemas Comuns

1. **Memória Insuficiente**
   ```bash
   # Aumentar limite de memória
   export POLIS_BENCHMARK_MEMORY_LIMIT=2048
   ```

2. **Timeout em Benchmarks**
   ```bash
   # Aumentar tempo de medição
   cargo bench -- --measurement-time 10s
   ```

3. **Benchmarks Muito Lentos**
   ```bash
   # Reduzir número de amostras
   cargo bench -- --sample-size 50
   ```

### Debug
```bash
# Executar com logs detalhados
RUST_LOG=debug cargo bench

# Executar benchmark específico
cargo bench --bench container_benchmarks -- --test create_containers
```

## Contribuindo

### Adicionando Novos Benchmarks

1. Criar novo arquivo em `benches/`
2. Implementar funções de benchmark
3. Adicionar ao script de execução
4. Documentar no README

### Exemplo de Benchmark
```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn my_benchmark(c: &mut Criterion) {
    c.bench_function("my_operation", |b| {
        b.iter(|| {
            // Código a ser medido
        });
    });
}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);
```

## Referências

- [Criterion.rs Documentation](https://docs.rs/criterion/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Benchmarking Best Practices](https://github.com/rust-lang/rustc-perf)
