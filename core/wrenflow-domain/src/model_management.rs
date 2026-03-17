//! Model management domain types.
//!
//! Pure data — no IO, no downloads. Infrastructure implements the actual operations.

/// Information about a downloadable model.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelInfo {
    /// Unique model identifier (e.g., "parakeet-tdt-0.6b-v3-onnx").
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// HuggingFace repo ID (e.g., "istupakov/parakeet-tdt-0.6b-v3-onnx").
    pub repo_id: String,
    /// Expected files in the model directory.
    pub expected_files: Vec<String>,
}

/// Progress of an ongoing download.
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadProgress {
    /// Bytes downloaded so far.
    pub bytes_downloaded: u64,
    /// Total bytes (if known).
    pub total_bytes: Option<u64>,
    /// Current file being downloaded (e.g., "encoder.onnx").
    pub current_file: String,
    /// Files completed / total files.
    pub files_completed: usize,
    pub files_total: usize,
}

impl DownloadProgress {
    /// Fraction 0.0..1.0, or None if total unknown.
    pub fn fraction(&self) -> Option<f64> {
        self.total_bytes.map(|total| {
            if total == 0 { 0.0 } else { self.bytes_downloaded as f64 / total as f64 }
        })
    }
}

/// Lifecycle state of a local model. Extends the simpler ModelState in transcription/.
#[derive(Debug, Clone, PartialEq)]
pub enum LocalModelState {
    /// No model files found locally.
    NotDownloaded,
    /// Download in progress.
    Downloading(DownloadProgress),
    /// Files downloaded, loading/compiling into runtime.
    Loading,
    /// Model is ready for inference.
    Ready,
    /// Something went wrong.
    Error(String),
}

impl LocalModelState {
    pub fn is_ready(&self) -> bool { matches!(self, Self::Ready) }
    pub fn is_busy(&self) -> bool { matches!(self, Self::Downloading(_) | Self::Loading) }
}

/// Callback trait for model download progress.
/// Infrastructure implements the actual download and calls these.
/// UI layer implements this to show progress.
pub trait ModelDownloadListener: Send + Sync {
    fn on_progress(&self, progress: DownloadProgress);
    fn on_state_changed(&self, state: LocalModelState);
}

/// The default Parakeet TDT ONNX model.
pub fn default_parakeet_model() -> ModelInfo {
    ModelInfo {
        id: "parakeet-tdt-0.6b-v3-onnx".to_string(),
        name: "Parakeet TDT 0.6B".to_string(),
        repo_id: "istupakov/parakeet-tdt-0.6b-v3-onnx".to_string(),
        expected_files: vec![
            "encoder-model.int8.onnx".to_string(),
            "decoder_joint-model.int8.onnx".to_string(),
            "nemo128.onnx".to_string(),
            "vocab.txt".to_string(),
            "config.json".to_string(),
        ],
    }
}
