use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use polis_core::{types::ContainerId, PolisConfig};
use polis_runtime::{ContainerRuntime, PolisRuntime};
use std::sync::Arc;
use tokio::runtime::Runtime;

fn container_creation_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let runtime = Arc::new(PolisRuntime::new(config));
        runtime.initialize().await.unwrap();

        let mut group = c.benchmark_group("container_creation");

        for size in [1, 10, 50, 100].iter() {
            group.bench_with_input(
                BenchmarkId::new("create_containers", size),
                size,
                |b, &size| {
                    b.iter(|| {
                        rt.block_on(async {
                            let mut containers = Vec::new();
                            for i in 0..size {
                                let container_id = runtime
                                    .create_container(
                                        format!("container-{}", i),
                                        "alpine:latest".to_string(),
                                        vec!["echo".to_string(), "hello".to_string()],
                                    )
                                    .await
                                    .unwrap();
                                containers.push(container_id);
                            }
                            containers
                        })
                    });
                },
            );
        }

        group.finish();
    });
}

fn container_lifecycle_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let runtime = Arc::new(PolisRuntime::new(config));
        runtime.initialize().await.unwrap();

        let mut group = c.benchmark_group("container_lifecycle");

        group.bench_function("create_start_stop_remove", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let container_id = runtime
                        .create_container(
                            "test-container".to_string(),
                            "alpine:latest".to_string(),
                            vec!["echo".to_string(), "hello".to_string()],
                        )
                        .await
                        .unwrap();

                    runtime.start_container(container_id.clone()).await.unwrap();
                    runtime.stop_container(container_id.clone()).await.unwrap();
                    runtime.remove_container(container_id).await.unwrap();
                })
            });
        });

        group.finish();
    });
}

fn container_listing_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let runtime = Arc::new(PolisRuntime::new(config));
        runtime.initialize().await.unwrap();

        // Pre-populate with containers
        let mut containers = Vec::new();
        for i in 0..100 {
            let container_id = runtime
                .create_container(
                    format!("container-{}", i),
                    "alpine:latest".to_string(),
                    vec!["echo".to_string(), format!("hello-{}", i)],
                )
                .await
                .unwrap();
            containers.push(container_id);
        }

        let mut group = c.benchmark_group("container_listing");

        group.bench_function("list_containers", |b| {
            b.iter(|| rt.block_on(async { runtime.list_containers().await }));
        });

        group.bench_function("get_container", |b| {
            b.iter(|| {
                rt.block_on(async {
                    if let Some(container_id) = containers.first() {
                        runtime.get_container(container_id.clone()).await
                    } else {
                        Ok(polis_core::types::Container {
                            id: ContainerId::new(),
                            name: "test".to_string(),
                            image: polis_core::types::ImageId::new("alpine", "latest"),
                            status: polis_core::types::ContainerStatus::Created,
                            created_at: chrono::Utc::now(),
                            started_at: None,
                            finished_at: None,
                            exit_code: None,
                            command: vec!["echo".to_string(), "hello".to_string()],
                            working_dir: std::path::PathBuf::from("/"),
                            environment: std::collections::HashMap::new(),
                            labels: std::collections::HashMap::new(),
                            resource_limits: polis_core::types::ResourceLimits::default(),
                            network_mode: polis_core::types::NetworkMode::Bridge,
                            ports: vec![],
                            volumes: vec![],
                        })
                    }
                })
            });
        });

        group.finish();
    });
}

fn concurrent_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let runtime = Arc::new(PolisRuntime::new(config));
        runtime.initialize().await.unwrap();

        let mut group = c.benchmark_group("concurrent_operations");

        group.bench_function("concurrent_container_creation", |b| {
            b.iter(|| {
                rt.block_on(async {
                    let mut handles = Vec::new();
                    for i in 0..10 {
                        let runtime = Arc::clone(&runtime);
                        let handle = tokio::spawn(async move {
                            runtime
                                .create_container(
                                    format!("container-{}", i),
                                    "alpine:latest".to_string(),
                                    vec!["echo".to_string(), format!("hello-{}", i)],
                                )
                                .await
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.await.unwrap().unwrap();
                    }
                })
            });
        });

        group.finish();
    });
}

criterion_group!(
    benches,
    container_creation_benchmark,
    container_lifecycle_benchmark,
    container_listing_benchmark,
    concurrent_operations_benchmark
);
criterion_main!(benches);
