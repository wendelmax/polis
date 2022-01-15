use tonic::{transport::Channel, Request};
use polis_api::grpc::{
    container_service_client::ContainerServiceClient,
    image_service_client::ImageServiceClient,
    system_service_client::SystemServiceClient,
    CreateContainerRequest, StartContainerRequest, StopContainerRequest,
    RemoveContainerRequest, ListContainersRequest, GetContainerRequest,
    StreamLogsRequest, PullImageRequest, ListImagesRequest,
    GetSystemInfoRequest, GetSystemStatsRequest, StreamMetricsRequest,
    HealthCheckRequest, PortMapping, Protocol, ResourceLimits,
    ContainerStatus, HealthStatus
};
use std::collections::HashMap;
use tokio;

const GRPC_ENDPOINT: &str = "http://localhost:9090";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Exemplo de Uso da API gRPC do Polis");
    println!("=====================================");

    // Conectar aos serviços gRPC
    let channel = Channel::from_static(GRPC_ENDPOINT).connect().await?;
    let mut container_client = ContainerServiceClient::new(channel.clone());
    let mut image_client = ImageServiceClient::new(channel.clone());
    let mut system_client = SystemServiceClient::new(channel.clone());

    // 1. Verificar saúde do sistema
    println!("\n1. Verificando saúde do sistema...");
    let health_request = Request::new(HealthCheckRequest {});
    let health_response = system_client.health_check(health_request).await?;
    let health = health_response.into_inner();
    
    match health.status {
        HealthStatus::Healthy => println!("✅ Sistema saudável"),
        HealthStatus::Unhealthy => println!("❌ Sistema não está saudável"),
        HealthStatus::Unknown => println!("⚠️ Status de saúde desconhecido"),
    }

    // 2. Obter informações do sistema
    println!("\n2. Obtendo informações do sistema...");
    let info_request = Request::new(GetSystemInfoRequest {});
    let info_response = system_client.get_system_info(info_request).await?;
    let system_info = info_response.into_inner();
    
    println!("📊 Informações do sistema:");
    println!("   Versão: {}", system_info.version);
    println!("   OS: {}", system_info.os);
    println!("   Arquitetura: {}", system_info.arch);
    println!("   Kernel: {}", system_info.kernel);
    println!("   Containers rodando: {}", system_info.containers_running);
    println!("   Total de containers: {}", system_info.containers_total);

    // 3. Listar imagens disponíveis
    println!("\n3. Listando imagens disponíveis...");
    let list_images_request = Request::new(ListImagesRequest {
        name: None,
        tag: None,
    });
    let images_response = image_client.list_images(list_images_request).await?;
    let images = images_response.into_inner();
    
    println!("📦 Imagens disponíveis: {}", images.images.len());
    for image in &images.images {
        println!("   - {}:{} ({} bytes)", image.name, image.tag, image.size);
    }

    // 4. Baixar uma imagem
    println!("\n4. Baixando imagem alpine:latest...");
    let pull_request = Request::new(PullImageRequest {
        name: "alpine".to_string(),
        tag: "latest".to_string(),
        registry: "docker.io".to_string(),
        insecure: false,
    });

    let mut pull_stream = image_client.pull_image(pull_request).await?.into_inner();
    while let Some(response) = pull_stream.message().await? {
        println!("   Status: {} - {}", response.status, response.progress);
        if !response.image_id.is_empty() {
            println!("✅ Imagem baixada: {}", response.image_id);
            break;
        }
    }

    // 5. Criar um container
    println!("\n5. Criando container...");
    let create_request = Request::new(CreateContainerRequest {
        name: "exemplo-grpc".to_string(),
        image: "alpine:latest".to_string(),
        command: vec!["echo".to_string(), "Hello from Polis gRPC!".to_string()],
        ports: vec![PortMapping {
            host_port: 8080,
            container_port: 80,
            protocol: Protocol::Tcp as i32,
            host_ip: "0.0.0.0".to_string(),
        }],
        environment: {
            let mut env = HashMap::new();
            env.insert("NODE_ENV".to_string(), "production".to_string());
            env.insert("GRPC_VERSION".to_string(), "v1".to_string());
            env
        },
        resource_limits: Some(ResourceLimits {
            memory_limit: Some(1073741824), // 1GB
            memory_swap: Some(2147483648),  // 2GB
            cpu_quota: Some(0.5),           // 50% CPU
            cpu_period: Some(100000),       // 100ms
            disk_quota: Some(10737418240),  // 10GB
            pids_limit: Some(100),          // 100 processos
        }),
    });

    let create_response = container_client.create_container(create_request).await?;
    let container = create_response.into_inner();
    let container_id = container.container_id.clone();
    println!("✅ Container criado: {}", container_id);

    // 6. Iniciar o container
    println!("\n6. Iniciando container...");
    let start_request = Request::new(StartContainerRequest {
        container_id: container_id.clone(),
    });
    let start_response = container_client.start_container(start_request).await?;
    let start_result = start_response.into_inner();
    
    if start_result.success {
        println!("✅ Container iniciado: {}", start_result.message);
    } else {
        println!("❌ Falha ao iniciar container: {}", start_result.message);
    }

    // 7. Aguardar um pouco
    println!("\n7. Aguardando execução...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // 8. Verificar status do container
    println!("\n8. Verificando status do container...");
    let get_request = Request::new(GetContainerRequest {
        container_id: container_id.clone(),
    });
    let get_response = container_client.get_container(get_request).await?;
    let container_info = get_response.into_inner().container.unwrap();
    
    let status = match container_info.status {
        0 => "CREATED",
        1 => "RUNNING",
        2 => "STOPPED",
        3 => "PAUSED",
        4 => "REMOVED",
        _ => "UNKNOWN",
    };
    println!("📊 Status do container: {}", status);

    // 9. Stream de logs do container
    println!("\n9. Obtendo logs do container...");
    let logs_request = Request::new(StreamLogsRequest {
        container_id: container_id.clone(),
        follow: false,
        tail: 10,
        since: None,
    });

    let mut logs_stream = container_client.stream_logs(logs_request).await?.into_inner();
    println!("📝 Logs do container:");
    while let Some(log_entry) = logs_stream.message().await? {
        println!("   [{}] {}: {}", 
            log_entry.timestamp, 
            log_entry.level, 
            log_entry.message
        );
    }

    // 10. Obter métricas do sistema
    println!("\n10. Obtendo métricas do sistema...");
    let metrics_request = Request::new(GetSystemStatsRequest {});
    let metrics_response = system_client.get_system_stats(metrics_request).await?;
    let metrics = metrics_response.into_inner().stats.unwrap();
    
    println!("📊 Métricas do sistema:");
    println!("   CPU: {:.2}%", metrics.cpu_usage);
    println!("   Memória: {:.2}%", metrics.memory_usage);
    println!("   Disco: {:.2}%", metrics.disk_usage);
    println!("   Containers rodando: {}", metrics.containers_running);
    println!("   Memória total: {} bytes", metrics.memory_total);
    println!("   Memória disponível: {} bytes", metrics.memory_available);

    // 11. Stream de métricas em tempo real
    println!("\n11. Iniciando stream de métricas (5 segundos)...");
    let stream_request = Request::new(StreamMetricsRequest {
        interval: 1, // 1 segundo
        container_ids: vec![container_id.clone()],
    });

    let mut metrics_stream = system_client.stream_metrics(stream_request).await?.into_inner();
    let mut count = 0;
    while let Some(metrics_data) = metrics_stream.message().await? {
        count += 1;
        println!("   [{}] CPU: {:.2}%, Memória: {:.2}%", 
            metrics_data.timestamp,
            metrics_data.system.as_ref().unwrap().cpu_usage,
            metrics_data.system.as_ref().unwrap().memory_usage
        );
        
        for container_metrics in &metrics_data.containers {
            println!("      Container {}: CPU {:.2}%, Memória {} bytes", 
                container_metrics.container_id,
                container_metrics.cpu_usage,
                container_metrics.memory_usage
            );
        }
        
        if count >= 5 {
            break;
        }
    }

    // 12. Listar containers
    println!("\n12. Listando todos os containers...");
    let list_request = Request::new(ListContainersRequest {
        status: None,
        image: None,
    });
    let containers_response = container_client.list_containers(list_request).await?;
    let containers = containers_response.into_inner();
    
    println!("📦 Containers disponíveis:");
    for container in &containers.containers {
        let status = match container.status {
            0 => "CREATED",
            1 => "RUNNING",
            2 => "STOPPED",
            3 => "PAUSED",
            4 => "REMOVED",
            _ => "UNKNOWN",
        };
        println!("   - {} ({}) - {}", 
            container.name, 
            container.id, 
            status
        );
    }

    // 13. Parar o container
    println!("\n13. Parando container...");
    let stop_request = Request::new(StopContainerRequest {
        container_id: container_id.clone(),
        timeout: 30,
    });
    let stop_response = container_client.stop_container(stop_request).await?;
    let stop_result = stop_response.into_inner();
    
    if stop_result.success {
        println!("✅ Container parado: {}", stop_result.message);
    } else {
        println!("❌ Falha ao parar container: {}", stop_result.message);
    }

    // 14. Remover o container
    println!("\n14. Removendo container...");
    let remove_request = Request::new(RemoveContainerRequest {
        container_id: container_id.clone(),
        force: false,
    });
    let remove_response = container_client.remove_container(remove_request).await?;
    let remove_result = remove_response.into_inner();
    
    if remove_result.success {
        println!("✅ Container removido: {}", remove_result.message);
    } else {
        println!("❌ Falha ao remover container: {}", remove_result.message);
    }

    // 15. Verificação final de saúde
    println!("\n15. Verificação final de saúde...");
    let final_health_request = Request::new(HealthCheckRequest {});
    let final_health_response = system_client.health_check(final_health_request).await?;
    let final_health = final_health_response.into_inner();
    
    match final_health.status {
        HealthStatus::Healthy => println!("✅ Sistema ainda saudável"),
        HealthStatus::Unhealthy => println!("❌ Sistema não está saudável"),
        HealthStatus::Unknown => println!("⚠️ Status de saúde desconhecido"),
    }

    println!("\n🎉 Exemplo da API gRPC concluído com sucesso!");
    Ok(())
}

