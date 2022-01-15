use polis_core::{ContainerId, ContainerStatus, PolisConfig, ResourceLimits};
use polis_runtime::{ContainerRuntime, PolisRuntime};
use std::sync::Arc;

#[tokio::test]
async fn test_runtime_initialization() {
    let config = PolisConfig::default();
    let runtime = PolisRuntime::new(config);

    // Test initialization
    assert!(runtime.initialize().await.is_ok());
}

#[tokio::test]
async fn test_container_lifecycle() {
    let config = PolisConfig::default();
    let runtime = PolisRuntime::new(config);
    runtime.initialize().await.unwrap();

    // Test container creation
    let container_id = runtime
        .create_container(
            "test-lifecycle".to_string(),
            "alpine:latest".to_string(),
            vec!["echo".to_string(), "hello".to_string()],
        )
        .await
        .unwrap();

    // Verify container was created
    let containers = runtime.list_containers().await.unwrap();
    assert_eq!(containers.len(), 1);
    assert_eq!(containers[0].id, container_id);
    assert_eq!(containers[0].name, "test-lifecycle");
    assert_eq!(containers[0].status, ContainerStatus::Created);

    // Test container start
    assert!(runtime.start_container(container_id.clone()).await.is_ok());

    // Verify container is running
    let container = runtime.get_container(container_id.clone()).await.unwrap();
    assert_eq!(container.status, ContainerStatus::Running);
    assert!(container.started_at.is_some());

    // Test container stop
    assert!(runtime.stop_container(container_id.clone()).await.is_ok());

    // Verify container is stopped
    let container = runtime.get_container(container_id.clone()).await.unwrap();
    assert_eq!(container.status, ContainerStatus::Stopped);
    assert!(container.finished_at.is_some());
    assert_eq!(container.exit_code, Some(0));

    // Test container removal
    assert!(runtime.remove_container(container_id).await.is_ok());

    // Verify container is removed
    let containers = runtime.list_containers().await.unwrap();
    assert_eq!(containers.len(), 0);
}

#[tokio::test]
async fn test_multiple_containers() {
    let config = PolisConfig::default();
    let runtime = PolisRuntime::new(config);
    runtime.initialize().await.unwrap();

    // Create multiple containers
    let container1 = runtime
        .create_container(
            "container1".to_string(),
            "alpine:latest".to_string(),
            vec!["echo".to_string(), "hello1".to_string()],
        )
        .await
        .unwrap();

    let container2 = runtime
        .create_container(
            "container2".to_string(),
            "ubuntu:20.04".to_string(),
            vec!["echo".to_string(), "hello2".to_string()],
        )
        .await
        .unwrap();

    // Verify both containers exist
    let containers = runtime.list_containers().await.unwrap();
    assert_eq!(containers.len(), 2);

    // Verify container details
    let container1_details = runtime.get_container(container1).await.unwrap();
    assert_eq!(container1_details.name, "container1");
    assert_eq!(container1_details.image.0, "alpine:latest");

    let container2_details = runtime.get_container(container2).await.unwrap();
    assert_eq!(container2_details.name, "container2");
    assert_eq!(container2_details.image.0, "ubuntu:20.04");
}

#[tokio::test]
async fn test_container_pause_unpause() {
    let config = PolisConfig::default();
    let runtime = PolisRuntime::new(config);
    runtime.initialize().await.unwrap();

    // Create and start container
    let container_id = runtime
        .create_container(
            "test-pause".to_string(),
            "alpine:latest".to_string(),
            vec!["sleep".to_string(), "10".to_string()],
        )
        .await
        .unwrap();

    runtime.start_container(container_id.clone()).await.unwrap();

    // Test pause
    assert!(runtime.pause_container(container_id.clone()).await.is_ok());

    let container = runtime.get_container(container_id.clone()).await.unwrap();
    assert_eq!(container.status, ContainerStatus::Paused);

    // Test unpause
    assert!(runtime
        .unpause_container(container_id.clone())
        .await
        .is_ok());

    let container = runtime.get_container(container_id.clone()).await.unwrap();
    assert_eq!(container.status, ContainerStatus::Running);

    // Cleanup
    runtime.stop_container(container_id).await.unwrap();
}

#[tokio::test]
async fn test_error_handling() {
    let config = PolisConfig::default();
    let runtime = PolisRuntime::new(config);
    runtime.initialize().await.unwrap();

    // Test getting non-existent container
    let fake_id = ContainerId::new();
    let result = runtime.get_container(fake_id.clone()).await;
    assert!(result.is_err());

    // Test starting non-existent container
    let result = runtime.start_container(fake_id.clone()).await;
    assert!(result.is_err());

    // Test stopping non-existent container
    let result = runtime.stop_container(fake_id.clone()).await;
    assert!(result.is_err());

    // Test removing non-existent container
    let result = runtime.remove_container(fake_id).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_container_with_environment() {
    let config = PolisConfig::default();
    let runtime = PolisRuntime::new(config);
    runtime.initialize().await.unwrap();

    // Create container with environment variables
    let container_id = runtime
        .create_container(
            "test-env".to_string(),
            "alpine:latest".to_string(),
            vec!["env".to_string()],
        )
        .await
        .unwrap();

    // Verify container was created
    let container = runtime.get_container(container_id.clone()).await.unwrap();
    assert_eq!(container.name, "test-env");
    assert_eq!(container.command, vec!["env"]);

    // Cleanup
    runtime.remove_container(container_id).await.unwrap();
}

#[tokio::test]
async fn test_container_with_resource_limits() {
    let config = PolisConfig::default();
    let runtime = PolisRuntime::new(config);
    runtime.initialize().await.unwrap();

    // Create container with resource limits
    let container_id = runtime
        .create_container(
            "test-limits".to_string(),
            "alpine:latest".to_string(),
            vec!["echo".to_string(), "test".to_string()],
        )
        .await
        .unwrap();

    // Verify container was created with default resource limits
    let container = runtime.get_container(container_id.clone()).await.unwrap();
    assert_eq!(container.name, "test-limits");
    assert_eq!(container.resource_limits, ResourceLimits::default());

    // Cleanup
    runtime.remove_container(container_id).await.unwrap();
}

#[tokio::test]
async fn test_concurrent_operations() {
    let config = PolisConfig::default();
    let runtime = PolisRuntime::new(config);
    runtime.initialize().await.unwrap();

    // Create multiple containers concurrently
    let runtime = Arc::new(runtime);
    let mut handles = Vec::new();

    for i in 0..5 {
        let runtime_clone = Arc::clone(&runtime);
        let handle = tokio::spawn(async move {
            runtime_clone
                .create_container(
                    format!("concurrent-{}", i),
                    "alpine:latest".to_string(),
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
    let containers = runtime.list_containers().await.unwrap();
    assert_eq!(containers.len(), 5);

    // Cleanup
    for container_id in container_ids {
        runtime.remove_container(container_id).await.unwrap();
    }
}
