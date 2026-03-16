use dioxus::prelude::*;
use std::sync::Arc;
use wrenflow_core::config::AppConfig;
use wrenflow_core::platform::*;

use crate::components::*;

// ---------------------------------------------------------------------------
// Launch at Login
// ---------------------------------------------------------------------------

#[component]
pub fn LaunchAtLoginCard() -> Element {
    let host = use_context::<Arc<dyn PlatformHost>>();
    let mut enabled = use_signal(|| host.get_launch_at_login());
    let needs_approval = host.launch_at_login_requires_approval();
    let h1 = host.clone();
    let h2 = host.clone();

    rsx! {
        Card { title: "Startup".to_string(),
            div { class: "flex flex-col gap-1.5",
                Toggle {
                    label: "Launch at login".to_string(),
                    checked: *enabled.read(),
                    on_change: move |v: bool| { h1.set_launch_at_login(v); enabled.set(v); },
                }
                if needs_approval {
                    div { class: "flex items-center gap-1.5",
                        span { class: "text-[11px] text-orange-600", "Requires approval in System Settings." }
                        button {
                            class: "text-[11px] text-frost-700 hover:underline",
                            onclick: move |_| h2.open_launch_at_login_settings(),
                            "Open Settings"
                        }
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Updates
// ---------------------------------------------------------------------------

#[component]
pub fn UpdatesCard() -> Element {
    let host = use_context::<Arc<dyn PlatformHost>>();
    let mut auto_check = use_signal(|| host.get_auto_check_updates());
    let mut status = use_signal(|| host.get_update_status());

    let hp = host.clone();
    use_future(move || {
        let h = hp.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                status.set(h.get_update_status());
            }
        }
    });

    let h1 = host.clone();
    let h2 = host.clone();
    let h3 = host.clone();
    let h4 = host.clone();
    let status_val = status.read().clone();
    let checking = matches!(status_val, UpdateStatus::Checking);

    rsx! {
        Card { title: "Updates".to_string(),
            div { class: "flex flex-col gap-1.5",
                Toggle {
                    label: "Check automatically".to_string(),
                    checked: *auto_check.read(),
                    on_change: move |v: bool| { h1.set_auto_check_updates(v); auto_check.set(v); },
                }
                button {
                    class: "h-7 px-2.5 rounded text-[11px] font-medium border border-ash-200 bg-white hover:bg-mint-50 disabled:opacity-35 self-start",
                    disabled: checking,
                    onclick: move |_| h2.check_for_updates(),
                    if checking { "Checking..." } else { "Check Now" }
                }
                match &status_val {
                    UpdateStatus::Available { version } => rsx! {
                        div { class: "flex items-center gap-1.5 bg-frost-50 border border-frost-200 rounded p-2",
                            span { class: "text-[11px]", "v{version} available" }
                            div { class: "flex-1" }
                            button {
                                class: "h-6 px-2 rounded text-[11px] font-medium bg-frost-700 text-white hover:opacity-90",
                                onclick: move |_| h3.download_and_install_update(),
                                "Update"
                            }
                        }
                    },
                    UpdateStatus::Downloading { progress } => rsx! {
                        div { class: "flex items-center gap-1.5",
                            Spinner {}
                            span { class: "text-[11px] text-ash-500",
                                if let Some(p) = progress { "Downloading {p:.0}%..." } else { "Downloading..." }
                            }
                            button {
                                class: "text-[11px] text-ash-500 hover:text-ash-700",
                                onclick: move |_| h4.cancel_update_download(),
                                "Cancel"
                            }
                        }
                    },
                    UpdateStatus::Installing => rsx! {
                        div { class: "flex items-center gap-1.5", Spinner {} span { class: "text-[11px] text-ash-500", "Installing..." } }
                    },
                    UpdateStatus::Error(msg) => rsx! { span { class: "text-[11px] text-red-600", "✗ {msg}" } },
                    UpdateStatus::UpToDate => rsx! { span { class: "text-[11px] text-frost-700", "✓ Up to date" } },
                    _ => rsx! {},
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Local Transcription Model
// ---------------------------------------------------------------------------

#[component]
pub fn LocalTranscriptionCard() -> Element {
    let host = use_context::<Arc<dyn PlatformHost>>();
    let mut state = use_signal(|| host.get_local_model_state());

    let hp = host.clone();
    use_future(move || {
        let h = hp.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                state.set(h.get_local_model_state());
            }
        }
    });

    let h1 = host.clone();
    let h2 = host.clone();
    let state_val = state.read().clone();

    rsx! {
        Card { title: "Local Transcription".to_string(),
            div { class: "flex flex-col gap-1.5",
                p { class: "text-[11px] text-ash-500", "On-device Parakeet model." }
                div { class: "flex items-center gap-1.5",
                    match &state_val {
                        LocalModelState::NotLoaded => rsx! {
                            span { class: "text-[11px] text-ash-500", "Not loaded" }
                            div { class: "flex-1" }
                            button {
                                class: "h-7 px-2.5 rounded text-[11px] font-medium border border-ash-200 bg-white hover:bg-mint-50",
                                onclick: move |_| h1.load_local_model(), "Load"
                            }
                        },
                        LocalModelState::Downloading { progress } => rsx! {
                            Spinner {}
                            span { class: "text-[11px] text-ash-500",
                                if let Some(p) = progress { "Downloading {p:.0}%..." } else { "Downloading..." }
                            }
                        },
                        LocalModelState::Compiling => rsx! {
                            Spinner {} span { class: "text-[11px] text-ash-500", "Compiling..." }
                        },
                        LocalModelState::Ready => rsx! { span { class: "text-[11px] text-frost-700", "✓ Ready" } },
                        LocalModelState::Error(msg) => rsx! {
                            span { class: "text-[11px] text-red-600", "✗ {msg}" }
                            div { class: "flex-1" }
                            button {
                                class: "h-7 px-2.5 rounded text-[11px] font-medium border border-ash-200 bg-white hover:bg-mint-50",
                                onclick: move |_| h2.retry_local_model(), "Retry"
                            }
                        },
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Microphone Selection
// ---------------------------------------------------------------------------

#[component]
pub fn MicrophoneCard(config: Signal<AppConfig>) -> Element {
    let host = use_context::<Arc<dyn PlatformHost>>();
    let mut devices = use_signal(|| host.list_microphones());
    let selected = config.read().selected_microphone_id.clone();
    let h1 = host.clone();

    rsx! {
        Card { title: "Microphone".to_string(),
            div { class: "flex flex-col gap-1",
                RadioOption {
                    label: "System Default".to_string(), description: String::new(),
                    selected: selected == "default" || selected.is_empty(),
                    on_click: move |_| { config.write().selected_microphone_id = "default".into(); },
                }
                for dev in devices.read().iter() {
                    {
                        let id = dev.id.clone();
                        let name = dev.name.clone();
                        let is_sel = selected == id;
                        rsx! {
                            RadioOption {
                                label: name, description: String::new(), selected: is_sel,
                                on_click: { let id = id.clone(); move |_| { config.write().selected_microphone_id = id.clone(); } },
                            }
                        }
                    }
                }
                button {
                    class: "text-[11px] text-ash-500 hover:text-ash-700 self-start mt-1",
                    onclick: move |_| { h1.refresh_microphones(); devices.set(h1.list_microphones()); },
                    "Refresh"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Permissions
// ---------------------------------------------------------------------------

#[component]
pub fn PermissionsCard() -> Element {
    let host = use_context::<Arc<dyn PlatformHost>>();
    let mut mic = use_signal(|| host.get_microphone_permission());
    let mut ax = use_signal(|| host.get_accessibility_permission());
    let mut sr = use_signal(|| host.get_screen_recording_permission());

    let hp = host.clone();
    use_future(move || {
        let h = hp.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                mic.set(h.get_microphone_permission());
                ax.set(h.get_accessibility_permission());
                sr.set(h.get_screen_recording_permission());
            }
        }
    });

    let h1 = host.clone();
    let h2 = host.clone();
    let h3 = host.clone();

    rsx! {
        Card { title: "Permissions".to_string(),
            div { class: "flex flex-col gap-1",
                PermissionRow { label: "Microphone".to_string(), status: mic.read().clone(),
                    on_request: move |_| { h1.request_microphone_permission(); },
                }
                PermissionRow { label: "Accessibility".to_string(), status: ax.read().clone(),
                    on_request: move |_| { h2.request_accessibility_permission(); },
                }
                PermissionRow { label: "Screen Recording".to_string(), status: sr.read().clone(),
                    on_request: move |_| { h3.request_screen_recording_permission(); },
                }
            }
        }
    }
}

#[component]
fn PermissionRow(label: String, status: PermissionStatus, on_request: EventHandler<()>) -> Element {
    if status == PermissionStatus::NotApplicable { return rsx! {}; }
    let granted = status == PermissionStatus::Granted;
    rsx! {
        div { class: "flex items-center gap-2 p-2 bg-mint-50 border border-ash-200 rounded",
            span { class: "text-xs", "{label}" }
            div { class: "flex-1" }
            if granted {
                span { class: "text-[11px] text-frost-700", "✓" }
            } else {
                button {
                    class: "h-6 px-2 rounded text-[11px] font-medium border border-ash-200 bg-white hover:bg-mint-50",
                    onclick: move |_| on_request.call(()), "Grant"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CLI Tool
// ---------------------------------------------------------------------------

#[component]
pub fn CliToolCard() -> Element {
    let host = use_context::<Arc<dyn PlatformHost>>();
    let mut status = use_signal(|| host.get_cli_status());
    let h1 = host.clone();

    rsx! {
        Card { title: "CLI Tool".to_string(),
            div { class: "flex flex-col gap-1.5",
                p { class: "text-[11px] text-ash-500", "Control Wrenflow from the terminal." }
                div { class: "flex items-center gap-1.5",
                    if status.read().installed {
                        { let p = status.read().path.clone().unwrap_or_default(); rsx! { span { class: "text-[11px] text-frost-700", "✓ {p}" } } }
                    } else {
                        span { class: "text-[11px] text-ash-500", "Not installed" }
                    }
                    div { class: "flex-1" }
                    button {
                        class: "h-7 px-2.5 rounded text-[11px] font-medium border border-ash-200 bg-white hover:bg-mint-50",
                        onclick: move |_| { h1.install_cli(); status.set(h1.get_cli_status()); },
                        if status.read().installed { "Reinstall" } else { "Install" }
                    }
                }
                p { class: "text-[10px] font-mono text-ash-400", "wrenflow start | stop | toggle | status" }
            }
        }
    }
}
