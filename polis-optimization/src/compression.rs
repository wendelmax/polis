use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// Compression algorithms supported
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    None,
    Gzip,
    Lz4,
    Zstd,
    Brotli,
}

impl CompressionAlgorithm {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(Self::None),
            "gzip" => Some(Self::Gzip),
            "lz4" => Some(Self::Lz4),
            "zstd" => Some(Self::Zstd),
            "brotli" => Some(Self::Brotli),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Gzip => "gzip",
            Self::Lz4 => "lz4",
            Self::Zstd => "zstd",
            Self::Brotli => "brotli",
        }
    }
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    pub algorithm: CompressionAlgorithm,
    pub level: u32,
    pub threshold: usize,
    pub enabled: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithm: CompressionAlgorithm::Zstd,
            level: 3,
            threshold: 1024, // Only compress data larger than 1KB
            enabled: true,
        }
    }
}

/// Compression utility
pub struct Compressor {
    config: CompressionConfig,
}

impl Compressor {
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>> {
        if !self.config.enabled || data.len() < self.config.threshold {
            return Ok(data.to_vec());
        }

        match self.config.algorithm {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Gzip => self.compress_gzip(data),
            CompressionAlgorithm::Lz4 => self.compress_lz4(data),
            CompressionAlgorithm::Zstd => self.compress_zstd(data),
            CompressionAlgorithm::Brotli => self.compress_brotli(data),
        }
    }

    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>> {
        match self.config.algorithm {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Gzip => self.decompress_gzip(data),
            CompressionAlgorithm::Lz4 => self.decompress_lz4(data),
            CompressionAlgorithm::Zstd => self.decompress_zstd(data),
            CompressionAlgorithm::Brotli => self.decompress_brotli(data),
        }
    }

    fn compress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.config.level as u32));
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress_gzip(&self, data: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    fn compress_lz4(&self, data: &[u8]) -> Result<Vec<u8>> {
        lz4::block::compress(data, Some(self.config.level as i32))
            .map_err(|e| anyhow::anyhow!("LZ4 compression failed: {}", e))
    }

    fn decompress_lz4(&self, data: &[u8]) -> Result<Vec<u8>> {
        lz4::block::decompress(data, None)
            .map_err(|e| anyhow::anyhow!("LZ4 decompression failed: {}", e))
    }

    fn compress_zstd(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::encode_all(data, self.config.level as i32)
            .map_err(|e| anyhow::anyhow!("Zstd compression failed: {}", e))
    }

    fn decompress_zstd(&self, data: &[u8]) -> Result<Vec<u8>> {
        zstd::decode_all(data).map_err(|e| anyhow::anyhow!("Zstd decompression failed: {}", e))
    }

    fn compress_brotli(&self, data: &[u8]) -> Result<Vec<u8>> {
        use brotli::enc::BrotliEncoderParams;

        let params = BrotliEncoderParams {
            quality: self.config.level as i32,
            ..Default::default()
        };

        let mut compressed = Vec::new();
        brotli::enc::BrotliCompress(&mut data.as_ref(), &mut compressed, &params)
            .map_err(|e| anyhow::anyhow!("Brotli compression failed: {}", e))?;
        Ok(compressed)
    }

    fn decompress_brotli(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut decompressed = Vec::new();
        brotli::Decompressor::new(data, 4096)
            .read_to_end(&mut decompressed)
            .map_err(|e| anyhow::anyhow!("Brotli decompression failed: {}", e))?;
        Ok(decompressed)
    }

    pub fn get_compression_ratio(&self, original: &[u8], compressed: &[u8]) -> f64 {
        if original.is_empty() {
            0.0
        } else {
            compressed.len() as f64 / original.len() as f64
        }
    }

    pub fn should_compress(&self, data: &[u8]) -> bool {
        self.config.enabled && data.len() >= self.config.threshold
    }
}

/// Compressed data wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedData {
    pub algorithm: String,
    pub level: u32,
    pub original_size: usize,
    pub compressed_size: usize,
    pub data: Vec<u8>,
}

impl CompressedData {
    pub fn new(
        algorithm: CompressionAlgorithm,
        level: u32,
        original_size: usize,
        data: Vec<u8>,
    ) -> Self {
        Self {
            algorithm: algorithm.as_str().to_string(),
            level,
            original_size,
            compressed_size: data.len(),
            data,
        }
    }

    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            self.compressed_size as f64 / self.original_size as f64
        }
    }

    pub fn space_saved(&self) -> usize {
        self.original_size.saturating_sub(self.compressed_size)
    }

    pub fn space_saved_percentage(&self) -> f64 {
        if self.original_size == 0 {
            0.0
        } else {
            (self.space_saved() as f64 / self.original_size as f64) * 100.0
        }
    }
}

/// Compression manager for different data types
pub struct CompressionManager {
    config: CompressionConfig,
    compressor: Compressor,
    stats: CompressionStats,
}

#[derive(Debug, Clone, Default)]
pub struct CompressionStats {
    pub total_compressed: usize,
    pub total_original: usize,
    pub compression_operations: u64,
    pub decompression_operations: u64,
    pub total_time: std::time::Duration,
}

impl CompressionManager {
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            compressor: Compressor::new(config.clone()),
            config,
            stats: CompressionStats::default(),
        }
    }

    pub fn compress_data(&mut self, data: &[u8]) -> Result<CompressedData> {
        let start = std::time::Instant::now();

        let compressed = self.compressor.compress(data)?;
        let compressed_data = CompressedData::new(
            self.config.algorithm,
            self.config.level,
            data.len(),
            compressed,
        );

        let duration = start.elapsed();
        self.stats.compression_operations += 1;
        self.stats.total_compressed += compressed_data.compressed_size;
        self.stats.total_original += data.len();
        self.stats.total_time += duration;

        Ok(compressed_data)
    }

    pub fn decompress_data(&mut self, compressed_data: &CompressedData) -> Result<Vec<u8>> {
        let start = std::time::Instant::now();

        let decompressed = self.compressor.decompress(&compressed_data.data)?;

        let duration = start.elapsed();
        self.stats.decompression_operations += 1;
        self.stats.total_time += duration;

        Ok(decompressed)
    }

    pub fn compress_serializable<T: Serialize>(&mut self, data: &T) -> Result<CompressedData> {
        let serialized = serde_json::to_vec(data)?;
        self.compress_data(&serialized)
    }

    pub fn decompress_deserializable<T: for<'de> Deserialize<'de>>(
        &mut self,
        compressed_data: &CompressedData,
    ) -> Result<T> {
        let decompressed = self.decompress_data(compressed_data)?;
        let deserialized = serde_json::from_slice(&decompressed)?;
        Ok(deserialized)
    }

    pub fn get_stats(&self) -> &CompressionStats {
        &self.stats
    }

    pub fn get_average_compression_ratio(&self) -> f64 {
        if self.stats.total_original == 0 {
            0.0
        } else {
            self.stats.total_compressed as f64 / self.stats.total_original as f64
        }
    }

    pub fn get_average_compression_time(&self) -> std::time::Duration {
        if self.stats.compression_operations == 0 {
            std::time::Duration::ZERO
        } else {
            self.stats.total_time / self.stats.compression_operations as u32
        }
    }

    pub fn reset_stats(&mut self) {
        self.stats = CompressionStats::default();
    }
}

/// Compression strategy selector
pub struct CompressionStrategy {
    strategies: Vec<(CompressionAlgorithm, f64)>, // algorithm and expected ratio
}

impl CompressionStrategy {
    pub fn new() -> Self {
        Self {
            strategies: vec![
                (CompressionAlgorithm::Zstd, 0.3),
                (CompressionAlgorithm::Lz4, 0.4),
                (CompressionAlgorithm::Gzip, 0.5),
                (CompressionAlgorithm::Brotli, 0.35),
                (CompressionAlgorithm::None, 1.0),
            ],
        }
    }

    pub fn select_algorithm(&self, data_size: usize, target_ratio: f64) -> CompressionAlgorithm {
        for (algorithm, expected_ratio) in &self.strategies {
            if *expected_ratio <= target_ratio {
                return *algorithm;
            }
        }
        CompressionAlgorithm::None
    }

    pub fn select_for_data_type(&self, data_type: &str) -> CompressionAlgorithm {
        match data_type {
            "json" => CompressionAlgorithm::Zstd,
            "text" => CompressionAlgorithm::Gzip,
            "binary" => CompressionAlgorithm::Lz4,
            "log" => CompressionAlgorithm::Brotli,
            _ => CompressionAlgorithm::Zstd,
        }
    }
}

/// Compression benchmark
pub struct CompressionBenchmark {
    strategies: CompressionStrategy,
}

impl CompressionBenchmark {
    pub fn new() -> Self {
        Self {
            strategies: CompressionStrategy::new(),
        }
    }

    pub fn benchmark_algorithm(
        &self,
        data: &[u8],
        algorithm: CompressionAlgorithm,
    ) -> Result<CompressionResult> {
        let config = CompressionConfig {
            algorithm,
            level: 3,
            threshold: 0,
            enabled: true,
        };

        let mut manager = CompressionManager::new(config);
        let start = std::time::Instant::now();

        let compressed = manager.compress_data(data)?;
        let compression_time = start.elapsed();

        let start = std::time::Instant::now();
        let _decompressed = manager.decompress_data(&compressed)?;
        let decompression_time = start.elapsed();

        Ok(CompressionResult {
            algorithm,
            original_size: data.len(),
            compressed_size: compressed.compressed_size,
            compression_ratio: compressed.compression_ratio(),
            compression_time,
            decompression_time,
            throughput_mbps: (data.len() as f64 / compression_time.as_secs_f64()) / 1_000_000.0,
        })
    }

    pub fn benchmark_all(&self, data: &[u8]) -> Result<Vec<CompressionResult>> {
        let mut results = Vec::new();

        for algorithm in [
            CompressionAlgorithm::None,
            CompressionAlgorithm::Gzip,
            CompressionAlgorithm::Lz4,
            CompressionAlgorithm::Zstd,
            CompressionAlgorithm::Brotli,
        ] {
            if let Ok(result) = self.benchmark_algorithm(data, algorithm) {
                results.push(result);
            }
        }

        results.sort_by(|a, b| {
            a.compression_ratio
                .partial_cmp(&b.compression_ratio)
                .unwrap()
        });
        Ok(results)
    }
}

#[derive(Debug, Clone)]
pub struct CompressionResult {
    pub algorithm: CompressionAlgorithm,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub compression_time: std::time::Duration,
    pub decompression_time: std::time::Duration,
    pub throughput_mbps: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_algorithms() {
        let data = b"Hello, World! This is a test string for compression.";
        let config = CompressionConfig::default();
        let mut manager = CompressionManager::new(config);

        let compressed = manager.compress_data(data).unwrap();
        assert!(compressed.compressed_size <= data.len());

        let decompressed = manager.decompress_data(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_compression_ratio() {
        let data = b"Hello, World! This is a test string for compression.";
        let config = CompressionConfig::default();
        let mut manager = CompressionManager::new(config);

        let compressed = manager.compress_data(data).unwrap();
        let ratio = compressed.compression_ratio();
        assert!(ratio > 0.0 && ratio <= 1.0);
    }

    #[test]
    fn test_serializable_compression() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: i32,
            items: Vec<String>,
        }

        let test_data = TestData {
            name: "test".to_string(),
            value: 42,
            items: vec!["item1".to_string(), "item2".to_string()],
        };

        let config = CompressionConfig::default();
        let mut manager = CompressionManager::new(config);

        let compressed = manager.compress_serializable(&test_data).unwrap();
        let decompressed: TestData = manager.decompress_deserializable(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
    }

    #[test]
    fn test_compression_strategy() {
        let strategy = CompressionStrategy::new();

        let algorithm = strategy.select_for_data_type("json");
        assert_eq!(algorithm, CompressionAlgorithm::Zstd);

        let algorithm = strategy.select_for_data_type("text");
        assert_eq!(algorithm, CompressionAlgorithm::Gzip);
    }

    #[test]
    fn test_compression_benchmark() {
        let data = b"Hello, World! This is a test string for compression. ".repeat(100);
        let benchmark = CompressionBenchmark::new();

        let results = benchmark.benchmark_all(&data).unwrap();
        assert!(!results.is_empty());

        // Zstd should have better compression than None
        let zstd_result = results
            .iter()
            .find(|r| r.algorithm == CompressionAlgorithm::Zstd)
            .unwrap();
        let none_result = results
            .iter()
            .find(|r| r.algorithm == CompressionAlgorithm::None)
            .unwrap();
        assert!(zstd_result.compression_ratio < none_result.compression_ratio);
    }
}
