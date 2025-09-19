#!/bin/bash
# Polis Installer for Linux
# Instala o Polis Container Runtime no Linux

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
CONFIG_PATH="/etc/polis"
SKIP_DEPS=false
FORCE=false

# Função para output colorido
print_color() {
    printf "${1}${2}${NC}\n"
}

# Função para detectar distribuição
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO=$ID
        VERSION_ID=$VERSION_ID
    elif [ -f /etc/redhat-release ]; then
        DISTRO="rhel"
    elif [ -f /etc/debian_version ]; then
        DISTRO="debian"
    else
        DISTRO="unknown"
    fi
}

# Função para instalar dependências
install_dependencies() {
    if [ "$SKIP_DEPS" = true ]; then
        print_color $YELLOW "  Pulando instalação de dependências"
        return
    fi

    print_color $BLUE " Instalando dependências..."

    case $DISTRO in
        "ubuntu"|"debian")
            sudo apt-get update
            sudo apt-get install -y curl wget git build-essential pkg-config libssl-dev
            ;;
        "centos"|"rhel"|"fedora")
            if command -v dnf &> /dev/null; then
                sudo dnf install -y curl wget git gcc gcc-c++ make pkgconfig openssl-devel
            else
                sudo yum install -y curl wget git gcc gcc-c++ make pkgconfig openssl-devel
            fi
            ;;
        "arch"|"manjaro")
            sudo pacman -S --noconfirm curl wget git base-devel pkgconf openssl
            ;;
        *)
            print_color $YELLOW "⚠  Distribuição não reconhecida. Instale manualmente:"
            print_color $YELLOW "   - curl, wget, git"
            print_color $YELLOW "   - build-essential (gcc, make, etc.)"
            print_color $YELLOW "   - pkg-config, libssl-dev"
            ;;
    esac

    # Instalar Rust se não estiver instalado
    if ! command -v rustc &> /dev/null; then
        print_color $BLUE " Instalando Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    else
        print_color $GREEN " Rust já instalado: $(rustc --version)"
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
        aarch64|arm64) ARCH="aarch64" ;;
        armv7l) ARCH="armv7" ;;
        *) print_color $RED " Arquitetura não suportada: $ARCH"; exit 1 ;;
    esac

    # URL de download
    DOWNLOAD_URL="https://github.com/wendelmax/polis/releases/download/$VERSION/polis-linux-$ARCH.tar.gz"
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
    sudo mkdir -p "$CONFIG_PATH"
    sudo chown $USER:$USER "$CONFIG_PATH"

    # Criar arquivo de configuração padrão
    cat > "$CONFIG_PATH/config.yaml" << EOF
# Polis Configuration
runtime:
  default_driver: "runc"
  log_level: "info"
  data_root: "/var/lib/polis"

network:
  bridge_name: "polis0"
  subnet: "172.17.0.0/16"
  gateway: "172.17.0.1"

security:
  enable_seccomp: true
  enable_apparmor: true
  enable_selinux: false

api:
  rest_port: 8080
  grpc_port: 9090
  enable_tls: false
EOF

    print_color $GREEN " Polis instalado em $INSTALL_PATH"
    print_color $GREEN " Configuração criada em $CONFIG_PATH"
}

# Função para configurar systemd
setup_systemd() {
    print_color $BLUE "⚙  Configurando serviço systemd..."

    sudo tee /etc/systemd/system/polis.service > /dev/null << EOF
[Unit]
Description=Polis Container Runtime
Documentation=https://github.com/wendelmax/polis
After=network.target

[Service]
Type=simple
User=root
ExecStart=$INSTALL_PATH/polis daemon
ExecReload=/bin/kill -HUP \$MAINPID
Restart=always
RestartSec=5
LimitNOFILE=infinity
LimitNPROC=infinity
LimitCORE=infinity
TasksMax=infinity
Delegate=yes
KillMode=process
OOMScoreAdjust=-999

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable polis

    print_color $GREEN " Serviço systemd configurado"
    print_color $BLUE "   Iniciar: sudo systemctl start polis"
    print_color $BLUE "   Status: sudo systemctl status polis"
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
Polis Container Runtime Installer

Uso: $0 [OPÇÕES]

Opções:
    -v, --version VERSION    Versão para instalar (padrão: latest)
    -p, --path PATH         Caminho de instalação (padrão: /usr/local/bin)
    -c, --config PATH       Caminho de configuração (padrão: /etc/polis)
    -s, --skip-deps         Pular instalação de dependências
    -f, --force             Forçar instalação (sobrescrever)
    -h, --help              Mostrar esta ajuda

Exemplos:
    $0                      # Instalar versão mais recente
    $0 -v v1.0.0           # Instalar versão específica
    $0 -s                   # Pular dependências
    $0 -f                   # Forçar instalação

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
print_color $BLUE " Polis Container Runtime Installer"
print_color $BLUE "===================================="

# Detectar distribuição
detect_distro
print_color $BLUE "� Distribuição detectada: $DISTRO"

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

# Configurar systemd
setup_systemd

# Testar instalação
test_installation

print_color $GREEN " Instalação concluída com sucesso!"
print_color $BLUE "   Reinicie o terminal ou execute: source ~/.bashrc"
