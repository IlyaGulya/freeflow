//! History actor — manages SQLite history store and routes signals.

use rinf::{DartSignal, RustSignal};
use std::path::PathBuf;
use wrenflow_core::history_store::HistoryStore;

use crate::signals;

pub struct HistoryActor {
    store: HistoryStore,
}

impl HistoryActor {
    pub fn new(db_path: PathBuf) -> Result<Self, String> {
        let store =
            HistoryStore::open(&db_path).map_err(|e| format!("Failed to open history db: {e}"))?;
        Ok(Self { store })
    }

    /// Run in a dedicated thread (rusqlite Connection is !Send).
    /// Receives commands via a channel from the async world.
    pub fn run_blocking(self) {
        let load_recv = signals::LoadHistory::get_dart_signal_receiver();
        let delete_recv = signals::DeleteHistoryEntry::get_dart_signal_receiver();
        let clear_recv = signals::ClearHistory::get_dart_signal_receiver();

        // Use a simple blocking loop with tokio runtime for signal receivers
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("history runtime");

        rt.block_on(async {
            loop {
                tokio::select! {
                    Some(_) = load_recv.recv() => {
                        self.handle_load();
                    }
                    Some(pack) = delete_recv.recv() => {
                        self.handle_delete(&pack.message.id);
                    }
                    Some(_) = clear_recv.recv() => {
                        self.handle_clear();
                    }
                    else => break,
                }
            }
        });
    }

    fn handle_load(&self) {
        match self.store.load_all() {
            Ok(entries) => {
                let signal_entries: Vec<signals::HistoryEntryData> = entries
                    .into_iter()
                    .map(|e| signals::HistoryEntryData {
                        id: e.id,
                        timestamp: e.timestamp,
                        transcript: e.transcript,
                        custom_vocabulary: e.custom_vocabulary,
                        audio_file_name: e.audio_file_name,
                        metrics_json: e.metrics_json,
                    })
                    .collect();
                signals::HistoryLoaded {
                    entries: signal_entries,
                }
                .send_signal_to_dart();
            }
            Err(e) => {
                log::error!("Failed to load history: {e}");
                signals::PipelineError {
                    message: format!("Failed to load history: {e}"),
                }
                .send_signal_to_dart();
            }
        }
    }

    fn handle_delete(&self, id: &str) {
        match self.store.delete(id) {
            Ok(_) => {
                // Reload and send updated list
                self.handle_load();
            }
            Err(e) => {
                log::error!("Failed to delete history entry: {e}");
            }
        }
    }

    fn handle_clear(&self) {
        match self.store.clear_all() {
            Ok(_) => {
                signals::HistoryLoaded {
                    entries: vec![],
                }
                .send_signal_to_dart();
            }
            Err(e) => {
                log::error!("Failed to clear history: {e}");
            }
        }
    }

    /// Insert a new entry (called by pipeline actor).
    pub fn insert(&self, entry: &wrenflow_domain::history::HistoryEntry) {
        if let Err(e) = self.store.insert(entry) {
            log::error!("Failed to insert history entry: {e}");
        }
        // Trim to keep max 50 entries
        if let Err(e) = self.store.trim(50) {
            log::error!("Failed to trim history: {e}");
        }
    }
}

/// Get the default history database path for the current platform.
pub fn default_history_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join("Library/Application Support/Wrenflow")
            .join("history.sqlite")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(appdata)
            .join("Wrenflow")
            .join("history.sqlite")
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".local/share/wrenflow")
            .join("history.sqlite")
    }
}
