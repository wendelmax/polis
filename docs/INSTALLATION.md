# Guia de Instalação do Polis

Este guia fornece instruções detalhadas para instalar o Polis Container Runtime em diferentes sistemas operacionais.

## 📋 Pré-requisitos

### Requisitos Mínimos
- **Rust**: 1.70 ou superior
- **Git**: 2.30 ou superior
- **Sistema Operacional**: Linux, macOS ou Windows
- **Memória RAM**: 4GB mínimo, 8GB recomendado
- **Espaço em Disco**: 2GB para instalação, 10GB para uso

### Requisitos Específicos por Plataforma

#### Linux
- **Kernel**: 4.15 ou superior
- **Privilégios**: Root ou usuário com sudo
- **Dependências**: `build-essential`, `pkg-config`, `libssl-dev`
- **Namespaces**: Suporte a namespaces do kernel

#### macOS
- **Versão**: macOS 10.15 (Catalina) ou superior
- **Xcode**: Command Line Tools instalados
- **Homebrew**: Recomendado para dependências

#### Windows
- **Versão**: Windows 10 1903 ou superior
- **PowerShell**: 5.1 ou superior
- **Visual Studio**: Build Tools 2019 ou superior
- **WSL2**: Recomendado para funcionalidades completas

## 🚀 Instalação Rápida

### Windows (Recomendado)

```powershell
# 1. Clone o repositório
git clone https://github.com/polis/polis.git
cd polis

# 2. Execute o instalador
.\installers\windows\install.ps1

# 3. Verifique a instalação
polis --version
```

### Linux/macOS

```bash
# 1. Clone o repositório
git clone https://github.com/polis/polis.git
cd polis

# 2. Execute o instalador
chmod +x installers/linux/install.sh
./installers/linux/install.sh

# 3. Verifique a instalação
polis --version
```

## 🔧 Instalação Manual

### 1. Instalar Rust

#### Linux/macOS
```bash
# Instalar Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verificar instalação
rustc --version
cargo --version
```

#### Windows
```powershell
# Baixar e executar rustup-init.exe
# Ou usar winget
winget install Rustlang.Rust.MSVC
```

### 2. Instalar Dependências

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev curl git
```

#### CentOS/RHEL/Fedora
```bash
# CentOS/RHEL
sudo yum groupinstall -y "Development Tools"
sudo yum install -y pkgconfig openssl-devel curl git

# Fedora
sudo dnf groupinstall -y "Development Tools"
sudo dnf install -y pkgconfig openssl-devel curl git
```

#### macOS
```bash
# Instalar Xcode Command Line Tools
xcode-select --install

# Instalar dependências via Homebrew
brew install pkg-config openssl curl git
```

#### Windows
```powershell
# Instalar Visual Studio Build Tools
# Baixar de: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022

# Instalar Git
winget install Git.Git

# Instalar Chocolatey (opcional)
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
```

### 3. Compilar o Polis

```bash
# Clone o repositório
git clone https://github.com/polis/polis.git
cd polis

# Compilar em modo release
cargo build --release

# Verificar compilação
ls target/release/polis*
```

### 4. Instalar os Binários

#### Linux/macOS
```bash
# Instalar binários
sudo cp target/release/polis /usr/local/bin/
sudo cp target/release/polis-api /usr/local/bin/
sudo cp target/release/polis-grpc /usr/local/bin/

# Verificar instalação
polis --version
polis-api --version
polis-grpc --version
```

#### Windows
```powershell
# Criar diretório de instalação
New-Item -ItemType Directory -Path "C:\Program Files\Polis" -Force

# Copiar binários
Copy-Item "target\release\polis.exe" "C:\Program Files\Polis\"
Copy-Item "target\release\polis-api.exe" "C:\Program Files\Polis\"
Copy-Item "target\release\polis-grpc.exe" "C:\Program Files\Polis\"

# Adicionar ao PATH
$env:PATH += ";C:\Program Files\Polis"
[Environment]::SetEnvironmentVariable("PATH", $env:PATH, [EnvironmentVariableTarget]::User)
```

## ⚙️ Configuração Inicial

### 1. Inicializar o Polis

```bash
# Inicializar configuração
polis init

# Verificar configuração
polis config show
```

### 2. Configurar Registries

```bash
# Configurar Docker Hub
polis registry add docker.io --username your-username --password your-token

# Configurar registry customizado
polis registry add my-registry.com --username admin --password secret

# Listar registries configurados
polis registry list
```

### 3. Testar Instalação

```bash
# Baixar imagem de teste
polis image pull alpine:latest

# Criar container de teste
polis container create --name test --image alpine:latest --command "echo 'Polis funcionando!'"

# Executar container
polis container start test

# Ver logs
polis container logs test

# Limpar teste
polis container stop test
polis container remove test
```

## 🔧 Configuração Avançada

### Arquivo de Configuração

O Polis usa arquivos de configuração em múltiplos formatos:

#### YAML (config.yaml)
```yaml
polis:
  runtime:
    default_driver: "runc"
    data_root: "/var/lib/polis"
    log_level: "info"
  
  network:
    default_bridge: "polis0"
    default_subnet: "172.17.0.0/16"
  
  registry:
    insecure_registries: []
    mirrors: {}
  
  security:
    seccomp_profile: "default"
    apparmor_profile: "docker-default"
```

#### TOML (config.toml)
```toml
[polis.runtime]
default_driver = "runc"
data_root = "/var/lib/polis"
log_level = "info"

[polis.network]
default_bridge = "polis0"
default_subnet = "172.17.0.0/16"

[polis.registry]
insecure_registries = []
mirrors = {}

[polis.security]
seccomp_profile = "default"
apparmor_profile = "docker-default"
```

#### JSON (config.json)
```json
{
  "polis": {
    "runtime": {
      "default_driver": "runc",
      "data_root": "/var/lib/polis",
      "log_level": "info"
    },
    "network": {
      "default_bridge": "polis0",
      "default_subnet": "172.17.0.0/16"
    },
    "registry": {
      "insecure_registries": [],
      "mirrors": {}
    },
    "security": {
      "seccomp_profile": "default",
      "apparmor_profile": "docker-default"
    }
  }
}
```

### Variáveis de Ambiente

```bash
# Configurar via variáveis de ambiente
export POLIS_DATA_ROOT="/var/lib/polis"
export POLIS_LOG_LEVEL="debug"
export POLIS_DEFAULT_REGISTRY="docker.io"
export POLIS_INSECURE_REGISTRIES="localhost:5000"
```

## 🐛 Resolução de Problemas

### Problemas Comuns

#### Erro de Compilação
```bash
# Limpar cache do Cargo
cargo clean

# Atualizar dependências
cargo update

# Recompilar
cargo build --release
```

#### Erro de Permissão
```bash
# Linux: Adicionar usuário ao grupo docker
sudo usermod -aG docker $USER
newgrp docker

# Ou executar com sudo
sudo polis --version
```

#### Erro de Dependências
```bash
# Ubuntu/Debian
sudo apt install -y build-essential pkg-config libssl-dev

# CentOS/RHEL
sudo yum groupinstall -y "Development Tools"
sudo yum install -y pkgconfig openssl-devel

# macOS
xcode-select --install
brew install pkg-config openssl
```

#### Erro de Registry
```bash
# Configurar registry inseguro
polis registry add localhost:5000 --insecure

# Verificar conectividade
polis registry ping docker.io
```

### Logs de Debug

```bash
# Executar com debug
RUST_LOG=debug polis --version

# Ver logs detalhados
polis logs --follow container-name

# Verificar status do sistema
polis system info
```

## 🔄 Atualização

### Atualizar Polis

```bash
# Parar serviços
polis system stop

# Fazer backup da configuração
cp -r ~/.polis ~/.polis.backup

# Atualizar código
git pull origin main

# Recompilar
cargo build --release

# Reinstalar
sudo cp target/release/polis* /usr/local/bin/

# Reiniciar serviços
polis system start
```

### Rollback

```bash
# Restaurar versão anterior
git checkout v0.0.9

# Recompilar versão anterior
cargo build --release

# Reinstalar
sudo cp target/release/polis* /usr/local/bin/
```

## 🧪 Verificação de Instalação

### Script de Verificação

```bash
#!/bin/bash
echo "=== Verificação da Instalação do Polis ==="

# Verificar binários
echo "1. Verificando binários..."
polis --version && echo "✅ polis CLI OK" || echo "❌ polis CLI FALHOU"
polis-api --version && echo "✅ polis-api OK" || echo "❌ polis-api FALHOU"
polis-grpc --version && echo "✅ polis-grpc OK" || echo "❌ polis-grpc FALHOU"

# Verificar configuração
echo "2. Verificando configuração..."
polis config show && echo "✅ Configuração OK" || echo "❌ Configuração FALHOU"

# Teste de funcionalidade
echo "3. Testando funcionalidade..."
polis image pull alpine:latest && echo "✅ Pull de imagem OK" || echo "❌ Pull de imagem FALHOU"

# Verificar sistema
echo "4. Verificando sistema..."
polis system info && echo "✅ Sistema OK" || echo "❌ Sistema FALHOU"

echo "=== Verificação Concluída ==="
```

## 📞 Suporte

### Canais de Suporte
- **GitHub Issues**: [github.com/polis/polis/issues](https://github.com/polis/polis/issues)
- **Discord**: [discord.gg/polis](https://discord.gg/polis)
- **Stack Overflow**: [stackoverflow.com/tags/polis](https://stackoverflow.com/tags/polis)
- **Email**: support@polis.dev

### Recursos Adicionais
- **Documentação**: [docs.polis.dev](https://docs.polis.dev)
- **Exemplos**: [examples.polis.dev](https://examples.polis.dev)
- **Tutoriais**: [tutorials.polis.dev](https://tutorials.polis.dev)

---

**Última atualização**: Janeiro 2025  
**Versão**: 1.0.0  
**Status**: Ativa e mantida

**Polis** - Container Runtime moderno, seguro e eficiente. Feito com ❤ no Brasil.
