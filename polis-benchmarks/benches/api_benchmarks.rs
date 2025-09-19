use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use hyper::Body;
use hyper::Request;
use polis_api::{GrpcServer, RestServer};
use polis_auth::AuthManager;
use polis_core::{types::ContainerId, PolisConfig};
use polis_image::ImageManager;
use polis_runtime::PolisRuntime;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::RwLock;

fn rest_api_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let runtime = Arc::new(PolisRuntime::new(config.clone()));
        runtime.initialize().await.unwrap();

        let image_cache_dir = config.storage.root_dir.join("images");
        let image_manager = Arc::new(ImageManager::new(image_cache_dir));
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
        let auth_manager = Arc::new(RwLock::new(AuthManager::new(jwt_secret)));

        let rest_server = RestServer::new(runtime, image_manager, auth_manager);

        let mut group = c.benchmark_group("rest_api");

        // Benchmark container creation via REST
        group.bench_function("create_container_rest", |b| {
            b.to_async(&rt).iter(|| async {
                let container_id = ContainerId::new();
                let request_body = serde_json::json!({
                    "image": "alpine:latest",
                    "command": ["echo", "hello"],
                    "name": format!("test-{}", container_id)
                });

                let request = Request::builder()
                    .method("POST")
                    .uri("/containers")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap();

                // Simulate request handling
                request
            });
        });

        // Benchmark container listing via REST
        group.bench_function("list_containers_rest", |b| {
            b.to_async(&rt).iter(|| async {
                let request = Request::builder()
                    .method("GET")
                    .uri("/containers")
                    .body(Body::empty())
                    .unwrap();

                // Simulate request handling
                request
            });
        });

        // Benchmark image listing via REST
        group.bench_function("list_images_rest", |b| {
            b.to_async(&rt).iter(|| async {
                let request = Request::builder()
                    .method("GET")
                    .uri("/images")
                    .body(Body::empty())
                    .unwrap();

                // Simulate request handling
                request
            });
        });

        group.finish();
    });
}

fn grpc_api_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let runtime = Arc::new(PolisRuntime::new(config.clone()));
        runtime.initialize().await.unwrap();

        let image_cache_dir = config.storage.root_dir.join("images");
        let image_manager = Arc::new(ImageManager::new(image_cache_dir));
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
        let auth_manager = Arc::new(RwLock::new(AuthManager::new(jwt_secret)));

        let grpc_server = GrpcServer::new(runtime, image_manager, auth_manager);

        let mut group = c.benchmark_group("grpc_api");

        // Benchmark gRPC service creation
        group.bench_function("grpc_service_creation", |b| {
            b.to_async(&rt).iter(|| async { grpc_server.clone() });
        });

        group.finish();
    });
}

fn authentication_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
        let mut auth_manager = AuthManager::new(jwt_secret);

        let mut group = c.benchmark_group("authentication");

        // Benchmark user creation
        group.bench_function("create_user", |b| {
            b.to_async(&rt).iter(|| async {
                let username = format!("user-{}", uuid::Uuid::new_v4());
                let password = std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string());
                auth_manager
                    .user_manager
                    .create_user(username, password)
                    .await
            });
        });

        // Benchmark user authentication
        group.bench_function("authenticate_user", |b| {
            b.to_async(&rt).iter(|| async {
                let username = "testuser".to_string();
                let password = std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string());

                // Create user first
                let _ = auth_manager
                    .user_manager
                    .create_user(username.clone(), password.clone())
                    .await;

                // Then authenticate
                auth_manager.authenticate(&username, &password).await
            });
        });

        // Benchmark token generation
        group.bench_function("generate_token", |b| {
            b.to_async(&rt).iter(|| async {
                let username = "testuser".to_string();
                let password = std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string());

                // Create user first
                let _ = auth_manager
                    .user_manager
                    .create_user(username.clone(), password.clone())
                    .await;

                // Generate token
                auth_manager.authenticate(&username, &password).await
            });
        });

        // Benchmark token validation
        group.bench_function("validate_token", |b| {
            b.to_async(&rt).iter(|| async {
                let username = "testuser".to_string();
                let password = std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string());

                // Create user and get token
                let _ = auth_manager
                    .user_manager
                    .create_user(username.clone(), password.clone())
                    .await;
                let auth_result = auth_manager
                    .authenticate(&username, &password)
                    .await
                    .unwrap();

                // Validate token
                auth_manager.jwt_manager.validate_token(&auth_result.token)
            });
        });

        group.finish();
    });
}

fn concurrent_api_requests_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let runtime = Arc::new(PolisRuntime::new(config.clone()));
        runtime.initialize().await.unwrap();

        let image_cache_dir = config.storage.root_dir.join("images");
        let image_manager = Arc::new(ImageManager::new(image_cache_dir));
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
        let auth_manager = Arc::new(RwLock::new(AuthManager::new(jwt_secret)));

        let rest_server = RestServer::new(runtime, image_manager, auth_manager);

        let mut group = c.benchmark_group("concurrent_api_requests");

        group.bench_function("concurrent_rest_requests", |b| {
            b.to_async(&rt).iter(|| async {
                let mut handles = Vec::new();

                for i in 0..10 {
                    let handle = tokio::spawn(async move {
                        let container_id = ContainerId::new();
                        let request_body = serde_json::json!({
                            "image": "alpine:latest",
                            "command": ["echo", format!("hello-{}", i)],
                            "name": format!("test-{}", container_id)
                        });

                        let request = Request::builder()
                            .method("POST")
                            .uri("/containers")
                            .header("content-type", "application/json")
                            .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                            .unwrap();

                        request
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.await.unwrap();
                }
            });
        });

        group.finish();
    });
}

fn json_serialization_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    // Benchmark container serialization
    group.bench_function("serialize_container", |b| {
        b.iter(|| {
            let container = polis_core::types::Container {
                id: ContainerId::new(),
                name: "test-container".to_string(),
                image: "alpine:latest".to_string(),
                status: polis_core::types::ContainerStatus::Running,
                created_at: chrono::Utc::now(),
                command: vec!["echo".to_string(), "hello".to_string()],
                environment: std::collections::HashMap::new(),
                ports: vec![],
                volumes: vec![],
                labels: std::collections::HashMap::new(),
            };

            serde_json::to_string(&container)
        });
    });

    // Benchmark image serialization
    group.bench_function("serialize_image", |b| {
        b.iter(|| {
            let image = polis_core::types::Image {
                id: polis_core::types::ImageId::new(),
                name: "alpine:latest".to_string(),
                tag: "latest".to_string(),
                size: 1024,
                created_at: chrono::Utc::now(),
                labels: std::collections::HashMap::new(),
            };

            serde_json::to_string(&image)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    rest_api_benchmark,
    grpc_api_benchmark,
    authentication_benchmark,
    concurrent_api_requests_benchmark,
    json_serialization_benchmark
);
criterion_main!(benches);
