# Tutorial 1: Instalação e Configuração Inicial do Polis

## 📖 Objetivos
- Instalar o Polis no seu sistema
- Configurar o ambiente de desenvolvimento
- Verificar a instalação
- Entender a estrutura básica do projeto

## ⏱️ Duração
- **Estimada**: 15 minutos
- **Nível**: Iniciante

## 📋 Pré-requisitos
- Sistema operacional Linux, macOS ou Windows
- Rust 1.70+ instalado
- Git instalado
- Acesso à internet

## 🎬 Roteiro do Vídeo

### Introdução (0:00 - 2:00)
- Apresentação do Polis
- O que é um container runtime
- Por que escolher o Polis
- O que será coberto neste tutorial

### Instalação do Rust (2:00 - 5:00)
```bash
# Instalar Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verificar instalação
rustc --version
cargo --version
```

### Clonagem do Repositório (5:00 - 7:00)
```bash
# Clonar o repositório
git clone https://github.com/polis-project/polis.git
cd polis

# Verificar estrutura do projeto
ls -la
```

### Compilação do Projeto (7:00 - 10:00)
```bash
# Compilar o projeto
cargo build --release

# Verificar se compilou corretamente
ls target/release/
```

### Configuração Inicial (10:00 - 12:00)
```bash
# Criar diretório de configuração
mkdir -p ~/.polis

# Criar arquivo de configuração básico
cat > ~/.polis/config.yaml << EOF
storage:
  root_dir: ~/.polis/data
  image_cache_dir: ~/.polis/images

network:
  bridge_name: polis0
  subnet: "172.17.0.0/16"

api:
  rest_port: 8080
  grpc_port: 9090

logging:
  level: info
  file: ~/.polis/logs/polis.log
EOF
```

### Verificação da Instalação (12:00 - 15:00)
```bash
# Executar o CLI
./target/release/polis --version

# Verificar comandos disponíveis
./target/release/polis --help

# Testar criação de container
./target/release/polis container create --name test --image alpine:latest
```

## 💻 Código de Exemplo

### Script de Instalação Automática
```bash
#!/bin/bash
# install_polis.sh

set -e

echo "🚀 Instalando Polis..."

# Verificar Rust
if ! command -v rustc &> /dev/null; then
    echo "📦 Instalando Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source ~/.cargo/env
fi

# Clonar repositório
if [ ! -d "polis" ]; then
    echo "📥 Clonando repositório..."
    git clone https://github.com/polis-project/polis.git
fi

cd polis

# Compilar
echo "🔨 Compilando Polis..."
cargo build --release

# Criar configuração
echo "⚙️ Criando configuração..."
mkdir -p ~/.polis
cp examples/config.yaml ~/.polis/

echo "✅ Instalação concluída!"
echo "Execute: ./target/release/polis --help"
```

### Configuração Avançada
```yaml
# ~/.polis/config.yaml
storage:
  root_dir: ~/.polis/data
  image_cache_dir: ~/.polis/images
  volume_dir: ~/.polis/volumes

network:
  bridge_name: polis0
  subnet: "172.17.0.0/16"
  gateway: "172.17.0.1"
  dns_servers:
    - "8.8.8.8"
    - "8.8.4.4"

api:
  rest_port: 8080
  grpc_port: 9090
  host: "0.0.0.0"

security:
  enable_apparmor: true
  enable_selinux: true
  default_capabilities:
    - "NET_ADMIN"
    - "SYS_ADMIN"

logging:
  level: info
  file: ~/.polis/logs/polis.log
  max_size: "100MB"
  max_files: 5

monitoring:
  enable_metrics: true
  metrics_port: 9091
  health_check_interval: "30s"
```

## 🔗 Recursos Adicionais
- [Documentação de Instalação](../INSTALACAO.md)
- [Configuração Avançada](../CONFIGURACAO.md)
- [Troubleshooting](../TROUBLESHOOTING.md)
- [Fórum da Comunidade](https://github.com/polis-project/polis/discussions)

## ❓ Exercícios
1. **Instalação Básica**: Siga o tutorial e instale o Polis
2. **Configuração Personalizada**: Modifique o arquivo de configuração
3. **Verificação**: Execute todos os comandos de verificação
4. **Exploração**: Use `polis --help` para ver todos os comandos

## 🎯 Próximos Tutoriais
- [Tutorial 2: Primeiros Passos com Containers](02-primeiros-passos.md)
- [Tutorial 3: Conceitos Fundamentais](03-conceitos-fundamentais.md)

## 📝 Notas do Instrutor

### Pontos Importantes
- Sempre verificar a versão do Rust antes de começar
- Explicar a diferença entre `cargo build` e `cargo build --release`
- Mostrar como interpretar mensagens de erro comuns
- Enfatizar a importância da configuração inicial

### Dicas de Apresentação
- Usar uma tela limpa e organizada
- Falar devagar e pausar entre comandos
- Explicar o que cada comando faz
- Mostrar a saída esperada de cada comando

### Possíveis Problemas
- **Erro de compilação**: Verificar versão do Rust
- **Permissões**: Usar `sudo` se necessário
- **Rede**: Verificar conectividade para clonar repositório
- **Espaço**: Verificar espaço em disco disponível

### Tempo Sugerido por Seção
- Introdução: 2 min
- Instalação Rust: 3 min
- Clonagem: 2 min
- Compilação: 3 min
- Configuração: 2 min
- Verificação: 3 min
- **Total**: 15 min
