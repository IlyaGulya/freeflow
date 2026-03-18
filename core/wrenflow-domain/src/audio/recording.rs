/// Metrics from a completed recording.
#[derive(Debug, Clone)]
pub struct RecordingMetrics {
    pub duration_ms: f64,
    pub file_size_bytes: u64,
    pub device_sample_rate: u32,
    pub buffer_count: u32,
    pub first_audio_ms: Option<f64>,
}

/// Result of a completed recording.
#[derive(Debug, Clone)]
pub struct RecordingResult {
    pub file_path: String,
    pub metrics: RecordingMetrics,
}
