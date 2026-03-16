use dioxus::prelude::*;
use wrenflow_core::config::AppConfig;
use wrenflow_core::http_client;
use wrenflow_core::post_processing;

use crate::components::*;

#[component]
pub fn PromptsSettings(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1.5",
            SystemPromptCard { config, api_key }
            ContextPromptCard { config }
        }
    }
}

#[component]
fn SystemPromptCard(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    let custom = config.read().custom_system_prompt.clone();
    let is_custom = !custom.is_empty();
    let mut prompt_input = use_signal(|| {
        if is_custom { custom.clone() } else { post_processing::DEFAULT_SYSTEM_PROMPT.to_string() }
    });
    let mut show_default = use_signal(|| false);
    let mut test_input = use_signal(|| "Um, so I was like, thinking we should uh, refactor the auth module, you know?".to_string());
    let mut test_running = use_signal(|| false);
    let mut test_output = use_signal(|| None::<String>);
    let mut test_error = use_signal(|| None::<String>);
    let mut test_prompt_text = use_signal(|| None::<String>);
    let mut show_test_prompt = use_signal(|| false);

    rsx! {
        Card { title: "System Prompt".to_string(),
            div { class: "flex flex-col gap-1.5",
                p { class: "text-[11px] text-ash-500", "Controls how raw transcriptions are cleaned up." }

                if *show_default.read() {
                    div { class: "bg-mint-50 border border-ash-200 rounded p-2",
                        div { class: "flex items-center justify-between mb-1",
                            span { class: "text-[11px] font-semibold", "Default" }
                            button { class: "text-[11px] text-ash-500 hover:text-ash-700",
                                onclick: move |_| show_default.set(false), "Hide" }
                        }
                        pre { class: "text-[11px] font-mono whitespace-pre-wrap text-ash-600 select-text",
                            "{post_processing::DEFAULT_SYSTEM_PROMPT}" }
                    }
                }

                textarea {
                    class: "w-full p-2 bg-mint-50 border border-ash-200 rounded text-xs font-mono leading-snug outline-none focus:border-frost-700 resize-y min-h-24",
                    rows: "6", value: "{prompt_input}",
                    oninput: move |e: Event<FormData>| {
                        let v = e.value(); prompt_input.set(v.clone());
                        let t = v.trim();
                        if t == post_processing::DEFAULT_SYSTEM_PROMPT.trim() || t.is_empty() {
                            config.write().custom_system_prompt = String::new();
                        } else { config.write().custom_system_prompt = t.to_string(); }
                    },
                }

                div { class: "flex items-center gap-2",
                    if is_custom {
                        span { class: "text-[11px] text-frost-700", "Custom" }
                    } else {
                        span { class: "text-[11px] text-ash-500", "Default" }
                    }
                    div { class: "flex-1" }
                    if !*show_default.read() {
                        button { class: "text-[11px] text-ash-500 hover:text-ash-700",
                            onclick: move |_| show_default.set(true), "View Default" }
                    }
                    if is_custom {
                        button { class: "text-[11px] text-ash-500 hover:text-ash-700",
                            onclick: move |_| {
                                prompt_input.set(post_processing::DEFAULT_SYSTEM_PROMPT.to_string());
                                config.write().custom_system_prompt = String::new();
                            }, "Reset" }
                    }
                }

                div { class: "h-px bg-ash-200 my-1" }

                p { class: "text-[11px] font-semibold", "Test" }
                textarea {
                    class: "w-full p-2 bg-mint-50 border border-ash-200 rounded text-xs font-mono leading-snug outline-none focus:border-frost-700 resize-y min-h-12",
                    rows: "2", value: "{test_input}",
                    oninput: move |e: Event<FormData>| { test_input.set(e.value()); },
                }
                button {
                    class: "h-7 px-2.5 rounded text-[11px] font-medium bg-frost-700 text-white border border-frost-700 hover:opacity-90 disabled:opacity-35 self-start",
                    disabled: *test_running.read() || api_key.read().trim().is_empty() || test_input.read().trim().is_empty(),
                    onclick: {
                        let key = api_key.read().clone();
                        let base_url = config.read().api_base_url.clone();
                        let model = config.read().post_processing_model.clone();
                        let custom_prompt = config.read().custom_system_prompt.clone();
                        let vocab = config.read().custom_vocabulary.clone();
                        move |_| {
                            let input = test_input.read().clone();
                            let key = key.clone(); let base_url = base_url.clone();
                            let model = model.clone(); let custom_prompt = custom_prompt.clone(); let vocab = vocab.clone();
                            test_running.set(true); test_output.set(None); test_error.set(None); test_prompt_text.set(None);
                            spawn(async move {
                                match http_client::build_client() {
                                    Ok(c) => match post_processing::post_process(&c, &key, &input, "Testing system prompt.", &model, &vocab, &custom_prompt, &base_url).await {
                                        Ok(r) => { test_output.set(Some(r.transcript)); test_prompt_text.set(Some(r.prompt)); }
                                        Err(e) => { test_error.set(Some(format!("{e}"))); }
                                    },
                                    Err(e) => { test_error.set(Some(format!("{e}"))); }
                                }
                                test_running.set(false);
                            });
                        }
                    },
                    if *test_running.read() { "Running..." } else { "Run Test" }
                }
                if let Some(ref e) = *test_error.read() {
                    span { class: "text-[11px] text-red-600", "✗ {e}" }
                }
                if let Some(ref out) = *test_output.read() {
                    div { class: "bg-frost-50 border border-frost-200 rounded p-2 text-xs font-mono select-text",
                        if out.is_empty() { "(empty)" } else { "{out}" }
                    }
                }
                if test_prompt_text.read().is_some() {
                    button { class: "text-[11px] text-ash-500 hover:text-ash-700 self-start",
                        onclick: move |_| { let c = *show_test_prompt.read(); show_test_prompt.set(!c); },
                        if *show_test_prompt.read() { "Hide prompt ▴" } else { "Show prompt ▾" }
                    }
                    if *show_test_prompt.read() {
                        if let Some(ref p) = *test_prompt_text.read() {
                            pre { class: "bg-mint-50 border border-ash-200 rounded p-2 text-[11px] font-mono whitespace-pre-wrap select-text",
                                "{p}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ContextPromptCard(config: Signal<AppConfig>) -> Element {
    let custom = config.read().custom_context_prompt.clone();
    let is_custom = !custom.is_empty();
    let mut prompt_input = use_signal(|| if is_custom { custom.clone() } else { String::new() });

    rsx! {
        Card { title: "Context Prompt".to_string(),
            div { class: "flex flex-col gap-1.5",
                p { class: "text-[11px] text-ash-500", "Controls activity inference from app metadata and screenshots." }
                textarea {
                    class: "w-full p-2 bg-mint-50 border border-ash-200 rounded text-xs font-mono leading-snug outline-none focus:border-frost-700 resize-y min-h-24",
                    rows: "6", placeholder: "Custom context prompt (leave empty for default)",
                    value: "{prompt_input}",
                    oninput: move |e: Event<FormData>| {
                        let v = e.value(); prompt_input.set(v.clone());
                        config.write().custom_context_prompt = v.trim().to_string();
                    },
                }
                div { class: "flex items-center gap-2",
                    if is_custom {
                        span { class: "text-[11px] text-frost-700", "Custom" }
                    } else {
                        span { class: "text-[11px] text-ash-500", "Default" }
                    }
                    div { class: "flex-1" }
                    if is_custom {
                        button { class: "text-[11px] text-ash-500 hover:text-ash-700",
                            onclick: move |_| { prompt_input.set(String::new()); config.write().custom_context_prompt = String::new(); },
                            "Reset" }
                    }
                }
            }
        }
    }
}
