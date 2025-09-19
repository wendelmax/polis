use polis_api::{
    ContainerServiceImpl, GrpcServer, ImageServiceImpl, RestServer, SystemServiceImpl,
};
use polis_auth::AuthManager;
use polis_core::PolisConfig;
use polis_image::ImageManager;
use polis_runtime::{ContainerRuntime, PolisRuntime};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("� Exemplo das APIs REST/gRPC do Polis");
    println!("=====================================\n");

    // 1. Inicializar componentes
    println!("1.  Inicializando componentes...");
    let config = PolisConfig::default();
    let runtime = Arc::new(PolisRuntime::new(config.clone()));
    runtime.initialize().await?;
    println!("    Runtime inicializado");

    let image_cache_dir = config.storage.root_dir.join("images");
    let image_manager = Arc::new(ImageManager::new(image_cache_dir));
    println!("    Image Manager inicializado");

    // 2. Testar operações de containers via runtime
    println!("\n2. � Testando operações de containers...");

    // Criar container
    let container_id = runtime
        .create_container(
            "test-container".to_string(),
            "alpine:latest".to_string(),
            vec![
                "cmd".to_string(),
                "/c".to_string(),
                "echo".to_string(),
                "Hello".to_string(),
            ],
        )
        .await?;
    println!("    Container criado: {}", container_id.0);

    // Listar containers
    let containers = runtime.list_containers().await?;
    println!("   � Containers encontrados: {}", containers.len());

    // Iniciar container
    runtime.start_container(container_id.clone()).await?;
    println!("    Container iniciado");

    // Parar container
    runtime.stop_container(container_id.clone()).await?;
    println!("    Container parado");

    // 3. Testar operações de imagens
    println!("\n3. �  Testando operações de imagens...");

    // Listar imagens
    let images = image_manager.list_images().await?;
    println!("   � Imagens encontradas: {}", images.len());

    // 4. Inicializar servidores de API
    println!("\n4.  Inicializando servidores de API...");

    // Servidor REST
    let auth_manager = Arc::new(RwLock::new(polis_auth::AuthManager::new(
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "example-secret".to_string()),
    )));
    let _rest_server = RestServer::new(
        Arc::clone(&runtime),
        Arc::clone(&image_manager),
        auth_manager,
    );
    println!("    Servidor REST configurado");

    // Servidor gRPC
    let _grpc_server = GrpcServer::new(Arc::clone(&runtime), Arc::clone(&image_manager));
    println!("    Servidor gRPC configurado");

    // 5. Testar serviços gRPC
    println!("\n5.  Testando serviços gRPC...");

    // Container Service
    let container_service = ContainerServiceImpl::new(Arc::clone(&runtime));
    let containers = container_service.list_containers().await?;
    println!("   � Container Service - Containers: {}", containers.len());

    // Image Service
    let image_service = ImageServiceImpl::new(Arc::clone(&image_manager));
    let images = image_service.list_images().await?;
    println!("   � Image Service - Imagens: {}", images.len());

    // System Service
    let system_service = SystemServiceImpl::new();
    let system_info = system_service.get_system_info().await?;
    println!("   ℹ  System Service - Versão: {}", system_info.version);
    println!(
        "   ℹ  System Service - Arquitetura: {}",
        system_info.architecture
    );

    let health = system_service.health_check().await?;
    println!("   ❤  Health Check - Status: {}", health.status);

    // 6. Demonstrar endpoints REST
    println!("\n6. � Endpoints REST disponíveis:");
    println!("   GET  /health              - Health check");
    println!("   GET  /containers          - Listar containers");
    println!("   POST /containers          - Criar container");
    println!("   GET  /containers/{{id}}     - Obter container");
    println!("   POST /containers/{{id}}/start - Iniciar container");
    println!("   POST /containers/{{id}}/stop  - Parar container");
    println!("   GET  /images              - Listar imagens");
    println!("   POST /images/pull         - Baixar imagem");
    println!("   GET  /system/info         - Informações do sistema");

    // 7. Demonstrar serviços gRPC
    println!("\n7.  Serviços gRPC disponíveis:");
    println!("   ContainerService:");
    println!("     - ListContainers()");
    println!("     - GetContainer()");
    println!("     - CreateContainer()");
    println!("     - StartContainer()");
    println!("     - StopContainer()");
    println!("     - RemoveContainer()");
    println!("     - PauseContainer()");
    println!("     - UnpauseContainer()");

    println!("   ImageService:");
    println!("     - ListImages()");
    println!("     - GetImage()");
    println!("     - PullImage()");
    println!("     - RemoveImage()");

    println!("   SystemService:");
    println!("     - GetSystemInfo()");
    println!("     - HealthCheck()");

    // 8. Limpeza
    println!("\n8. � Limpando recursos...");
    runtime.remove_container(container_id).await?;
    println!("    Container removido");

    println!("\n Exemplo das APIs concluído com sucesso!");
    println!("\n� APIs Implementadas:");
    println!("   - API REST completa com endpoints para containers, imagens e sistema");
    println!("   - API gRPC com serviços especializados");
    println!("   - Integração total com runtime e image manager");
    println!("   - Tratamento de erros robusto");
    println!("   - Suporte a JSON e Protocol Buffers");

    Ok(())
}
