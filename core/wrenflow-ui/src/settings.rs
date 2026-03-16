use dioxus::prelude::*;
use wrenflow_core::config::AppConfig;
use wrenflow_core::http_client;
use wrenflow_core::models;

use crate::components::*;

/// General settings screen — transcription provider, API key, hotkey, etc.
#[component]
pub fn GeneralSettings(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    rsx! {
        div { class: "flex-col",
            TranscriptionProviderCard { config }
            ApiKeyCard { config, api_key }
            PostProcessingCard { config, api_key }
            HotkeyCard { config }
            VocabularyCard { config }
        }
    }
}

// --- Transcription Provider ---

#[component]
fn TranscriptionProviderCard(config: Signal<AppConfig>) -> Element {
    let provider = config.read().transcription_provider.clone();
    let is_local = provider == "local";

    rsx! {
        SettingsCard { title: "Transcription".to_string(), icon: "\u{1f399}".to_string(),
            div { class: "flex-col",
                SegmentedControl {
                    options: vec![
                        ("local".to_string(), "Local (Parakeet)".to_string()),
                        ("groq".to_string(), "Cloud (Groq)".to_string()),
                    ],
                    selected: provider.clone(),
                    on_change: move |val: String| {
                        config.write().transcription_provider = val;
                    },
                }
                p { class: "caption",
                    if is_local {
                        "Speech-to-text runs entirely on your device using the Parakeet model. No API key required."
                    } else {
                        "Uses Groq's Whisper API for fast, accurate cloud transcription. Requires an API key."
                    }
                }
            }
        }
    }
}

// --- API Key ---

#[component]
fn ApiKeyCard(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    let mut key_input = use_signal(|| api_key.read().clone());
    let mut base_url_input = use_signal(|| config.read().api_base_url.clone());
    let mut is_validating = use_signal(|| false);
    let mut validation_result = use_signal(|| None::<bool>);
    let mut validation_error = use_signal(|| None::<String>);

    rsx! {
        SettingsCard { title: "API Key".to_string(), icon: "\u{1f511}".to_string(),
            div { class: "flex-col",
                p { class: "caption",
                    "Used for Groq transcription (when selected) and text cleanup (post-processing)."
                }

                div { class: "input-row",
                    input {
                        class: "input input-mono",
                        r#type: "password",
                        placeholder: "Enter your Groq API key",
                        value: "{key_input}",
                        disabled: *is_validating.read(),
                        oninput: move |evt: Event<FormData>| {
                            key_input.set(evt.value().clone());
                            validation_result.set(None);
                            validation_error.set(None);
                        },
                    }
                    button {
                        class: "btn btn-primary",
                        disabled: key_input.read().trim().is_empty() || *is_validating.read(),
                        onclick: move |_| {
                            let key = key_input.read().trim().to_string();
                            let base_url = base_url_input.read().trim().to_string();
                            let base_url = if base_url.is_empty() {
                                http_client::GROQ_BASE_URL.to_string()
                            } else {
                                base_url
                            };
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
                                            validation_error.set(Some("Invalid API key. Please check and try again.".to_string()));
                                        }
                                    }
                                    Err(e) => {
                                        is_validating.set(false);
                                        validation_error.set(Some(format!("HTTP client error: {e}")));
                                    }
                                }
                            });
                        },
                        if *is_validating.read() {
                            "Validating..."
                        } else {
                            "Save"
                        }
                    }
                }

                if let Some(true) = *validation_result.read() {
                    span { class: "badge badge-green", "\u{2713} API key saved" }
                }
                if let Some(ref err) = *validation_error.read() {
                    span { class: "badge badge-red", "\u{2717} {err}" }
                }

                Divider {}

                p { class: "caption-bold", "API Base URL" }
                p { class: "caption", "Change this to use a different OpenAI-compatible API provider." }

                div { class: "input-row",
                    input {
                        class: "input input-mono",
                        placeholder: "https://api.groq.com/openai/v1",
                        value: "{base_url_input}",
                        oninput: move |evt: Event<FormData>| {
                            let val = evt.value().clone();
                            base_url_input.set(val.clone());
                            let trimmed = val.trim().to_string();
                            if !trimmed.is_empty() {
                                config.write().api_base_url = trimmed;
                            }
                        },
                    }
                    button {
                        class: "btn",
                        onclick: move |_| {
                            let default = http_client::GROQ_BASE_URL.to_string();
                            base_url_input.set(default.clone());
                            config.write().api_base_url = default;
                        },
                        "Reset"
                    }
                }
            }
        }
    }
}

// --- Post-Processing ---

#[component]
fn PostProcessingCard(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    let key = api_key.read().clone();
    if key.trim().is_empty() {
        return rsx! {};
    }

    let enabled = config.read().post_processing_enabled;

    rsx! {
        SettingsCard { title: "Post-Processing".to_string(), icon: "\u{2728}".to_string(),
            div { class: "flex-col",
                Toggle {
                    label: "Enable LLM post-processing".to_string(),
                    checked: enabled,
                    on_change: move |val: bool| {
                        config.write().post_processing_enabled = val;
                    },
                }
                p { class: "caption",
                    "When enabled, an LLM cleans up transcriptions using screen context. When disabled, raw transcription is pasted directly."
                }
            }
        }

        if enabled {
            PostProcessingModelCard { config, api_key }
        }
    }
}

#[component]
fn PostProcessingModelCard(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    let mut available_models = use_signal(Vec::<models::GroqModel>::new);
    let mut is_fetching = use_signal(|| false);
    let mut fetch_failed = use_signal(|| false);

    // Auto-fetch models on mount
    let key = api_key.read().clone();
    let base_url = config.read().api_base_url.clone();
    use_effect(move || {
        let key = key.clone();
        let base_url = base_url.clone();
        if !key.trim().is_empty() && available_models.read().is_empty() {
            spawn(async move {
                is_fetching.set(true);
                fetch_failed.set(false);
                match http_client::build_client() {
                    Ok(client) => {
                        match models::fetch_models(&client, &key, &base_url).await {
                            Ok(m) => {
                                if m.is_empty() {
                                    fetch_failed.set(true);
                                } else {
                                    available_models.set(m);
                                }
                            }
                            Err(_) => fetch_failed.set(true),
                        }
                    }
                    Err(_) => fetch_failed.set(true),
                }
                is_fetching.set(false);
            });
        }
    });

    let model = config.read().post_processing_model.clone();

    rsx! {
        SettingsCard { title: "Post-Processing Model".to_string(), icon: "\u{1f4bb}".to_string(),
            div { class: "flex-col",
                p { class: "caption", "The LLM used to clean up raw transcriptions." }

                if *is_fetching.read() {
                    div { class: "flex-row",
                        Spinner {}
                        span { class: "caption", "Loading models..." }
                    }
                } else if !available_models.read().is_empty() {
                    select {
                        class: "select",
                        value: "{model}",
                        onchange: move |evt: Event<FormData>| {
                            config.write().post_processing_model = evt.value().clone();
                        },
                        for m in available_models.read().iter() {
                            option { value: "{m.id}", "{m.id}" }
                        }
                    }
                } else {
                    div { class: "input-row",
                        input {
                            class: "input input-mono",
                            placeholder: "Model ID",
                            value: "{model}",
                            oninput: move |evt: Event<FormData>| {
                                config.write().post_processing_model = evt.value().clone();
                            },
                        }
                        button {
                            class: "btn",
                            onclick: {
                                let key = api_key.read().clone();
                                let base_url = config.read().api_base_url.clone();
                                move |_| {
                                    let key = key.clone();
                                    let base_url = base_url.clone();
                                    is_fetching.set(true);
                                    fetch_failed.set(false);
                                    spawn(async move {
                                        match http_client::build_client() {
                                            Ok(client) => {
                                                match models::fetch_models(&client, &key, &base_url).await {
                                                    Ok(m) => {
                                                        if m.is_empty() {
                                                            fetch_failed.set(true);
                                                        } else {
                                                            available_models.set(m);
                                                        }
                                                    }
                                                    Err(_) => fetch_failed.set(true),
                                                }
                                            }
                                            Err(_) => fetch_failed.set(true),
                                        }
                                        is_fetching.set(false);
                                    });
                                }
                            },
                            "Fetch Models"
                        }
                    }
                    if *fetch_failed.read() {
                        p { class: "caption", "Could not load model list. You can type a model ID manually." }
                    }
                }
            }
        }
    }
}

// --- Hotkey ---

pub(crate) const HOTKEY_OPTIONS: &[(&str, &str, &str)] = &[
    ("fn", "Fn Key", "Hold Fn to record"),
    ("ctrl+shift+space", "Ctrl+Shift+Space", "Hold Ctrl+Shift+Space to record"),
    ("f13", "F13", "Hold F13 to record"),
    ("f14", "F14", "Hold F14 to record"),
    ("f15", "F15", "Hold F15 to record"),
];

#[component]
fn HotkeyCard(config: Signal<AppConfig>) -> Element {
    let selected = config.read().selected_hotkey.clone();
    let min_duration = config.read().minimum_recording_duration_ms;

    rsx! {
        SettingsCard { title: "Push-to-Talk Key".to_string(), icon: "\u{2328}".to_string(),
            div { class: "flex-col",
                p { class: "caption", "Hold this key to record, release to transcribe." }

                div { class: "flex-col gap-sm",
                    for &(value, label, desc) in HOTKEY_OPTIONS {
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
                }

                if selected == "fn" {
                    p { class: "caption text-orange",
                        "Tip: If Fn opens Emoji picker, go to System Settings > Keyboard and change \"Press fn key to\" to \"Do Nothing\"."
                    }
                }

                Divider {}

                div { class: "flex-col",
                    div { class: "flex-row",
                        span { "Minimum recording duration" }
                        span { class: "spacer" }
                        span { class: "text-secondary", "{min_duration:.0}ms" }
                    }
                    input {
                        class: "slider",
                        r#type: "range",
                        min: "50",
                        max: "500",
                        step: "50",
                        value: "{min_duration}",
                        oninput: move |evt: Event<FormData>| {
                            if let Ok(val) = evt.value().parse::<f64>() {
                                config.write().minimum_recording_duration_ms = val;
                            }
                        },
                    }
                    p { class: "caption",
                        "Recordings shorter than this are treated as accidental and cancelled."
                    }
                }
            }
        }
    }
}

// --- Custom Vocabulary ---

#[component]
fn VocabularyCard(config: Signal<AppConfig>) -> Element {
    let vocab = config.read().custom_vocabulary.clone();

    rsx! {
        SettingsCard { title: "Custom Vocabulary".to_string(), icon: "\u{1f4d6}".to_string(),
            div { class: "flex-col",
                p { class: "caption", "Words and phrases to preserve during post-processing." }

                textarea {
                    class: "textarea",
                    rows: "4",
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
