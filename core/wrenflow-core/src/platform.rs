//! Platform host abstraction for settings UI.
//!
//! The native shell (macOS, Windows, Android) implements this trait
//! and provides it to wrenflow-ui via Dioxus context.
//! All methods have default no-op implementations so platforms only
//! override what they support.

// ---------------------------------------------------------------------------
// Shared data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LocalModelState {
    NotLoaded,
    Downloading { progress: Option<f64> },
    Compiling,
    Ready,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionStatus {
    Granted,
    NotGranted,
    /// Platform doesn't have this permission concept.
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    Available { version: String },
    Downloading { progress: Option<f64> },
    Installing,
    ReadyToRelaunch,
    Error(String),
    UpToDate,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CliToolStatus {
    pub installed: bool,
    pub path: Option<String>,
}

// ---------------------------------------------------------------------------
// Capability flags
// ---------------------------------------------------------------------------

/// Which optional platform features are supported.
#[derive(Debug, Clone, Default)]
pub struct PlatformCapabilities {
    pub launch_at_login: bool,
    pub updates: bool,
    pub local_transcription: bool,
    pub microphone_selection: bool,
    pub permissions: bool,
    pub cli_tool: bool,
}

// ---------------------------------------------------------------------------
// PlatformHost trait
// ---------------------------------------------------------------------------

/// Callback interface for platform-specific settings operations.
/// The native layer implements this trait and passes it into the Dioxus
/// app via `provide_context`.
pub trait PlatformHost: Send + Sync + 'static {
    /// Declare which optional features this platform supports.
    fn capabilities(&self) -> PlatformCapabilities;

    // -- Launch at Login --

    fn get_launch_at_login(&self) -> bool { false }
    fn set_launch_at_login(&self, _enabled: bool) {}
    /// macOS: SMAppService may require approval in System Settings.
    fn launch_at_login_requires_approval(&self) -> bool { false }
    fn open_launch_at_login_settings(&self) {}

    // -- Updates --

    fn get_auto_check_updates(&self) -> bool { true }
    fn set_auto_check_updates(&self, _enabled: bool) {}
    fn check_for_updates(&self) {}
    fn get_update_status(&self) -> UpdateStatus { UpdateStatus::Idle }
    fn download_and_install_update(&self) {}
    fn cancel_update_download(&self) {}

    // -- Local Transcription Model --

    fn get_local_model_state(&self) -> LocalModelState { LocalModelState::NotLoaded }
    fn load_local_model(&self) {}
    fn retry_local_model(&self) {}

    // -- Microphone --

    fn list_microphones(&self) -> Vec<AudioDevice> { vec![] }
    fn refresh_microphones(&self) {}

    // -- Permissions --

    fn get_microphone_permission(&self) -> PermissionStatus { PermissionStatus::NotApplicable }
    fn request_microphone_permission(&self) {}

    fn get_accessibility_permission(&self) -> PermissionStatus { PermissionStatus::NotApplicable }
    fn request_accessibility_permission(&self) {}

    fn get_screen_recording_permission(&self) -> PermissionStatus { PermissionStatus::NotApplicable }
    fn request_screen_recording_permission(&self) {}

    // -- CLI Tool --

    fn get_cli_status(&self) -> CliToolStatus { CliToolStatus { installed: false, path: None } }
    fn install_cli(&self) {}
}

// ---------------------------------------------------------------------------
// Stub (all capabilities disabled)
// ---------------------------------------------------------------------------

/// No-op implementation for standalone dev/testing.
pub struct StubPlatformHost;

impl PlatformHost for StubPlatformHost {
    fn capabilities(&self) -> PlatformCapabilities {
        PlatformCapabilities::default()
    }
}
