//! Actor system for Wrenflow hub.

pub mod audio_actor;
pub mod hotkey_actor;
pub mod paste_actor;
mod pipeline_actor;

use audio_actor::AudioActor;
use hotkey_actor::HotkeyActor;
use pipeline_actor::PipelineActor;
use rinf::{DartSignal, RustSignal};
use tokio::spawn;

use crate::signals;

pub async fn create_actors() {
    let mut pipeline = PipelineActor::new();
    let _audio = AudioActor::new();
    let mut hotkey = HotkeyActor::new("fn");

    // Listen for device listing requests
    spawn(async {
        let recv = signals::ListAudioDevices::get_dart_signal_receiver();
        while let Some(_) = recv.recv().await {
            let devices = AudioActor::list_devices();
            signals::AudioDevicesListed { devices }.send_signal_to_dart();
        }
    });

    // Main loop: hotkey events drive the pipeline
    spawn(async move {
        let config_recv = signals::UpdateConfig::get_dart_signal_receiver();

        loop {
            tokio::select! {
                Some(event) = hotkey.recv() => {
                    match event {
                        hotkey_actor::HotkeyEvent::KeyDown => {
                            pipeline.handle_hotkey_down();
                        }
                        hotkey_actor::HotkeyEvent::KeyUp { duration_ms } => {
                            pipeline.handle_hotkey_up(duration_ms);
                        }
                    }
                }
                Some(pack) = config_recv.recv() => {
                    pipeline.handle_config_update(pack.message);
                }
                else => break,
            }

            pipeline.check_timers().await;
        }
    });
}
