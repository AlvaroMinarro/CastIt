use iced::widget::Id;
use iced::Task;
use std::process::Command;

use crate::infra::runner;
use super::message::Message;
use super::state::{
    CastIt, Mode, RunnerState, cycle_terminal, cycle_theme, launch_selected,
    update_filtered_entries, update_filtered_files,
};

pub fn update(state: &mut CastIt, message: Message) -> Task<Message> {
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
