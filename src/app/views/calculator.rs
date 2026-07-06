use iced::widget::{container, text, Column, row, Space};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::message::Message;
use crate::app::state::CastIt;

pub fn calculator_view<'a>(
    state: &'a CastIt,
    palette: iced::theme::Palette,
    lang: &str,
) -> Element<'a, Message> {
    let result_val = state.calculator_result.unwrap_or(0.0);
    
    let expression_label = text(&state.query)
        .size(14)
        .font(Font::MONOSPACE)
        .color(Color { a: 0.5, ..palette.text });

    let result_label = text(format!("= {}", result_val))
        .size(28)
        .font(Font {
            weight: iced::font::Weight::Bold,
            ..Default::default()
        })
        .color(palette.primary);

    let copy_tag = container(
        text(if lang == "ES" { "⏎ Copiar" } else { "⏎ Copy" })
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

    let footer_text = text(if lang == "ES" {
        "Presiona Enter para copiar el resultado al portapapeles"
    } else {
        "Press Enter to copy the result to your clipboard"
    })
    .size(11)
    .color(Color { a: 0.4, ..palette.text });

    let footer_row = row![
        footer_text,
        Space::new().width(Length::Fill),
        copy_tag,
    ]
    .align_y(iced::Alignment::Center);

    let body = Column::new()
        .spacing(12)
        .push(expression_label)
        .push(result_label)
        .push(Space::new().height(Length::Fixed(8.0)))
        .push(footer_row);

    container(body)
        .width(Length::Fill)
        .padding(Padding::from([18, 22]))
        .style(move |theme: &iced::Theme| {
            let bg = theme.palette().background;
            container::Style {
                background: Some(iced::Background::Color(Color { a: 0.05, ..bg })),
                ..Default::default()
            }
        })
        .into()
}
