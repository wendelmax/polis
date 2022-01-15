use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: u64,
    pub level: LogLevel,
    pub message: String,
    pub source: String,
    pub container_id: Option<String>,
    pub fields: HashMap<String, String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogQuery {
    pub level_filter: Option<LogLevel>,
    pub source_filter: Option<String>,
    pub container_filter: Option<String>,
    pub message_filter: Option<String>,
    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStats {
    pub total_entries: usize,
    pub entries_by_level: HashMap<LogLevel, usize>,
    pub entries_by_source: HashMap<String, usize>,
    pub entries_by_container: HashMap<String, usize>,
    pub oldest_entry: Option<u64>,
    pub newest_entry: Option<u64>,
}

pub struct LogManager {
    logs: Vec<LogEntry>,
    next_log_id: u64,
    max_logs: usize,
}

impl LogManager {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Vec::new(),
            next_log_id: 1,
            max_logs,
        }
    }

    pub async fn add_log(
        &mut self,
        level: LogLevel,
        message: &str,
        source: &str,
        container_id: Option<&str>,
    ) -> Result<String> {
        let log_id = format!("log-{}", self.next_log_id);
        self.next_log_id += 1;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let log_entry = LogEntry {
            id: log_id.clone(),
            timestamp,
            level: level.clone(),
            message: message.to_string(),
            source: source.to_string(),
            container_id: container_id.map(|s| s.to_string()),
            fields: HashMap::new(),
            tags: Vec::new(),
        };

        self.logs.push(log_entry);

        // Maintain max logs limit
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }

        println!("📝 Log adicionado: [{:?}] {} - {}", level, source, message);

        Ok(log_id)
    }

    pub async fn add_log_with_fields(
        &mut self,
        level: LogLevel,
        message: &str,
        source: &str,
        container_id: Option<&str>,
        fields: HashMap<String, String>,
        tags: Vec<String>,
    ) -> Result<String> {
        let log_id = format!("log-{}", self.next_log_id);
        self.next_log_id += 1;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let log_entry = LogEntry {
            id: log_id.clone(),
            timestamp,
            level: level.clone(),
            message: message.to_string(),
            source: source.to_string(),
            container_id: container_id.map(|s| s.to_string()),
            fields,
            tags,
        };

        self.logs.push(log_entry);

        // Maintain max logs limit
        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }

        println!(
            "📝 Log estruturado adicionado: [{:?}] {} - {}",
            level, source, message
        );

        Ok(log_id)
    }

    pub async fn query_logs(&self, query: &LogQuery) -> Result<Vec<LogEntry>> {
        let mut filtered_logs: Vec<LogEntry> = self
            .logs
            .iter()
            .filter(|log| {
                // Level filter
                if let Some(ref level_filter) = query.level_filter {
                    if log.level != *level_filter {
                        return false;
                    }
                }

                // Source filter
                if let Some(ref source_filter) = query.source_filter {
                    if !log.source.contains(source_filter) {
                        return false;
                    }
                }

                // Container filter
                if let Some(ref container_filter) = query.container_filter {
                    if let Some(ref container_id) = log.container_id {
                        if !container_id.contains(container_filter) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }

                // Message filter
                if let Some(ref message_filter) = query.message_filter {
                    if !log
                        .message
                        .to_lowercase()
                        .contains(&message_filter.to_lowercase())
                    {
                        return false;
                    }
                }

                // Time range filter
                if let Some(start_time) = query.start_time {
                    if log.timestamp < start_time {
                        return false;
                    }
                }

                if let Some(end_time) = query.end_time {
                    if log.timestamp > end_time {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        filtered_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(100);

        let start = offset;
        let end = std::cmp::min(start + limit, filtered_logs.len());

        if start >= filtered_logs.len() {
            return Ok(Vec::new());
        }

        Ok(filtered_logs[start..end].to_vec())
    }

    pub async fn get_log_stats(&self) -> Result<LogStats> {
        let total_entries = self.logs.len();

        let mut entries_by_level = HashMap::new();
        let mut entries_by_source = HashMap::new();
        let mut entries_by_container = HashMap::new();

        let mut oldest_entry = None;
        let mut newest_entry = None;

        for log in &self.logs {
            // Count by level
            *entries_by_level.entry(log.level.clone()).or_insert(0) += 1;

            // Count by source
            *entries_by_source.entry(log.source.clone()).or_insert(0) += 1;

            // Count by container
            if let Some(ref container_id) = log.container_id {
                *entries_by_container
                    .entry(container_id.clone())
                    .or_insert(0) += 1;
            }

            // Track oldest and newest
            if oldest_entry.is_none() || log.timestamp < oldest_entry.unwrap() {
                oldest_entry = Some(log.timestamp);
            }
            if newest_entry.is_none() || log.timestamp > newest_entry.unwrap() {
                newest_entry = Some(log.timestamp);
            }
        }

        Ok(LogStats {
            total_entries,
            entries_by_level,
            entries_by_source,
            entries_by_container,
            oldest_entry,
            newest_entry,
        })
    }

    pub async fn search_logs(&self, search_term: &str) -> Result<Vec<LogEntry>> {
        let search_lower = search_term.to_lowercase();

        let filtered_logs: Vec<LogEntry> = self
            .logs
            .iter()
            .filter(|log| {
                log.message.to_lowercase().contains(&search_lower)
                    || log.source.to_lowercase().contains(&search_lower)
                    || log
                        .container_id
                        .as_ref()
                        .map(|id| id.to_lowercase().contains(&search_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        Ok(filtered_logs)
    }

    pub async fn get_logs_by_container(&self, container_id: &str) -> Result<Vec<LogEntry>> {
        let container_logs: Vec<LogEntry> = self
            .logs
            .iter()
            .filter(|log| {
                log.container_id
                    .as_ref()
                    .map(|id| id == container_id)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        Ok(container_logs)
    }

    pub async fn get_logs_by_level(&self, level: LogLevel) -> Result<Vec<LogEntry>> {
        let level_logs: Vec<LogEntry> = self
            .logs
            .iter()
            .filter(|log| log.level == level)
            .cloned()
            .collect();

        Ok(level_logs)
    }

    pub async fn clear_old_logs(&mut self, older_than_seconds: u64) -> Result<usize> {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - older_than_seconds;

        let initial_count = self.logs.len();
        self.logs.retain(|log| log.timestamp >= cutoff_time);
        let removed_count = initial_count - self.logs.len();

        println!(
            "🧹 {} logs antigos removidos (mais antigos que {} segundos)",
            removed_count, older_than_seconds
        );

        Ok(removed_count)
    }

    pub async fn export_logs(&self, format: &str) -> Result<String> {
        match format.to_lowercase().as_str() {
            "json" => serde_json::to_string_pretty(&self.logs).map_err(PolisError::Serialization),
            "csv" => {
                let mut csv = String::from("timestamp,level,source,container_id,message\n");
                for log in &self.logs {
                    csv.push_str(&format!(
                        "{},{:?},{},{},{}\n",
                        log.timestamp,
                        log.level,
                        log.source,
                        log.container_id.as_deref().unwrap_or(""),
                        log.message.replace("\"", "\"\"")
                    ));
                }
                Ok(csv)
            }
            _ => Err(PolisError::Api(format!(
                "Formato de exportação não suportado: {}",
                format
            ))),
        }
    }

    pub async fn get_recent_logs(&self, count: usize) -> Result<Vec<LogEntry>> {
        let mut recent_logs = self.logs.clone();
        recent_logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if recent_logs.len() > count {
            recent_logs.truncate(count);
        }

        Ok(recent_logs)
    }
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new(10000) // Default max 10,000 logs
    }
}
