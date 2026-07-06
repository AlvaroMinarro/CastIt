use iced::widget::{container, scrollable, text, Column};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::message::Message;
use crate::app::state::{CastIt, RunnerState};
use crate::app::view::translate;

/// Renders the Terminal Output preview block.
pub fn command_runner_view<'a>(
    state: &'a CastIt,
    palette: iced::theme::Palette,
    lang: &str,
) -> Element<'a, Message> {
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
