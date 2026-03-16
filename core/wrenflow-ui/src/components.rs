use dioxus::prelude::*;

#[component]
pub fn Card(title: String, children: Element) -> Element {
    rsx! {
        div { class: "bg-white border border-ash-200 rounded p-3 mb-2",
            div { class: "text-xs font-semibold text-ash-900 mb-2", "{title}" }
            {children}
        }
    }
}

#[component]
pub fn Toggle(label: String, checked: bool, on_change: EventHandler<bool>) -> Element {
    rsx! {
        div { class: "flex items-center gap-2 h-7 text-xs",
            label { class: "toggle",
                input {
                    r#type: "checkbox",
                    checked: checked,
                    onchange: move |evt: Event<FormData>| on_change.call(evt.checked()),
                }
                span { class: "toggle-track" }
            }
            "{label}"
        }
    }
}

#[component]
pub fn Segmented(
    options: Vec<(String, String)>,
    selected: String,
    on_change: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "flex bg-mint-50 border border-ash-200 rounded h-7 p-0.5",
            for (value, label) in options.iter() {
                button {
                    class: if *selected == **value {
                        "flex-1 text-center text-[11px] font-medium rounded-sm bg-white text-ash-900 shadow-xs"
                    } else {
                        "flex-1 text-center text-[11px] font-medium rounded-sm text-ash-500 hover:text-ash-700"
                    },
                    onclick: {
                        let v = value.clone();
                        move |_| on_change.call(v.clone())
                    },
                    "{label}"
                }
            }
        }
    }
}

#[component]
pub fn RadioOption(
    label: String,
    description: String,
    selected: bool,
    on_click: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: if selected {
                "flex items-start gap-2 p-2 rounded border border-frost-700 bg-frost-50 cursor-pointer"
            } else {
                "flex items-start gap-2 p-2 rounded border border-ash-200 bg-white hover:border-ash-300 cursor-pointer"
            },
            onclick: move |_| on_click.call(()),
            div {
                class: if selected {
                    "w-3.5 h-3.5 rounded-full border-[1.5px] border-frost-700 bg-frost-700 flex items-center justify-center mt-0.5 shrink-0"
                } else {
                    "w-3.5 h-3.5 rounded-full border-[1.5px] border-ash-300 flex items-center justify-center mt-0.5 shrink-0"
                },
                if selected {
                    div { class: "w-1.5 h-1.5 rounded-full bg-white" }
                }
            }
            div {
                div { class: "text-xs font-medium", "{label}" }
                if !description.is_empty() {
                    p { class: "text-[11px] text-ash-500 mt-0.5", "{description}" }
                }
            }
        }
    }
}

#[component]
pub fn Spinner() -> Element {
    rsx! { span { class: "spinner" } }
}

#[component]
pub fn PipelineStep(
    number: u32,
    title: String,
    duration_ms: Option<f64>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "flex gap-2 p-2 bg-mint-50 border border-ash-200 rounded",
            div { class: "w-[18px] h-[18px] rounded-full bg-ash-100 text-ash-500 flex items-center justify-center text-[10px] font-semibold shrink-0",
                "{number}"
            }
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-1.5 mb-1",
                    span { class: "text-[11px] font-semibold", "{title}" }
                    if let Some(ms) = duration_ms {
                        span { class: "text-[10px] font-mono text-ash-500 bg-ash-100 px-1.5 rounded",
                            "{format_duration(ms)}"
                        }
                    }
                }
                {children}
            }
        }
    }
}

pub fn format_duration(ms: f64) -> String {
    if ms >= 1000.0 { format!("{:.1}s", ms / 1000.0) }
    else { format!("{:.0}ms", ms) }
}

pub fn format_file_size(bytes: i64) -> String {
    if bytes >= 1_048_576 { format!("{:.1} MB", bytes as f64 / 1_048_576.0) }
    else if bytes >= 1024 { format!("{:.1} KB", bytes as f64 / 1024.0) }
    else { format!("{} B", bytes) }
}
