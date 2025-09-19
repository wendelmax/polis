#!/bin/bash
# Cross-Platform Build Script for Polis
# Compila Polis para diferentes plataformas e arquiteturas

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configurações
VERSION=${1:-"v1.0.0"}
BUILD_DIR="build"
DIST_DIR="dist"
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
)

# Função para output colorido
print_color() {
    printf "${1}${2}${NC}\n"
}

# Função para instalar Rust targets
install_targets() {
    print_color $BLUE " Instalando targets Rust..."
    
    for target in "${TARGETS[@]}"; do
        print_color $YELLOW "   Instalando $target..."
        rustup target add "$target" || true
    done
}

# Função para instalar dependências cross-compilation
install_cross_deps() {
    print_color $BLUE " Instalando dependências de cross-compilation..."
    
    # Linux
    if command -v apt-get &> /dev/null; then
        sudo apt-get update
        sudo apt-get install -y gcc-multilib gcc-aarch64-linux-gnu gcc-x86-64-linux-gnu
    elif command -v dnf &> /dev/null; then
        sudo dnf install -y gcc-c++ gcc-aarch64-linux-gnu gcc-x86_64-linux-gnu
    fi
    
    # macOS (se estiver no macOS)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        if command -v brew &> /dev/null; then
            brew install FiloSottile/musl-cross/musl-cross
        fi
    fi
}

# Função para compilar para um target específico
build_target() {
    local target=$1
    local os_arch=$(echo $target | cut -d'-' -f1,3)
    
    print_color $BLUE "� Compilando para $target..."
    
    # Configurar variáveis de ambiente
    case $target in
        *-linux-gnu)
            export CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
            export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
            ;;
        *-linux-musl)
            export CC_x86_64_unknown_linux_musl=x86_64-linux-musl-gcc
            ;;
        *-apple-darwin)
            # macOS targets
            ;;
        *-pc-windows-gnu)
            export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
            ;;
    esac
    
    # Compilar
    cargo build --release --target "$target" --bin polis
    
    # Criar diretório de distribuição
    local dist_path="$DIST_DIR/$os_arch"
    mkdir -p "$dist_path"
    
    # Copiar binário
    local binary_name="polis"
    if [[ $target == *"windows"* ]]; then
        binary_name="polis.exe"
    fi
    
    cp "target/$target/release/$binary_name" "$dist_path/"
    
    # Criar arquivo de configuração
    cat > "$dist_path/config.yaml" << EOF
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

    # Criar README
    cat > "$dist_path/README.md" << EOF
# Polis Container Runtime - $os_arch

Versão: $VERSION
Arquitetura: $os_arch
Target: $target

## Instalação

### Linux
\`\`\`bash
sudo cp polis /usr/local/bin/
sudo chmod +x /usr/local/bin/polis
\`\`\`

### macOS
\`\`\`bash
sudo cp polis /usr/local/bin/
sudo chmod +x /usr/local/bin/polis
\`\`\`

### Windows
Copie \`polis.exe\` para uma pasta no PATH.

## Uso
\`\`\`bash
polis --help
\`\`\`
EOF

    print_color $GREEN " Compilado para $target"
}

# Função para criar pacotes de distribuição
create_packages() {
    print_color $BLUE " Criando pacotes de distribuição..."
    
    # Linux x86_64
    if [ -d "$DIST_DIR/x86_64-linux" ]; then
        print_color $YELLOW "   Criando pacote Linux x86_64..."
        cd "$DIST_DIR/x86_64-linux"
        tar -czf "../polis-linux-x86_64.tar.gz" *
        cd - > /dev/null
    fi
    
    # Linux aarch64
    if [ -d "$DIST_DIR/aarch64-linux" ]; then
        print_color $YELLOW "   Criando pacote Linux aarch64..."
        cd "$DIST_DIR/aarch64-linux"
        tar -czf "../polis-linux-aarch64.tar.gz" *
        cd - > /dev/null
    fi
    
    # macOS x86_64
    if [ -d "$DIST_DIR/x86_64-macos" ]; then
        print_color $YELLOW "   Criando pacote macOS x86_64..."
        cd "$DIST_DIR/x86_64-macos"
        tar -czf "../polis-macos-x86_64.tar.gz" *
        cd - > /dev/null
    fi
    
    # macOS aarch64
    if [ -d "$DIST_DIR/aarch64-macos" ]; then
        print_color $YELLOW "   Criando pacote macOS aarch64..."
        cd "$DIST_DIR/aarch64-macos"
        tar -czf "../polis-macos-aarch64.tar.gz" *
        cd - > /dev/null
    fi
    
    # Windows x86_64
    if [ -d "$DIST_DIR/x86_64-windows" ]; then
        print_color $YELLOW "   Criando pacote Windows x86_64..."
        cd "$DIST_DIR/x86_64-windows"
        zip -r "../polis-windows-x86_64.zip" *
        cd - > /dev/null
    fi
}

# Função para criar checksums
create_checksums() {
    print_color $BLUE "� Criando checksums..."
    
    cd "$DIST_DIR"
    for file in *.tar.gz *.zip; do
        if [ -f "$file" ]; then
            sha256sum "$file" > "${file}.sha256"
            print_color $GREEN "   $file.sha256"
        fi
    done
    cd - > /dev/null
}

# Função para mostrar ajuda
show_help() {
    cat << EOF
Cross-Platform Build Script for Polis

Uso: $0 [VERSÃO]

Argumentos:
    VERSÃO    Versão para compilar (padrão: v1.0.0)

Exemplos:
    $0                # Compilar versão v1.0.0
    $0 v1.2.0         # Compilar versão v1.2.0

Targets suportados:
    - x86_64-unknown-linux-gnu
    - x86_64-unknown-linux-musl
    - aarch64-unknown-linux-gnu
    - x86_64-apple-darwin
    - aarch64-apple-darwin
    - x86_64-pc-windows-gnu

EOF
}

# Main
if [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
    show_help
    exit 0
fi

print_color $BLUE " Cross-Platform Build for Polis $VERSION"
print_color $BLUE "=========================================="

# Criar diretórios
mkdir -p "$BUILD_DIR" "$DIST_DIR"

# Instalar targets e dependências
install_targets
install_cross_deps

# Compilar para cada target
for target in "${TARGETS[@]}"; do
    build_target "$target"
done

# Criar pacotes
create_packages

# Criar checksums
create_checksums

print_color $GREEN " Build concluído com sucesso!"
print_color $BLUE "   Pacotes criados em: $DIST_DIR/"
print_color $BLUE "   Lista de arquivos:"
ls -la "$DIST_DIR"/*.tar.gz "$DIST_DIR"/*.zip 2>/dev/null || true
