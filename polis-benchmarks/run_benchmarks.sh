#!/bin/bash

# Script para executar todos os benchmarks do Polis
# Gera relatórios HTML e CSV para análise de performance

set -e

echo "🚀 Iniciando benchmarks do Polis..."

# Criar diretório para relatórios
mkdir -p reports

# Executar benchmarks individuais
echo "📊 Executando benchmarks de containers..."
cargo bench --bench container_benchmarks -- --output-format html --output reports/container_benchmarks.html

echo "📊 Executando benchmarks de imagens..."
cargo bench --bench image_benchmarks -- --output-format html --output reports/image_benchmarks.html

echo "📊 Executando benchmarks de APIs..."
cargo bench --bench api_benchmarks -- --output-format html --output reports/api_benchmarks.html

echo "📊 Executando benchmarks de segurança..."
cargo bench --bench security_benchmarks -- --output-format html --output reports/security_benchmarks.html

echo "📊 Executando benchmarks de rede..."
cargo bench --bench network_benchmarks -- --output-format html --output reports/network_benchmarks.html

echo "📊 Executando benchmarks de armazenamento..."
cargo bench --bench storage_benchmarks -- --output-format html --output reports/storage_benchmarks.html

# Executar todos os benchmarks juntos para comparação
echo "📊 Executando todos os benchmarks..."
cargo bench -- --output-format html --output reports/all_benchmarks.html

# Gerar relatório consolidado
echo "📋 Gerando relatório consolidado..."
cat > reports/benchmark_summary.md << EOF
# Relatório de Benchmarks - Polis

## Resumo Executivo

Este relatório apresenta os resultados dos benchmarks de performance do sistema Polis, incluindo:

- **Containers**: Criação, lifecycle, listagem e operações concorrentes
- **Imagens**: Download, listagem, parsing e conversão
- **APIs**: REST, gRPC, autenticação e serialização
- **Segurança**: AppArmor, SELinux e gerenciamento de perfis
- **Rede**: Criação de bridges, firewall, DNS e port forwarding
- **Armazenamento**: Volumes, snapshots, backups e operações de arquivo

## Arquivos de Relatório

- [Container Benchmarks](container_benchmarks.html)
- [Image Benchmarks](image_benchmarks.html)
- [API Benchmarks](api_benchmarks.html)
- [Security Benchmarks](security_benchmarks.html)
- [Network Benchmarks](network_benchmarks.html)
- [Storage Benchmarks](storage_benchmarks.html)
- [All Benchmarks](all_benchmarks.html)

## Como Interpretar os Resultados

### Métricas Importantes

1. **Throughput**: Operações por segundo
2. **Latência**: Tempo médio de resposta
3. **Uso de Memória**: Consumo de RAM durante operações
4. **Escalabilidade**: Performance com diferentes cargas

### Benchmarks por Categoria

#### Containers
- Criação de containers (1, 10, 50, 100 containers)
- Lifecycle completo (criar → iniciar → parar → remover)
- Listagem e busca de containers
- Operações concorrentes

#### Imagens
- Download de imagens populares (alpine, ubuntu, nginx, redis, postgres)
- Listagem e busca de imagens
- Parsing de manifests OCI
- Conversão Docker → OCI

#### APIs
- Endpoints REST (containers, imagens, sistema)
- Serviços gRPC
- Autenticação e autorização
- Serialização JSON

#### Segurança
- Criação de perfis AppArmor
- Políticas SELinux
- Gerenciamento de perfis de segurança
- Operações concorrentes de segurança

#### Rede
- Criação de bridges
- Alocação de IPs e subnets
- Regras de firewall
- Registros DNS
- Port forwarding

#### Armazenamento
- Criação e gerenciamento de volumes
- Snapshots e restauração
- Backups e recuperação
- Operações de arquivo

## Recomendações de Otimização

Com base nos resultados dos benchmarks, as seguintes otimizações são recomendadas:

1. **Cache de Imagens**: Implementar cache inteligente para reduzir downloads
2. **Pool de Conexões**: Otimizar conexões de rede e banco de dados
3. **Serialização**: Usar formatos mais eficientes (MessagePack, Protocol Buffers)
4. **Concorrência**: Ajustar limites de threads e workers
5. **Memória**: Implementar garbage collection otimizado

## Próximos Passos

1. Analisar resultados detalhados em cada relatório HTML
2. Identificar gargalos de performance
3. Implementar otimizações baseadas nos resultados
4. Re-executar benchmarks para validar melhorias
5. Estabelecer métricas de performance como parte do CI/CD

---
*Relatório gerado em: $(date)*
*Versão do Polis: 0.1.0*
EOF

echo "✅ Benchmarks concluídos!"
echo "📁 Relatórios disponíveis em: reports/"
echo "🌐 Abra reports/benchmark_summary.md para visão geral"
echo "📊 Abra os arquivos .html para detalhes específicos"
