//! Wrenflow Windows — system tray app entry point.
//!
//! Architecture
//! ──────────────────────────────────────────────────────────────────────────
//!  ┌──────────────┐   hotkey down/up   ┌──────────────────┐
//!  │ HotkeyManager│ ─────────────────► │  PipelineEngine  │ (wrenflow-core)
//!  └──────────────┘                    └────────┬─────────┘
//!                                               │ on_paste_text
//!  ┌──────────────┐   samples          ┌────────▼─────────┐
//!  │  AudioCapture│ ─────────────────► │  TextInserter    │
//!  └──────────────┘                    └──────────────────┘
//!
//!  The system tray (TrayIcon) runs on the main thread's Win32 message loop.
//!  Audio capture and transcription run on Tokio worker threads.
//!  Hot-key events are dispatched through the Win32 WM_HOTKEY message.
//!
//! Platform guard: this binary is Windows-only.  On non-Windows hosts it
//! compiles to a stub that explains the situation, so that `cargo check`
//! succeeds on macOS/Linux CI without Windows-specific dependencies.

#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"  // suppress the console window in release
)]

mod audio;
mod hotkey;
mod text_insert;
mod tray;

// ---------------------------------------------------------------------------
// Windows-only real implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod app {
    use std::sync::{Arc, Mutex};

    use anyhow::Result;
    use log::info;
    use tokio::sync::mpsc;
    use wrenflow_core::{
        config::AppConfig,
        pipeline::{PipelineEngine, PipelineListener, PipelineSound, PipelineState},
        history::HistoryEntry,
    };

    use crate::audio::AudioCapture;
    use crate::hotkey::{HotkeyEvent, HotkeyManager};
    use crate::text_insert::TextInserter;
    use crate::tray::{TrayCommand, TrayIcon};

    // -----------------------------------------------------------------------
    // PipelineListener implementation
    // -----------------------------------------------------------------------

    /// Bridges wrenflow-core pipeline callbacks to Windows-native actions.
    struct WindowsListener {
        inserter: Arc<TextInserter>,
    }

    impl PipelineListener for WindowsListener {
        fn on_state_changed(&self, old: &PipelineState, new: &PipelineState) {
            info!("pipeline: {} → {}", old.name(), new.name());
            // TODO: update tray icon tooltip / animation to reflect state
        }

        fn on_paste_text(&self, text: &str) {
            if let Err(e) = self.inserter.insert(text) {
                log::error!("text insertion failed: {e}");
            }
        }

        fn on_play_sound(&self, sound: PipelineSound) {
            // Windows system sounds via MessageBeep
            #[cfg(target_os = "windows")]
            {
                use windows::Win32::UI::WindowsAndMessaging::{MessageBeep, MB_OK, MB_ICONINFORMATION};
                match sound {
                    PipelineSound::RecordingStarted => unsafe { let _ = MessageBeep(MB_OK); },
                    PipelineSound::RecordingStopped => unsafe { let _ = MessageBeep(MB_ICONINFORMATION); },
                }
            }
        }

        fn on_error(&self, message: &str) {
            log::error!("pipeline error: {message}");
            // TODO: show Windows balloon notification via Shell_NotifyIcon
        }

        fn on_history_entry_added(&self, entry: &HistoryEntry) {
            info!(
                "history entry: id={} transcript={}",
                entry.id,
                &entry.post_processed_transcript.chars().take(60).collect::<String>()
            );
            // TODO: persist via wrenflow-core HistoryStore
        }
    }

    // -----------------------------------------------------------------------
    // Event loop
    // -----------------------------------------------------------------------

    pub fn run() -> Result<()> {
        env_logger::init();
        info!("Wrenflow Windows starting");

        // Load (or create default) configuration
        let config_path = AppConfig::default_path("Wrenflow");
        let config = AppConfig::load_or_default(&config_path);
        info!("config loaded from {config_path:?}");

        // Shared components
        let inserter = Arc::new(TextInserter::new());
        let listener = Arc::new(WindowsListener { inserter: inserter.clone() });
        let engine = Arc::new(Mutex::new(PipelineEngine::new(config.clone())));

        // Channel: audio capture → pipeline
        let (audio_tx, mut audio_rx) = mpsc::unbounded_channel::<Vec<f32>>();
        // Channel: tray commands → event loop
        let (tray_tx, mut tray_rx) = mpsc::unbounded_channel::<TrayCommand>();

        // --- Tokio runtime for async work (transcription, audio collection) ---
        let rt = tokio::runtime::Runtime::new()?;

        // Spawn audio-collection task: accumulates samples until signalled
        let engine_clone = engine.clone();
        let listener_clone = listener.clone();
        let config_clone = config.clone();
        rt.spawn(async move {
            let mut samples: Vec<f32> = Vec::new();
            let mut recording = false;
            let mut capture: Option<AudioCapture> = None;

            loop {
                tokio::select! {
                    Some(chunk) = audio_rx.recv() => {
                        if recording {
                            samples.extend_from_slice(&chunk);
                        }
                    }
                }
                // Note: the stop-recording signal comes through the hotkey path
                // below; this task simply drains the channel.
                let _ = (engine_clone.clone(), listener_clone.clone(), config_clone.clone(), capture.take());
                break; // placeholder — real loop exits on channel close
            }
        });

        // --- Hotkey manager (Win32 RegisterHotKey) ---
        let mut hotkey_mgr = HotkeyManager::new()?;
        let hotkey_id = hotkey_mgr.register_from_config(&config)?;
        info!("registered hotkey id={hotkey_id} for key={}", config.selected_hotkey);

        // --- System tray ---
        let mut tray = TrayIcon::new(tray_tx.clone())?;
        tray.show()?;

        // --- Main Win32 message loop ---
        //
        // WM_HOTKEY → start/stop recording
        // Tray commands → settings / quit
        let mut audio_capture: Option<AudioCapture> = None;
        let mut recording_start: Option<std::time::Instant> = None;

        loop {
            // Poll hotkey events (non-blocking)
            while let Ok(event) = hotkey_mgr.try_recv() {
                let mut eng = engine.lock().unwrap();
                match event {
                    HotkeyEvent::Pressed => {
                        if eng.handle_hotkey_down(&*listener) {
                            // Start WASAPI capture
                            match AudioCapture::start(audio_tx.clone()) {
                                Ok(cap) => {
                                    audio_capture = Some(cap);
                                    recording_start = Some(std::time::Instant::now());
                                    eng.on_first_audio(&*listener);
                                }
                                Err(e) => {
                                    eng.on_pipeline_error(&format!("audio init failed: {e}"), &*listener);
                                }
                            }
                        }
                    }
                    HotkeyEvent::Released => {
                        let duration_ms = recording_start
                            .take()
                            .map(|s| s.elapsed().as_secs_f64() * 1000.0)
                            .unwrap_or(0.0);

                        if eng.handle_hotkey_up(duration_ms, &*listener) {
                            // Stop capture, collect samples
                            if let Some(cap) = audio_capture.take() {
                                let collected = cap.stop();
                                drop(eng); // release lock before async work

                                // Run transcription on the Tokio thread pool
                                let engine2 = engine.clone();
                                let listener2 = listener.clone();
                                let config2 = config.clone();
                                rt.spawn(async move {
                                    transcribe_and_finish(
                                        collected, &config2, engine2, listener2,
                                    ).await;
                                });
                            }
                        }
                    }
                }
            }

            // Poll tray commands
            while let Ok(cmd) = tray_rx.try_recv() {
                match cmd {
                    TrayCommand::OpenSettings => {
                        info!("settings requested (TODO: open settings window)");
                    }
                    TrayCommand::Quit => {
                        info!("quit requested");
                        tray.hide()?;
                        return Ok(());
                    }
                }
            }

            // Pump Win32 message queue (needed for WM_HOTKEY + tray messages)
            tray.pump_messages();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    /// Transcribes collected samples and feeds the result back to the pipeline.
    async fn transcribe_and_finish(
        samples: Vec<f32>,
        config: &AppConfig,
        engine: Arc<Mutex<PipelineEngine>>,
        listener: Arc<WindowsListener>,
    ) {
        use wrenflow_core::audio::{resample_to_16khz, pad_to_minimum_duration, TARGET_SAMPLE_RATE};
        use wrenflow_core::pipeline::TranscriptionResult;

        let start = std::time::Instant::now();

        // Resample to 16 kHz if needed (WASAPI may return 44.1kHz or 48kHz)
        // AudioCapture already captures at 16kHz if the device supports it;
        // this is a safety resampling step.
        let samples_16k = resample_to_16khz(&samples, 48_000, TARGET_SAMPLE_RATE);
        let padded = pad_to_minimum_duration(&samples_16k, TARGET_SAMPLE_RATE, 1.0);

        let transcript = match config.transcription_provider.as_str() {
            "groq" => {
                transcribe_cloud(&padded, config).await
            }
            _ => {
                transcribe_local(&padded).await
            }
        };

        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        match transcript {
            Ok(text) => {
                let result = TranscriptionResult {
                    raw_transcript: text,
                    duration_ms,
                    provider: config.transcription_provider.clone(),
                };
                let mut eng = engine.lock().unwrap();
                eng.on_transcription_complete(result, &*listener);
            }
            Err(e) => {
                let mut eng = engine.lock().unwrap();
                eng.on_pipeline_error(&e.to_string(), &*listener);
            }
        }
    }

    async fn transcribe_cloud(samples: &[f32], config: &AppConfig) -> Result<String, anyhow::Error> {
        use wrenflow_core::audio::wav::encode_wav_to_vec;
        use wrenflow_core::transcription::cloud::transcribe;
        use wrenflow_core::http_client::build_http_client;

        let wav = encode_wav_to_vec(samples)?;
        let tmp = std::env::temp_dir().join("wrenflow_recording.wav");
        tokio::fs::write(&tmp, &wav).await?;

        let api_key = std::env::var("GROQ_API_KEY")
            .map_err(|_| anyhow::anyhow!("GROQ_API_KEY not set"))?;
        let client = build_http_client()?;
        let text = transcribe(&client, &api_key, &tmp, &config.api_base_url).await?;
        let _ = tokio::fs::remove_file(&tmp).await;
        Ok(text)
    }

    async fn transcribe_local(samples: &[f32]) -> Result<String, anyhow::Error> {
        use wrenflow_core::transcription::local::LocalTranscriber;

        let transcriber = LocalTranscriber::new()?;
        let text = transcriber.transcribe(samples).await?;
        Ok(text)
    }
}

// ---------------------------------------------------------------------------
// Non-Windows stub — lets `cargo check` pass on macOS/Linux
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "windows"))]
mod app {
    use anyhow::Result;

    pub fn run() -> Result<()> {
        eprintln!(
            "wrenflow-windows must be compiled for target x86_64-pc-windows-msvc \
             or aarch64-pc-windows-msvc."
        );
        eprintln!("Cross-check passed on non-Windows host — no runtime behaviour available.");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> anyhow::Result<()> {
    app::run()
}
