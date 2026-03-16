use std::sync::Arc;
use dioxus::prelude::*;
use wrenflow_core::config::AppConfig;
use wrenflow_core::http_client;
use wrenflow_core::models;
use wrenflow_core::platform::PlatformHost;

use crate::components::*;
use crate::platform_cards::*;

#[component]
pub fn GeneralSettings(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    let host = use_context::<Arc<dyn PlatformHost>>();
    let caps = host.capabilities();
    let is_local = config.read().transcription_provider == "local";

    rsx! {
        div { class: "flex flex-col gap-1.5",
            if caps.launch_at_login { LaunchAtLoginCard {} }
            if caps.updates { UpdatesCard {} }
            TranscriptionCard { config }
            if caps.local_transcription && is_local { LocalTranscriptionCard {} }
            ApiKeyCard { config, api_key }
            PostProcessingCard { config, api_key }
            HotkeyCard { config }
            if caps.microphone_selection { MicrophoneCard { config } }
            VocabularyCard { config }
            if caps.permissions { PermissionsCard {} }
            if caps.cli_tool { CliToolCard {} }
        }
    }
}

#[component]
fn TranscriptionCard(config: Signal<AppConfig>) -> Element {
    let provider = config.read().transcription_provider.clone();
    let is_local = provider == "local";
    rsx! {
        Card { title: "Transcription".to_string(),
            div { class: "flex flex-col gap-1.5",
                Segmented {
                    options: vec![
                        ("local".to_string(), "Local (Parakeet)".to_string()),
                        ("groq".to_string(), "Cloud (Groq)".to_string()),
                    ],
                    selected: provider,
                    on_change: move |v: String| { config.write().transcription_provider = v; },
                }
                p { class: "text-[11px] text-ash-500",
                    if is_local { "Runs on-device via Parakeet. No API key needed." }
                    else { "Groq Whisper API. Requires API key." }
                }
            }
        }
    }
}

#[component]
fn ApiKeyCard(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    let mut key_input = use_signal(|| api_key.read().clone());
    let mut base_url_input = use_signal(|| config.read().api_base_url.clone());
    let mut validating = use_signal(|| false);
    let mut result = use_signal(|| None::<bool>);
    let mut error = use_signal(|| None::<String>);

    rsx! {
        Card { title: "API Key".to_string(),
            div { class: "flex flex-col gap-1.5",
                p { class: "text-[11px] text-ash-500", "For Groq transcription and post-processing." }
                div { class: "flex gap-1.5 items-center",
                    input {
                        class: "flex-1 h-7 px-2 bg-mint-50 border border-ash-200 rounded text-xs font-mono outline-none focus:border-frost-700",
                        r#type: "password",
                        placeholder: "Groq API key",
                        value: "{key_input}",
                        disabled: *validating.read(),
                        oninput: move |e: Event<FormData>| {
                            key_input.set(e.value()); result.set(None); error.set(None);
                        },
                    }
                    button {
                        class: "h-7 px-2.5 rounded text-[11px] font-medium bg-frost-700 text-white border border-frost-700 hover:opacity-90 disabled:opacity-35",
                        disabled: key_input.read().trim().is_empty() || *validating.read(),
                        onclick: move |_| {
                            let key = key_input.read().trim().to_string();
                            let url = base_url_input.read().trim().to_string();
                            let url = if url.is_empty() { http_client::GROQ_BASE_URL.to_string() } else { url };
                            validating.set(true); result.set(None); error.set(None);
                            spawn(async move {
                                match http_client::build_client() {
                                    Ok(c) => {
                                        let ok = http_client::validate_api_key(&c, &key, &url).await;
                                        validating.set(false);
                                        if ok { api_key.set(key); result.set(Some(true)); }
                                        else { error.set(Some("Invalid API key.".into())); }
                                    }
                                    Err(e) => { validating.set(false); error.set(Some(format!("{e}"))); }
                                }
                            });
                        },
                        if *validating.read() { "..." } else { "Save" }
                    }
                }
                if let Some(true) = *result.read() {
                    span { class: "text-[11px] text-frost-700", "✓ Saved" }
                }
                if let Some(ref e) = *error.read() {
                    span { class: "text-[11px] text-red-600", "✗ {e}" }
                }
                div { class: "h-px bg-ash-200 my-1" }
                p { class: "text-[11px] font-semibold", "API Base URL" }
                div { class: "flex gap-1.5 items-center",
                    input {
                        class: "flex-1 h-7 px-2 bg-mint-50 border border-ash-200 rounded text-xs font-mono outline-none focus:border-frost-700",
                        placeholder: "https://api.groq.com/openai/v1",
                        value: "{base_url_input}",
                        oninput: move |e: Event<FormData>| {
                            let v = e.value(); base_url_input.set(v.clone());
                            let t = v.trim().to_string();
                            if !t.is_empty() { config.write().api_base_url = t; }
                        },
                    }
                    button {
                        class: "h-7 px-2.5 rounded text-[11px] font-medium border border-ash-200 bg-white hover:bg-mint-50",
                        onclick: move |_| {
                            let d = http_client::GROQ_BASE_URL.to_string();
                            base_url_input.set(d.clone()); config.write().api_base_url = d;
                        },
                        "Reset"
                    }
                }
            }
        }
    }
}

#[component]
fn PostProcessingCard(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    if api_key.read().trim().is_empty() { return rsx! {}; }
    let enabled = config.read().post_processing_enabled;
    rsx! {
        Card { title: "Post-Processing".to_string(),
            div { class: "flex flex-col gap-1.5",
                Toggle {
                    label: "Enable LLM cleanup".to_string(),
                    checked: enabled,
                    on_change: move |v: bool| { config.write().post_processing_enabled = v; },
                }
            }
        }
        if enabled { ModelCard { config, api_key } }
    }
}

#[component]
fn ModelCard(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    let mut models_list = use_signal(Vec::<models::GroqModel>::new);
    let mut fetching = use_signal(|| false);
    let mut failed = use_signal(|| false);
    let key = api_key.read().clone();
    let base_url = config.read().api_base_url.clone();
    use_effect(move || {
        let key = key.clone(); let base_url = base_url.clone();
        if !key.trim().is_empty() && models_list.read().is_empty() {
            spawn(async move {
                fetching.set(true); failed.set(false);
                if let Ok(c) = http_client::build_client() {
                    match models::fetch_models(&c, &key, &base_url).await {
                        Ok(m) if !m.is_empty() => models_list.set(m),
                        _ => failed.set(true),
                    }
                } else { failed.set(true); }
                fetching.set(false);
            });
        }
    });
    let model = config.read().post_processing_model.clone();
    rsx! {
        Card { title: "Model".to_string(),
            div { class: "flex flex-col gap-1.5",
                if *fetching.read() {
                    div { class: "flex items-center gap-1.5", Spinner {} span { class: "text-[11px] text-ash-500", "Loading..." } }
                } else if !models_list.read().is_empty() {
                    select {
                        class: "h-7 px-2 bg-mint-50 border border-ash-200 rounded text-xs outline-none focus:border-frost-700",
                        value: "{model}",
                        onchange: move |e: Event<FormData>| { config.write().post_processing_model = e.value(); },
                        for m in models_list.read().iter() { option { value: "{m.id}", "{m.id}" } }
                    }
                } else {
                    div { class: "flex gap-1.5 items-center",
                        input {
                            class: "flex-1 h-7 px-2 bg-mint-50 border border-ash-200 rounded text-xs font-mono outline-none focus:border-frost-700",
                            placeholder: "Model ID", value: "{model}",
                            oninput: move |e: Event<FormData>| { config.write().post_processing_model = e.value(); },
                        }
                        button {
                            class: "h-7 px-2.5 rounded text-[11px] font-medium border border-ash-200 bg-white hover:bg-mint-50",
                            onclick: {
                                let key = api_key.read().clone(); let base_url = config.read().api_base_url.clone();
                                move |_| {
                                    let key = key.clone(); let base_url = base_url.clone();
                                    fetching.set(true); failed.set(false);
                                    spawn(async move {
                                        if let Ok(c) = http_client::build_client() {
                                            match models::fetch_models(&c, &key, &base_url).await {
                                                Ok(m) if !m.is_empty() => models_list.set(m), _ => failed.set(true),
                                            }
                                        } else { failed.set(true); }
                                        fetching.set(false);
                                    });
                                }
                            },
                            "Fetch"
                        }
                    }
                }
            }
        }
    }
}

pub(crate) const HOTKEY_OPTIONS: &[(&str, &str, &str)] = &[
    ("fn", "Fn / Globe Key", "Hold Fn to record"),
    ("rightOption", "Right Option Key", "Hold Right Option to record"),
    ("f5", "F5 Key", "Hold F5 to record"),
];

#[component]
fn HotkeyCard(config: Signal<AppConfig>) -> Element {
    let selected = config.read().selected_hotkey.clone();
    let min_dur = config.read().minimum_recording_duration_ms;
    rsx! {
        Card { title: "Push-to-Talk Key".to_string(),
            div { class: "flex flex-col gap-1.5",
                div { class: "flex flex-col gap-1",
                    for &(val, label, desc) in HOTKEY_OPTIONS {
                        RadioOption {
                            label: label.to_string(), description: desc.to_string(),
                            selected: selected == val,
                            on_click: { let val = val.to_string(); move |_| { config.write().selected_hotkey = val.clone(); } },
                        }
                    }
                }
                if selected == "fn" {
                    p { class: "text-[11px] text-orange-600",
                        "If Fn opens Emoji picker, change it to \"Do Nothing\" in System Settings → Keyboard."
                    }
                }
                div { class: "h-px bg-ash-200 my-1" }
                div { class: "flex items-center justify-between text-xs",
                    span { "Min duration" }
                    span { class: "text-ash-500 font-mono", "{min_dur:.0}ms" }
                }
                input {
                    class: "w-full accent-frost-700",
                    r#type: "range", min: "50", max: "500", step: "50", value: "{min_dur}",
                    oninput: move |e: Event<FormData>| {
                        if let Ok(v) = e.value().parse::<f64>() { config.write().minimum_recording_duration_ms = v; }
                    },
                }
            }
        }
    }
}

#[component]
fn VocabularyCard(config: Signal<AppConfig>) -> Element {
    let vocab = config.read().custom_vocabulary.clone();
    rsx! {
        Card { title: "Custom Vocabulary".to_string(),
            div { class: "flex flex-col gap-1.5",
                textarea {
                    class: "w-full p-2 bg-mint-50 border border-ash-200 rounded text-xs font-mono leading-snug outline-none focus:border-frost-700 resize-y min-h-14",
                    rows: "3", value: "{vocab}",
                    oninput: move |e: Event<FormData>| { config.write().custom_vocabulary = e.value().trim().to_string(); },
                }
                p { class: "text-[11px] text-ash-500", "Comma, newline, or semicolon separated." }
            }
        }
    }
}
