//! Model actor — manages local Parakeet model download, loading, and lifecycle.
//!
//! Listens for `InitializeLocalModel` and `CancelModelDownload` signals from Dart,
//! drives the download via `wrenflow_core::model_downloader`, loads the model via
//! `wrenflow_core::transcription_local::LocalTranscriptionEngine`, and sends
//! `ModelStateChanged` signals back to Dart with progress updates.

use rinf::{DartSignal, RustSignal};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use wrenflow_core::model_downloader;
use wrenflow_core::model_management::{
    default_parakeet_model, DownloadProgress, LocalModelState, ModelDownloadListener,
};
use wrenflow_core::transcription_local::LocalTranscriptionEngine;

use crate::signals;

/// Bridges `ModelDownloadListener` to rinf `ModelStateChanged` signals.
struct SignalDownloadListener {
    /// Throttle progress signals to avoid flooding Dart.
    last_signal: std::sync::Mutex<Instant>,
}

impl SignalDownloadListener {
    fn new() -> Self {
        Self {
            last_signal: std::sync::Mutex::new(Instant::now()),
        }
    }

    fn send_model_state(state: &signals::ModelState) {
        signals::ModelStateChanged {
            state: state.clone(),
        }
        .send_signal_to_dart();
    }
}

impl ModelDownloadListener for SignalDownloadListener {
    fn on_progress(&self, progress: DownloadProgress) {
        // Throttle to ~20 updates/sec to avoid swamping the Dart side.
        let now = Instant::now();
        let should_send = {
            let Ok(mut last) = self.last_signal.lock() else {
                return;
            };
            if now.duration_since(*last).as_millis() >= 50 {
                *last = now;
                true
            } else {
                false
            }
        };

        if !should_send {
            return;
        }

        let fraction = progress.fraction().unwrap_or(0.0);
        let speed_bps = 0.0; // Could be computed from a rolling window; left as 0 for now
        let eta_secs = if fraction > 0.0 {
            // Rough estimate; refinement possible later
            0.0
        } else {
            0.0
        };

        Self::send_model_state(&signals::ModelState::Downloading {
            progress: fraction,
            speed_bps,
            eta_secs,
        });
    }

    fn on_state_changed(&self, state: LocalModelState) {
        let signal_state = domain_state_to_signal(&state);
        Self::send_model_state(&signal_state);
    }
}

/// Return the directory where model files should live.
fn model_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("wrenflow").join("models").join("parakeet-tdt")
}

/// Run the model actor. Spawned once from `create_actors`.
pub async fn run() {
    let init_recv = signals::InitializeLocalModel::get_dart_signal_receiver();
    let cancel_recv = signals::CancelModelDownload::get_dart_signal_receiver();

    // Shared cancel flag — reset on each new initialization attempt.
    let cancel_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    loop {
        tokio::select! {
            Some(_) = init_recv.recv() => {
                // Reset cancel flag for this attempt
                cancel_flag.store(false, Ordering::Relaxed);
                handle_initialize(cancel_flag.clone()).await;
            }
            Some(_) = cancel_recv.recv() => {
                log::info!("Model download cancel requested");
                cancel_flag.store(true, Ordering::Relaxed);
                // The download loop in handle_initialize checks the flag and will bail out.
            }
            else => break,
        }
    }
}

/// Full initialize flow: check presence → download if needed → load model.
async fn handle_initialize(cancel_flag: Arc<AtomicBool>) {
    let model = default_parakeet_model();
    let dir = model_dir();

    // 1. Check if already downloaded
    let already_present = model_downloader::is_model_present(&model, &dir);

    if !already_present {
        // Tell Dart we're starting the download
        signals::ModelStateChanged {
            state: signals::ModelState::Downloading {
                progress: 0.0,
                speed_bps: 0.0,
                eta_secs: 0.0,
            },
        }
        .send_signal_to_dart();

        let listener = Arc::new(SignalDownloadListener::new());

        match model_downloader::download_model(&model, &dir, listener, cancel_flag.clone()).await {
            Ok(_path) => {
                log::info!("Model download complete");
            }
            Err(e) if e == "Cancelled" => {
                log::info!("Model download cancelled by user");
                signals::ModelStateChanged {
                    state: signals::ModelState::NotDownloaded,
                }
                .send_signal_to_dart();
                return;
            }
            Err(e) => {
                log::error!("Model download failed: {e}");
                signals::ModelStateChanged {
                    state: signals::ModelState::Error { message: e },
                }
                .send_signal_to_dart();
                return;
            }
        }
    }

    // 2. Load/compile the model (CPU-intensive — run on blocking thread)
    signals::ModelStateChanged {
        state: signals::ModelState::Loading,
    }
    .send_signal_to_dart();

    let load_dir = dir.clone();
    let load_result = tokio::task::spawn_blocking(move || {
        let mut engine = LocalTranscriptionEngine::new();
        engine.initialize(&load_dir, None)?;
        Ok::<(), wrenflow_core::transcription_local::LocalTranscriptionError>(())
    })
    .await;

    match load_result {
        Ok(Ok(())) => {
            log::info!("Local transcription model ready");
            signals::ModelStateChanged {
                state: signals::ModelState::Ready,
            }
            .send_signal_to_dart();
        }
        Ok(Err(e)) => {
            log::error!("Model load failed: {e}");
            signals::ModelStateChanged {
                state: signals::ModelState::Error {
                    message: e.to_string(),
                },
            }
            .send_signal_to_dart();
        }
        Err(e) => {
            log::error!("Model load task panicked: {e}");
            signals::ModelStateChanged {
                state: signals::ModelState::Error {
                    message: format!("Internal error: {e}"),
                },
            }
            .send_signal_to_dart();
        }
    }
}

fn domain_state_to_signal(state: &LocalModelState) -> signals::ModelState {
    match state {
        LocalModelState::NotDownloaded => signals::ModelState::NotDownloaded,
        LocalModelState::Downloading(progress) => {
            let fraction = progress.fraction().unwrap_or(0.0);
            signals::ModelState::Downloading {
                progress: fraction,
                speed_bps: 0.0,
                eta_secs: 0.0,
            }
        }
        LocalModelState::Loading => signals::ModelState::Loading,
        LocalModelState::Ready => signals::ModelState::Ready,
        LocalModelState::Error(msg) => signals::ModelState::Error {
            message: msg.clone(),
        },
    }
}
