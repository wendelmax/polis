use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use polis_core::PolisConfig;
use polis_storage::{BackupManager, SnapshotManager, VolumeManager};
use tempfile::TempDir;
use tokio::runtime::Runtime;

fn volume_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let temp_dir = TempDir::new().unwrap();
        let volume_manager = VolumeManager::new(temp_dir.path().to_path_buf());

        let mut group = c.benchmark_group("volume_operations");

        // Benchmark volume creation
        group.bench_function("create_volume", |b| {
            b.to_async(&rt)
                .iter(|| async { volume_manager.create_volume("test-volume").await });
        });

        // Benchmark volume listing
        group.bench_function("list_volumes", |b| {
            b.to_async(&rt)
                .iter(|| async { volume_manager.list_volumes().await });
        });

        // Benchmark volume mounting
        group.bench_function("mount_volume", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = volume_manager.create_volume("test-volume").await;
                volume_manager
                    .mount_volume("test-volume", "/mnt/test")
                    .await
            });
        });

        // Benchmark volume unmounting
        group.bench_function("unmount_volume", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = volume_manager.create_volume("test-volume").await;
                let _ = volume_manager
                    .mount_volume("test-volume", "/mnt/test")
                    .await;
                volume_manager.unmount_volume("test-volume").await
            });
        });

        // Benchmark volume deletion
        group.bench_function("delete_volume", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = volume_manager.create_volume("test-volume").await;
                volume_manager.delete_volume("test-volume").await
            });
        });

        group.finish();
    });
}

fn snapshot_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let temp_dir = TempDir::new().unwrap();
        let snapshot_manager = SnapshotManager::new(temp_dir.path().to_path_buf());

        let mut group = c.benchmark_group("snapshot_operations");

        // Benchmark snapshot creation
        group.bench_function("create_snapshot", |b| {
            b.to_async(&rt).iter(|| async {
                snapshot_manager
                    .create_snapshot("test-volume", "snapshot-1")
                    .await
            });
        });

        // Benchmark snapshot listing
        group.bench_function("list_snapshots", |b| {
            b.to_async(&rt)
                .iter(|| async { snapshot_manager.list_snapshots("test-volume").await });
        });

        // Benchmark snapshot restoration
        group.bench_function("restore_snapshot", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = snapshot_manager
                    .create_snapshot("test-volume", "snapshot-1")
                    .await;
                snapshot_manager
                    .restore_snapshot("test-volume", "snapshot-1")
                    .await
            });
        });

        // Benchmark snapshot deletion
        group.bench_function("delete_snapshot", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = snapshot_manager
                    .create_snapshot("test-volume", "snapshot-1")
                    .await;
                snapshot_manager
                    .delete_snapshot("test-volume", "snapshot-1")
                    .await
            });
        });

        group.finish();
    });
}

fn backup_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let temp_dir = TempDir::new().unwrap();
        let backup_manager = BackupManager::new(temp_dir.path().to_path_buf());

        let mut group = c.benchmark_group("backup_operations");

        // Benchmark backup creation
        group.bench_function("create_backup", |b| {
            b.to_async(&rt).iter(|| async {
                backup_manager
                    .create_backup("test-volume", "backup-1")
                    .await
            });
        });

        // Benchmark backup listing
        group.bench_function("list_backups", |b| {
            b.to_async(&rt)
                .iter(|| async { backup_manager.list_backups("test-volume").await });
        });

        // Benchmark backup restoration
        group.bench_function("restore_backup", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = backup_manager
                    .create_backup("test-volume", "backup-1")
                    .await;
                backup_manager
                    .restore_backup("test-volume", "backup-1")
                    .await
            });
        });

        // Benchmark backup deletion
        group.bench_function("delete_backup", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = backup_manager
                    .create_backup("test-volume", "backup-1")
                    .await;
                backup_manager
                    .delete_backup("test-volume", "backup-1")
                    .await
            });
        });

        group.finish();
    });
}

fn concurrent_storage_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let temp_dir = TempDir::new().unwrap();
        let volume_manager = VolumeManager::new(temp_dir.path().to_path_buf());
        let snapshot_manager = SnapshotManager::new(temp_dir.path().to_path_buf());
        let backup_manager = BackupManager::new(temp_dir.path().to_path_buf());

        let mut group = c.benchmark_group("concurrent_storage_operations");

        group.bench_function("concurrent_storage_operations", |b| {
            b.to_async(&rt).iter(|| async {
                let mut handles = Vec::new();

                // Create volumes
                for i in 0..5 {
                    let volume_manager = volume_manager.clone();
                    let handle = tokio::spawn(async move {
                        volume_manager.create_volume(&format!("volume-{}", i)).await
                    });
                    handles.push(handle);
                }

                // Create snapshots
                for i in 0..5 {
                    let snapshot_manager = snapshot_manager.clone();
                    let handle = tokio::spawn(async move {
                        snapshot_manager
                            .create_snapshot(&format!("volume-{}", i), &format!("snapshot-{}", i))
                            .await
                    });
                    handles.push(handle);
                }

                // Create backups
                for i in 0..5 {
                    let backup_manager = backup_manager.clone();
                    let handle = tokio::spawn(async move {
                        backup_manager
                            .create_backup(&format!("volume-{}", i), &format!("backup-{}", i))
                            .await
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

fn storage_serialization_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_serialization");

    // Benchmark volume serialization
    group.bench_function("serialize_volume", |b| {
        b.iter(|| {
            let volume = polis_storage::Volume {
                name: "test-volume".to_string(),
                path: "/var/lib/polis/volumes/test-volume".to_string(),
                size: 1024 * 1024 * 1024, // 1GB
                created_at: chrono::Utc::now(),
                labels: std::collections::HashMap::new(),
            };

            serde_json::to_string(&volume)
        });
    });

    // Benchmark snapshot serialization
    group.bench_function("serialize_snapshot", |b| {
        b.iter(|| {
            let snapshot = polis_storage::Snapshot {
                name: "snapshot-1".to_string(),
                volume_name: "test-volume".to_string(),
                path: "/var/lib/polis/snapshots/test-volume/snapshot-1".to_string(),
                size: 512 * 1024 * 1024, // 512MB
                created_at: chrono::Utc::now(),
            };

            serde_json::to_string(&snapshot)
        });
    });

    // Benchmark backup serialization
    group.bench_function("serialize_backup", |b| {
        b.iter(|| {
            let backup = polis_storage::Backup {
                name: "backup-1".to_string(),
                volume_name: "test-volume".to_string(),
                path: "/var/lib/polis/backups/test-volume/backup-1".to_string(),
                size: 256 * 1024 * 1024, // 256MB
                created_at: chrono::Utc::now(),
                compression: Some("gzip".to_string()),
            };

            serde_json::to_string(&backup)
        });
    });

    group.finish();
}

fn file_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let temp_dir = TempDir::new().unwrap();
        let volume_manager = VolumeManager::new(temp_dir.path().to_path_buf());

        let mut group = c.benchmark_group("file_operations");

        // Benchmark file writing
        group.bench_function("write_file", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = volume_manager.create_volume("test-volume").await;
                let _ = volume_manager
                    .mount_volume("test-volume", "/mnt/test")
                    .await;

                let content = "Hello, World!".repeat(1000); // 13KB
                volume_manager
                    .write_file("test-volume", "test.txt", &content)
                    .await
            });
        });

        // Benchmark file reading
        group.bench_function("read_file", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = volume_manager.create_volume("test-volume").await;
                let _ = volume_manager
                    .mount_volume("test-volume", "/mnt/test")
                    .await;
                let content = "Hello, World!".repeat(1000);
                let _ = volume_manager
                    .write_file("test-volume", "test.txt", &content)
                    .await;

                volume_manager.read_file("test-volume", "test.txt").await
            });
        });

        // Benchmark file deletion
        group.bench_function("delete_file", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = volume_manager.create_volume("test-volume").await;
                let _ = volume_manager
                    .mount_volume("test-volume", "/mnt/test")
                    .await;
                let content = "Hello, World!".repeat(1000);
                let _ = volume_manager
                    .write_file("test-volume", "test.txt", &content)
                    .await;

                volume_manager.delete_file("test-volume", "test.txt").await
            });
        });

        group.finish();
    });
}

criterion_group!(
    benches,
    volume_operations_benchmark,
    snapshot_operations_benchmark,
    backup_operations_benchmark,
    concurrent_storage_operations_benchmark,
    storage_serialization_benchmark,
    file_operations_benchmark
);
criterion_main!(benches);
