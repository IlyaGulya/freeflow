//! Pipeline actor — owns the PipelineEngine and routes signals.

use rinf::{DartSignal, RustSignal};
use tokio::select;
use wrenflow_domain::config::AppConfig;
use wrenflow_domain::history::HistoryEntry;
use wrenflow_domain::pipeline::{PipelineEngine, PipelineListener, PipelineSound, PipelineState};

use crate::signals;

/// Bridges PipelineListener trait to rinf signals.
struct SignalListener;

impl PipelineListener for SignalListener {
    fn on_state_changed(&self, old: PipelineState, new: PipelineState) {
        signals::PipelineStateChanged {
            old_state: domain_state_to_signal(old),
            new_state: domain_state_to_signal(new),
        }
        .send_signal_to_dart();
    }

    fn on_paste_text(&self, text: String) {
        // TODO: Use enigo+arboard to paste, then notify Dart
        signals::TranscriptReady {
            transcript: text,
        }
        .send_signal_to_dart();
    }

    fn on_play_sound(&self, sound: PipelineSound) {
        let sound_type = match sound {
            PipelineSound::RecordingStarted => signals::SoundType::RecordingStarted,
            PipelineSound::RecordingStopped => signals::SoundType::RecordingStopped,
        };
        signals::PlaySound { sound: sound_type }.send_signal_to_dart();
    }

    fn on_error(&self, message: String) {
        signals::PipelineError { message }.send_signal_to_dart();
    }

    fn on_history_entry_added(&self, entry: HistoryEntry) {
        signals::HistoryEntryAdded {
            entry: signals::HistoryEntryData {
                id: entry.id,
                timestamp: entry.timestamp,
                transcript: entry.transcript,
                custom_vocabulary: entry.custom_vocabulary,
                audio_file_name: entry.audio_file_name,
                metrics_json: entry.metrics_json,
            },
        }
        .send_signal_to_dart();
    }
}

pub struct PipelineActor {
    engine: PipelineEngine,
    listener: SignalListener,
}

impl PipelineActor {
    pub fn new() -> Self {
        Self {
            engine: PipelineEngine::new(AppConfig::default()),
            listener: SignalListener,
        }
    }

    pub async fn run(&mut self) {
        let start_recv = signals::StartRecording::get_dart_signal_receiver();
        let stop_recv = signals::StopRecording::get_dart_signal_receiver();
        let config_recv = signals::UpdateConfig::get_dart_signal_receiver();

        loop {
            select! {
                Some(_pack) = start_recv.recv() => {
                    self.engine.handle_hotkey_down(&self.listener);
                }
                Some(pack) = stop_recv.recv() => {
                    self.engine.handle_hotkey_up(pack.message.duration_ms, &self.listener);
                }
                Some(pack) = config_recv.recv() => {
                    let c = pack.message;
                    self.engine.update_config(AppConfig {
                        api_key: c.api_key,
                        api_base_url: c.api_base_url,
                        selected_hotkey: c.selected_hotkey,
                        selected_microphone_id: c.selected_microphone_id,
                        sound_enabled: c.sound_enabled,
                        custom_vocabulary: c.custom_vocabulary,
                        transcription_provider: c.transcription_provider,
                        transcription_model: c.transcription_model,
                        minimum_recording_duration_ms: c.minimum_recording_duration_ms,
                    });
                }
                else => break,
            }
        }
    }
}

fn domain_state_to_signal(state: PipelineState) -> signals::PipelineState {
    match state {
        PipelineState::Idle => signals::PipelineState::Idle,
        PipelineState::Starting => signals::PipelineState::Starting,
        PipelineState::Initializing => signals::PipelineState::Initializing,
        PipelineState::Recording => signals::PipelineState::Recording,
        PipelineState::Transcribing { showing_indicator } => {
            signals::PipelineState::Transcribing { showing_indicator }
        }
        PipelineState::Pasting => signals::PipelineState::Pasting,
        PipelineState::Error { message } => signals::PipelineState::Error { message },
    }
}
