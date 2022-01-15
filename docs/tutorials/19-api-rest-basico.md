# Tutorial 19: API REST - Básico

## 📖 Objetivos
- Entender a API REST do Polis
- Fazer requisições básicas
- Gerenciar containers via API
- Trabalhar com autenticação

## ⏱️ Duração
- **Estimada**: 25 minutos
- **Nível**: Intermediário

## 📋 Pré-requisitos
- Polis instalado e configurado
- Conhecimento básico de HTTP
- Familiaridade com JSON
- Tutorial 4 concluído

## 🎬 Roteiro do Vídeo

### Introdução (0:00 - 3:00)
- O que é uma API REST
- Vantagens da API do Polis
- Casos de uso comuns
- O que será coberto

### Conceitos da API (3:00 - 8:00)
- Endpoints principais
- Métodos HTTP
- Códigos de status
- Formato de resposta

### Configurando a API (8:00 - 12:00)
```bash
# Iniciar o servidor API
polis api start --port 8080

# Verificar se está rodando
curl http://localhost:8080/health

# Ver endpoints disponíveis
curl http://localhost:8080/api/v1/
```

### Gerenciando Containers (12:00 - 20:00)
```bash
# Listar containers
curl -X GET http://localhost:8080/api/v1/containers

# Criar container
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "api-test",
    "image": "alpine:latest",
    "command": ["echo", "Hello API"]
  }'

# Iniciar container
curl -X POST http://localhost:8080/api/v1/containers/api-test/start

# Ver logs
curl -X GET http://localhost:8080/api/v1/containers/api-test/logs

# Parar container
curl -X POST http://localhost:8080/api/v1/containers/api-test/stop
```

### Autenticação (20:00 - 25:00)
```bash
# Fazer login
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "password"
  }'

# Usar token
curl -X GET http://localhost:8080/api/v1/containers \
  -H "Authorization: Bearer <token>"
```

## 💻 Código de Exemplo

### Script de Demonstração
```bash
#!/bin/bash
# api_demo.sh

API_BASE="http://localhost:8080/api/v1"

echo "🚀 Demonstração da API REST do Polis"

# 1. Verificar saúde da API
echo "📊 Verificando saúde da API..."
curl -s "$API_BASE/health" | jq .

# 2. Listar containers
echo "📦 Listando containers..."
curl -s "$API_BASE/containers" | jq .

# 3. Criar container
echo "➕ Criando container..."
CONTAINER_ID=$(curl -s -X POST "$API_BASE/containers" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "api-demo",
    "image": "alpine:latest",
    "command": ["sh", "-c", "echo Hello from API && sleep 10"]
  }' | jq -r '.id')

echo "Container criado: $CONTAINER_ID"

# 4. Iniciar container
echo "▶️ Iniciando container..."
curl -s -X POST "$API_BASE/containers/$CONTAINER_ID/start" | jq .

# 5. Ver status
echo "📊 Status do container..."
curl -s "$API_BASE/containers/$CONTAINER_ID" | jq .

# 6. Ver logs
echo "📝 Logs do container..."
curl -s "$API_BASE/containers/$CONTAINER_ID/logs" | jq .

# 7. Parar container
echo "⏹️ Parando container..."
curl -s -X POST "$API_BASE/containers/$CONTAINER_ID/stop" | jq .

# 8. Remover container
echo "🗑️ Removendo container..."
curl -s -X DELETE "$API_BASE/containers/$CONTAINER_ID" | jq .

echo "✅ Demonstração concluída!"
```

### Cliente Python
```python
#!/usr/bin/env python3
# polis_api_client.py

import requests
import json
import time

class PolisAPIClient:
    def __init__(self, base_url="http://localhost:8080/api/v1"):
        self.base_url = base_url
        self.session = requests.Session()
        self.token = None
    
    def login(self, username, password):
        """Fazer login e obter token"""
        response = self.session.post(
            f"{self.base_url}/auth/login",
            json={"username": username, "password": password}
        )
        response.raise_for_status()
        data = response.json()
        self.token = data["token"]
        self.session.headers.update({
            "Authorization": f"Bearer {self.token}"
        })
        return data
    
    def list_containers(self):
        """Listar todos os containers"""
        response = self.session.get(f"{self.base_url}/containers")
        response.raise_for_status()
        return response.json()
    
    def create_container(self, name, image, command):
        """Criar um novo container"""
        data = {
            "name": name,
            "image": image,
            "command": command
        }
        response = self.session.post(
            f"{self.base_url}/containers",
            json=data
        )
        response.raise_for_status()
        return response.json()
    
    def start_container(self, container_id):
        """Iniciar um container"""
        response = self.session.post(
            f"{self.base_url}/containers/{container_id}/start"
        )
        response.raise_for_status()
        return response.json()
    
    def stop_container(self, container_id):
        """Parar um container"""
        response = self.session.post(
            f"{self.base_url}/containers/{container_id}/stop"
        )
        response.raise_for_status()
        return response.json()
    
    def get_container_logs(self, container_id):
        """Obter logs de um container"""
        response = self.session.get(
            f"{self.base_url}/containers/{container_id}/logs"
        )
        response.raise_for_status()
        return response.json()
    
    def delete_container(self, container_id):
        """Remover um container"""
        response = self.session.delete(
            f"{self.base_url}/containers/{container_id}"
        )
        response.raise_for_status()
        return response.json()

# Exemplo de uso
if __name__ == "__main__":
    client = PolisAPIClient()
    
    # Fazer login
    client.login("admin", "password")
    
    # Criar container
    container = client.create_container(
        "python-demo",
        "alpine:latest",
        ["echo", "Hello from Python API client"]
    )
    
    print(f"Container criado: {container['id']}")
    
    # Iniciar container
    client.start_container(container["id"])
    
    # Aguardar um pouco
    time.sleep(2)
    
    # Ver logs
    logs = client.get_container_logs(container["id"])
    print(f"Logs: {logs}")
    
    # Parar e remover container
    client.stop_container(container["id"])
    client.delete_container(container["id"])
    
    print("Container removido!")
```

### Cliente JavaScript
```javascript
// polis_api_client.js

class PolisAPIClient {
    constructor(baseUrl = 'http://localhost:8080/api/v1') {
        this.baseUrl = baseUrl;
        this.token = null;
    }
    
    async login(username, password) {
        const response = await fetch(`${this.baseUrl}/auth/login`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ username, password })
        });
        
        if (!response.ok) {
            throw new Error('Login failed');
        }
        
        const data = await response.json();
        this.token = data.token;
        return data;
    }
    
    async request(endpoint, options = {}) {
        const url = `${this.baseUrl}${endpoint}`;
        const config = {
            headers: {
                'Content-Type': 'application/json',
                ...(this.token && { 'Authorization': `Bearer ${this.token}` }),
                ...options.headers
            },
            ...options
        };
        
        const response = await fetch(url, config);
        
        if (!response.ok) {
            throw new Error(`API request failed: ${response.statusText}`);
        }
        
        return response.json();
    }
    
    async listContainers() {
        return this.request('/containers');
    }
    
    async createContainer(name, image, command) {
        return this.request('/containers', {
            method: 'POST',
            body: JSON.stringify({ name, image, command })
        });
    }
    
    async startContainer(containerId) {
        return this.request(`/containers/${containerId}/start`, {
            method: 'POST'
        });
    }
    
    async stopContainer(containerId) {
        return this.request(`/containers/${containerId}/stop`, {
            method: 'POST'
        });
    }
    
    async getContainerLogs(containerId) {
        return this.request(`/containers/${containerId}/logs`);
    }
    
    async deleteContainer(containerId) {
        return this.request(`/containers/${containerId}`, {
            method: 'DELETE'
        });
    }
}

// Exemplo de uso
async function demo() {
    const client = new PolisAPIClient();
    
    try {
        // Fazer login
        await client.login('admin', 'password');
        console.log('Login successful');
        
        // Criar container
        const container = await client.createContainer(
            'js-demo',
            'alpine:latest',
            ['echo', 'Hello from JavaScript API client']
        );
        console.log('Container created:', container.id);
        
        // Iniciar container
        await client.startContainer(container.id);
        console.log('Container started');
        
        // Aguardar um pouco
        await new Promise(resolve => setTimeout(resolve, 2000));
        
        // Ver logs
        const logs = await client.getContainerLogs(container.id);
        console.log('Logs:', logs);
        
        // Parar e remover container
        await client.stopContainer(container.id);
        await client.deleteContainer(container.id);
        console.log('Container removed');
        
    } catch (error) {
        console.error('Error:', error.message);
    }
}

// Executar demo
demo();
```

## 🔗 Recursos Adicionais
- [Documentação da API REST](../API_REST.md)
- [Referência de Endpoints](../ENDPOINTS.md)
- [Exemplos Avançados](../EXEMPLOS_API.md)
- [Autenticação e Segurança](../AUTENTICACAO.md)

## ❓ Exercícios
1. **API Básica**: Faça requisições para listar e criar containers
2. **Autenticação**: Implemente login e use tokens
3. **Cliente Personalizado**: Crie um cliente em sua linguagem preferida
4. **Automação**: Automatize o gerenciamento de containers

## 🎯 Próximos Tutoriais
- [Tutorial 20: API REST - Avançado](20-api-rest-avancado.md)
- [Tutorial 21: gRPC e Integração](21-grpc-integracao.md)

## 📝 Notas do Instrutor

### Pontos Importantes
- Explicar a diferença entre GET, POST, PUT, DELETE
- Mostrar como interpretar códigos de status HTTP
- Enfatizar a importância da autenticação
- Demonstrar tratamento de erros

### Dicas de Apresentação
- Usar ferramentas como Postman ou curl
- Mostrar a resposta JSON de cada requisição
- Explicar cada campo da resposta
- Demonstrar diferentes cenários de erro

### Possíveis Problemas
- **API não responde**: Verificar se o servidor está rodando
- **Erro 401**: Verificar autenticação
- **Erro 404**: Verificar URL e endpoints
- **Erro 500**: Verificar logs do servidor

### Tempo Sugerido por Seção
- Introdução: 3 min
- Conceitos: 5 min
- Configuração: 4 min
- Gerenciamento: 8 min
- Autenticação: 5 min
- **Total**: 25 min
