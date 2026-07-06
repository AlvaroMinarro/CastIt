use iced::widget::{container, image, row, scrollable, svg, text, text_input, Column, Space, Id};
use iced::{Element, Length, Padding, Task, Font, Color, ContentFit};
use iced::alignment::{Horizontal, Vertical};
use iced_layershell::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};
use iced_layershell::to_layer_message;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use std::process::Command;

use crate::config;
use crate::domain::models::AppEntry;
use crate::infra::{desktop, runner};

const THEMES: &[&str] = &[
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

// ---------------------------------------------------------------------------
// File & Folder Entry Models
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileEntry {
    name: String,
    path: String,
    is_dir: bool,
    icon_path: Option<String>,
}

// ---------------------------------------------------------------------------
// Translations Helper
// ---------------------------------------------------------------------------

fn translate<'a>(key: &'a str, lang: &str) -> &'a str {
    match lang {
        "ES" => match key {
            "search_placeholder" => "Busca apps, comandos ('>') o archivos/carpetas ('/','~')...",
            "no_results" => "No se encontraron resultados",
            "no_files" => "Directorio vacío o no encontrado",
            "cmd_idle" => "Pulsa Enter para segundo plano, Ctrl+Enter para terminal",
            "cmd_running" => "Ejecutando comando...",
            "cmd_success" => "Comando ejecutado con éxito (sin salida)",
            "cmd_failed" => "El comando falló (sin salida de error)",
            "info_tip" => "Usa Arriba/Abajo para navegar. Izquierda/Derecha para cambiar valores. Auto-guardado.",
            "setting_theme" => "Tema",
            "setting_terminal" => "Terminal Preferida",
            "setting_opacity" => "Opacidad de Fondo",
            "setting_width" => "Ancho de Ventana",
            "setting_height" => "Alto de Ventana",
            "setting_language" => "Idioma",
            "launch_tag" => "⏎ Lanzar",
            _ => key,
        },
        _ => match key {
            "search_placeholder" => "Search apps, commands ('>') or files/folders ('/','~')...",
            "no_results" => "No results found",
            "no_files" => "Empty directory or not found",
            "cmd_idle" => "Press Enter to run in background, Ctrl+Enter to run in terminal",
            "cmd_running" => "Running command...",
            "cmd_success" => "Command executed successfully (no output)",
            "cmd_failed" => "Command failed (no error output)",
            "info_tip" => "Use Up/Down to navigate. Use Left/Right to change values. Changes auto-saved.",
            "setting_theme" => "Theme",
            "setting_terminal" => "Preferred Terminal",
            "setting_opacity" => "Background Opacity",
            "setting_width" => "Window Width",
            "setting_height" => "Window Height",
            "setting_language" => "Language",
            "launch_tag" => "⏎ Launch",
            _ => key,
        },
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Launcher,
    CommandRunner,
    FileBrowser,
    Settings,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RunnerState {
    Idle,
    Running { command: String },
    Finished { command: String, output: String },
    Failed { command: String, error: String },
}

struct CastIt {
    query: String,
    all_entries: Vec<AppEntry>,
    filtered_entries: Vec<(AppEntry, u32)>, // (entry, score)
    selected_index: usize,
    mode: Mode,
    runner_state: RunnerState,
    config: config::Config,
    selected_setting: usize, // 0: Theme, 1: Terminal, 2: Opacity, 3: Width, 4: Height, 5: Language
    current_parent_dir: String,
    directory_entries: Vec<FileEntry>,
    filtered_files: Vec<(FileEntry, u32)>, // (entry, score)
    preview_active: bool,
}

impl CastIt {
    fn new() -> (Self, Task<Message>) {
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
            },
            iced::widget::operation::focus(Id::new("search-input")),
        )
    }
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

#[to_layer_message]
#[derive(Debug, Clone)]
enum Message {
    QueryChanged(String),
    Submit,
    SubmitInTerminal,
    CommandFinished { command: String, result: Result<String, String> },
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    SelectEntry(usize),
    WindowFocused,
    ClearQuery,
    TogglePreview,
    ScrollPreviewUp,
    ScrollPreviewDown,
}

// ---------------------------------------------------------------------------
// Core functions (TEA)
// ---------------------------------------------------------------------------

fn namespace() -> String {
    String::from("castit")
}

fn resolve_iced_theme(name: &str) -> iced::Theme {
    match name {
        "Light" => iced::Theme::Light,
        "Dark" => iced::Theme::Dark,
        "Dracula" => iced::Theme::Dracula,
        "Nord" => iced::Theme::Nord,
        "SolarizedLight" => iced::Theme::SolarizedLight,
        "SolarizedDark" => iced::Theme::SolarizedDark,
        "GruvboxLight" => iced::Theme::GruvboxLight,
        "GruvboxDark" => iced::Theme::GruvboxDark,
        "CatppuccinLatte" => iced::Theme::CatppuccinLatte,
        "CatppuccinFrappe" => iced::Theme::CatppuccinFrappe,
        "CatppuccinMacchiato" => iced::Theme::CatppuccinMacchiato,
        "CatppuccinMocha" => iced::Theme::CatppuccinMocha,
        "TokyoNight" => iced::Theme::TokyoNight,
        "TokyoNightStorm" => iced::Theme::TokyoNightStorm,
        "TokyoNightLight" => iced::Theme::TokyoNightLight,
        "KanagawaWave" => iced::Theme::KanagawaWave,
        "KanagawaDragon" => iced::Theme::KanagawaDragon,
        "KanagawaLotus" => iced::Theme::KanagawaLotus,
        "Moonfly" => iced::Theme::Moonfly,
        "Nightfly" => iced::Theme::Nightfly,
        "Oxocarbon" => iced::Theme::Oxocarbon,
        "Ferra" => iced::Theme::Ferra,
        _ => iced::Theme::TokyoNight,
    }
}

fn update(state: &mut CastIt, message: Message) -> Task<Message> {
    match message {
        Message::ClearQuery => {
            state.query.clear();
            state.selected_index = 0;
            state.mode = Mode::Launcher;
            state.runner_state = RunnerState::Idle;
            state.preview_active = false;
            state.filtered_entries.clear();
            state.filtered_files.clear();
            Task::none()
        }
        Message::QueryChanged(value) => {
            state.query = value;
            state.selected_index = 0;
            state.preview_active = false;
            if state.query.starts_with("??") {
                state.mode = Mode::Help;
            } else if state.query.starts_with("..") {
                state.mode = Mode::Settings;
            } else if state.query.starts_with('>') {
                state.mode = Mode::CommandRunner;
                if state.query.trim() == ">" {
                    state.runner_state = RunnerState::Idle;
                }
            } else if state.query.starts_with('/') || state.query.starts_with('~') {
                state.mode = Mode::FileBrowser;
                update_filtered_files(state);
            } else {
                state.mode = Mode::Launcher;
                state.runner_state = RunnerState::Idle;
                update_filtered_entries(state);
            }
            Task::none()
        }
        Message::Submit => {
            match state.mode {
                Mode::Launcher => {
                    launch_selected(state);
                    Task::none()
                }
                Mode::CommandRunner => {
                    let cmd = state.query.strip_prefix('>').unwrap_or(&state.query).trim().to_string();
                    if !cmd.is_empty() {
                        state.runner_state = RunnerState::Running { command: cmd.clone() };
                        state.query = "> ".to_string(); // Reset input back to prompt
                        let cmd_clone = cmd.clone();
                        Task::perform(
                            async move {
                                runner::run_in_background(&cmd_clone)
                            },
                            move |res| Message::CommandFinished {
                                command: cmd.clone(),
                                result: res,
                            },
                        )
                    } else {
                        Task::none()
                    }
                }
                Mode::FileBrowser => {
                    if let Some((entry, _)) = state.filtered_files.get(state.selected_index) {
                        // Open file or directory using system default (xdg-open) and exit
                        let _ = Command::new("xdg-open")
                            .arg(&entry.path)
                            .stdin(std::process::Stdio::null())
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .spawn();
                        std::process::exit(0);
                    }
                    Task::none()
                }
                Mode::Settings => Task::none(),
                Mode::Help => Task::none(),
            }
        }
        Message::SubmitInTerminal => {
            if state.mode == Mode::CommandRunner {
                let cmd = state.query.strip_prefix('>').unwrap_or(&state.query).trim().to_string();
                if !cmd.is_empty() {
                    let term_override = state.config.terminal.as_deref();
                    let _ = runner::run_in_terminal(&cmd, term_override);
                    std::process::exit(0);
                }
            }
            Task::none()
        }
        Message::CommandFinished { command, result } => {
            match result {
                Ok(output) => state.runner_state = RunnerState::Finished { command, output },
                Err(err) => state.runner_state = RunnerState::Failed { command, error: err },
            }
            Task::none()
        }
        Message::Escape => {
            std::process::exit(0);
        }
        Message::ArrowDown => {
            match state.mode {
                Mode::Launcher => {
                    if !state.filtered_entries.is_empty() {
                        state.selected_index =
                            (state.selected_index + 1).min(state.filtered_entries.len() - 1);
                        let total = state.filtered_entries.len();
                        if total > 1 {
                            let ratio = state.selected_index as f32 / (total - 1) as f32;
                            return iced::widget::operation::snap_to(
                                Id::new("scroll-list"),
                                iced::widget::scrollable::RelativeOffset { x: 0.0, y: ratio }
                            );
                        }
                    }
                }
                Mode::FileBrowser => {
                    if !state.filtered_files.is_empty() {
                        state.selected_index =
                            (state.selected_index + 1).min(state.filtered_files.len() - 1);
                        let total = state.filtered_files.len();
                        if total > 1 {
                            let ratio = state.selected_index as f32 / (total - 1) as f32;
                            return iced::widget::operation::snap_to(
                                Id::new("scroll-list"),
                                iced::widget::scrollable::RelativeOffset { x: 0.0, y: ratio }
                            );
                        }
                    }
                }
                Mode::Help => {
                    return iced::widget::operation::scroll_by(
                        Id::new("help-scroll"),
                        iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: 40.0 }
                    );
                }
                _ => {}
            }
            Task::none()
        }
        Message::ArrowUp => {
            match state.mode {
                Mode::Launcher => {
                    state.selected_index = state.selected_index.saturating_sub(1);
                    let total = state.filtered_entries.len();
                    if total > 1 {
                        let ratio = state.selected_index as f32 / (total - 1) as f32;
                        return iced::widget::operation::snap_to(
                            Id::new("scroll-list"),
                            iced::widget::scrollable::RelativeOffset { x: 0.0, y: ratio }
                        );
                    }
                }
                Mode::FileBrowser => {
                    state.selected_index = state.selected_index.saturating_sub(1);
                    let total = state.filtered_files.len();
                    if total > 1 {
                        let ratio = state.selected_index as f32 / (total - 1) as f32;
                        return iced::widget::operation::snap_to(
                            Id::new("scroll-list"),
                            iced::widget::scrollable::RelativeOffset { x: 0.0, y: ratio }
                        );
                    }
                }
                Mode::Settings => {
                    state.selected_setting = state.selected_setting.saturating_sub(1);
                }
                Mode::Help => {
                    return iced::widget::operation::scroll_by(
                        Id::new("help-scroll"),
                        iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: -40.0 }
                    );
                }
                _ => {}
            }
            Task::none()
        }
        Message::ArrowLeft => {
            if state.mode == Mode::Settings {
                match state.selected_setting {
                    0 => { // Theme
                        cycle_theme(state, -1);
                    }
                    1 => { // Terminal
                        cycle_terminal(state, -1);
                    }
                    2 => { // Opacity
                        let opacity = state.config.opacity.unwrap_or(0.92);
                        state.config.opacity = Some((opacity - 0.05).max(0.1));
                        state.config.save();
                    }
                    3 => { // Width
                        let width = state.config.width.unwrap_or(800);
                        state.config.width = Some(width.saturating_sub(50).max(400));
                        state.config.save();
                    }
                    4 => { // Height
                        let height = state.config.height.unwrap_or(500);
                        state.config.height = Some(height.saturating_sub(50).max(300));
                        state.config.save();
                    }
                    5 => { // Language
                        let lang = state.config.language.as_deref().unwrap_or("EN");
                        state.config.language = Some(if lang == "EN" { "ES".to_string() } else { "EN".to_string() });
                        state.config.save();
                    }
                    _ => {}
                }
            } else if state.mode == Mode::FileBrowser {
                let mut path_str = state.query.clone();
                if path_str.ends_with('/') {
                    path_str.pop();
                }
                if let Some(idx) = path_str.rfind('/') {
                    let parent = &path_str[..=idx];
                    let mut display_path = parent.to_string();
                    let home = std::env::var("HOME").unwrap_or_default();
                    if !home.is_empty() && display_path.starts_with(&home) {
                        display_path = display_path.replacen(&home, "~", 1);
                    }
                    state.query = display_path;
                    state.selected_index = 0;
                    update_filtered_files(state);
                    return iced::widget::operation::move_cursor_to_end(Id::new("search-input"));
                }
            }
            Task::none()
        }
        Message::ArrowRight => {
            match state.mode {
                Mode::Settings => {
                    match state.selected_setting {
                        0 => { // Theme
                            cycle_theme(state, 1);
                        }
                        1 => { // Terminal
                            cycle_terminal(state, 1);
                        }
                        2 => { // Opacity
                            let opacity = state.config.opacity.unwrap_or(0.92);
                            state.config.opacity = Some((opacity + 0.05).min(1.0));
                            state.config.save();
                        }
                        3 => { // Width
                            let width = state.config.width.unwrap_or(800);
                            state.config.width = Some((width + 50).min(1920));
                            state.config.save();
                        }
                        4 => { // Height
                            let height = state.config.height.unwrap_or(500);
                            state.config.height = Some((height + 50).min(1080));
                            state.config.save();
                        }
                        5 => { // Language
                            let lang = state.config.language.as_deref().unwrap_or("EN");
                            state.config.language = Some(if lang == "EN" { "ES".to_string() } else { "EN".to_string() });
                            state.config.save();
                        }
                        _ => {}
                    }
                }
                Mode::FileBrowser => {
                    if let Some((entry, _)) = state.filtered_files.get(state.selected_index) {
                        if entry.is_dir {
                            // Autocomplete directory path
                            let mut display_path = entry.path.clone();
                            let home = std::env::var("HOME").unwrap_or_default();
                            if !home.is_empty() && display_path.starts_with(&home) {
                                display_path = display_path.replacen(&home, "~", 1);
                            }
                            state.query = format!("{}/", display_path);
                            state.selected_index = 0;
                            update_filtered_files(state);
                            return iced::widget::operation::move_cursor_to_end(Id::new("search-input"));
                        }
                    }
                }
                _ => {}
            }
            Task::none()
        }
        Message::TogglePreview => {
            if state.mode == Mode::FileBrowser && !state.filtered_files.is_empty() {
                state.preview_active = !state.preview_active;
            }
            Task::none()
        }
        Message::ScrollPreviewUp => {
            if state.mode == Mode::FileBrowser && state.preview_active {
                return iced::widget::operation::scroll_by(
                    Id::new("preview-scroll"),
                    iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: -40.0 }
                );
            }
            Task::none()
        }
        Message::ScrollPreviewDown => {
            if state.mode == Mode::FileBrowser && state.preview_active {
                return iced::widget::operation::scroll_by(
                    Id::new("preview-scroll"),
                    iced::widget::scrollable::AbsoluteOffset { x: 0.0, y: 40.0 }
                );
            }
            Task::none()
        }
        Message::SelectEntry(index) => {
            if state.mode == Mode::Launcher {
                state.selected_index = index;
                launch_selected(state);
            }
            Task::none()
        }
        Message::WindowFocused => {
            iced::widget::operation::focus(Id::new("search-input"))
        }
        _ => Task::none(),
    }
}

fn cycle_theme(state: &mut CastIt, direction: i32) {
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

fn cycle_terminal(state: &mut CastIt, direction: i32) {
    let terminals = [
        "Auto",
        "kitty",
        "alacritty",
        "wezterm",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "xterm",
    ];
    let current = state.config.terminal.as_deref().unwrap_or("Auto");
    let current_idx = terminals.iter().position(|&t| t == current).unwrap_or(0);
    let new_idx = if direction > 0 {
        (current_idx + 1) % terminals.len()
    } else {
        (current_idx + terminals.len() - 1) % terminals.len()
    };
    state.config.terminal = if terminals[new_idx] == "Auto" {
        None
    } else {
        Some(terminals[new_idx].to_string())
    };
    state.config.save();
}

/// Helper to parse path query into parent folder and filter query
fn parse_path_query(query: &str) -> Option<(String, String)> {
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

/// Reads directory entries from disk and sorts them (directories first, then alphabetically)
fn read_directory(path: &str) -> Vec<FileEntry> {
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

/// Performs fuzzy filtering on the directory entries using nucleo-matcher
fn update_filtered_files(state: &mut CastIt) {
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

/// Runs fuzzy matching against all entries using nucleo-matcher.
fn update_filtered_entries(state: &mut CastIt) {
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

/// Launches the currently selected application.
fn launch_selected(state: &CastIt) {
    if let Some((entry, _)) = state.filtered_entries.get(state.selected_index) {
        // Split the exec command and spawn it detached
        let parts: Vec<&str> = entry.exec.split_whitespace().collect();
        if let Some((program, args)) = parts.split_first() {
            let _ = Command::new(program)
                .args(args)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
        std::process::exit(0);
    }
}

// ---------------------------------------------------------------------------
// View
// ---------------------------------------------------------------------------

fn view(state: &CastIt) -> Element<'_, Message> {
    let lang = state.config.language.as_deref().unwrap_or("EN");
    let active_theme = resolve_iced_theme(state.config.theme.as_deref().unwrap_or("TokyoNight"));
    let palette = active_theme.palette();
    let opacity = state.config.opacity.unwrap_or(0.92);

    let input = text_input(translate("search_placeholder", lang), &state.query)
        .on_input(Message::QueryChanged)
        .on_submit(Message::Submit)
        .padding(Padding { top: 16.0, right: 20.0, bottom: 16.0, left: 20.0 })
        .size(16)
        .id(Id::new("search-input"))
        .style(move |theme: &iced::Theme, _status| {
            let pal = theme.palette();
            text_input::Style {
                background: iced::Background::Color(Color::TRANSPARENT),
                border: iced::Border::default(),
                icon: pal.text,
                placeholder: Color { a: 0.35, ..pal.text },
                value: pal.text,
                selection: Color { a: 0.25, ..pal.primary },
            }
        });

    let search_pill = container(input)
        .width(Length::Fill)
        .style(move |theme: &iced::Theme| {
            let bg = theme.palette().background;
            container::Style {
                background: Some(iced::Background::Color(Color { a: opacity, ..bg })),
                border: iced::Border {
                    radius: 26.0.into(),
                    width: 1.0,
                    color: Color { a: 0.12, ..theme.palette().text },
                },
                ..Default::default()
            }
        });

    let show_results_card = match state.mode {
        Mode::Launcher => !state.filtered_entries.is_empty() || !state.query.is_empty(),
        Mode::FileBrowser => !state.filtered_files.is_empty() || !state.query.is_empty(),
        _ => true,
    };

    let mut main_layout = Column::new()
        .spacing(10)
        .width(720);

    main_layout = main_layout.push(search_pill);

    if show_results_card {
        let card_content = match state.mode {
            Mode::Launcher => {
                if !state.filtered_entries.is_empty() {
                    let mut results = Column::new()
                        .spacing(4)
                        .padding(Padding { top: 10.0, right: 10.0, bottom: 10.0, left: 10.0 });

                    for (i, (entry, _score)) in state.filtered_entries.iter().enumerate() {
                        let is_selected = i == state.selected_index;
                        results = results.push(result_row(entry, is_selected, i, palette, lang));
                    }

                    scrollable(results).height(Length::Shrink).id(Id::new("scroll-list")).into()
                } else {
                    container(
                        text(translate("no_results", lang))
                            .size(14)
                            .color(Color { a: 0.5, ..palette.text }),
                    )
                    .padding(Padding { top: 16.0, right: 20.0, bottom: 16.0, left: 20.0 })
                    .into()
                }
            }
            Mode::FileBrowser => {
                if state.preview_active {
                    if let Some((entry, _)) = state.filtered_files.get(state.selected_index) {
                        preview_pane(entry, palette, lang)
                    } else {
                        container(
                            text(translate("no_files", lang))
                                .size(14)
                                .color(Color { a: 0.5, ..palette.text }),
                        )
                        .padding(Padding { top: 16.0, right: 20.0, bottom: 16.0, left: 20.0 })
                        .into()
                    }
                } else if !state.filtered_files.is_empty() {
                    let mut results = Column::new()
                        .spacing(4)
                        .padding(Padding { top: 10.0, right: 10.0, bottom: 10.0, left: 10.0 });

                    for (i, (entry, _score)) in state.filtered_files.iter().enumerate() {
                        let is_selected = i == state.selected_index;
                        results = results.push(file_row(entry, is_selected, i, palette, lang));
                    }

                    scrollable(results).height(Length::Shrink).id(Id::new("scroll-list")).into()
                } else {
                    container(
                        text(translate("no_files", lang))
                            .size(14)
                            .color(Color { a: 0.5, ..palette.text }),
                    )
                    .padding(Padding { top: 16.0, right: 20.0, bottom: 16.0, left: 20.0 })
                    .into()
                }
            }
            Mode::CommandRunner => command_runner_view(state, palette, lang),
            Mode::Settings => settings_view(state, palette, lang),
            Mode::Help => help_view(palette, lang),
        };

        let results_card = container(card_content)
            .width(Length::Fill)
            .style(move |theme: &iced::Theme| {
                let bg = theme.palette().background;
                container::Style {
                    background: Some(iced::Background::Color(Color { a: opacity, ..bg })),
                    border: iced::Border {
                        radius: 16.0.into(),
                        width: 1.0,
                        color: Color { a: 0.1, ..theme.palette().text },
                    },
                    ..Default::default()
                }
            });

        main_layout = main_layout.push(results_card);
    }

    container(main_layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(Padding { top: 40.0, right: 16.0, bottom: 16.0, left: 16.0 })
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(Color::TRANSPARENT)),
            ..Default::default()
        })
        .into()
}

/// Renders the Terminal Output preview block.
fn command_runner_view<'a>(state: &'a CastIt, palette: iced::theme::Palette, lang: &str) -> Element<'a, Message> {
    let mut console_content = Column::new().spacing(8);

    // Render the executed command prompt at the top of the box
    if let Some(cmd) = match &state.runner_state {
        RunnerState::Running { command } => Some(command),
        RunnerState::Finished { command, .. } => Some(command),
        RunnerState::Failed { command, .. } => Some(command),
        _ => None,
    } {
        let prompt_line = text(format!("$ {}", cmd))
            .size(13)
            .font(Font::MONOSPACE)
            .color(palette.primary);
        console_content = console_content.push(prompt_line);
    }

    let mut text_color = palette.text;
    let content_text = match &state.runner_state {
        RunnerState::Idle => {
            text_color = Color { a: 0.5, ..palette.text };
            translate("cmd_idle", lang).to_string()
        }
        RunnerState::Running { .. } => {
            text_color = palette.text;
            translate("cmd_running", lang).to_string()
        }
        RunnerState::Finished { output, .. } => {
            if output.trim().is_empty() {
                text_color = palette.success;
                translate("cmd_success", lang).to_string()
            } else {
                output.clone()
            }
        }
        RunnerState::Failed { error, .. } => {
            text_color = palette.danger;
            if error.trim().is_empty() {
                translate("cmd_failed", lang).to_string()
            } else {
                error.clone()
            }
        }
    };

    let console_text = text(content_text)
        .size(13)
        .font(Font::MONOSPACE)
        .color(text_color);

    console_content = console_content.push(console_text);

    let console = container(
        scrollable(console_content)
            .height(Length::Fixed(350.0))
    )
    .padding(16)
    .width(Length::Fill)
    .style(|theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Color { a: 0.04, ..theme.palette().text })),
        border: iced::Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    container(console)
        .padding(Padding { top: 12.0, right: 12.0, bottom: 12.0, left: 12.0 })
        .into()
}

/// Renders the Settings panel.
fn settings_view<'a>(state: &'a CastIt, palette: iced::theme::Palette, lang: &str) -> Element<'a, Message> {
    let mut settings_list = Column::new()
        .spacing(4)
        .padding(Padding { top: 12.0, right: 10.0, bottom: 12.0, left: 10.0 });

    let active_theme = state.config.theme.as_deref().unwrap_or("TokyoNight");
    let active_term = state.config.terminal.as_deref().unwrap_or("Auto");
    let opacity_val = format!("{:.2}", state.config.opacity.unwrap_or(0.92));
    let width_val = format!("{} px", state.config.width.unwrap_or(800));
    let height_val = format!("{} px", state.config.height.unwrap_or(500));
    
    let config_lang = state.config.language.as_deref().unwrap_or("EN");
    let lang_display = if config_lang == "ES" { "Español (ES)" } else { "English (EN)" };

    settings_list = settings_list.push(settings_row(translate("setting_theme", lang), active_theme, state.selected_setting == 0, palette));
    settings_list = settings_list.push(settings_row(translate("setting_terminal", lang), active_term, state.selected_setting == 1, palette));
    settings_list = settings_list.push(settings_row(translate("setting_opacity", lang), &opacity_val, state.selected_setting == 2, palette));
    settings_list = settings_list.push(settings_row(translate("setting_width", lang), &width_val, state.selected_setting == 3, palette));
    settings_list = settings_list.push(settings_row(translate("setting_height", lang), &height_val, state.selected_setting == 4, palette));
    settings_list = settings_list.push(settings_row(translate("setting_language", lang), lang_display, state.selected_setting == 5, palette));

    let info_text = text(translate("info_tip", lang))
        .size(11)
        .color(Color { a: 0.4, ..palette.text });

    let settings_box = Column::new()
        .spacing(2)
        .push(scrollable(settings_list).height(Length::Shrink))
        .push(container(info_text).padding(Padding { top: 4.0, right: 20.0, bottom: 12.0, left: 20.0 }));

    container(settings_box)
        .width(Length::Fill)
        .into()
}

/// Renders the keyboard shortcuts cheatsheet view in 2 columns.
fn help_view<'a>(palette: iced::theme::Palette, lang: &str) -> Element<'a, Message> {
    let mut left_column = Column::new().spacing(12).width(Length::FillPortion(1));
    let mut right_column = Column::new().spacing(12).width(Length::FillPortion(1));

    let title = text(if lang == "ES" { "Atajos de Teclado" } else { "Keyboard Shortcuts" })
        .size(15)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .color(palette.primary);

    let categories = if lang == "ES" {
        vec![
            ("Global", vec![
                ("Esc", "Salir de CastIt"),
                ("Shift + Del", "Limpiar barra de búsqueda"),
                ("..", "Ir a Ajustes"),
                ("??", "Ver atajos de teclado (esta pantalla)"),
            ], true),
            ("Launcher / Apps", vec![
                ("↑ / ↓", "Navegar resultados"),
                ("Enter", "Lanzar aplicación seleccionada"),
            ], true),
            ("Comandos ('>')", vec![
                ("Enter", "Ejecutar comando en segundo plano"),
                ("Ctrl + Enter", "Ejecutar comando en terminal externa"),
            ], true),
            ("Archivos ('/', '~')", vec![
                ("↑ / ↓", "Navegar archivos / carpetas"),
                ("→", "Autocompletar / Entrar en carpeta"),
                ("Shift + ←", "Subir al directorio superior"),
                ("Enter", "Abrir archivo / Carpeta nativa"),
                ("Ctrl + Espacio", "Previsualizar archivo (Quick Look)"),
            ], false),
            ("Previsualización de Archivo", vec![
                ("Shift + ↑ / ↓", "Deslizar (scroll) contenido de archivo"),
                ("↑ / ↓", "Cambiar previsualización al anterior/sig"),
                ("Ctrl + Espacio", "Cerrar previsualización"),
            ], false),
        ]
    } else {
        vec![
            ("Global", vec![
                ("Esc", "Exit CastIt"),
                ("Shift + Del", "Clear search bar input"),
                ("..", "Go to Settings"),
                ("??", "View keyboard shortcuts (this screen)"),
            ], true),
            ("Launcher / Apps", vec![
                ("↑ / ↓", "Navigate launcher results"),
                ("Enter", "Launch selected application"),
            ], true),
            ("Commands ('>')", vec![
                ("Enter", "Run command in background"),
                ("Ctrl + Enter", "Run command in external terminal"),
            ], true),
            ("Files ('/', '~')", vec![
                ("↑ / ↓", "Navigate file list"),
                ("→", "Autocomplete / Enter folder"),
                ("Shift + ←", "Navigate to parent folder"),
                ("Enter", "Open file / Native folder manager"),
                ("Ctrl + Space", "Preview file (Quick Look)"),
            ], false),
            ("File Preview", vec![
                ("Shift + ↑ / ↓", "Scroll through file content preview"),
                ("↑ / ↓", "Switch preview to previous/next file"),
                ("Ctrl + Space", "Close file preview"),
            ], false),
        ]
    };

    for (cat_name, items, is_left) in categories {
        let mut cat_section = Column::new().spacing(6);
        let cat_label = text(cat_name)
            .size(11)
            .font(iced::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            })
            .color(Color { a: 0.5, ..palette.text });
        cat_section = cat_section.push(cat_label);

        for (keys, desc) in items {
            let keys_badge = container(
                text(keys)
                    .size(9)
                    .font(Font::MONOSPACE)
                    .color(palette.primary)
            )
            .padding(Padding::from([2, 5]))
            .style(move |theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(Color { a: 0.08, ..theme.palette().primary })),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

            let desc_text = text(desc)
                .size(11)
                .color(Color { a: 0.85, ..palette.text });

            let item_row = row![
                keys_badge,
                Space::new().width(Length::Fixed(10.0)),
                desc_text,
            ]
            .align_y(iced::Alignment::Center);

            cat_section = cat_section.push(
                container(item_row)
                    .padding(Padding::from([3, 4]))
            );
        }

        if is_left {
            left_column = left_column.push(cat_section).push(Space::new().height(Length::Fixed(8.0)));
        } else {
            right_column = right_column.push(cat_section).push(Space::new().height(Length::Fixed(8.0)));
        }
    }

    let grid_layout = row![
        left_column,
        Space::new().width(Length::Fixed(24.0)),
        right_column,
    ]
    .spacing(0);

    let main_body = Column::new()
        .spacing(12)
        .padding(Padding { top: 12.0, right: 14.0, bottom: 12.0, left: 14.0 })
        .push(title)
        .push(grid_layout);

    container(scrollable(main_body).height(Length::Fixed(350.0)).id(Id::new("help-scroll")))
        .width(Length::Fill)
        .into()
}

fn settings_row(label: &str, value: &str, selected: bool, palette: iced::theme::Palette) -> Element<'static, Message> {
    let label_text = text(label.to_string())
        .size(14)
        .color(if selected { palette.text } else { Color { a: 0.8, ..palette.text } });

    let value_text = text(value.to_string())
        .size(13)
        .font(Font::MONOSPACE)
        .color(if selected { palette.primary } else { Color { a: 0.6, ..palette.text } });

    let shortcut_tag = if selected {
        container(text("← / →").size(10).color(palette.primary))
            .padding(Padding::from([2, 6]))
            .style(move |theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(Color { a: 0.1, ..theme.palette().primary })),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            })
    } else {
        container(Space::new().width(0).height(0))
    };

    let content = row![
        label_text,
        Space::new().width(Length::Fill),
        value_text,
        Space::new().width(Length::Fixed(12.0)),
        shortcut_tag,
    ]
    .spacing(0)
    .align_y(iced::Alignment::Center);

    container(content)
        .width(Length::Fill)
        .padding(Padding::from([10, 14]))
        .style(move |theme: &iced::Theme| {
            let pal = theme.palette();
            let bg = if selected {
                Color { a: 0.08, ..pal.primary }
            } else {
                Color::TRANSPARENT
            };
            container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

/// Renders a single application result row
fn result_row(entry: &AppEntry, selected: bool, _index: usize, palette: iced::theme::Palette, lang: &str) -> Element<'static, Message> {
    let name_text = text(entry.name.clone())
        .size(14)
        .color(if selected { palette.text } else { Color { a: 0.9, ..palette.text } });

    let mut row_content = row![].spacing(12).align_y(iced::Alignment::Center);

    if let Some(path) = &entry.icon_path {
        if path.ends_with(".svg") {
            let svg_handle = svg::Handle::from_path(path);
            row_content = row_content.push(svg(svg_handle).width(28).height(28));
        } else {
            row_content = row_content.push(image(path).width(28).height(28));
        }
    } else {
        row_content = row_content.push(Space::new().width(Length::Fixed(28.0)));
    }

    let mut details = Column::new().spacing(2);
    details = details.push(name_text);

    if let Some(desc) = &entry.description {
        let desc_text = text(desc.clone())
            .size(11)
            .color(Color { a: 0.45, ..palette.text });
        details = details.push(desc_text);
    }

    row_content = row_content.push(details);
    row_content = row_content.push(Space::new().width(Length::Fill));

    if selected {
        let tag = container(text(translate("launch_tag", lang)).size(10).color(palette.primary))
            .padding(Padding::from([3, 8]))
            .style(move |theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(Color { a: 0.1, ..theme.palette().primary })),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        row_content = row_content.push(tag);
    }

    container(row_content)
        .width(Length::Fill)
        .padding(Padding::from([8, 12]))
        .style(move |theme: &iced::Theme| {
            let pal = theme.palette();
            let bg = if selected {
                Color { a: 0.08, ..pal.primary }
            } else {
                Color::TRANSPARENT
            };
            container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

/// Renders a single file/directory result row
fn file_row(entry: &FileEntry, selected: bool, _index: usize, palette: iced::theme::Palette, lang: &str) -> Element<'static, Message> {
    let name_text = text(entry.name.clone())
        .size(14)
        .color(if selected { palette.text } else { Color { a: 0.9, ..palette.text } });

    let mut row_content = row![].spacing(12).align_y(iced::Alignment::Center);

    if let Some(path) = &entry.icon_path {
        if path.ends_with(".svg") {
            let svg_handle = svg::Handle::from_path(path);
            row_content = row_content.push(svg(svg_handle).width(28).height(28));
        } else {
            row_content = row_content.push(image(path).width(28).height(28));
        }
    } else {
        row_content = row_content.push(Space::new().width(Length::Fixed(28.0)));
    }

    let mut details = Column::new().spacing(2);
    details = details.push(name_text);

    let clean_path = if entry.is_dir {
        entry.path.clone()
    } else {
        if let Some(idx) = entry.path.rfind('/') {
            entry.path[..idx].to_string()
        } else {
            entry.path.clone()
        }
    };

    let mut display_path = clean_path;
    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() && display_path.starts_with(&home) {
        display_path = display_path.replacen(&home, "~", 1);
    }

    let desc_text = text(display_path)
        .size(11)
        .color(Color { a: 0.45, ..palette.text });
    details = details.push(desc_text);

    row_content = row_content.push(details);
    row_content = row_content.push(Space::new().width(Length::Fill));

    if selected {
        let tag_text = if entry.is_dir {
            if lang == "ES" { "⏎ Abrir | → Navegar" } else { "⏎ Open | → Navigate" }
        } else {
            if lang == "ES" { "⏎ Lanzar | Ctrl+Espacio Vista previa" } else { "⏎ Launch | Ctrl+Space Preview" }
        };
        let tag = container(text(tag_text).size(10).color(palette.primary))
            .padding(Padding::from([3, 8]))
            .style(move |theme: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(Color { a: 0.1, ..theme.palette().primary })),
                border: iced::Border {
                    radius: 4.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });
        row_content = row_content.push(tag);
    }

    container(row_content)
        .width(Length::Fill)
        .padding(Padding::from([8, 12]))
        .style(move |theme: &iced::Theme| {
            let pal = theme.palette();
            let bg = if selected {
                Color { a: 0.08, ..pal.primary }
            } else {
                Color::TRANSPARENT
            };
            container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    radius: 6.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

/// Renders the Quick Look Document/Folder Preview pane.
fn preview_pane<'a>(entry: &FileEntry, palette: iced::theme::Palette, lang: &str) -> Element<'a, Message> {
    let mut layout = Column::new()
        .spacing(10)
        .padding(16)
        .width(Length::Fill)
        .align_x(iced::Alignment::Center);

    // Filename Header
    let filename = text(entry.name.clone())
        .size(15)
        .font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .color(palette.text);
    layout = layout.push(filename);

    // Metadata loading (size and type)
    let mut metadata_info = String::new();
    if let Ok(meta) = std::fs::metadata(&entry.path) {
        let size = meta.len();
        let size_str = if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        };
        
        let type_display = if entry.is_dir {
            (if lang == "ES" { "Carpeta" } else { "Folder" }).to_string()
        } else {
            if let Some(ext) = std::path::Path::new(&entry.path).extension() {
                format!("{} File", ext.to_string_lossy().to_uppercase())
            } else {
                (if lang == "ES" { "Archivo" } else { "File" }).to_string()
            }
        };
        metadata_info = format!("{}  •  {}", type_display, size_str);
    }

    if !metadata_info.is_empty() {
        let meta_text = text(metadata_info)
            .size(11)
            .color(Color { a: 0.45, ..palette.text });
        layout = layout.push(meta_text);
    }

    layout = layout.push(Space::new().height(Length::Fixed(4.0)));

    // Preview Pane Body depending on extension
    let preview_body: Element<'a, Message> = if entry.is_dir {
        // Folder big icon
        let icon_widget = if let Some(path) = &entry.icon_path {
            if path.ends_with(".svg") {
                let svg_handle = svg::Handle::from_path(path);
                Element::from(svg(svg_handle).width(128).height(128))
            } else {
                Element::from(image(path).width(128).height(128))
            }
        } else {
            Element::from(text("📁").size(64))
        };
        Element::from(container(icon_widget).padding(10))
    } else {
        let path = std::path::Path::new(&entry.path);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        
        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "gif" | "webp" => {
                let img = image(entry.path.clone())
                    .width(Length::Fixed(450.0))
                    .height(Length::Fixed(250.0))
                    .content_fit(ContentFit::Contain);
                Element::from(container(img)
                    .width(Length::Fill)
                    .height(Length::Fixed(250.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center))
            }
            "svg" => {
                let svg_handle = svg::Handle::from_path(&entry.path);
                let svg_widget = svg(svg_handle)
                    .width(Length::Fixed(250.0))
                    .height(Length::Fixed(250.0));
                Element::from(container(svg_widget)
                    .width(Length::Fill)
                    .height(Length::Fixed(250.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center))
            }
            // Code & Text Files (Expanded Dev extensions)
            "rs" | "toml" | "json" | "txt" | "md" | "css" | "js" | "ts" | "tsx" | "jsx" | "html" | "sh" | "bash" | "py" | "c" | "cpp" | "h" | "hpp" | "go" | "java" | "kt" | "scala" | "xml" | "gradle" | "properties" | "conf" | "ini" | "sql" | "yaml" | "yml" => {
                let content = std::fs::read_to_string(&entry.path)
                    .map(|s| {
                        s.lines()
                            .take(300)
                            .collect::<Vec<&str>>()
                            .join("\n")
                    })
                    .unwrap_or_else(|_| {
                        if lang == "ES" { "No se puede leer el archivo" } else { "Cannot read file contents" }.to_string()
                    });

                let text_box = container(
                    scrollable(
                        text(content)
                            .size(11)
                            .font(Font::MONOSPACE)
                            .color(palette.text)
                    )
                    .height(Length::Fixed(250.0))
                    .id(Id::new("preview-scroll"))
                )
                .padding(12)
                .width(Length::Fill)
                .style(|theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(Color { a: 0.04, ..theme.palette().text })),
                    border: iced::Border {
                        radius: 8.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

                Element::from(text_box)
            }
            // Unsupported binary files
            _ => {
                let icon_widget = if let Some(path) = &entry.icon_path {
                    if path.ends_with(".svg") {
                        let svg_handle = svg::Handle::from_path(path);
                        Element::from(svg(svg_handle).width(96).height(96))
                    } else {
                        Element::from(image(path).width(96).height(96))
                    }
                } else {
                    Element::from(text("📄").size(64))
                };

                let extension_tag = container(
                    text(ext.to_uppercase())
                        .size(11)
                        .font(iced::Font {
                            weight: iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .color(palette.primary)
                )
                .padding(Padding::from([4, 10]))
                .style(|theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(Color { a: 0.1, ..theme.palette().primary })),
                    border: iced::Border {
                        radius: 6.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                });

                let info_box = Column::new()
                    .spacing(12)
                    .align_x(iced::Alignment::Center)
                    .push(icon_widget)
                    .push(extension_tag);

                Element::from(container(info_box)
                    .height(Length::Fixed(250.0))
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center))
            }
        }
    };

    layout = layout.push(preview_body);
    layout = layout.push(Space::new().height(Length::Fixed(4.0)));

    // Return tip
    let tip = text(if lang == "ES" { "Presiona Ctrl + Espacio para volver" } else { "Press Ctrl + Space to return" })
        .size(10)
        .color(Color { a: 0.35, ..palette.text });
    layout = layout.push(tip);

    container(layout)
        .width(Length::Fill)
        .into()
}

// ---------------------------------------------------------------------------
// Keyboard subscription
// ---------------------------------------------------------------------------

fn subscription(state: &CastIt) -> iced::Subscription<Message> {
    match state.mode {
        Mode::Settings => {
            iced::event::listen_with(|event, _status, _id| match event {
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    match key {
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => Some(Message::Escape),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) => Some(Message::ArrowDown),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) => Some(Message::ArrowUp),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft) => Some(Message::ArrowLeft),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight) => Some(Message::ArrowRight),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete) if modifiers.shift() => Some(Message::ClearQuery),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace) if modifiers.shift() => Some(Message::ClearQuery),
                        _ => None,
                    }
                }
                iced::Event::Window(iced::window::Event::Focused) => Some(Message::WindowFocused),
                _ => None,
            })
        }
        Mode::FileBrowser => {
            iced::event::listen_with(|event, _status, _id| match event {
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    match key {
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => Some(Message::Escape),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                            if modifiers.shift() {
                                Some(Message::ScrollPreviewDown)
                            } else {
                                Some(Message::ArrowDown)
                            }
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                            if modifiers.shift() {
                                Some(Message::ScrollPreviewUp)
                            } else {
                                Some(Message::ArrowUp)
                            }
                        }
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft) if modifiers.shift() => Some(Message::ArrowLeft),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight) => Some(Message::ArrowRight),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Space) if modifiers.control() => Some(Message::TogglePreview),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete) if modifiers.shift() => Some(Message::ClearQuery),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace) if modifiers.shift() => Some(Message::ClearQuery),
                        _ => None,
                    }
                }
                iced::Event::Window(iced::window::Event::Focused) => Some(Message::WindowFocused),
                _ => None,
            })
        }
        _ => {
            iced::event::listen_with(|event, _status, _id| match event {
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    match key {
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => Some(Message::Escape),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) => Some(Message::ArrowDown),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) => Some(Message::ArrowUp),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete) if modifiers.shift() => Some(Message::ClearQuery),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace) if modifiers.shift() => Some(Message::ClearQuery),
                        iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) if modifiers.control() => Some(Message::SubmitInTerminal),
                        _ => None,
                    }
                }
                iced::Event::Window(iced::window::Event::Focused) => Some(Message::WindowFocused),
                _ => None,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run() -> Result<(), iced_layershell::Error> {
    let config = config::Config::load();
    let width = config.width.unwrap_or(800) as u32;
    let height = config.height.unwrap_or(500) as u32;

    application(CastIt::new, namespace, update, view)
        .subscription(subscription)
        .theme(|state: &CastIt| {
            Some(resolve_iced_theme(state.config.theme.as_deref().unwrap_or("TokyoNight")))
        })
        .style(|_state, _theme| iced::theme::Style {
            background_color: iced::Color::TRANSPARENT,
            text_color: iced::Color::WHITE,
        })
        .settings(Settings {
            layer_settings: LayerShellSettings {
                anchor: Anchor::empty(),
                layer: Layer::Overlay,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                exclusive_zone: -1,
                size: Some((width, height)),
                start_mode: StartMode::Active,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}
