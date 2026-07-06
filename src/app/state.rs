use iced::widget::Id;
use iced::Task;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::config;
use crate::domain::models::AppEntry;
use crate::infra::desktop;
use super::message::Message;

pub const THEMES: &[&str] = &[
    "TokyoNight",
    "TokyoNightStorm",
    "TokyoNightLight",
    "Dark",
    "Light",
    "Dracula",
    "Nord",
    "SolarizedLight",
    "SolarizedDark",
    "GruvboxLight",
    "GruvboxDark",
    "CatppuccinLatte",
    "CatppuccinFrappe",
    "CatppuccinMacchiato",
    "CatppuccinMocha",
    "KanagawaWave",
    "KanagawaDragon",
    "KanagawaLotus",
    "Moonfly",
    "Nightfly",
    "Oxocarbon",
    "Ferra",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Launcher,
    CommandRunner,
    FileBrowser,
    Settings,
    Help,
    WebSearch,
    Calculator,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnerState {
    Idle,
    Running { command: String },
    Finished { command: String, output: String },
    Failed { command: String, error: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub icon_path: Option<String>,
}

pub struct CastIt {
    pub query: String,
    pub all_entries: Vec<AppEntry>,
    pub filtered_entries: Vec<(AppEntry, u32)>, // (entry, score)
    pub selected_index: usize,
    pub mode: Mode,
    pub runner_state: RunnerState,
    pub config: config::Config,
    pub selected_setting: usize, // 0: Theme, 1: Terminal, 2: Opacity, 3: Width, 4: Height, 5: Language, 6: Browser
    pub current_parent_dir: String,
    pub directory_entries: Vec<FileEntry>,
    pub filtered_files: Vec<(FileEntry, u32)>, // (entry, score)
    pub preview_active: bool,
    pub calculator_result: Option<f64>,
    pub modifiers: iced::keyboard::Modifiers,
}

impl CastIt {
    pub fn new() -> (Self, Task<Message>) {
        let entries = desktop::scan_desktop_entries();
        let loaded_config = config::Config::load();
        (
            Self {
                query: String::new(),
                all_entries: entries,
                filtered_entries: Vec::new(),
                selected_index: 0,
                mode: Mode::Launcher,
                runner_state: RunnerState::Idle,
                config: loaded_config,
                selected_setting: 0,
                current_parent_dir: String::new(),
                directory_entries: Vec::new(),
                filtered_files: Vec::new(),
                preview_active: false,
                calculator_result: None,
                modifiers: iced::keyboard::Modifiers::default(),
            },
            iced::widget::operation::focus(Id::new("search-input")),
        )
    }
}

pub fn cycle_theme(state: &mut CastIt, direction: i32) {
    let current = state.config.theme.as_deref().unwrap_or("TokyoNight");
    let current_idx = THEMES.iter().position(|&t| t == current).unwrap_or(0);
    let new_idx = if direction > 0 {
        (current_idx + 1) % THEMES.len()
    } else {
        (current_idx + THEMES.len() - 1) % THEMES.len()
    };
    state.config.theme = Some(THEMES[new_idx].to_string());
    state.config.save();
}

pub fn cycle_terminal(state: &mut CastIt, direction: i32) {
    let mut list = crate::infra::runner::get_available_terminals();
    let current = state.config.terminal.as_deref().unwrap_or("Auto");
    if !list.iter().any(|t| t == current) {
        list.push(current.to_string());
    }

    let current_idx = list.iter().position(|t| t == current).unwrap_or(0);
    let new_idx = if direction > 0 {
        (current_idx + 1) % list.len()
    } else {
        (current_idx + list.len() - 1) % list.len()
    };
    state.config.terminal = if list[new_idx] == "Auto" {
        None
    } else {
        Some(list[new_idx].to_string())
    };
    state.config.save();
}

pub fn parse_path_query(query: &str) -> Option<(String, String)> {
    if !query.starts_with('/') && !query.starts_with('~') {
        return None;
    }

    let expanded = if let Some(stripped) = query.strip_prefix('~') {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/alvaro".to_string());
        format!("{}{}", home, stripped)
    } else {
        query.to_string()
    };

    if let Some(idx) = expanded.rfind('/') {
        let (parent, filter) = expanded.split_at(idx + 1);
        Some((parent.to_string(), filter.to_string()))
    } else {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/alvaro".to_string());
        if let Some(idx) = home.rfind('/') {
            let (parent, _) = home.split_at(idx + 1);
            Some((parent.to_string(), home.strip_prefix(&parent).unwrap_or("").to_string()))
        } else {
            None
        }
    }
}

pub fn read_directory(path: &str) -> Vec<FileEntry> {
    let mut entries = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(path) {
        for entry_result in read_dir {
            if let Ok(dir_entry) = entry_result {
                let file_name = dir_entry.file_name().to_string_lossy().to_string();
                let full_path = dir_entry.path().to_string_lossy().to_string();
                let is_dir = dir_entry.file_type().map(|t| t.is_dir()).unwrap_or(false);

                entries.push(FileEntry {
                    name: file_name,
                    path: full_path,
                    is_dir,
                    icon_path: None,
                });
            }
        }
    }

    entries.sort_by(|a, b| {
        if a.is_dir && !b.is_dir {
            std::cmp::Ordering::Less
        } else if !a.is_dir && b.is_dir {
            std::cmp::Ordering::Greater
        } else {
            a.name.to_lowercase().cmp(&b.name.to_lowercase())
        }
    });

    entries
}

pub fn update_filtered_files(state: &mut CastIt) {
    state.filtered_files.clear();

    if let Some((parent, filter)) = parse_path_query(&state.query) {
        if parent != state.current_parent_dir {
            state.current_parent_dir = parent.clone();
            state.directory_entries = read_directory(&parent);
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let pattern = Pattern::parse(&filter, CaseMatching::Ignore, Normalization::Smart);
        let mut buf = Vec::new();

        let filter_starts_with_dot = filter.starts_with('.');

        for entry in &state.directory_entries {
            // Hide dotfiles by default, unless requested
            if entry.name.starts_with('.') && !filter_starts_with_dot {
                continue;
            }

            let haystack = Utf32Str::new(&entry.name, &mut buf);
            if filter.is_empty() {
                state.filtered_files.push((entry.clone(), 1));
            } else if let Some(score) = pattern.score(haystack, &mut matcher) {
                state.filtered_files.push((entry.clone(), score));
            }
        }

        if !filter.is_empty() {
            state.filtered_files.sort_by(|a, b| b.1.cmp(&a.1));
        }

        state.filtered_files.truncate(8);

        // Resolve icon paths for the top results
        for (entry, _) in &mut state.filtered_files {
            if entry.icon_path.is_none() {
                let icon_name = if entry.is_dir { "folder" } else { "text-x-generic" };
                if let Some(result) = linicon::lookup_icon(icon_name).with_size(64).next() {
                    if let Ok(icon_info) = result {
                        entry.icon_path = Some(icon_info.path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
}

pub fn update_filtered_entries(state: &mut CastIt) {
    state.filtered_entries.clear();

    if state.query.is_empty() {
        return;
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(&state.query, CaseMatching::Ignore, Normalization::Smart);

    let mut buf = Vec::new();

    for entry in &state.all_entries {
        let haystack = Utf32Str::new(&entry.name, &mut buf);
        if let Some(score) = pattern.score(haystack, &mut matcher) {
            state.filtered_entries.push((entry.clone(), score));
        }
    }

    // Sort by score descending (best match first)
    state
        .filtered_entries
        .sort_by(|a, b| b.1.cmp(&a.1));

    // Limit visible results
    state.filtered_entries.truncate(8);

    // Lazily resolve icon paths ONLY for the visible results
    for (entry, _) in &mut state.filtered_entries {
        if entry.icon_path.is_none() {
            if let Some(ref name) = entry.icon {
                if name.starts_with('/') {
                    entry.icon_path = Some(name.clone());
                } else if let Some(result) = linicon::lookup_icon(name).with_size(64).next() {
                    if let Ok(icon_info) = result {
                        entry.icon_path = Some(icon_info.path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
}

pub fn launch_selected(state: &CastIt) {
    if let Some((entry, _)) = state.filtered_entries.get(state.selected_index) {
        // Split the exec command and spawn it detached
        let parts: Vec<&str> = entry.exec.split_whitespace().collect();
        if let Some((program, args)) = parts.split_first() {
            let _ = Command::new(program)
                .args(args)
                .process_group(0)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
        std::process::exit(0);
    }
}

pub fn cycle_browser(state: &mut CastIt, direction: i32) {
    let mut list = crate::infra::runner::get_available_browsers();
    let current = state.config.browser.as_deref().unwrap_or("Auto");
    if !list.iter().any(|b| b == current) {
        list.push(current.to_string());
    }

    let current_idx = list.iter().position(|b| b == current).unwrap_or(0);
    let new_idx = if direction > 0 {
        (current_idx + 1) % list.len()
    } else {
        (current_idx + list.len() - 1) % list.len()
    };
    state.config.browser = if list[new_idx] == "Auto" {
        None
    } else {
        Some(list[new_idx].to_string())
    };
    state.config.save();
}
