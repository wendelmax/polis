#!/bin/bash
# Universal Polis Installer
# Detecta a plataforma e executa o instalador apropriado

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Função para output colorido
print_color() {
    printf "${1}${2}${NC}\n"
}

# Função para detectar plataforma
detect_platform() {
    local os=$(uname -s)
    local arch=$(uname -m)
    
    case $os in
        "Linux")
            PLATFORM="linux"
            ;;
        "Darwin")
            PLATFORM="macos"
            ;;
        "CYGWIN"*|"MINGW"*|"MSYS"*)
            PLATFORM="windows"
            ;;
        *)
            print_color $RED " Sistema operacional não suportado: $os"
            exit 1
            ;;
    esac
    
    case $arch in
        "x86_64"|"amd64")
            ARCH="x86_64"
            ;;
        "aarch64"|"arm64")
            ARCH="aarch64"
            ;;
        "armv7l")
            ARCH="armv7"
            ;;
        *)
            print_color $YELLOW "⚠  Arquitetura não testada: $arch"
            ARCH="$arch"
            ;;
    esac
}

# Função para baixar instalador específico
download_installer() {
    local platform=$1
    local installer_url="https://raw.githubusercontent.com/wendelmax/polis/main/installers/$platform/install.sh"
    local installer_path="/tmp/polis-installer-$platform.sh"
    
    print_color $BLUE " Baixando instalador para $platform..."
    
    if curl -fsSL "$installer_url" -o "$installer_path"; then
        chmod +x "$installer_path"
        echo "$installer_path"
    else
        print_color $RED " Erro ao baixar instalador para $platform"
        return 1
    fi
}

# Função para executar instalador local
run_local_installer() {
    local platform=$1
    local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local installer_path="$script_dir/../$platform/install.sh"
    
    if [ -f "$installer_path" ]; then
        print_color $BLUE " Executando instalador local para $platform..."
        bash "$installer_path" "$@"
    else
        print_color $RED " Instalador local não encontrado: $installer_path"
        return 1
    fi
}

# Função para mostrar ajuda
show_help() {
    cat << EOF
Universal Polis Installer

Uso: $0 [OPÇÕES]

Este script detecta automaticamente sua plataforma e executa o instalador apropriado.

Opções:
    -p, --platform PLATFORM  Forçar plataforma (linux|macos|windows)
    -v, --version VERSION     Versão para instalar (padrão: latest)
    -s, --skip-deps          Pular instalação de dependências
    -f, --force              Forçar instalação (sobrescrever)
    -l, --local              Usar instaladores locais
    -h, --help               Mostrar esta ajuda

Exemplos:
    $0                       # Instalação automática
    $0 -p linux             # Forçar Linux
    $0 -v v1.0.0            # Versão específica
    $0 -l                   # Usar instaladores locais

EOF
}

# Parse argumentos
PLATFORM=""
VERSION="latest"
SKIP_DEPS=false
FORCE=false
LOCAL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--platform)
            PLATFORM="$2"
            shift 2
            ;;
        -v|--version)
            VERSION="$2"
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
        -l|--local)
            LOCAL=true
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
print_color $BLUE " Universal Polis Installer"
print_color $BLUE "============================"

# Detectar plataforma se não especificada
if [ -z "$PLATFORM" ]; then
    detect_platform
    print_color $GREEN " Plataforma detectada: $PLATFORM ($ARCH)"
else
    print_color $GREEN " Plataforma especificada: $PLATFORM"
fi

# Preparar argumentos para o instalador
INSTALLER_ARGS=()
if [ "$VERSION" != "latest" ]; then
    INSTALLER_ARGS+=("-v" "$VERSION")
fi
if [ "$SKIP_DEPS" = true ]; then
    INSTALLER_ARGS+=("-s")
fi
if [ "$FORCE" = true ]; then
    INSTALLER_ARGS+=("-f")
fi

# Executar instalador apropriado
case $PLATFORM in
    "linux")
        if [ "$LOCAL" = true ]; then
            run_local_installer "linux" "${INSTALLER_ARGS[@]}"
        else
            installer_path=$(download_installer "linux")
            bash "$installer_path" "${INSTALLER_ARGS[@]}"
            rm -f "$installer_path"
        fi
        ;;
    "macos")
        if [ "$LOCAL" = true ]; then
            run_local_installer "macos" "${INSTALLER_ARGS[@]}"
        else
            installer_path=$(download_installer "macos")
            bash "$installer_path" "${INSTALLER_ARGS[@]}"
            rm -f "$installer_path"
        fi
        ;;
    "windows")
        print_color $YELLOW "⚠  Windows requer PowerShell"
        print_color $BLUE "   Execute: powershell -ExecutionPolicy Bypass -File install.ps1"
        if [ "$LOCAL" = true ]; then
            print_color $BLUE "   Ou: powershell -ExecutionPolicy Bypass -File installers/windows/install.ps1"
        else
            print_color $BLUE "   Ou baixe: https://raw.githubusercontent.com/wendelmax/polis/main/installers/windows/install.ps1"
        fi
        ;;
    *)
        print_color $RED " Plataforma não suportada: $PLATFORM"
        exit 1
        ;;
esac

print_color $GREEN " Instalação concluída!"
