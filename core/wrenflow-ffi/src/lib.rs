uniffi::setup_scaffolding!();

/// Get the default app configuration as JSON
#[uniffi::export]
pub fn default_config() -> String {
    serde_json::to_string_pretty(&wrenflow_core::config::AppConfig::default()).unwrap_or_default()
}

/// Get pipeline state status text
#[uniffi::export]
pub fn pipeline_status_text(state: String) -> String {
    match state.as_str() {
        "idle" => "Ready",
        "starting" | "initializing" => "Starting...",
        "recording" => "Recording...",
        "transcribing" => "Transcribing...",
        "post_processing" => "Processing...",
        "pasting" => "Copied to clipboard!",
        _ => "Error",
    }.to_string()
}

/// Parse metrics JSON and return a display-formatted summary
#[uniffi::export]
pub fn format_metrics(metrics_json: String) -> String {
    let metrics = wrenflow_core::metrics::PipelineMetrics::from_json(&metrics_json);
    let mut lines = Vec::new();
    for key in metrics.all_keys() {
        if let Some(val) = metrics.get_double(&key) {
            lines.push(format!("{key}: {}", wrenflow_core::metrics::MetricValue::Double(val).display_value()));
        } else if let Some(val) = metrics.get_int(&key) {
            lines.push(format!("{key}: {val}"));
        } else if let Some(val) = metrics.get_string(&key) {
            lines.push(format!("{key}: {val}"));
        } else if let Some(val) = metrics.get_bool(&key) {
            lines.push(format!("{key}: {val}"));
        }
    }
    lines.join("\n")
}
