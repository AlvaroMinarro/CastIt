use iced::widget::{container, image, scrollable, svg, text, Column, Id, Space};
use iced::{Color, ContentFit, Element, Font, Length, Padding};
use iced::alignment::{Horizontal, Vertical};
use std::fs;
use std::path::Path;

use super::message::Message;
use super::state::FileEntry;

pub fn preview_pane<'a>(entry: &FileEntry, palette: iced::theme::Palette, lang: &str) -> Element<'a, Message> {
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
    if let Ok(meta) = fs::metadata(&entry.path) {
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
            if let Some(ext) = Path::new(&entry.path).extension() {
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
        let path = Path::new(&entry.path);
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
                let content = fs::read_to_string(&entry.path)
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
