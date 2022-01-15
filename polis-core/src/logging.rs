use crate::config::LogLevel;
use std::path::PathBuf;
use tracing::{debug, error, info, trace, warn};
use tracing_subscriber::fmt::{self, format::FmtSpan};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub struct Logger {
    level: LogLevel,
    log_file: Option<PathBuf>,
}

impl Logger {
    pub fn new(level: LogLevel, log_file: Option<PathBuf>) -> Self {
        Self { level, log_file }
    }

    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let filter = match self.level {
            LogLevel::Error => EnvFilter::new("error"),
            LogLevel::Warn => EnvFilter::new("warn"),
            LogLevel::Info => EnvFilter::new("info"),
            LogLevel::Debug => EnvFilter::new("debug"),
            LogLevel::Trace => EnvFilter::new("trace"),
        };

        let registry = Registry::default().with(filter);

        if let Some(log_file) = &self.log_file {
            let file_appender =
                tracing_appender::rolling::daily(log_file.parent().unwrap(), "polis.log");
            let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

            let file_layer = fmt::layer()
                .with_writer(non_blocking)
                .with_span_events(FmtSpan::CLOSE)
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true);

            let _ = registry.with(file_layer).try_init();
        } else {
            let stdout_layer = fmt::layer()
                .with_span_events(FmtSpan::CLOSE)
                .with_target(false)
                .with_thread_ids(true)
                .with_thread_names(true);

            let _ = registry.with(stdout_layer).try_init();
        }

        Ok(())
    }
}

pub fn log_container_created(container_id: &str, name: &str) {
    info!(
        container_id = %container_id,
        container_name = %name,
        "Container criado com sucesso"
    );
}

pub fn log_container_started(container_id: &str, name: &str) {
    info!(
        container_id = %container_id,
        container_name = %name,
        "Container iniciado"
    );
}

pub fn log_container_stopped(container_id: &str, name: &str, exit_code: Option<i32>) {
    info!(
        container_id = %container_id,
        container_name = %name,
        exit_code = ?exit_code,
        "Container parado"
    );
}

pub fn log_container_removed(container_id: &str, name: &str) {
    info!(
        container_id = %container_id,
        container_name = %name,
        "Container removido"
    );
}

pub fn log_image_pulled(image_name: &str, tag: &str) {
    info!(
        image = %image_name,
        tag = %tag,
        "Imagem baixada com sucesso"
    );
}

pub fn log_network_created(network_id: &str, name: &str) {
    info!(
        network_id = %network_id,
        network_name = %name,
        "Rede criada"
    );
}

pub fn log_volume_created(volume_id: &str, name: &str) {
    info!(
        volume_id = %volume_id,
        volume_name = %name,
        "Volume criado"
    );
}

pub fn log_error(error: &str, context: Option<&str>) {
    error!(
        error = %error,
        context = ?context,
        "Erro ocorreu"
    );
}

pub fn log_warning(warning: &str, context: Option<&str>) {
    warn!(
        warning = %warning,
        context = ?context,
        "Aviso"
    );
}

pub fn log_debug(message: &str, context: Option<&str>) {
    debug!(
        message = %message,
        context = ?context,
        "Debug"
    );
}

pub fn log_trace(message: &str, context: Option<&str>) {
    trace!(
        message = %message,
        context = ?context,
        "Trace"
    );
}
