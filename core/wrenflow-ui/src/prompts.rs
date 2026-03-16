use dioxus::prelude::*;
use wrenflow_core::config::AppConfig;
use wrenflow_core::http_client;
use wrenflow_core::post_processing;

use crate::components::*;

/// Prompts settings screen — system prompt and context prompt editors.
#[component]
pub fn PromptsSettings(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    rsx! {
        div { class: "flex-col",
            SystemPromptCard { config, api_key }
            ContextPromptCard { config }
        }
    }
}

// --- System Prompt ---

#[component]
fn SystemPromptCard(config: Signal<AppConfig>, api_key: Signal<String>) -> Element {
    let custom = config.read().custom_system_prompt.clone();
    let is_custom = !custom.is_empty();

    let mut prompt_input = use_signal(|| {
        if is_custom {
            custom.clone()
        } else {
            post_processing::DEFAULT_SYSTEM_PROMPT.to_string()
        }
    });

    let mut show_default = use_signal(|| false);

    // Test state
    let mut test_input = use_signal(|| {
        "Um, so I was like, thinking we should uh, refactor the authentication module, you know?"
            .to_string()
    });
    let mut test_running = use_signal(|| false);
    let mut test_output = use_signal(|| None::<String>);
    let mut test_error = use_signal(|| None::<String>);
    let mut test_prompt = use_signal(|| None::<String>);
    let mut show_test_prompt = use_signal(|| false);

    rsx! {
        SettingsCard { title: "System Prompt".to_string(), icon: "\u{1f4ac}".to_string(),
            div { class: "flex-col",
                p { class: "caption", "Controls how raw transcriptions are cleaned up." }

                if *show_default.read() {
                    div { class: "card",
                        div { class: "flex-row",
                            span { class: "caption-bold", "Default System Prompt" }
                            span { class: "spacer" }
                            button {
                                class: "btn",
                                onclick: move |_| show_default.set(false),
                                "Hide"
                            }
                        }
                        pre { class: "code-block mt-md", "{post_processing::DEFAULT_SYSTEM_PROMPT}" }
                    }
                }

                textarea {
                    class: "textarea",
                    rows: "8",
                    value: "{prompt_input}",
                    oninput: move |evt: Event<FormData>| {
                        let val = evt.value().clone();
                        prompt_input.set(val.clone());
                        let trimmed = val.trim();
                        let default_trimmed = post_processing::DEFAULT_SYSTEM_PROMPT.trim();
                        if trimmed == default_trimmed || trimmed.is_empty() {
                            config.write().custom_system_prompt = String::new();
                        } else {
                            config.write().custom_system_prompt = trimmed.to_string();
                        }
                    },
                }

                div { class: "flex-row",
                    if is_custom {
                        span { class: "badge badge-blue", "\u{270f} Using custom prompt" }
                    } else {
                        span { class: "badge badge-neutral", "\u{2713} Using default" }
                    }
                    span { class: "spacer" }
                    if !*show_default.read() {
                        button {
                            class: "btn",
                            onclick: move |_| show_default.set(true),
                            "View Default"
                        }
                    }
                    if is_custom {
                        button {
                            class: "btn",
                            onclick: move |_| {
                                let default = post_processing::DEFAULT_SYSTEM_PROMPT.to_string();
                                prompt_input.set(default);
                                config.write().custom_system_prompt = String::new();
                            },
                            "Reset to Default"
                        }
                    }
                }

                Divider {}

                // Test section
                div { class: "flex-col",
                    p { class: "caption-bold", "Test System Prompt" }
                    p { class: "caption", "Enter sample text to see how the current prompt cleans it up." }

                    textarea {
                        class: "textarea",
                        rows: "3",
                        value: "{test_input}",
                        oninput: move |evt: Event<FormData>| {
                            test_input.set(evt.value().clone());
                        },
                    }

                    button {
                        class: "btn btn-primary",
                        disabled: *test_running.read()
                            || api_key.read().trim().is_empty()
                            || test_input.read().trim().is_empty(),
                        onclick: {
                            let key = api_key.read().clone();
                            let base_url = config.read().api_base_url.clone();
                            let model = config.read().post_processing_model.clone();
                            let custom_prompt = config.read().custom_system_prompt.clone();
                            let vocab = config.read().custom_vocabulary.clone();
                            move |_| {
                                let input = test_input.read().clone();
                                let key = key.clone();
                                let base_url = base_url.clone();
                                let model = model.clone();
                                let custom_prompt = custom_prompt.clone();
                                let vocab = vocab.clone();
                                test_running.set(true);
                                test_output.set(None);
                                test_error.set(None);
                                test_prompt.set(None);
                                spawn(async move {
                                    match http_client::build_client() {
                                        Ok(client) => {
                                            match post_processing::post_process(
                                                &client, &key, &input,
                                                "User is testing the system prompt in Wrenflow settings.",
                                                &model, &vocab, &custom_prompt, &base_url,
                                            ).await {
                                                Ok(result) => {
                                                    test_output.set(Some(result.transcript));
                                                    test_prompt.set(Some(result.prompt));
                                                }
                                                Err(e) => {
                                                    test_error.set(Some(format!("{e}")));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            test_error.set(Some(format!("{e}")));
                                        }
                                    }
                                    test_running.set(false);
                                });
                            }
                        },
                        if *test_running.read() {
                            span { class: "flex-row",
                                Spinner {}
                                "Running..."
                            }
                        } else {
                            "\u{25b6} Test System Prompt"
                        }
                    }

                    if api_key.read().trim().is_empty() {
                        span { class: "badge badge-orange", "\u{26a0} API key required to test" }
                    }

                    if let Some(ref err) = *test_error.read() {
                        span { class: "badge badge-red", "\u{2717} {err}" }
                    }

                    if let Some(ref output) = *test_output.read() {
                        div { class: "flex-col gap-sm",
                            p { class: "caption-bold", "Result:" }
                            div {
                                class: "code-block",
                                style: "background: var(--green-bg);",
                                if output.is_empty() {
                                    "(empty \u{2014} no output)"
                                } else {
                                    "{output}"
                                }
                            }
                        }
                    }

                    if let Some(ref prompt) = *test_prompt.read() {
                        div { class: "flex-col gap-sm",
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    let current = *show_test_prompt.read();
                                    show_test_prompt.set(!current);
                                },
                                if *show_test_prompt.read() { "Hide Full Prompt" } else { "Show Full Prompt" }
                            }
                            if *show_test_prompt.read() {
                                pre { class: "code-block", "{prompt}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// --- Context Prompt ---

#[component]
fn ContextPromptCard(config: Signal<AppConfig>) -> Element {
    let custom = config.read().custom_context_prompt.clone();
    let is_custom = !custom.is_empty();

    let mut prompt_input = use_signal(|| {
        if is_custom {
            custom.clone()
        } else {
            // Use a placeholder since context prompt default is platform-specific
            String::new()
        }
    });

    rsx! {
        SettingsCard { title: "Context Prompt".to_string(), icon: "\u{1f441}".to_string(),
            div { class: "flex-col",
                p { class: "caption",
                    "Controls how Wrenflow infers your current activity from app metadata and screenshots."
                }

                textarea {
                    class: "textarea",
                    rows: "8",
                    placeholder: "Enter a custom context prompt, or leave empty to use the default.",
                    value: "{prompt_input}",
                    oninput: move |evt: Event<FormData>| {
                        let val = evt.value().clone();
                        prompt_input.set(val.clone());
                        config.write().custom_context_prompt = val.trim().to_string();
                    },
                }

                div { class: "flex-row",
                    if is_custom {
                        span { class: "badge badge-blue", "\u{270f} Using custom prompt" }
                    } else {
                        span { class: "badge badge-neutral", "\u{2713} Using default" }
                    }
                    span { class: "spacer" }
                    if is_custom {
                        button {
                            class: "btn",
                            onclick: move |_| {
                                prompt_input.set(String::new());
                                config.write().custom_context_prompt = String::new();
                            },
                            "Reset to Default"
                        }
                    }
                }
            }
        }
    }
}
