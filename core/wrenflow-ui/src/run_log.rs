use dioxus::prelude::*;
use wrenflow_core::history::HistoryEntry;
use wrenflow_core::metrics::PipelineMetrics;

use crate::components::*;

#[component]
pub fn RunLog(history: Signal<Vec<HistoryEntry>>, on_clear: EventHandler<()>) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1.5",
            div { class: "flex items-center justify-between",
                div {
                    p { class: "text-xs font-semibold", "Run Log" }
                    p { class: "text-[11px] text-ash-500", "Most recent runs stored locally." }
                }
                button {
                    class: "h-7 px-2.5 rounded text-[11px] font-medium border border-ash-200 text-red-600 bg-white hover:bg-red-50 disabled:opacity-35",
                    disabled: history.read().is_empty(),
                    onclick: move |_| on_clear.call(()),
                    "Clear"
                }
            }
            div { class: "h-px bg-ash-200" }
            if history.read().is_empty() {
                div { class: "flex items-center justify-center py-8 text-[11px] text-ash-400",
                    "No runs yet."
                }
            } else {
                div { class: "flex flex-col gap-1",
                    for entry in history.read().iter() {
                        EntryRow { key: "{entry.id}", entry: entry.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn EntryRow(entry: HistoryEntry) -> Element {
    let mut expanded = use_signal(|| false);
    let metrics = entry.metrics();
    let total_ms = metrics.get_double("pipeline.totalMs");
    let is_err = entry.post_processing_status.starts_with("Error:");
    let ts = format_ts(entry.timestamp);
    let preview = if entry.post_processed_transcript.is_empty() { "(no transcript)".into() }
        else if entry.post_processed_transcript.len() > 80 { format!("{}…", &entry.post_processed_transcript[..80]) }
        else { entry.post_processed_transcript.clone() };

    rsx! {
        div { class: "border border-ash-200 rounded bg-white overflow-hidden",
            div {
                class: "flex items-center px-2.5 h-8 gap-1.5 cursor-pointer hover:bg-mint-50 transition-colors",
                onclick: move |_| { let c = *expanded.read(); expanded.set(!c); },
                if is_err { span { class: "text-red-500 text-[11px]", "!" } }
                div { class: "flex-1 min-w-0",
                    div { class: "flex items-center gap-1.5",
                        span { class: "text-[11px] font-mono text-ash-600", "{ts}" }
                        if let Some(ms) = total_ms {
                            span { class: "text-[10px] font-mono text-ash-400 bg-ash-50 px-1 rounded", "{format_duration(ms)}" }
                        }
                    }
                    p { class: "text-[11px] text-ash-500 truncate", "{preview}" }
                }
                span { class: if *expanded.read() { "text-[9px] text-ash-400 rotate-90 transition-transform" } else { "text-[9px] text-ash-400 transition-transform" },
                    "▸"
                }
            }
            if *expanded.read() {
                div { class: "border-t border-ash-200 bg-mint-50 p-2.5",
                    EntryDetail { entry: entry.clone() }
                }
            }
        }
    }
}

#[component]
fn EntryDetail(entry: HistoryEntry) -> Element {
    let m = entry.metrics();
    let mut show_pp = use_signal(|| false);
    let rec_dur = m.get_double("recording.durationMs");
    let file_sz = m.get_int("recording.fileSizeBytes");
    let ctx_total = m.get_double("context.totalMs");
    let tr_dur = m.get_double("transcription.durationMs");
    let tr_prov = m.get_string("transcription.provider").map(|s| s.to_string());
    let pp_dur = m.get_double("postProcessing.durationMs");
    let pp_model = m.get_string("postProcessing.model").map(|s| s.to_string());
    let diff = !entry.post_processed_transcript.is_empty() && entry.post_processed_transcript != entry.raw_transcript;
    let reasoning = entry.post_processing_reasoning.clone().unwrap_or_default();
    let pp_prompt = entry.post_processing_prompt.clone().unwrap_or_default();

    rsx! {
        div { class: "flex flex-col gap-2.5",
            PipelineStep { number: 1, title: "Record".to_string(), duration_ms: rec_dur,
                div { class: "flex flex-col gap-0.5",
                    if let Some(sz) = file_sz { p { class: "text-[11px] text-ash-500", "Size: {format_file_size(sz)}" } }
                }
            }
            PipelineStep { number: 2, title: "Context".to_string(), duration_ms: ctx_total,
                if !entry.context_summary.is_empty() {
                    p { class: "text-[11px] text-ash-600", "{entry.context_summary}" }
                } else {
                    p { class: "text-[11px] text-ash-400", "None" }
                }
            }
            PipelineStep { number: 3, title: "Transcribe".to_string(), duration_ms: tr_dur,
                div { class: "flex flex-col gap-1",
                    if let Some(ref prov) = tr_prov {
                        p { class: "text-[11px] text-ash-500", "Provider: {prov}" }
                    }
                    if !entry.raw_transcript.is_empty() {
                        div { class: "text-[11px] font-mono bg-white border border-ash-200 rounded p-1.5 select-text whitespace-pre-wrap",
                            "{entry.raw_transcript}" }
                    }
                }
            }
            PipelineStep { number: 4, title: "Post-Process".to_string(), duration_ms: pp_dur,
                div { class: "flex flex-col gap-1",
                    if let Some(ref mdl) = pp_model { p { class: "text-[11px] text-ash-500", "{mdl}" } }
                    p { class: "text-[11px] text-ash-500", "{entry.post_processing_status}" }
                    if diff {
                        div { class: "text-[11px] font-mono bg-frost-50 border border-frost-200 rounded p-1.5 select-text whitespace-pre-wrap",
                            "{entry.post_processed_transcript}" }
                    }
                    if !reasoning.is_empty() { p { class: "text-[11px] text-ash-500 italic", "{reasoning}" } }
                    if !pp_prompt.is_empty() {
                        button { class: "text-[11px] text-ash-500 hover:text-ash-700 self-start",
                            onclick: move |_| { let c = *show_pp.read(); show_pp.set(!c); },
                            if *show_pp.read() { "Hide prompt ▴" } else { "Show prompt ▾" }
                        }
                        if *show_pp.read() {
                            pre { class: "text-[11px] font-mono bg-white border border-ash-200 rounded p-1.5 whitespace-pre-wrap select-text",
                                "{pp_prompt}" }
                        }
                    }
                }
            }
            if !m.is_empty() { MetricsBlock { metrics: m.clone() } }
        }
    }
}

#[component]
fn MetricsBlock(metrics: PipelineMetrics) -> Element {
    let mut show = use_signal(|| false);
    let keys = metrics.all_keys();
    let n = keys.len();
    let lines: Vec<String> = keys.iter().map(|k| {
        let v = if let Some(v) = metrics.get_double(k) { format_duration(v) }
            else if let Some(v) = metrics.get_int(k) { v.to_string() }
            else if let Some(v) = metrics.get_string(k) { v.to_string() }
            else if let Some(v) = metrics.get_bool(k) { v.to_string() }
            else { "?".into() };
        format!("{k}: {v}")
    }).collect();
    rsx! {
        button { class: "text-[11px] text-ash-500 hover:text-ash-700 self-start",
            onclick: move |_| { let c = *show.read(); show.set(!c); },
            if *show.read() { "Hide metrics ▴" } else { "All metrics ({n}) ▾" }
        }
        if *show.read() {
            div { class: "text-[11px] font-mono bg-white border border-ash-200 rounded p-1.5 whitespace-pre-wrap select-text",
                for l in lines.iter() { div { "{l}" } }
            }
        }
    }
}

fn format_ts(ts: f64) -> String {
    use chrono::{Local, TimeZone};
    let s = ts as i64;
    let ns = ((ts - s as f64) * 1e9) as u32;
    match Local.timestamp_opt(s, ns) {
        chrono::LocalResult::Single(dt) => dt.format("%m-%d %H:%M:%S").to_string(),
        _ => format!("{:.0}", ts),
    }
}
