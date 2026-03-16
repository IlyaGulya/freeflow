use dioxus::prelude::*;
use wrenflow_core::history::HistoryEntry;
use wrenflow_core::metrics::PipelineMetrics;

use crate::components::*;

/// Run log screen — shows pipeline history entries with expandable details.
#[component]
pub fn RunLog(history_entries: Signal<Vec<HistoryEntry>>, on_clear: EventHandler<()>) -> Element {
    rsx! {
        div { class: "flex-col",
            div { class: "flex-row",
                div {
                    p { style: "font-size: 14px; font-weight: 600;", "Run Log" }
                    p { class: "caption text-tertiary",
                        "Stored locally. Only the most recent runs are kept."
                    }
                }
                span { class: "spacer" }
                button {
                    class: "btn btn-danger",
                    disabled: history_entries.read().is_empty(),
                    onclick: move |_| on_clear.call(()),
                    "Clear History"
                }
            }

            Divider {}

            if history_entries.read().is_empty() {
                div { class: "empty-state",
                    p { "No runs yet. Use dictation to populate history." }
                }
            } else {
                div { class: "flex-col",
                    for entry in history_entries.read().iter() {
                        HistoryEntryView {
                            key: "{entry.id}",
                            entry: entry.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn HistoryEntryView(entry: HistoryEntry) -> Element {
    let mut expanded = use_signal(|| false);

    let metrics = entry.metrics();
    let total_ms = metrics.get_double("pipeline.totalMs");
    let is_error = entry.post_processing_status.starts_with("Error:");

    let timestamp = format_timestamp(entry.timestamp);
    let transcript_preview = if entry.post_processed_transcript.is_empty() {
        "(no transcript)".to_string()
    } else {
        let s = &entry.post_processed_transcript;
        if s.len() > 100 {
            format!("{}...", &s[..100])
        } else {
            s.clone()
        }
    };

    rsx! {
        div { class: "history-entry",
            div {
                class: "history-header",
                onclick: move |_| {
                    let current = *expanded.read();
                    expanded.set(!current);
                },

                if is_error {
                    span { class: "text-red", "\u{26a0}" }
                }
                div { style: "flex: 1; min-width: 0;",
                    div { class: "flex-row gap-sm",
                        span { style: "font-size: 12px; font-weight: 600;", "{timestamp}" }
                        if let Some(ms) = total_ms {
                            span { class: "badge badge-neutral", "{format_duration(ms)}" }
                        }
                    }
                    p {
                        class: "caption",
                        style: "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                        "{transcript_preview}"
                    }
                }
                span {
                    class: if *expanded.read() { "chevron expanded" } else { "chevron" },
                    "\u{25b8}"
                }
            }

            if *expanded.read() {
                div { class: "history-body",
                    HistoryDetail { entry: entry.clone() }
                }
            }
        }
    }
}

#[component]
fn HistoryDetail(entry: HistoryEntry) -> Element {
    let metrics = entry.metrics();
    let mut show_pp_prompt = use_signal(|| false);

    // Pre-compute values to avoid complex expressions in RSX
    let recording_dur = metrics.get_double("recording.durationMs");
    let file_size = metrics.get_int("recording.fileSizeBytes");
    let engine_reused = metrics.get_bool("engine.reused");
    let engine_init = metrics.get_double("engine.initMs");
    let first_buffer = metrics.get_double("engine.firstBufferMs");
    let context_total = metrics.get_double("context.totalMs");
    let screenshot_ms = metrics.get_double("context.screenshotMs");
    let llm_ms = metrics.get_double("context.llmMs");
    let transcription_dur = metrics.get_double("transcription.durationMs");
    let transcription_provider = metrics.get_string("transcription.provider").map(|s| s.to_string());
    let pp_dur = metrics.get_double("postProcessing.durationMs");
    let pp_model = metrics.get_string("postProcessing.model").map(|s| s.to_string());
    let has_different_pp = !entry.post_processed_transcript.is_empty()
        && entry.post_processed_transcript != entry.raw_transcript;
    let pp_reasoning = entry.post_processing_reasoning.clone().unwrap_or_default();
    let pp_prompt = entry.post_processing_prompt.clone().unwrap_or_default();
    let has_reasoning = !pp_reasoning.is_empty();
    let has_pp_prompt = !pp_prompt.is_empty();

    rsx! {
        div { class: "flex-col gap-lg",

            // Step 1: Recording
            PipelineStep {
                number: 1,
                title: "Record Audio".to_string(),
                duration_ms: recording_dur,
                div { class: "flex-col gap-sm",
                    if let Some(size) = file_size {
                        p { class: "caption", "File size: {format_file_size(size)}" }
                    }
                    if let Some(reused) = engine_reused {
                        {
                            let label = if reused { "reused" } else { "new" };
                            rsx! { p { class: "caption", "Engine: {label}" } }
                        }
                    }
                    if let Some(init_ms) = engine_init {
                        p { class: "caption", "Engine init: {format_duration(init_ms)}" }
                    }
                    if let Some(first_ms) = first_buffer {
                        p { class: "caption", "First buffer: {format_duration(first_ms)}" }
                    }
                }
            }

            // Step 2: Context Capture
            PipelineStep {
                number: 2,
                title: "Capture Context".to_string(),
                duration_ms: context_total,
                div { class: "flex-col gap-sm",
                    div { class: "flex-row gap-md",
                        if let Some(ss_ms) = screenshot_ms {
                            span { class: "caption", "Screenshot: {format_duration(ss_ms)}" }
                        }
                        if let Some(lms) = llm_ms {
                            span { class: "caption", "LLM: {format_duration(lms)}" }
                        }
                    }
                    if !entry.context_summary.is_empty() {
                        p { class: "caption", "{entry.context_summary}" }
                    } else {
                        p { class: "caption text-secondary", "No context captured" }
                    }
                }
            }

            // Step 3: Transcribe Audio
            PipelineStep {
                number: 3,
                title: "Transcribe Audio".to_string(),
                duration_ms: transcription_dur,
                div { class: "flex-col gap-sm",
                    if let Some(ref provider) = transcription_provider {
                        p { class: "caption", "Provider: {provider}" }
                    }
                    if !entry.raw_transcript.is_empty() {
                        div { class: "code-block", "{entry.raw_transcript}" }
                    } else {
                        p { class: "caption text-secondary", "(empty transcript)" }
                    }
                }
            }

            // Step 4: Post-Process
            PipelineStep {
                number: 4,
                title: "Post-Process".to_string(),
                duration_ms: pp_dur,
                div { class: "flex-col gap-sm",
                    if let Some(ref model) = pp_model {
                        p { class: "caption", "Model: {model}" }
                    }
                    p { class: "caption", "{entry.post_processing_status}" }

                    if has_different_pp {
                        div { class: "flex-col gap-sm",
                            p { class: "caption-bold", "Result:" }
                            div {
                                class: "code-block",
                                style: "background: var(--green-bg);",
                                "{entry.post_processed_transcript}"
                            }
                        }
                    }

                    if has_reasoning {
                        div { class: "flex-col gap-sm",
                            p { class: "caption-bold", "Reasoning:" }
                            p { class: "caption", "{pp_reasoning}" }
                        }
                    }

                    if has_pp_prompt {
                        button {
                            class: "btn",
                            onclick: move |_| {
                                let current = *show_pp_prompt.read();
                                show_pp_prompt.set(!current);
                            },
                            if *show_pp_prompt.read() { "Hide Prompt" } else { "Show Prompt" }
                        }
                        if *show_pp_prompt.read() {
                            pre { class: "code-block", "{pp_prompt}" }
                        }
                    }
                }
            }

            // All Metrics
            if !metrics.is_empty() {
                AllMetrics { metrics: metrics.clone() }
            }
        }
    }
}

#[component]
fn AllMetrics(metrics: PipelineMetrics) -> Element {
    let mut show = use_signal(|| false);
    let keys = metrics.all_keys();
    let count = keys.len();

    // Pre-render metrics as strings
    let metric_lines: Vec<String> = keys
        .iter()
        .map(|key| {
            let val = if let Some(v) = metrics.get_double(key) {
                format_duration(v)
            } else if let Some(v) = metrics.get_int(key) {
                v.to_string()
            } else if let Some(v) = metrics.get_string(key) {
                v.to_string()
            } else if let Some(v) = metrics.get_bool(key) {
                v.to_string()
            } else {
                "?".to_string()
            };
            format!("{key}: {val}")
        })
        .collect();

    rsx! {
        div { class: "flex-col gap-sm",
            button {
                class: "btn",
                onclick: move |_| {
                    let current = *show.read();
                    show.set(!current);
                },
                if *show.read() { "Hide All Metrics" } else { "Show All Metrics ({count})" }
            }
            if *show.read() {
                div { class: "code-block",
                    for line in metric_lines.iter() {
                        div { "{line}" }
                    }
                }
            }
        }
    }
}

/// Format a Unix timestamp to a human-readable string.
fn format_timestamp(ts: f64) -> String {
    use chrono::{Local, TimeZone};
    let secs = ts as i64;
    let nanos = ((ts - secs as f64) * 1_000_000_000.0) as u32;
    match Local.timestamp_opt(secs, nanos) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => format!("{:.0}", ts),
    }
}
