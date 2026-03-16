use dioxus::prelude::*;
use wrenflow_core::config::AppConfig;
use wrenflow_core::http_client;

use crate::components::*;

/// Setup step enum — mirrors Swift's SetupStep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SetupStep {
    Welcome,
    TranscriptionProvider,
    ApiKey,
    Hotkey,
    Vocabulary,
    Ready,
}

impl SetupStep {
    fn index(self) -> usize {
        match self {
            Self::Welcome => 0,
            Self::TranscriptionProvider => 1,
            Self::ApiKey => 2,
            Self::Hotkey => 3,
            Self::Vocabulary => 4,
            Self::Ready => 5,
        }
    }

    fn count() -> usize {
        6
    }

    fn next(self) -> Option<Self> {
        match self {
            Self::Welcome => Some(Self::TranscriptionProvider),
            Self::TranscriptionProvider => Some(Self::ApiKey),
            Self::ApiKey => Some(Self::Hotkey),
            Self::Hotkey => Some(Self::Vocabulary),
            Self::Vocabulary => Some(Self::Ready),
            Self::Ready => None,
        }
    }

    fn prev(self) -> Option<Self> {
        match self {
            Self::Welcome => None,
            Self::TranscriptionProvider => Some(Self::Welcome),
            Self::ApiKey => Some(Self::TranscriptionProvider),
            Self::Hotkey => Some(Self::ApiKey),
            Self::Vocabulary => Some(Self::Hotkey),
            Self::Ready => Some(Self::Vocabulary),
        }
    }
}

/// Setup wizard — guides user through initial configuration.
#[component]
pub fn SetupWizard(
    config: Signal<AppConfig>,
    api_key: Signal<String>,
    on_complete: EventHandler<()>,
) -> Element {
    let mut step = use_signal(|| SetupStep::Welcome);
    let current = *step.read();

    rsx! {
        div { class: "wizard-container",
            // Progress dots
            div { class: "wizard-progress",
                for i in 0..SetupStep::count() {
                    div {
                        class: {
                            if i == current.index() {
                                "progress-dot active"
                            } else if i < current.index() {
                                "progress-dot completed"
                            } else {
                                "progress-dot"
                            }
                        },
                    }
                }
            }

            // Step content
            match current {
                SetupStep::Welcome => rsx! { WelcomeStep {} },
                SetupStep::TranscriptionProvider => rsx! {
                    TranscriptionProviderStep { config }
                },
                SetupStep::ApiKey => rsx! {
                    ApiKeyStep { config, api_key }
                },
                SetupStep::Hotkey => rsx! {
                    HotkeyStep { config }
                },
                SetupStep::Vocabulary => rsx! {
                    VocabularyStep { config }
                },
                SetupStep::Ready => rsx! { ReadyStep {} },
            }

            // Navigation
            div { class: "wizard-footer",
                if let Some(prev) = current.prev() {
                    button {
                        class: "btn",
                        onclick: move |_| step.set(prev),
                        "\u{2190} Back"
                    }
                } else {
                    div {}
                }

                if current == SetupStep::Ready {
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| on_complete.call(()),
                        "Get Started \u{2192}"
                    }
                } else if let Some(next) = current.next() {
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| step.set(next),
                        "Continue \u{2192}"
                    }
                }
            }
        }
    }
}

// --- Step components ---

#[component]
fn WelcomeStep() -> Element {
    rsx! {
        div { class: "wizard-header",
            div {
                style: "font-size: 48px; margin-bottom: 16px;",
                "\u{1f399}"
            }
            h1 { class: "wizard-title", "Welcome to Wrenflow" }
            p { class: "wizard-subtitle",
                "Hold a key to record, release to transcribe. Let\u{2019}s get you set up."
            }
        }
    }
}

#[component]
fn TranscriptionProviderStep(config: Signal<AppConfig>) -> Element {
    let provider = config.read().transcription_provider.clone();

    rsx! {
        div {
            div { class: "wizard-header",
                h1 { class: "wizard-title", "Transcription" }
                p { class: "wizard-subtitle", "Choose how speech is converted to text." }
            }

            div { class: "flex-col",
                RadioOption {
                    label: "Local (Parakeet)".to_string(),
                    description: "Runs entirely on your device. No API key needed. Downloads a ~500MB model.".to_string(),
                    selected: provider == "local",
                    on_click: move |_| {
                        config.write().transcription_provider = "local".to_string();
                    },
                }
                RadioOption {
                    label: "Cloud (Groq Whisper)".to_string(),
                    description: "Fast, accurate cloud transcription. Requires a free Groq API key.".to_string(),
                    selected: provider == "groq",
                    on_click: move |_| {
                        config.write().transcription_provider = "groq".to_string();
                    },
                }
            }
        }
    }
}

#[component]
fn ApiKeyStep(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    let mut key_input = use_signal(|| api_key.read().clone());
    let mut is_validating = use_signal(|| false);
    let mut validation_result = use_signal(|| None::<bool>);
    let mut validation_error = use_signal(|| None::<String>);

    rsx! {
        div {
            div { class: "wizard-header",
                h1 { class: "wizard-title", "API Key" }
                p { class: "wizard-subtitle",
                    "Enter your Groq API key for cloud transcription and post-processing. You can skip this and add it later."
                }
            }

            div { class: "flex-col",
                div { class: "input-row",
                    input {
                        class: "input input-mono",
                        r#type: "password",
                        placeholder: "gsk_...",
                        value: "{key_input}",
                        disabled: *is_validating.read(),
                        oninput: move |evt: Event<FormData>| {
                            key_input.set(evt.value().clone());
                            validation_result.set(None);
                            validation_error.set(None);
                        },
                    }
                }

                button {
                    class: "btn btn-primary",
                    disabled: key_input.read().trim().is_empty() || *is_validating.read(),
                    onclick: {
                        let base_url = config.read().api_base_url.clone();
                        move |_| {
                            let key = key_input.read().trim().to_string();
                            let base_url = base_url.clone();
                            is_validating.set(true);
                            validation_result.set(None);
                            validation_error.set(None);
                            spawn(async move {
                                match http_client::build_client() {
                                    Ok(client) => {
                                        let valid = http_client::validate_api_key(&client, &key, &base_url).await;
                                        is_validating.set(false);
                                        if valid {
                                            api_key.set(key);
                                            validation_result.set(Some(true));
                                        } else {
                                            validation_error.set(Some("Invalid API key.".to_string()));
                                        }
                                    }
                                    Err(e) => {
                                        is_validating.set(false);
                                        validation_error.set(Some(format!("{e}")));
                                    }
                                }
                            });
                        }
                    },
                    if *is_validating.read() {
                        span { class: "flex-row",
                            Spinner {}
                            "Validating..."
                        }
                    } else {
                        "Validate & Save"
                    }
                }

                if let Some(true) = *validation_result.read() {
                    span { class: "badge badge-green", "\u{2713} API key saved" }
                }
                if let Some(ref err) = *validation_error.read() {
                    span { class: "badge badge-red", "\u{2717} {err}" }
                }
            }
        }
    }
}

#[component]
fn HotkeyStep(config: Signal<AppConfig>) -> Element {
    let selected = config.read().selected_hotkey.clone();

    rsx! {
        div {
            div { class: "wizard-header",
                h1 { class: "wizard-title", "Push-to-Talk Key" }
                p { class: "wizard-subtitle", "Hold this key to record, release to transcribe." }
            }

            div { class: "flex-col",
                for &(value, label, desc) in super::settings::HOTKEY_OPTIONS {
                    RadioOption {
                        label: label.to_string(),
                        description: desc.to_string(),
                        selected: selected == value,
                        on_click: {
                            let value = value.to_string();
                            move |_| {
                                config.write().selected_hotkey = value.clone();
                            }
                        },
                    }
                }

                if selected == "fn" {
                    p { class: "caption text-orange",
                        "Tip: If Fn opens Emoji picker, go to System Settings > Keyboard and change \"Press fn key to\" to \"Do Nothing\"."
                    }
                }
            }
        }
    }
}

#[component]
fn VocabularyStep(config: Signal<AppConfig>) -> Element {
    let vocab = config.read().custom_vocabulary.clone();

    rsx! {
        div {
            div { class: "wizard-header",
                h1 { class: "wizard-title", "Custom Vocabulary" }
                p { class: "wizard-subtitle",
                    "Add words and phrases to preserve during post-processing. You can update these later in Settings."
                }
            }

            div { class: "flex-col",
                textarea {
                    class: "textarea",
                    rows: "5",
                    placeholder: "e.g. Wrenflow, Parakeet, gRPC",
                    value: "{vocab}",
                    oninput: move |evt: Event<FormData>| {
                        config.write().custom_vocabulary = evt.value().trim().to_string();
                    },
                }
                p { class: "caption", "Separate entries with commas, new lines, or semicolons." }
            }
        }
    }
}

#[component]
fn ReadyStep() -> Element {
    rsx! {
        div { class: "wizard-header",
            div {
                style: "font-size: 48px; margin-bottom: 16px;",
                "\u{2705}"
            }
            h1 { class: "wizard-title", "You\u{2019}re All Set!" }
            p { class: "wizard-subtitle",
                "Wrenflow is ready to use. Hold your hotkey to record, release to transcribe."
            }
            p { class: "caption mt-md",
                "You can change any of these settings later from the Settings window."
            }
        }
    }
}
