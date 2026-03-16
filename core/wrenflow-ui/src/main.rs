mod components;
mod prompts;
mod run_log;
mod settings;
mod setup;
mod theme;

use dioxus::prelude::*;
use wrenflow_core::config::AppConfig;
use wrenflow_core::history::{HistoryEntry, HistoryStore};
use std::path::PathBuf;

/// Settings tab enum — mirrors Swift's SettingsTab.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SettingsTab {
    General,
    Prompts,
    RunLog,
}

impl SettingsTab {
    fn label(self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Prompts => "Prompts",
            Self::RunLog => "Run Log",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::General => "\u{2699}",
            Self::Prompts => "\u{1f4ac}",
            Self::RunLog => "\u{1f4cb}",
        }
    }
}

const ALL_TABS: [SettingsTab; 3] = [SettingsTab::General, SettingsTab::Prompts, SettingsTab::RunLog];

/// CLI argument: --setup to launch setup wizard, otherwise settings.
fn is_setup_mode() -> bool {
    std::env::args().any(|a| a == "--setup")
}

/// Get the config file path.
fn config_path() -> PathBuf {
    AppConfig::default_path("Wrenflow")
}

/// Get the history database path.
fn history_db_path() -> PathBuf {
    let config_dir = config_path().parent().unwrap().to_path_buf();
    config_dir.join("PipelineHistory.sqlite")
}

fn main() {
    env_logger::init();

    let cfg = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::tao::window::WindowBuilder::new()
                .with_title("Wrenflow Settings")
                .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(800.0, 600.0))
                .with_min_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(600.0, 400.0)),
        );

    dioxus::LaunchBuilder::desktop()
        .with_cfg(cfg)
        .launch(App);
}

#[component]
fn App() -> Element {
    // Load config
    let config = use_signal(|| AppConfig::load_or_default(&config_path()));

    // API key — stored separately for cross-platform.
    // The native shell should inject this from secure storage in production.
    let api_key = use_signal(String::new);

    // History
    let mut history_entries = use_signal(Vec::<HistoryEntry>::new);

    // Load history on mount
    use_effect(move || {
        let db_path = history_db_path();
        if let Ok(store) = HistoryStore::open(&db_path) {
            if let Ok(entries) = store.load_all() {
                history_entries.set(entries);
            }
        }
    });

    // Setup vs settings mode
    let setup_mode = is_setup_mode();
    let mut setup_complete = use_signal(|| !setup_mode);

    if !*setup_complete.read() {
        return rsx! {
            style { {theme::GLOBAL_CSS} }
            setup::SetupWizard {
                config,
                api_key,
                on_complete: move |_| {
                    let path = config_path();
                    let _ = config.read().save(&path);
                    setup_complete.set(true);
                },
            }
        };
    }

    rsx! {
        style { {theme::GLOBAL_CSS} }
        SettingsApp { config, api_key, history_entries }
    }
}

#[component]
fn SettingsApp(
    config: Signal<AppConfig>,
    api_key: Signal<String>,
    history_entries: Signal<Vec<HistoryEntry>>,
) -> Element {
    let mut selected_tab = use_signal(|| SettingsTab::General);

    rsx! {
        div { class: "settings-layout",
            // Sidebar
            div { class: "sidebar",
                for tab in ALL_TABS {
                    button {
                        class: if *selected_tab.read() == tab { "sidebar-item active" } else { "sidebar-item" },
                        onclick: move |_| selected_tab.set(tab),
                        span { "{tab.icon()}" }
                        "{tab.label()}"
                    }
                }
            }

            // Content
            div { class: "content-area",
                match *selected_tab.read() {
                    SettingsTab::General => rsx! {
                        settings::GeneralSettings { config, api_key }
                    },
                    SettingsTab::Prompts => rsx! {
                        prompts::PromptsSettings { config, api_key }
                    },
                    SettingsTab::RunLog => rsx! {
                        run_log::RunLog {
                            history_entries,
                            on_clear: move |_| {
                                let db_path = history_db_path();
                                if let Ok(store) = HistoryStore::open(&db_path) {
                                    let _ = store.clear_all();
                                }
                                history_entries.set(Vec::new());
                            },
                        }
                    },
                }
            }
        }
    }
}
