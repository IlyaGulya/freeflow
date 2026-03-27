//! Hotkey actor — listens for global key events via rdev.
//! Sends hotkey down/up events to the pipeline actor.

use rdev::{EventType, Key, listen};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Events sent from the hotkey listener to the pipeline.
#[derive(Debug)]
pub enum HotkeyEvent {
    KeyDown,
    KeyUp { duration_ms: f64 },
}

pub struct HotkeyActor {
    event_rx: mpsc::UnboundedReceiver<HotkeyEvent>,
    target_key: Key,
}

impl HotkeyActor {
    /// Create a new hotkey listener for the specified key.
    /// Spawns a background thread for rdev::listen (blocking).
    pub fn new(hotkey_name: &str) -> Self {
        let target_key = hotkey_from_name(hotkey_name);
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let key = target_key;
        std::thread::spawn(move || {
            let is_pressed = Arc::new(AtomicBool::new(false));
            let press_time = Arc::new(std::sync::Mutex::new(None::<std::time::Instant>));

            let tx = event_tx;
            let pressed = is_pressed;
            let time = press_time;

            let callback = move |event: rdev::Event| {
                match event.event_type {
                    EventType::KeyPress(k) if k == key => {
                        if !pressed.swap(true, Ordering::Relaxed) {
                            *time.lock().unwrap_or_else(|e| e.into_inner()) =
                                Some(std::time::Instant::now());
                            let _ = tx.send(HotkeyEvent::KeyDown);
                        }
                    }
                    EventType::KeyRelease(k) if k == key => {
                        if pressed.swap(false, Ordering::Relaxed) {
                            let duration_ms = time
                                .lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .take()
                                .map(|t| t.elapsed().as_secs_f64() * 1000.0)
                                .unwrap_or(0.0);
                            let _ = tx.send(HotkeyEvent::KeyUp { duration_ms });
                        }
                    }
                    _ => {}
                }
            };

            if let Err(e) = listen(callback) {
                log::error!("rdev listen error: {:?}", e);
            }
        });

        Self {
            event_rx,
            target_key,
        }
    }

    /// Receive next hotkey event.
    pub async fn recv(&mut self) -> Option<HotkeyEvent> {
        self.event_rx.recv().await
    }

    /// Update the target key (requires restart — for now just log).
    pub fn set_hotkey(&mut self, name: &str) {
        self.target_key = hotkey_from_name(name);
        // Note: rdev::listen can't be reconfigured at runtime.
        // A full restart of the listener thread would be needed.
        log::info!("hotkey changed to: {:?} (takes effect on restart)", self.target_key);
    }
}

fn hotkey_from_name(name: &str) -> Key {
    match name {
        "fn" | "fnKey" => Key::Function,
        "rightOption" => Key::AltGr,
        "f5" => Key::F5,
        _ => Key::Function,
    }
}
