use iced::widget::{container, image, row, svg, text, Column, Space};
use iced::{Color, Element, Length, Padding};

use crate::app::message::Message;
use crate::app::state::FileEntry;

/// Renders a single file/directory result row
pub fn file_row(
    entry: &FileEntry,
    selected: bool,
    _index: usize,
    palette: iced::theme::Palette,
    lang: &str,
) -> Element<'static, Message> {
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
