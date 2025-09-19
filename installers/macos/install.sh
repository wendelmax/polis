#!/bin/bash
# Polis Installer for macOS
# Instala o Polis Container Runtime no macOS

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configurações padrão
VERSION="latest"
INSTALL_PATH="/usr/local/bin"
CONFIG_PATH="$HOME/.polis"
SKIP_DEPS=false
FORCE=false
USE_HOMEBREW=true

# Função para output colorido
print_color() {
    printf "${1}${2}${NC}\n"
}

# Função para verificar se Homebrew está instalado
check_homebrew() {
    if command -v brew &> /dev/null; then
        print_color $GREEN " Homebrew encontrado: $(brew --version | head -n1)"
        return 0
    else
        print_color $YELLOW "⚠  Homebrew não encontrado"
        return 1
    fi
}

# Função para instalar Homebrew
install_homebrew() {
    if [ "$USE_HOMEBREW" = false ]; then
        return 1
    fi

    print_color $BLUE "� Instalando Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    
    # Adicionar Homebrew ao PATH se necessário
    if [[ ":$PATH:" != *":/opt/homebrew/bin:"* ]]; then
        echo 'export PATH="/opt/homebrew/bin:$PATH"' >> ~/.zshrc
        export PATH="/opt/homebrew/bin:$PATH"
    fi
}

# Função para instalar dependências
install_dependencies() {
    if [ "$SKIP_DEPS" = true ]; then
        print_color $YELLOW "  Pulando instalação de dependências"
        return
    fi

    print_color $BLUE " Instalando dependências..."

    # Verificar/instalar Homebrew
    if ! check_homebrew; then
        if [ "$USE_HOMEBREW" = true ]; then
            install_homebrew
        else
            print_color $YELLOW "⚠  Instale manualmente: Xcode Command Line Tools"
            print_color $YELLOW "   xcode-select --install"
            return
        fi
    fi

    # Instalar dependências via Homebrew
    print_color $BLUE "� Instalando dependências via Homebrew..."
    brew install curl wget git pkg-config openssl

    # Instalar Rust se não estiver instalado
    if ! command -v rustc &> /dev/null; then
        print_color $BLUE " Instalando Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    else
        print_color $GREEN " Rust já instalado: $(rustc --version)"
    fi

    # Verificar Xcode Command Line Tools
    if ! xcode-select -p &> /dev/null; then
        print_color $BLUE " Instalando Xcode Command Line Tools..."
        xcode-select --install
        print_color $YELLOW "⚠  Aguarde a instalação do Xcode Command Line Tools e execute novamente"
        exit 0
    fi
}

# Função para obter última versão
get_latest_version() {
    local version=$(curl -s https://api.github.com/repos/wendelmax/polis/releases/latest | grep '"tag_name"' | cut -d'"' -f4)
    echo ${version:-"v1.0.0"}
}

# Função para instalar Polis
install_polis() {
    print_color $BLUE " Instalando Polis..."

    # Obter versão
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(get_latest_version)
    fi

    print_color $BLUE " Baixando Polis $VERSION..."

    # Detectar arquitetura
    ARCH=$(uname -m)
    case $ARCH in
        x86_64) ARCH="x86_64" ;;
        arm64) ARCH="aarch64" ;;
        *) print_color $RED " Arquitetura não suportada: $ARCH"; exit 1 ;;
    esac

    # URL de download
    DOWNLOAD_URL="https://github.com/wendelmax/polis/releases/download/$VERSION/polis-macos-$ARCH.tar.gz"
    TEMP_DIR=$(mktemp -d)
    TARBALL="$TEMP_DIR/polis.tar.gz"

    # Baixar e extrair
    if curl -L "$DOWNLOAD_URL" -o "$TARBALL" 2>/dev/null; then
        tar -xzf "$TARBALL" -C "$TEMP_DIR"
        sudo cp "$TEMP_DIR/polis" "$INSTALL_PATH/polis"
        sudo chmod +x "$INSTALL_PATH/polis"
        rm -rf "$TEMP_DIR"
    else
        print_color $YELLOW "⚠  Erro ao baixar. Compilando a partir do código fonte..."
        
        # Fallback: compilar a partir do código fonte
        git clone https://github.com/wendelmax/polis.git "$TEMP_DIR/polis"
        cd "$TEMP_DIR/polis"
        cargo build --release
        sudo cp "target/release/polis" "$INSTALL_PATH/polis"
        sudo chmod +x "$INSTALL_PATH/polis"
        cd - > /dev/null
        rm -rf "$TEMP_DIR"
    fi

    # Criar diretório de configuração
    mkdir -p "$CONFIG_PATH"

    # Criar arquivo de configuração padrão
    cat > "$CONFIG_PATH/config.yaml" << EOF
# Polis Configuration for macOS
runtime:
  default_driver: "runc"
  log_level: "info"
  data_root: "$HOME/.polis/data"

network:
  bridge_name: "polis0"
  subnet: "172.17.0.0/16"
  gateway: "172.17.0.1"

security:
  enable_seccomp: false
  enable_apparmor: false
  enable_selinux: false

api:
  rest_port: 8080
  grpc_port: 9090
  enable_tls: false

macos:
  enable_hypervisor_framework: true
  use_vmnet: true
EOF

    print_color $GREEN " Polis instalado em $INSTALL_PATH"
    print_color $GREEN " Configuração criada em $CONFIG_PATH"
}

# Função para configurar LaunchAgent
setup_launchagent() {
    print_color $BLUE "⚙  Configurando LaunchAgent..."

    # Criar diretório LaunchAgents se não existir
    mkdir -p "$HOME/Library/LaunchAgents"

    # Criar plist para LaunchAgent
    cat > "$HOME/Library/LaunchAgents/com.polis.daemon.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.polis.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_PATH/polis</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$HOME/.polis/daemon.log</string>
    <key>StandardErrorPath</key>
    <string>$HOME/.polis/daemon.error.log</string>
</dict>
</plist>
EOF

    # Carregar LaunchAgent
    launchctl load "$HOME/Library/LaunchAgents/com.polis.daemon.plist" 2>/dev/null || true

    print_color $GREEN " LaunchAgent configurado"
    print_color $BLUE "   Iniciar: launchctl start com.polis.daemon"
    print_color $BLUE "   Parar: launchctl stop com.polis.daemon"
    print_color $BLUE "   Status: launchctl list | grep polis"
}

# Função para criar aplicativo no Applications
create_app() {
    print_color $BLUE "� Criando aplicativo no Applications..."

    APP_PATH="/Applications/Polis.app"
    APP_CONTENTS="$APP_PATH/Contents"
    APP_MACOS="$APP_CONTENTS/MacOS"
    APP_RESOURCES="$APP_CONTENTS/Resources"

    # Criar estrutura do app
    sudo mkdir -p "$APP_MACOS" "$APP_RESOURCES"

    # Criar Info.plist
    sudo tee "$APP_CONTENTS/Info.plist" > /dev/null << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>polis</string>
    <key>CFBundleIdentifier</key>
    <string>com.polis.container-runtime</string>
    <key>CFBundleName</key>
    <string>Polis</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
</dict>
</plist>
EOF

    # Copiar binário
    sudo cp "$INSTALL_PATH/polis" "$APP_MACOS/polis"
    sudo chmod +x "$APP_MACOS/polis"

    # Criar script wrapper
    sudo tee "$APP_MACOS/polis" > /dev/null << 'EOF'
#!/bin/bash
exec /usr/local/bin/polis "$@"
EOF
    sudo chmod +x "$APP_MACOS/polis"

    print_color $GREEN " Aplicativo criado em $APP_PATH"
}

# Função para testar instalação
test_installation() {
    print_color $BLUE "� Testando instalação..."

    if command -v polis &> /dev/null; then
        local version=$(polis --version 2>/dev/null || echo "unknown")
        print_color $GREEN " Polis instalado com sucesso!"
        print_color $GREEN "   Versão: $version"
        print_color $GREEN "   Localização: $INSTALL_PATH/polis"
        print_color $BLUE "   Execute: polis --help"
    else
        print_color $RED " Erro: polis não encontrado no PATH"
        exit 1
    fi
}

# Função para mostrar ajuda
show_help() {
    cat << EOF
Polis Container Runtime Installer for macOS

Uso: $0 [OPÇÕES]

Opções:
    -v, --version VERSION    Versão para instalar (padrão: latest)
    -p, --path PATH         Caminho de instalação (padrão: /usr/local/bin)
    -c, --config PATH       Caminho de configuração (padrão: ~/.polis)
    -s, --skip-deps         Pular instalação de dependências
    -f, --force             Forçar instalação (sobrescrever)
    --no-homebrew           Não usar Homebrew
    -h, --help              Mostrar esta ajuda

Exemplos:
    $0                      # Instalar versão mais recente
    $0 -v v1.0.0           # Instalar versão específica
    $0 -s                   # Pular dependências
    $0 --no-homebrew        # Não usar Homebrew

EOF
}

# Parse argumentos
while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--version)
            VERSION="$2"
            shift 2
            ;;
        -p|--path)
            INSTALL_PATH="$2"
            shift 2
            ;;
        -c|--config)
            CONFIG_PATH="$2"
            shift 2
            ;;
        -s|--skip-deps)
            SKIP_DEPS=true
            shift
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        --no-homebrew)
            USE_HOMEBREW=false
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            print_color $RED " Opção desconhecida: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main
print_color $BLUE " Polis Container Runtime Installer for macOS"
print_color $BLUE "=============================================="

# Verificar se já está instalado
if command -v polis &> /dev/null && [ "$FORCE" = false ]; then
    print_color $YELLOW "⚠  Polis já está instalado"
    print_color $YELLOW "   Use -f para forçar reinstalação"
    exit 1
fi

# Instalar dependências
install_dependencies

# Instalar Polis
install_polis

# Configurar LaunchAgent
setup_launchagent

# Criar aplicativo
create_app

# Testar instalação
test_installation

print_color $GREEN " Instalação concluída com sucesso!"
print_color $BLUE "   Reinicie o terminal ou execute: source ~/.zshrc"
