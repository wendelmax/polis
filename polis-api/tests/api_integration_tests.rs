use polis_api::{
    ContainerServiceImpl, GrpcServer, ImageServiceImpl, RestServer, SystemServiceImpl,
};
use polis_auth::AuthManager;
use polis_core::PolisConfig;
use polis_image::ImageManager;
use polis_runtime::PolisRuntime;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_rest_server_creation() {
    let config = PolisConfig::default();
    let runtime = Arc::new(PolisRuntime::new(config.clone()));
    runtime.initialize().await.unwrap();

    let image_cache_dir = config.storage.root_dir.join("images");
    let image_manager = Arc::new(ImageManager::new(image_cache_dir));

    let auth_manager = Arc::new(RwLock::new(polis_auth::AuthManager::new(
        "test-secret".to_string(),
    )));
    let _rest_server = RestServer::new(runtime, image_manager, auth_manager);

    // Test that server was created successfully (without actually starting it)
    // In a real test, we would use a test server or mock
}

#[tokio::test]
async fn test_grpc_server_creation() {
    let config = PolisConfig::default();
    let runtime = Arc::new(PolisRuntime::new(config.clone()));
    runtime.initialize().await.unwrap();

    let image_cache_dir = config.storage.root_dir.join("images");
    let image_manager = Arc::new(ImageManager::new(image_cache_dir));

    let _grpc_server = GrpcServer::new(runtime, image_manager);

    // Test that server was created successfully (without actually starting it)
    // In a real test, we would use a test server or mock
}

#[tokio::test]
async fn test_container_service() {
    let config = PolisConfig::default();
    let runtime = Arc::new(PolisRuntime::new(config));
    runtime.initialize().await.unwrap();

    let container_service = ContainerServiceImpl::new(runtime.clone());

    // Test listing containers (should be empty initially)
    let containers = container_service.list_containers().await.unwrap();
    assert_eq!(containers.len(), 0);

    // Test creating a container
    let container_id = container_service
        .create_container(
            "test-api-container".to_string(),
            "test-image".to_string(),
            vec!["echo".to_string(), "hello".to_string()],
        )
        .await
        .unwrap();

    // Test listing containers after creation
    let containers = container_service.list_containers().await.unwrap();
    assert_eq!(containers.len(), 1);
    assert_eq!(containers[0].name, "test-api-container");

    // Test getting specific container
    let container = container_service
        .get_container(&container_id.0.to_string())
        .await
        .unwrap();
    assert_eq!(container.name, "test-api-container");
    assert_eq!(container.image.0, "test-image");

    // Test starting container
    assert!(container_service
        .start_container(&container_id.0.to_string())
        .await
        .is_ok());

    // Test stopping container
    assert!(container_service
        .stop_container(&container_id.0.to_string())
        .await
        .is_ok());

    // Test removing container
    assert!(container_service
        .remove_container(&container_id.0.to_string())
        .await
        .is_ok());

    // Verify container was removed
    let containers = container_service.list_containers().await.unwrap();
    assert_eq!(containers.len(), 0);
}

#[tokio::test]
async fn test_image_service() {
    let config = PolisConfig::default();
    let image_cache_dir = config.storage.root_dir.join("images");
    let image_manager = Arc::new(ImageManager::new(image_cache_dir));

    let image_service = ImageServiceImpl::new(image_manager);

    // Test listing images (should be empty initially)
    let images = image_service.list_images().await.unwrap();
    assert_eq!(images.len(), 0);

    // Test getting non-existent image
    let result = image_service.get_image("nonexistent-image").await;
    assert!(result.is_err());

    // Test removing non-existent image
    let result = image_service.remove_image("nonexistent-image").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_system_service() {
    let system_service = SystemServiceImpl::new();

    // Test getting system info
    let system_info = system_service.get_system_info().await.unwrap();
    assert_eq!(system_info.service, "polis");
    assert_eq!(system_info.version, "0.3.0");
    assert_eq!(system_info.runtime, "rust");
    assert_eq!(system_info.status, "running");

    // Test health check
    let health = system_service.health_check().await.unwrap();
    assert_eq!(health.status, "healthy");
    assert_eq!(health.service, "polis");
}

#[tokio::test]
async fn test_error_handling() {
    let config = PolisConfig::default();
    let runtime = Arc::new(PolisRuntime::new(config));
    runtime.initialize().await.unwrap();

    let container_service = ContainerServiceImpl::new(runtime);

    // Test getting non-existent container
    let result = container_service.get_container("nonexistent-id").await;
    assert!(result.is_err());

    // Test starting non-existent container
    let result = container_service.start_container("nonexistent-id").await;
    assert!(result.is_err());

    // Test stopping non-existent container
    let result = container_service.stop_container("nonexistent-id").await;
    assert!(result.is_err());

    // Test removing non-existent container
    let result = container_service.remove_container("nonexistent-id").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_api_operations() {
    let config = PolisConfig::default();
    let runtime = Arc::new(PolisRuntime::new(config));
    runtime.initialize().await.unwrap();

    let container_service = ContainerServiceImpl::new(runtime.clone());

    // Create multiple containers concurrently
    let mut handles = Vec::new();

    for i in 0..3 {
        let service_clone = ContainerServiceImpl::new(Arc::clone(&runtime));
        let handle = tokio::spawn(async move {
            service_clone
                .create_container(
                    format!("api-concurrent-{}", i),
                    "test-image".to_string(),
                    vec!["echo".to_string(), format!("hello-{}", i)],
                )
                .await
        });
        handles.push(handle);
    }

    // Wait for all containers to be created
    let mut container_ids = Vec::new();
    for handle in handles {
        let container_id = handle.await.unwrap().unwrap();
        container_ids.push(container_id);
    }

    // Verify all containers were created
    let containers = container_service.list_containers().await.unwrap();
    assert_eq!(containers.len(), 3);

    // Cleanup
    for container_id in container_ids {
        container_service
            .remove_container(&container_id.0.to_string())
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn test_container_pause_unpause_via_api() {
    let config = PolisConfig::default();
    let runtime = Arc::new(PolisRuntime::new(config));
    runtime.initialize().await.unwrap();

    let container_service = ContainerServiceImpl::new(runtime);

    // Create and start container
    let container_id = container_service
        .create_container(
            "test-pause-api".to_string(),
            "test-image".to_string(),
            vec!["echo".to_string(), "test".to_string()],
        )
        .await
        .unwrap();

    container_service
        .start_container(&container_id.0.to_string())
        .await
        .unwrap();

    // Test pause
    assert!(container_service
        .pause_container(&container_id.0.to_string())
        .await
        .is_ok());

    // Test unpause
    assert!(container_service
        .unpause_container(&container_id.0.to_string())
        .await
        .is_ok());

    // Cleanup
    container_service
        .stop_container(&container_id.0.to_string())
        .await
        .unwrap();
    container_service
        .remove_container(&container_id.0.to_string())
        .await
        .unwrap();
}
