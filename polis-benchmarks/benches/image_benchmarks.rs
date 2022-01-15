use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use polis_core::PolisConfig;
use polis_image::{ImageManager, RegistryClient};
use std::path::PathBuf;
use tokio::runtime::Runtime;

fn image_download_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let image_cache_dir = config.storage.root_dir.join("images");
        let image_manager = ImageManager::new(image_cache_dir);
        let registry_client = RegistryClient::new("https://registry-1.docker.io".to_string());

        let mut group = c.benchmark_group("image_download");

        let images = vec![
            "alpine:latest",
            "ubuntu:20.04",
            "nginx:alpine",
            "redis:alpine",
            "postgres:13-alpine",
        ];

        for image in images {
            group.bench_with_input(
                BenchmarkId::new("download_image", image),
                &image,
                |b, &image| {
                    b.to_async(&rt).iter(|| async {
                        let image_ref = image.parse().unwrap();
                        registry_client.pull_image(&image_ref, &image_manager).await
                    });
                },
            );
        }

        group.finish();
    });
}

fn image_listing_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let image_cache_dir = config.storage.root_dir.join("images");
        let image_manager = ImageManager::new(image_cache_dir);

        // Pre-populate with images
        let images = vec![
            "alpine:latest",
            "ubuntu:20.04",
            "nginx:alpine",
            "redis:alpine",
            "postgres:13-alpine",
        ];

        for image in images {
            let image_ref = image.parse().unwrap();
            let _ = image_manager.add_image(image_ref).await;
        }

        let mut group = c.benchmark_group("image_listing");

        group.bench_function("list_images", |b| {
            b.to_async(&rt)
                .iter(|| async { image_manager.list_images().await });
        });

        group.bench_function("get_image", |b| {
            b.to_async(&rt).iter(|| async {
                let image_ref = "alpine:latest".parse().unwrap();
                image_manager.get_image(&image_ref).await
            });
        });

        group.finish();
    });
}

fn image_parsing_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("image_parsing");

    // Sample OCI manifest JSON
    let manifest_json = r#"{
        "schemaVersion": 2,
        "mediaType": "application/vnd.oci.image.manifest.v1+json",
        "config": {
            "mediaType": "application/vnd.oci.image.config.v1+json",
            "size": 1234,
            "digest": "sha256:abcd1234"
        },
        "layers": [
            {
                "mediaType": "application/vnd.oci.image.layer.v1.tar+gzip",
                "size": 5678,
                "digest": "sha256:efgh5678"
            }
        ]
    }"#;

    group.bench_function("parse_oci_manifest", |b| {
        b.iter(|| serde_json::from_str::<polis_image::OciManifest>(manifest_json));
    });

    // Sample OCI config JSON
    let config_json = r#"{
        "architecture": "amd64",
        "os": "linux",
        "config": {
            "Env": ["PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"],
            "Cmd": ["/bin/sh"]
        },
        "rootfs": {
            "type": "layers",
            "diff_ids": ["sha256:abcd1234"]
        }
    }"#;

    group.bench_function("parse_oci_config", |b| {
        b.iter(|| serde_json::from_str::<polis_image::OciImageConfig>(config_json));
    });

    group.finish();
}

fn image_conversion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let image_cache_dir = config.storage.root_dir.join("images");
        let image_manager = ImageManager::new(image_cache_dir);

        let mut group = c.benchmark_group("image_conversion");

        group.bench_function("docker_to_oci", |b| {
            b.to_async(&rt).iter(|| async {
                let docker_image = polis_image::DockerImage {
                    id: "alpine:latest".to_string(),
                    created: "2023-01-01T00:00:00Z".to_string(),
                    size: 1024,
                    labels: std::collections::HashMap::new(),
                };
                image_manager.convert_docker_to_oci(&docker_image).await
            });
        });

        group.finish();
    });
}

fn concurrent_image_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let image_cache_dir = config.storage.root_dir.join("images");
        let image_manager = ImageManager::new(image_cache_dir);

        let mut group = c.benchmark_group("concurrent_image_operations");

        group.bench_function("concurrent_image_operations", |b| {
            b.to_async(&rt).iter(|| async {
                let mut handles = Vec::new();
                let images = vec![
                    "alpine:latest",
                    "ubuntu:20.04",
                    "nginx:alpine",
                    "redis:alpine",
                    "postgres:13-alpine",
                ];

                for image in images {
                    let image_manager = image_manager.clone();
                    let handle = tokio::spawn(async move {
                        let image_ref = image.parse().unwrap();
                        image_manager.add_image(image_ref).await
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.await.unwrap().unwrap();
                }
            });
        });

        group.finish();
    });
}

criterion_group!(
    benches,
    image_download_benchmark,
    image_listing_benchmark,
    image_parsing_benchmark,
    image_conversion_benchmark,
    concurrent_image_operations_benchmark
);
criterion_main!(benches);
