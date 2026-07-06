use iced::widget::{container, scrollable, text, Column, row, Space};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::message::Message;
use crate::app::state::CastIt;
use crate::app::view::translate;

/// Renders the Settings panel.
pub fn settings_view<'a>(
    state: &'a CastIt,
    palette: iced::theme::Palette,
    lang: &str,
) -> Element<'a, Message> {
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

pub fn settings_row(
    label: &str,
    value: &str,
    selected: bool,
    palette: iced::theme::Palette,
) -> Element<'static, Message> {
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
