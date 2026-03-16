use dioxus::prelude::*;
use wrenflow_core::config::AppConfig;
use wrenflow_core::http_client;

use crate::components::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step { Welcome, Provider, ApiKey, Hotkey, Vocab, Ready }

impl Step {
    fn idx(self) -> usize { match self { Self::Welcome=>0, Self::Provider=>1, Self::ApiKey=>2, Self::Hotkey=>3, Self::Vocab=>4, Self::Ready=>5 } }
    fn count() -> usize { 6 }
    fn next(self) -> Option<Self> { match self { Self::Welcome=>Some(Self::Provider), Self::Provider=>Some(Self::ApiKey), Self::ApiKey=>Some(Self::Hotkey), Self::Hotkey=>Some(Self::Vocab), Self::Vocab=>Some(Self::Ready), Self::Ready=>None } }
    fn prev(self) -> Option<Self> { match self { Self::Welcome=>None, Self::Provider=>Some(Self::Welcome), Self::ApiKey=>Some(Self::Provider), Self::Hotkey=>Some(Self::ApiKey), Self::Vocab=>Some(Self::Hotkey), Self::Ready=>Some(Self::Vocab) } }
}

#[component]
pub fn SetupWizard(config: Signal<AppConfig>, api_key: Signal<String>, on_complete: EventHandler<()>) -> Element {
    let mut step = use_signal(|| Step::Welcome);
    let cur = *step.read();
    let icon_url = crate::icon_data_url();

    rsx! {
        div { class: "max-w-md mx-auto py-8 px-4",
            // Progress dots
            div { class: "flex gap-1 justify-center mb-5",
                for i in 0..Step::count() {
                    {
                        let cls = if i == cur.idx() { "w-1.5 h-1.5 rounded-full bg-frost-700" }
                            else if i < cur.idx() { "w-1.5 h-1.5 rounded-full bg-ash-400" }
                            else { "w-1.5 h-1.5 rounded-full bg-ash-200" };
                        rsx! { div { class: cls } }
                    }
                }
            }
            // Icon
            div { class: "flex justify-center mb-4",
                img { class: "w-10 h-10 opacity-40", src: "{icon_url}", alt: "Wrenflow" }
            }
            // Content
            match cur {
                Step::Welcome => rsx! {
                    div { class: "text-center mb-5",
                        h1 { class: "text-base font-semibold mb-1", "Welcome to Wrenflow" }
                        p { class: "text-xs text-ash-500", "Hold a key to record, release to transcribe." }
                    }
                },
                Step::Provider => rsx! {
                    div { class: "text-center mb-4",
                        h1 { class: "text-base font-semibold mb-1", "Transcription" }
                        p { class: "text-xs text-ash-500", "Choose speech-to-text engine." }
                    }
                    div { class: "flex flex-col gap-1.5",
                        {
                            let provider = config.read().transcription_provider.clone();
                            rsx! {
                                RadioOption { label: "Local (Parakeet)".to_string(), description: "On-device, ~500MB model download.".to_string(),
                                    selected: provider == "local",
                                    on_click: move |_| { config.write().transcription_provider = "local".into(); },
                                }
                                RadioOption { label: "Cloud (Groq)".to_string(), description: "Fast cloud API. Requires key.".to_string(),
                                    selected: provider == "groq",
                                    on_click: move |_| { config.write().transcription_provider = "groq".into(); },
                                }
                            }
                        }
                    }
                },
                Step::ApiKey => rsx! {
                    div { class: "text-center mb-4",
                        h1 { class: "text-base font-semibold mb-1", "API Key" }
                        p { class: "text-xs text-ash-500", "Optional. Skip to add later." }
                    }
                    ApiKeyWizardStep { config, api_key }
                },
                Step::Hotkey => rsx! {
                    div { class: "text-center mb-4",
                        h1 { class: "text-base font-semibold mb-1", "Push-to-Talk Key" }
                    }
                    div { class: "flex flex-col gap-1",
                        {
                            let sel = config.read().selected_hotkey.clone();
                            rsx! {
                                for &(val, label, desc) in super::settings::HOTKEY_OPTIONS {
                                    RadioOption {
                                        label: label.to_string(), description: desc.to_string(),
                                        selected: sel == val,
                                        on_click: { let val = val.to_string(); move |_| { config.write().selected_hotkey = val.clone(); } },
                                    }
                                }
                            }
                        }
                    }
                },
                Step::Vocab => rsx! {
                    div { class: "text-center mb-4",
                        h1 { class: "text-base font-semibold mb-1", "Custom Vocabulary" }
                        p { class: "text-xs text-ash-500", "Words to preserve. Update later in Settings." }
                    }
                    {
                        let vocab = config.read().custom_vocabulary.clone();
                        rsx! {
                            textarea {
                                class: "w-full p-2 bg-mint-50 border border-ash-200 rounded text-xs font-mono leading-snug outline-none focus:border-frost-700 resize-y min-h-16",
                                rows: "4", placeholder: "Wrenflow, Parakeet, gRPC", value: "{vocab}",
                                oninput: move |e: Event<FormData>| { config.write().custom_vocabulary = e.value().trim().to_string(); },
                            }
                        }
                    }
                },
                Step::Ready => rsx! {
                    div { class: "text-center mb-5",
                        h1 { class: "text-base font-semibold mb-1", "All set" }
                        p { class: "text-xs text-ash-500", "Hold your hotkey to record, release to transcribe." }
                    }
                },
            }
            // Nav
            div { class: "flex justify-between items-center mt-6",
                if let Some(prev) = cur.prev() {
                    button { class: "text-xs text-ash-500 hover:text-ash-700",
                        onclick: move |_| step.set(prev), "← Back" }
                } else { div {} }
                if cur == Step::Ready {
                    button { class: "h-7 px-3 rounded text-[11px] font-medium bg-frost-700 text-white hover:opacity-90",
                        onclick: move |_| on_complete.call(()), "Get Started →" }
                } else if let Some(next) = cur.next() {
                    button { class: "h-7 px-3 rounded text-[11px] font-medium bg-frost-700 text-white hover:opacity-90",
                        onclick: move |_| step.set(next), "Continue →" }
                }
            }
        }
    }
}

#[component]
fn ApiKeyWizardStep(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    let mut key_input = use_signal(|| api_key.read().clone());
    let mut validating = use_signal(|| false);
    let mut result = use_signal(|| None::<bool>);
    let mut error = use_signal(|| None::<String>);
    rsx! {
        div { class: "flex flex-col gap-1.5",
            input {
                class: "h-7 px-2 bg-mint-50 border border-ash-200 rounded text-xs font-mono outline-none focus:border-frost-700",
                r#type: "password", placeholder: "gsk_...", value: "{key_input}",
                disabled: *validating.read(),
                oninput: move |e: Event<FormData>| { key_input.set(e.value()); result.set(None); error.set(None); },
            }
            button {
                class: "h-7 px-3 rounded text-[11px] font-medium bg-frost-700 text-white hover:opacity-90 disabled:opacity-35 self-start",
                disabled: key_input.read().trim().is_empty() || *validating.read(),
                onclick: {
                    let base_url = config.read().api_base_url.clone();
                    move |_| {
                        let key = key_input.read().trim().to_string();
                        let url = base_url.clone();
                        validating.set(true); result.set(None); error.set(None);
                        spawn(async move {
                            match http_client::build_client() {
                                Ok(c) => {
                                    let ok = http_client::validate_api_key(&c, &key, &url).await;
                                    validating.set(false);
                                    if ok { api_key.set(key); result.set(Some(true)); }
                                    else { error.set(Some("Invalid key.".into())); }
                                }
                                Err(e) => { validating.set(false); error.set(Some(format!("{e}"))); }
                            }
                        });
                    }
                },
                if *validating.read() { "..." } else { "Validate" }
            }
            if let Some(true) = *result.read() { span { class: "text-[11px] text-frost-700", "✓ Saved" } }
            if let Some(ref e) = *error.read() { span { class: "text-[11px] text-red-600", "✗ {e}" } }
        }
    }
}
