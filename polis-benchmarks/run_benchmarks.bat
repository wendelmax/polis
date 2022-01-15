@echo off
REM Script para executar todos os benchmarks do Polis no Windows
REM Gera relatórios HTML e CSV para análise de performance

echo 🚀 Iniciando benchmarks do Polis...

REM Criar diretório para relatórios
if not exist reports mkdir reports

REM Executar benchmarks individuais
echo 📊 Executando benchmarks de containers...
cargo bench --bench container_benchmarks -- --output-format html --output reports/container_benchmarks.html

echo 📊 Executando benchmarks de imagens...
cargo bench --bench image_benchmarks -- --output-format html --output reports/image_benchmarks.html

echo 📊 Executando benchmarks de APIs...
cargo bench --bench api_benchmarks -- --output-format html --output reports/api_benchmarks.html

echo 📊 Executando benchmarks de segurança...
cargo bench --bench security_benchmarks -- --output-format html --output reports/security_benchmarks.html

echo 📊 Executando benchmarks de rede...
cargo bench --bench network_benchmarks -- --output-format html --output reports/network_benchmarks.html

echo 📊 Executando benchmarks de armazenamento...
cargo bench --bench storage_benchmarks -- --output-format html --output reports/storage_benchmarks.html

REM Executar todos os benchmarks juntos para comparação
echo 📊 Executando todos os benchmarks...
cargo bench -- --output-format html --output reports/all_benchmarks.html

REM Gerar relatório consolidado
echo 📋 Gerando relatório consolidado...
(
echo # Relatório de Benchmarks - Polis
echo.
echo ## Resumo Executivo
echo.
echo Este relatório apresenta os resultados dos benchmarks de performance do sistema Polis, incluindo:
echo.
echo - **Containers**: Criação, lifecycle, listagem e operações concorrentes
echo - **Imagens**: Download, listagem, parsing e conversão
echo - **APIs**: REST, gRPC, autenticação e serialização
echo - **Segurança**: AppArmor, SELinux e gerenciamento de perfis
echo - **Rede**: Criação de bridges, firewall, DNS e port forwarding
echo - **Armazenamento**: Volumes, snapshots, backups e operações de arquivo
echo.
echo ## Arquivos de Relatório
echo.
echo - [Container Benchmarks](container_benchmarks.html^)
echo - [Image Benchmarks](image_benchmarks.html^)
echo - [API Benchmarks](api_benchmarks.html^)
echo - [Security Benchmarks](security_benchmarks.html^)
echo - [Network Benchmarks](network_benchmarks.html^)
echo - [Storage Benchmarks](storage_benchmarks.html^)
echo - [All Benchmarks](all_benchmarks.html^)
echo.
echo ## Como Interpretar os Resultados
echo.
echo ### Métricas Importantes
echo.
echo 1. **Throughput**: Operações por segundo
echo 2. **Latência**: Tempo médio de resposta
echo 3. **Uso de Memória**: Consumo de RAM durante operações
echo 4. **Escalabilidade**: Performance com diferentes cargas
echo.
echo ### Benchmarks por Categoria
echo.
echo #### Containers
echo - Criação de containers (1, 10, 50, 100 containers^)
echo - Lifecycle completo (criar → iniciar → parar → remover^)
echo - Listagem e busca de containers
echo - Operações concorrentes
echo.
echo #### Imagens
echo - Download de imagens populares (alpine, ubuntu, nginx, redis, postgres^)
echo - Listagem e busca de imagens
echo - Parsing de manifests OCI
echo - Conversão Docker → OCI
echo.
echo #### APIs
echo - Endpoints REST (containers, imagens, sistema^)
echo - Serviços gRPC
echo - Autenticação e autorização
echo - Serialização JSON
echo.
echo #### Segurança
echo - Criação de perfis AppArmor
echo - Políticas SELinux
echo - Gerenciamento de perfis de segurança
echo - Operações concorrentes de segurança
echo.
echo #### Rede
echo - Criação de bridges
echo - Alocação de IPs e subnets
echo - Regras de firewall
echo - Registros DNS
echo - Port forwarding
echo.
echo #### Armazenamento
echo - Criação e gerenciamento de volumes
echo - Snapshots e restauração
echo - Backups e recuperação
echo - Operações de arquivo
echo.
echo ## Recomendações de Otimização
echo.
echo Com base nos resultados dos benchmarks, as seguintes otimizações são recomendadas:
echo.
echo 1. **Cache de Imagens**: Implementar cache inteligente para reduzir downloads
echo 2. **Pool de Conexões**: Otimizar conexões de rede e banco de dados
echo 3. **Serialização**: Usar formatos mais eficientes (MessagePack, Protocol Buffers^)
echo 4. **Concorrência**: Ajustar limites de threads e workers
echo 5. **Memória**: Implementar garbage collection otimizado
echo.
echo ## Próximos Passos
echo.
echo 1. Analisar resultados detalhados em cada relatório HTML
echo 2. Identificar gargalos de performance
echo 3. Implementar otimizações baseadas nos resultados
echo 4. Re-executar benchmarks para validar melhorias
echo 5. Estabelecer métricas de performance como parte do CI/CD
echo.
echo ---
echo *Relatório gerado em: %date% %time%*
echo *Versão do Polis: 0.1.0*
) > reports/benchmark_summary.md

echo ✅ Benchmarks concluídos!
echo 📁 Relatórios disponíveis em: reports/
echo 🌐 Abra reports/benchmark_summary.md para visão geral
echo 📊 Abra os arquivos .html para detalhes específicos
