use iced::widget::{container, text, row, Space};
use iced::{Color, Element, Length, Padding};

use crate::app::message::Message;
use crate::app::state::CastIt;

pub fn web_search_view<'a>(
    state: &'a CastIt,
    palette: iced::theme::Palette,
    lang: &str,
) -> Element<'a, Message> {
    let query_val = state.query.strip_prefix('?').unwrap_or(&state.query).trim();
    
    let icon = text("🌐")
        .size(24)
        .color(palette.primary);

    let label_prefix = if lang == "ES" {
        "Buscar en Google: "
    } else {
        "Search on Google: "
    };

    let label = text(format!("{}{}", label_prefix, query_val))
        .size(14)
        .color(palette.text);

    let submit_tag = container(
        text("⏎ Google")
            .size(10)
            .color(palette.primary)
    )
    .padding(Padding::from([3, 7]))
    .style(move |theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(Color { a: 0.1, ..theme.palette().primary })),
        border: iced::Border {
            radius: 4.0.into(),
            ..Default::default()
        },
        ..Default::default()
    });

    let row_content = row![
        icon,
        Space::new().width(Length::Fixed(16.0)),
        label,
        Space::new().width(Length::Fill),
        submit_tag,
    ]
    .align_y(iced::Alignment::Center);

    container(row_content)
        .width(Length::Fill)
        .padding(Padding::from([14, 18]))
        .style(move |theme: &iced::Theme| {
            let bg = theme.palette().background;
            container::Style {
                background: Some(iced::Background::Color(Color { a: 0.05, ..bg })),
                ..Default::default()
            }
        })
        .into()
}
