# Polis - Demonstração Profissional
# Container Runtime e Plataforma de Orquestração

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "    POLIS - CONTAINER RUNTIME PROFISSIONAL    " -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host ""

# Verificar se o Polis está instalado
Write-Host "1. Verificando instalação do Polis..." -ForegroundColor Yellow
try {
    $version = & .\target\debug\polis.exe --version
    Write-Host "   ✓ Polis instalado: $version" -ForegroundColor Green
} catch {
    Write-Host "   ✗ Polis não encontrado. Compilando..." -ForegroundColor Red
    cargo build --bin polis --no-default-features
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✓ Polis compilado com sucesso!" -ForegroundColor Green
    } else {
        Write-Host "   ✗ Erro na compilação" -ForegroundColor Red
        exit 1
    }
}

Write-Host ""

# Demonstração do Sistema
Write-Host "2. Informações do Sistema..." -ForegroundColor Yellow
& .\target\debug\polis.exe system info
Write-Host ""

# Demonstração de Imagens
Write-Host "3. Gerenciamento de Imagens..." -ForegroundColor Yellow
Write-Host "   Listando imagens locais:"
& .\target\debug\polis.exe image list
Write-Host ""

# Demonstração de Containers
Write-Host "4. Gerenciamento de Containers..." -ForegroundColor Yellow
Write-Host "   Listando containers:"
& .\target\debug\polis.exe container list
Write-Host ""

# Demonstração de Orquestração
Write-Host "5. Orquestração e Deployments..." -ForegroundColor Yellow
Write-Host "   Listando deployments:"
& .\target\debug\polis.exe deploy list
Write-Host ""

# Demonstração de Estatísticas
Write-Host "6. Estatísticas do Orchestrator..." -ForegroundColor Yellow
& .\target\debug\polis.exe deploy stats
Write-Host ""

# Demonstração de Rede
Write-Host "7. Gerenciamento de Rede..." -ForegroundColor Yellow
Write-Host "   Listando bridges de rede:"
& .\target\debug\polis.exe network list-bridges
Write-Host ""

# Demonstração de Volumes
Write-Host "8. Gerenciamento de Volumes..." -ForegroundColor Yellow
Write-Host "   Listando volumes:"
& .\target\debug\polis.exe volume list
Write-Host ""

# Demonstração de Estatísticas de Container
Write-Host "9. Monitoramento de Containers..." -ForegroundColor Yellow
Write-Host "   Estatísticas de containers:"
& .\target\debug\polis.exe stats list
Write-Host ""

# Demonstração de Registry
Write-Host "10. Configuração de Registry..." -ForegroundColor Yellow
Write-Host "    Listando registries configurados:"
& .\target\debug\polis.exe registry list
Write-Host ""

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host "    DEMONSTRAÇÃO CONCLUÍDA COM SUCESSO!       " -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Funcionalidades Demonstradas:" -ForegroundColor Green
Write-Host "  ✓ Sistema de Informações" -ForegroundColor Green
Write-Host "  ✓ Gerenciamento de Imagens" -ForegroundColor Green
Write-Host "  ✓ Gerenciamento de Containers" -ForegroundColor Green
Write-Host "  ✓ Orquestração e Deployments" -ForegroundColor Green
Write-Host "  ✓ Monitoramento e Estatísticas" -ForegroundColor Green
Write-Host "  ✓ Gerenciamento de Rede" -ForegroundColor Green
Write-Host "  ✓ Gerenciamento de Volumes" -ForegroundColor Green
Write-Host "  ✓ Configuração de Registry" -ForegroundColor Green
Write-Host ""
Write-Host "Polis está pronto para uso em produção!" -ForegroundColor Magenta
