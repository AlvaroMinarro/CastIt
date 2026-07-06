use iced::widget::{container, image, row, svg, text, Column, Space};
use iced::{Color, Element, Length, Padding};

use crate::domain::models::AppEntry;
use crate::app::message::Message;
use crate::app::view::translate;

/// Renders a single application result row
pub fn result_row(
    entry: &AppEntry,
    selected: bool,
    is_favorite: bool,
    is_recent: bool,
    _index: usize,
    palette: iced::theme::Palette,
    lang: &str,
) -> Element<'static, Message> {
    let name_text = text(entry.name.clone())
        .size(14)
        .color(if selected { palette.text } else { Color { a: 0.9, ..palette.text } });

    let icon_element: Element<'static, Message> = if let Some(path) = &entry.icon_path {
        if path.ends_with(".svg") {
            let svg_handle = svg::Handle::from_path(path);
            svg(svg_handle).width(28).height(28).into()
        } else {
            image(path).width(28).height(28).into()
        }
    } else {
        Space::new().width(Length::Fixed(28.0)).height(Length::Fixed(28.0)).into()
    };

    let styled_icon = container(icon_element)
        .padding(3)
        .style(move |theme: &iced::Theme| {
            let pal = theme.palette();
            if is_favorite {
                container::Style {
                    border: iced::Border {
                        color: Color { r: 0.95, g: 0.75, b: 0.2, a: 0.8 }, // Gold
                        width: 1.5,
                        radius: 6.0.into(),
                    },
                    background: Some(iced::Background::Color(Color { r: 0.95, g: 0.75, b: 0.2, a: 0.1 })),
                    ..Default::default()
                }
            } else if is_recent {
                container::Style {
                    border: iced::Border {
                        color: Color { r: pal.primary.r, g: pal.primary.g, b: pal.primary.b, a: 0.5 }, // Soft primary
                        width: 1.5,
                        radius: 6.0.into(),
                    },
                    background: Some(iced::Background::Color(Color { r: pal.primary.r, g: pal.primary.g, b: pal.primary.b, a: 0.06 })),
                    ..Default::default()
                }
            } else {
                container::Style::default()
            }
        });

    let mut row_content = row![styled_icon].spacing(12).align_y(iced::Alignment::Center);

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
