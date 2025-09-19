# Polis Installer for Windows
# Instala o Polis Container Runtime no Windows

param(
    [string]$Version = "latest",
    [string]$InstallPath = "$env:ProgramFiles\Polis",
    [switch]$Force,
    [switch]$SkipDependencies
)

$ErrorActionPreference = "Stop"

# Cores para output
$Red = "`e[31m"
$Green = "`e[32m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Reset = "`e[0m"

function Write-ColorOutput {
    param([string]$Message, [string]$Color = $Reset)
    Write-Host "${Color}${Message}${Reset}"
}

function Test-Administrator {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Ajustar caminho de instalação se não for administrador
if (-not (Test-Administrator) -and $SkipDependencies) {
    $InstallPath = "$env:USERPROFILE\Polis"
}

function Get-LatestRelease {
    try {
        $response = Invoke-RestMethod -Uri "https://api.github.com/repos/wendelmax/polis/releases/latest"
        return $response.tag_name
    }
    catch {
        Write-ColorOutput " Erro ao obter versão mais recente: $($_.Exception.Message)" $Red
        return "v1.0.0"
    }
}

function Install-Dependencies {
    if ($SkipDependencies) {
        Write-ColorOutput "  Pulando instalação de dependências" $Yellow
        return
    }

    Write-ColorOutput " Verificando dependências..." $Blue

    # Verificar se o Rust está instalado
    try {
        $rustVersion = rustc --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput " Rust já instalado: $rustVersion" $Green
        } else {
            Write-ColorOutput " Rust não encontrado. Instalando..." $Red
            Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
            .\rustup-init.exe -y
            Remove-Item "rustup-init.exe" -Force
        }
    }
    catch {
        Write-ColorOutput " Erro ao instalar Rust: $($_.Exception.Message)" $Red
        exit 1
    }

    # Verificar se o Git está instalado
    try {
        $gitVersion = git --version 2>$null
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput " Git já instalado: $gitVersion" $Green
        } else {
            Write-ColorOutput " Git não encontrado. Instale o Git primeiro." $Red
            Write-ColorOutput "   Download: https://git-scm.com/download/win" $Blue
            exit 1
        }
    }
    catch {
        Write-ColorOutput " Git não encontrado. Instale o Git primeiro." $Red
        exit 1
    }
}

function Install-Polis {
    Write-ColorOutput "Instalando Polis..." $Blue

    # Criar diretório de instalação
    if (Test-Path $InstallPath) {
        if ($Force) {
            Write-ColorOutput "Removendo instalacao anterior..." $Yellow
            Remove-Item $InstallPath -Recurse -Force
        } else {
            Write-ColorOutput "Diretorio de instalacao ja existe: $InstallPath" $Red
            Write-ColorOutput "   Use -Force para sobrescrever" $Yellow
            exit 1
        }
    }

    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null

    # Obter versão
    if ($Version -eq "latest") {
        $Version = Get-LatestRelease
    }

    Write-ColorOutput "Baixando Polis $Version..." $Blue

    # Baixar binários
    $downloadUrl = "https://github.com/wendelmax/polis/releases/download/$Version/polis-windows-x86_64.zip"
    $zipPath = "$env:TEMP\polis-windows.zip"

    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath
        Expand-Archive -Path $zipPath -DestinationPath $InstallPath -Force
        Remove-Item $zipPath -Force
    }
    catch {
        Write-ColorOutput " Erro ao baixar Polis: $($_.Exception.Message)" $Red
        Write-ColorOutput "   Tentando compilar a partir do código fonte..." $Yellow
        
        # Fallback: compilar a partir do código fonte
        git clone https://github.com/wendelmax/polis.git "$env:TEMP\polis"
        Set-Location "$env:TEMP\polis"
        
        # Desabilitar jemalloc no Windows
        $cargoToml = Get-Content "Cargo.toml" -Raw
        $cargoToml = $cargoToml -replace 'default = \[.*?\]', 'default = []'
        $cargoToml = $cargoToml -replace 'jemalloc = ".*?"', ''
        $cargoToml = $cargoToml -replace '\[features\]\s*jemalloc = \[.*?\]', ''
        $cargoToml = $cargoToml -replace '\[dependencies\]\s*jemalloc-sys.*?\n', ''
        
        # Remover jemalloc de todos os Cargo.toml dos crates
        Get-ChildItem -Recurse -Name "Cargo.toml" | ForEach-Object {
            $crateToml = Get-Content $_ -Raw
            $crateToml = $crateToml -replace 'jemalloc = ".*?"', ''
            $crateToml = $crateToml -replace '\[features\]\s*jemalloc = \[.*?\]', ''
            $crateToml = $crateToml -replace '\[dependencies\]\s*jemalloc-sys.*?\n', ''
            $crateToml | Set-Content $_
        }
        
        # Remover jemalloc do Cargo.lock
        if (Test-Path "Cargo.lock") {
            $cargoLock = Get-Content "Cargo.lock" -Raw
            $cargoLock = $cargoLock -replace '\[\[package\]\]\s*name = "jemalloc-sys".*?\[\[package\]\]', '[[package]]'
            $cargoLock = $cargoLock -replace '\[\[package\]\]\s*name = "jemalloc-sys".*?$', ''
            $cargoLock | Set-Content "Cargo.lock"
        }
        
        $cargoToml | Set-Content "Cargo.toml"
        
        # Compilar apenas o CLI sem dependências problemáticas
        cargo build --release --bin polis --no-default-features
        Copy-Item "target\release\polis.exe" $InstallPath
        Set-Location $PSScriptRoot
        Remove-Item "$env:TEMP\polis" -Recurse -Force
    }

    # Adicionar ao PATH
    if (Test-Administrator) {
        # Se for admin, adicionar ao PATH do sistema
        try {
            $currentPath = [Environment]::GetEnvironmentVariable("PATH", "Machine")
            if ($currentPath -notlike "*$InstallPath*") {
                [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$InstallPath", "Machine")
                Write-ColorOutput " Adicionado ao PATH do sistema" $Green
            }
        } catch {
            Write-ColorOutput " Aviso: Não foi possível adicionar ao PATH do sistema" $Yellow
        }
    } else {
        # Se não for admin, adicionar ao PATH do usuário
        try {
            $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
            if ($currentPath -notlike "*$InstallPath*") {
                [Environment]::SetEnvironmentVariable("PATH", "$currentPath;$InstallPath", "User")
                Write-ColorOutput " Adicionado ao PATH do usuário" $Green
            }
        } catch {
            Write-ColorOutput " Aviso: Não foi possível adicionar ao PATH automaticamente. Adicione manualmente: $InstallPath" $Yellow
        }
    }

    # Criar atalho no Desktop
    $desktopPath = [Environment]::GetFolderPath("Desktop")
    $shortcutPath = "$desktopPath\Polis.lnk"
    $WshShell = New-Object -comObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($shortcutPath)
    $Shortcut.TargetPath = "$InstallPath\polis.exe"
    $Shortcut.Description = "Polis Container Runtime"
    $Shortcut.Save()

    Write-ColorOutput " Atalho criado no Desktop" $Green
}

function Test-Installation {
    Write-ColorOutput "� Testando instalação..." $Blue
    
    try {
        $polisVersion = & "$InstallPath\polis.exe" --version
        if ($LASTEXITCODE -eq 0) {
            Write-ColorOutput " Polis instalado com sucesso!" $Green
            Write-ColorOutput "   Versão: $polisVersion" $Blue
            Write-ColorOutput "   Localização: $InstallPath" $Blue
            Write-ColorOutput "   Execute: polis --help" $Blue
        } else {
            Write-ColorOutput " Erro ao executar Polis" $Red
            exit 1
        }
    }
    catch {
        Write-ColorOutput " Erro ao testar instalação: $($_.Exception.Message)" $Red
        exit 1
    }
}

# Main
Write-ColorOutput " Polis Container Runtime Installer" $Blue
Write-ColorOutput "=====================================" $Blue

# Verificar privilégios de administrador
if (-not (Test-Administrator) -and -not $SkipDependencies) {
    Write-ColorOutput " Este script requer privilégios de administrador" $Red
    Write-ColorOutput "   Execute como administrador ou use -SkipDependencies" $Yellow
    exit 1
}

# Instalar dependências
Install-Dependencies

# Instalar Polis
Install-Polis

# Testar instalação
Test-Installation

Write-ColorOutput " Instalação concluída com sucesso!" $Green
Write-ColorOutput "   Reinicie o terminal ou execute: refreshenv" $Yellow
