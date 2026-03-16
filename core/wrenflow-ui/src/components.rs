use dioxus::prelude::*;

/// A settings card with title, icon, and body content.
#[component]
pub fn SettingsCard(title: String, icon: String, children: Element) -> Element {
    rsx! {
        div { class: "card",
            div { class: "card-header",
                span { class: "card-icon", "{icon}" }
                "{title}"
            }
            {children}
        }
    }
}

/// A toggle switch with label.
#[component]
pub fn Toggle(label: String, checked: bool, on_change: EventHandler<bool>) -> Element {
    rsx! {
        div { class: "toggle-row",
            label { class: "toggle",
                input {
                    r#type: "checkbox",
                    checked: checked,
                    onchange: move |evt: Event<FormData>| {
                        on_change.call(evt.checked());
                    },
                }
                span { class: "toggle-slider" }
            }
            "{label}"
        }
    }
}

/// A segmented control (radio group styled as tabs).
#[component]
pub fn SegmentedControl(
    options: Vec<(String, String)>,
    selected: String,
    on_change: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "segmented",
            for (value, label) in options.iter() {
                button {
                    class: if *selected == **value { "segmented-option active" } else { "segmented-option" },
                    onclick: {
                        let value = value.clone();
                        move |_| on_change.call(value.clone())
                    },
                    "{label}"
                }
            }
        }
    }
}

/// A radio-style option row.
#[component]
pub fn RadioOption(
    label: String,
    description: String,
    selected: bool,
    on_click: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: if selected { "radio-option selected" } else { "radio-option" },
            onclick: move |_| on_click.call(()),
            div { class: "radio-dot" }
            div {
                div { "{label}" }
                if !description.is_empty() {
                    p { class: "caption mt-sm", "{description}" }
                }
            }
        }
    }
}

/// Status badge (green checkmark, red error, etc.)
#[component]
pub fn StatusBadge(text: String, variant: String) -> Element {
    let class = match variant.as_str() {
        "green" => "badge badge-green",
        "red" => "badge badge-red",
        "orange" => "badge badge-orange",
        "blue" => "badge badge-blue",
        _ => "badge badge-neutral",
    };
    rsx! {
        span { class: class, "{text}" }
    }
}

/// A small loading spinner.
#[component]
pub fn Spinner() -> Element {
    rsx! {
        span { class: "spinner" }
    }
}

/// A divider line.
#[component]
pub fn Divider() -> Element {
    rsx! {
        div { class: "divider" }
    }
}

/// A pipeline step in the run log detail view.
#[component]
pub fn PipelineStep(
    number: u32,
    title: String,
    duration_ms: Option<f64>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "pipeline-step",
            div { class: "step-number", "{number}" }
            div { class: "step-content",
                div { class: "step-header",
                    span { class: "step-title", "{title}" }
                    if let Some(ms) = duration_ms {
                        span { class: "badge badge-neutral", "{format_duration(ms)}" }
                    }
                }
                {children}
            }
        }
    }
}

/// Format a duration in milliseconds for display.
pub fn format_duration(ms: f64) -> String {
    if ms >= 1000.0 {
        format!("{:.1}s", ms / 1000.0)
    } else {
        format!("{:.0}ms", ms)
    }
}

/// Format a file size in bytes for display.
pub fn format_file_size(bytes: i64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
