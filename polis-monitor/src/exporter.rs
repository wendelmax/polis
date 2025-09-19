use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Prometheus,
    InfluxDB,
    Elasticsearch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub endpoint: Option<String>,
    pub headers: HashMap<String, String>,
    pub batch_size: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub timestamp: u64,
    pub metrics: HashMap<String, f64>,
    pub labels: HashMap<String, String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub records_exported: usize,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

pub struct MetricsExporter {
    config: ExportConfig,
    export_history: Vec<ExportResult>,
}

impl MetricsExporter {
    pub fn new(config: ExportConfig) -> Self {
        Self {
            config,
            export_history: Vec::new(),
        }
    }

    pub async fn export_metrics(&mut self, data: &[ExportData]) -> Result<ExportResult> {
        let start_time = SystemTime::now();
        let mut errors = Vec::new();
        let mut records_exported = 0;

        // Process data in batches
        for batch in data.chunks(self.config.batch_size) {
            match self.export_batch(batch).await {
                Ok(count) => {
                    records_exported += count;
                }
                Err(e) => {
                    errors.push(format!("Batch export failed: {}", e));
                }
            }
        }

        let duration = SystemTime::now()
            .duration_since(start_time)
            .unwrap()
            .as_millis() as u64;

        let result = ExportResult {
            success: errors.is_empty(),
            records_exported,
            errors,
            duration_ms: duration,
        };

        self.export_history.push(result.clone());

        // Keep only last 100 export results
        if self.export_history.len() > 100 {
            self.export_history.remove(0);
        }

        println!(
            "� Exportação concluída: {} registros em {}ms",
            records_exported, duration
        );

        Ok(result)
    }

    async fn export_batch(&self, batch: &[ExportData]) -> Result<usize> {
        match self.config.format {
            ExportFormat::Json => self.export_json(batch).await,
            ExportFormat::Csv => self.export_csv(batch).await,
            ExportFormat::Prometheus => self.export_prometheus(batch).await,
            ExportFormat::InfluxDB => self.export_influxdb(batch).await,
            ExportFormat::Elasticsearch => self.export_elasticsearch(batch).await,
        }
    }

    async fn export_json(&self, batch: &[ExportData]) -> Result<usize> {
        let json_data = serde_json::to_string_pretty(batch).map_err(PolisError::Serialization)?;

        if let Some(_endpoint) = &self.config.endpoint {
            self.send_to_endpoint(&json_data, "application/json")
                .await?;
        } else {
            println!("� JSON Export:\n{}", json_data);
        }

        Ok(batch.len())
    }

    async fn export_csv(&self, batch: &[ExportData]) -> Result<usize> {
        let mut csv = String::from("timestamp,source");

        // Get all unique metric names and labels
        let mut metric_names = std::collections::HashSet::new();
        let mut label_names = std::collections::HashSet::new();

        for data in batch {
            for metric_name in data.metrics.keys() {
                metric_names.insert(metric_name);
            }
            for label_name in data.labels.keys() {
                label_names.insert(label_name);
            }
        }

        // Add metric columns
        for metric_name in &metric_names {
            csv.push_str(&format!(",{}", metric_name));
        }

        // Add label columns
        for label_name in &label_names {
            csv.push_str(&format!(",label_{}", label_name));
        }
        csv.push('\n');

        // Add data rows
        for data in batch {
            csv.push_str(&format!("{},{}", data.timestamp, data.source));

            // Add metric values
            for metric_name in &metric_names {
                let value = data.metrics.get(metric_name.as_str()).unwrap_or(&0.0);
                csv.push_str(&format!(",{}", value));
            }

            // Add label values
            for label_name in &label_names {
                let empty_string = "".to_string();
                let value = data
                    .labels
                    .get(label_name.as_str())
                    .unwrap_or(&empty_string);
                csv.push_str(&format!(",{}", value));
            }
            csv.push('\n');
        }

        if let Some(_endpoint) = &self.config.endpoint {
            self.send_to_endpoint(&csv, "text/csv").await?;
        } else {
            println!(" CSV Export:\n{}", csv);
        }

        Ok(batch.len())
    }

    async fn export_prometheus(&self, batch: &[ExportData]) -> Result<usize> {
        let mut prometheus_data = String::new();

        for data in batch {
            for (metric_name, value) in &data.metrics {
                let mut labels = String::new();
                for (label_name, label_value) in &data.labels {
                    labels.push_str(&format!("{}=\"{}\",", label_name, label_value));
                }
                labels.push_str(&format!("source=\"{}\"", data.source));

                prometheus_data.push_str(&format!(
                    "{}_polis{{{}}} {} {}\n",
                    metric_name.replace("-", "_"),
                    labels,
                    value,
                    data.timestamp * 1000 // Prometheus expects milliseconds
                ));
            }
        }

        if let Some(_endpoint) = &self.config.endpoint {
            self.send_to_endpoint(&prometheus_data, "text/plain")
                .await?;
        } else {
            println!(" Prometheus Export:\n{}", prometheus_data);
        }

        Ok(batch.len())
    }

    async fn export_influxdb(&self, batch: &[ExportData]) -> Result<usize> {
        let mut influx_data = String::new();

        for data in batch {
            let measurement = "polis_metrics";
            let mut tags = String::new();
            for (tag_name, tag_value) in &data.labels {
                tags.push_str(&format!(",{}={}", tag_name, tag_value));
            }
            tags.push_str(&format!(",source={}", data.source));

            let mut fields = String::new();
            for (field_name, field_value) in &data.metrics {
                fields.push_str(&format!(",{}={}", field_name, field_value));
            }

            influx_data.push_str(&format!(
                "{} {} {}{}\n",
                measurement,
                data.timestamp * 1000000000, // InfluxDB expects nanoseconds
                fields.trim_start_matches(','),
                tags
            ));
        }

        if let Some(_endpoint) = &self.config.endpoint {
            self.send_to_endpoint(&influx_data, "text/plain").await?;
        } else {
            println!(" InfluxDB Export:\n{}", influx_data);
        }

        Ok(batch.len())
    }

    async fn export_elasticsearch(&self, batch: &[ExportData]) -> Result<usize> {
        let mut elastic_data = String::new();

        for data in batch {
            let index = format!(
                "polis-metrics-{}",
                chrono::DateTime::from_timestamp(data.timestamp as i64, 0)
                    .unwrap()
                    .format("%Y.%m.%d")
            );

            let doc = serde_json::json!({
                "@timestamp": data.timestamp * 1000, // Elasticsearch expects milliseconds
                "source": data.source,
                "metrics": data.metrics,
                "labels": data.labels
            });

            elastic_data.push_str(&format!(
                "{{ \"index\": {{ \"_index\": \"{}\" }} }}\n{}\n",
                index,
                serde_json::to_string(&doc).unwrap()
            ));
        }

        if let Some(_endpoint) = &self.config.endpoint {
            self.send_to_endpoint(&elastic_data, "application/x-ndjson")
                .await?;
        } else {
            println!(" Elasticsearch Export:\n{}", elastic_data);
        }

        Ok(batch.len())
    }

    async fn send_to_endpoint(&self, _data: &str, content_type: &str) -> Result<()> {
        // Simulate sending data to endpoint
        println!(
            "� Enviando dados para {} (Content-Type: {})",
            self.config.endpoint.as_deref().unwrap_or("unknown"),
            content_type
        );

        // In a real implementation, this would make an HTTP request
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(())
    }

    pub async fn get_export_history(&self) -> Result<Vec<ExportResult>> {
        Ok(self.export_history.clone())
    }

    pub async fn get_export_stats(&self) -> Result<ExportStats> {
        let total_exports = self.export_history.len();
        let successful_exports = self.export_history.iter().filter(|r| r.success).count();
        let failed_exports = total_exports - successful_exports;

        let total_records = self.export_history.iter().map(|r| r.records_exported).sum();

        let avg_duration = if total_exports > 0 {
            self.export_history
                .iter()
                .map(|r| r.duration_ms)
                .sum::<u64>()
                / total_exports as u64
        } else {
            0
        };

        Ok(ExportStats {
            total_exports,
            successful_exports,
            failed_exports,
            total_records_exported: total_records,
            average_duration_ms: avg_duration,
            last_export: self.export_history.last().cloned(),
        })
    }

    pub fn update_config(&mut self, config: ExportConfig) {
        self.config = config;
        println!("⚙ Configuração de exportação atualizada");
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStats {
    pub total_exports: usize,
    pub successful_exports: usize,
    pub failed_exports: usize,
    pub total_records_exported: usize,
    pub average_duration_ms: u64,
    pub last_export: Option<ExportResult>,
}

impl Default for MetricsExporter {
    fn default() -> Self {
        Self::new(ExportConfig {
            format: ExportFormat::Json,
            endpoint: None,
            headers: HashMap::new(),
            batch_size: 100,
            timeout_seconds: 30,
        })
    }
}
