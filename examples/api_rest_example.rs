use reqwest;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio;

const BASE_URL: &str = "http://localhost:8080/api/v1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Exemplo de Uso da API REST do Polis");
    println!("=====================================");

    let client = reqwest::Client::new();

    // 1. Verificar saúde do sistema
    println!("\n1. Verificando saúde do sistema...");
    let health_response = client
        .get(&format!("{}/health", BASE_URL))
        .send()
        .await?;
    
    if health_response.status().is_success() {
        let health: Value = health_response.json().await?;
        println!("✅ Sistema saudável: {:?}", health["status"]);
    } else {
        println!("❌ Sistema não está saudável");
        return Ok(());
    }

    // 2. Obter informações do sistema
    println!("\n2. Obtendo informações do sistema...");
    let info_response = client
        .get(&format!("{}/system/info", BASE_URL))
        .send()
        .await?;
    
    let system_info: Value = info_response.json().await?;
    println!("📊 Informações do sistema:");
    println!("   Versão: {}", system_info["version"]);
    println!("   OS: {}", system_info["os"]);
    println!("   Containers rodando: {}", system_info["containers_running"]);

    // 3. Listar imagens disponíveis
    println!("\n3. Listando imagens disponíveis...");
    let images_response = client
        .get(&format!("{}/images", BASE_URL))
        .send()
        .await?;
    
    let images: Value = images_response.json().await?;
    println!("📦 Imagens disponíveis: {}", images["images"].as_array().unwrap().len());

    // 4. Baixar uma imagem
    println!("\n4. Baixando imagem alpine:latest...");
    let pull_request = json!({
        "name": "alpine:latest",
        "registry": "docker.io"
    });

    let pull_response = client
        .post(&format!("{}/images/pull", BASE_URL))
        .json(&pull_request)
        .send()
        .await?;
    
    if pull_response.status().is_success() {
        let pull_result: Value = pull_response.json().await?;
        println!("✅ Imagem baixada: {}", pull_result["image_id"]);
    } else {
        println!("❌ Falha ao baixar imagem");
    }

    // 5. Criar um container
    println!("\n5. Criando container...");
    let container_request = json!({
        "name": "exemplo-api",
        "image": "alpine:latest",
        "command": ["echo", "Hello from Polis API!"],
        "ports": [
            {
                "host_port": 8080,
                "container_port": 80,
                "protocol": "Tcp"
            }
        ],
        "environment": {
            "NODE_ENV": "production",
            "API_VERSION": "v1"
        },
        "resource_limits": {
            "memory_limit": 1073741824,
            "cpu_quota": 0.5
        }
    });

    let create_response = client
        .post(&format!("{}/containers", BASE_URL))
        .json(&container_request)
        .send()
        .await?;
    
    let container: Value = create_response.json().await?;
    let container_id = container["id"].as_str().unwrap();
    println!("✅ Container criado: {}", container_id);

    // 6. Iniciar o container
    println!("\n6. Iniciando container...");
    let start_response = client
        .post(&format!("{}/containers/{}/start", BASE_URL, container_id))
        .send()
        .await?;
    
    if start_response.status().is_success() {
        let start_result: Value = start_response.json().await?;
        println!("✅ Container iniciado: {}", start_result["message"]);
    } else {
        println!("❌ Falha ao iniciar container");
    }

    // 7. Aguardar um pouco
    println!("\n7. Aguardando execução...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // 8. Verificar status do container
    println!("\n8. Verificando status do container...");
    let status_response = client
        .get(&format!("{}/containers/{}", BASE_URL, container_id))
        .send()
        .await?;
    
    let container_status: Value = status_response.json().await?;
    println!("📊 Status do container: {}", container_status["status"]);

    // 9. Obter logs do container
    println!("\n9. Obtendo logs do container...");
    let logs_response = client
        .get(&format!("{}/containers/{}/logs", BASE_URL, container_id))
        .query(&[("tail", "10")])
        .send()
        .await?;
    
    if logs_response.status().is_success() {
        let logs: Value = logs_response.json().await?;
        println!("📝 Logs do container:");
        for log_entry in logs["logs"].as_array().unwrap() {
            println!("   [{}] {}: {}", 
                log_entry["timestamp"], 
                log_entry["level"], 
                log_entry["message"]
            );
        }
    }

    // 10. Obter métricas do sistema
    println!("\n10. Obtendo métricas do sistema...");
    let metrics_response = client
        .get(&format!("{}/metrics/system", BASE_URL))
        .send()
        .await?;
    
    let metrics: Value = metrics_response.json().await?;
    println!("📊 Métricas do sistema:");
    println!("   CPU: {:.2}%", metrics["cpu_usage"]);
    println!("   Memória: {:.2}%", metrics["memory_usage"]);
    println!("   Disco: {:.2}%", metrics["disk_usage"]);
    println!("   Containers rodando: {}", metrics["containers_running"]);

    // 11. Obter métricas do container
    println!("\n11. Obtendo métricas do container...");
    let container_metrics_response = client
        .get(&format!("{}/metrics/containers/{}", BASE_URL, container_id))
        .send()
        .await?;
    
    if container_metrics_response.status().is_success() {
        let container_metrics: Value = container_metrics_response.json().await?;
        println!("📊 Métricas do container:");
        println!("   CPU: {:.2}%", container_metrics["cpu_usage"]);
        println!("   Memória: {} bytes", container_metrics["memory_usage"]);
        println!("   Limite de memória: {} bytes", container_metrics["memory_limit"]);
    }

    // 12. Listar containers
    println!("\n12. Listando todos os containers...");
    let containers_response = client
        .get(&format!("{}/containers", BASE_URL))
        .send()
        .await?;
    
    let containers: Value = containers_response.json().await?;
    println!("📦 Containers disponíveis:");
    for container in containers["containers"].as_array().unwrap() {
        println!("   - {} ({}) - {}", 
            container["name"], 
            container["id"], 
            container["status"]
        );
    }

    // 13. Parar o container
    println!("\n13. Parando container...");
    let stop_response = client
        .post(&format!("{}/containers/{}/stop", BASE_URL, container_id))
        .send()
        .await?;
    
    if stop_response.status().is_success() {
        let stop_result: Value = stop_response.json().await?;
        println!("✅ Container parado: {}", stop_result["message"]);
    } else {
        println!("❌ Falha ao parar container");
    }

    // 14. Remover o container
    println!("\n14. Removendo container...");
    let remove_response = client
        .delete(&format!("{}/containers/{}", BASE_URL, container_id))
        .send()
        .await?;
    
    if remove_response.status().is_success() {
        let remove_result: Value = remove_response.json().await?;
        println!("✅ Container removido: {}", remove_result["message"]);
    } else {
        println!("❌ Falha ao remover container");
    }

    // 15. Obter estatísticas finais
    println!("\n15. Estatísticas finais do sistema...");
    let stats_response = client
        .get(&format!("{}/system/stats", BASE_URL))
        .send()
        .await?;
    
    let stats: Value = stats_response.json().await?;
    println!("📊 Estatísticas finais:");
    println!("   Containers rodando: {}", stats["containers"]["running"]);
    println!("   Containers parados: {}", stats["containers"]["stopped"]);
    println!("   Total de containers: {}", stats["containers"]["total"]);
    println!("   Total de imagens: {}", stats["images"]["total"]);
    println!("   Uso de armazenamento: {} bytes", stats["storage"]["used"]);

    println!("\n🎉 Exemplo da API REST concluído com sucesso!");
    Ok(())
}

