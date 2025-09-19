# Polis Installers

Este diretório contém instaladores para diferentes plataformas e scripts de build cross-platform.

##  Instalação Rápida

### Instalação Universal (Recomendado)
```bash
curl -fsSL https://raw.githubusercontent.com/wendelmax/polis/main/installers/scripts/install.sh | bash
```

### Instalação por Plataforma

#### Linux
```bash
curl -fsSL https://raw.githubusercontent.com/wendelmax/polis/main/installers/linux/install.sh | bash
```

#### macOS
```bash
curl -fsSL https://raw.githubusercontent.com/wendelmax/polis/main/installers/macos/install.sh | bash
```

#### Windows (PowerShell)
```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/wendelmax/polis/main/installers/windows/install.ps1" -OutFile "install.ps1"
powershell -ExecutionPolicy Bypass -File install.ps1
```

## � Estrutura de Diretórios

```
installers/
├── windows/           # Instalador para Windows
│   └── install.ps1   # Script PowerShell
├── linux/            # Instalador para Linux
│   └── install.sh    # Script Bash
├── macos/            # Instalador para macOS
│   └── install.sh    # Script Bash
├── scripts/          # Scripts universais
│   └── install.sh    # Instalador universal
└── build-cross-platform.sh  # Script de build
```

##  Opções de Instalação

### Opções Comuns
- `-v, --version VERSION`: Versão específica para instalar
- `-s, --skip-deps`: Pular instalação de dependências
- `-f, --force`: Forçar instalação (sobrescrever)
- `-h, --help`: Mostrar ajuda

### Exemplos
```bash
# Instalar versão específica
./install.sh -v v1.2.0

# Pular dependências
./install.sh -s

# Forçar reinstalação
./install.sh -f
```

##  Build Cross-Platform

### Compilar para Todas as Plataformas
```bash
./build-cross-platform.sh v1.0.0
```

### Targets Suportados
- `x86_64-unknown-linux-gnu` - Linux x86_64
- `aarch64-unknown-linux-gnu` - Linux ARM64
- `x86_64-apple-darwin` - macOS Intel
- `aarch64-apple-darwin` - macOS Apple Silicon
- `x86_64-pc-windows-gnu` - Windows x86_64

### Dependências de Build
```bash
# Ubuntu/Debian
sudo apt-get install gcc-aarch64-linux-gnu gcc-x86-64-linux-gnu

# macOS
brew install FiloSottile/musl-cross/musl-cross

# Windows (via Chocolatey)
choco install mingw
```

##  Pacotes de Distribuição

Os builds criam pacotes prontos para distribuição:

### Linux
- `polis-linux-x86_64.tar.gz` - Linux x86_64
- `polis-linux-aarch64.tar.gz` - Linux ARM64

### macOS
- `polis-macos-x86_64.tar.gz` - macOS Intel
- `polis-macos-aarch64.tar.gz` - macOS Apple Silicon

### Windows
- `polis-windows-x86_64.zip` - Windows x86_64

## � Verificação de Integridade

Cada pacote inclui um arquivo `.sha256` para verificação:

```bash
# Verificar checksum
sha256sum -c polis-linux-x86_64.tar.gz.sha256
```

##  GitHub Actions

O workflow `.github/workflows/build-installers.yml` automatiza:

1. **Build** para todas as plataformas
2. **Criação** de pacotes de distribuição
3. **Upload** de artifacts
4. **Release** automático com downloads

### Trigger
- Push de tags (`v*`)
- Workflow dispatch manual

##  Desenvolvimento

### Testar Instaladores Localmente
```bash
# Linux
./linux/install.sh -v v1.0.0 -s

# macOS
./macos/install.sh -v v1.0.0 -s

# Windows
powershell -ExecutionPolicy Bypass -File windows/install.ps1 -Version v1.0.0 -SkipDependencies
```

### Adicionar Nova Plataforma
1. Criar diretório `installers/nova-plataforma/`
2. Criar script `install.sh` ou `install.ps1`
3. Adicionar target ao `build-cross-platform.sh`
4. Atualizar GitHub Actions workflow

## � Requisitos

### Linux
- curl, wget, git
- build-essential (gcc, make, etc.)
- pkg-config, libssl-dev

### macOS
- Xcode Command Line Tools
- Homebrew (opcional)
- curl, wget, git

### Windows
- PowerShell 5.1+
- Git for Windows
- Visual Studio Build Tools (para compilação)

## � Troubleshooting

### Problemas Comuns

#### Erro de Permissão (Linux/macOS)
```bash
chmod +x install.sh
sudo ./install.sh
```

#### Erro de Política de Execução (Windows)
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

#### Erro de Dependências
```bash
# Instalar dependências manualmente
sudo apt-get install curl wget git build-essential  # Linux
brew install curl wget git                          # macOS
```

#### Erro de Compilação
```bash
# Limpar cache do Cargo
cargo clean
# Reinstalar Rust
rustup self uninstall
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## � Suporte

- **Issues**: [GitHub Issues](https://github.com/wendelmax/polis/issues)
- **Documentação**: [Wiki](https://github.com/wendelmax/polis/wiki)
- **Discussões**: [GitHub Discussions](https://github.com/wendelmax/polis/discussions)
