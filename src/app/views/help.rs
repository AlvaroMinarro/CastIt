use iced::widget::{container, scrollable, text, Column, row, Space, Id};
use iced::{Color, Element, Font, Length, Padding};

use crate::app::message::Message;

/// Renders the keyboard shortcuts cheatsheet view in 2 columns.
pub fn help_view<'a>(
    palette: iced::theme::Palette,
    lang: &str,
) -> Element<'a, Message> {
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
                ("Ctrl + D", "Añadir/quitar app favorita (Dashboard)"),
            ], true),
            ("Comandos ('>')", vec![
                ("Enter", "Ejecutar comando en segundo plano"),
                ("Ctrl + Enter", "Ejecutar comando en terminal externa"),
            ], true),
            ("Calculadora", vec![
                ("Enter", "Copiar resultado al portapapeles"),
            ], true),
            ("Archivos ('/', '~')", vec![
                ("↑ / ↓", "Navegar archivos / carpetas"),
                ("→", "Autocompletar / Entrar en carpeta"),
                ("Shift + ←", "Subir al directorio superior"),
                ("Enter", "Abrir archivo / Carpeta nativa"),
                ("Ctrl + Espacio", "Previsualizar archivo (Quick Look)"),
            ], false),
            ("Búsqueda Global ('f ')", vec![
                ("f <archivo>", "Buscar archivos recursivamente en ~"),
            ], false),
            ("Buscador Web ('?')", vec![
                ("? <texto>", "Buscar consulta en Google"),
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
                ("Ctrl + D", "Toggle app favorite (Dashboard)"),
            ], true),
            ("Commands ('>')", vec![
                ("Enter", "Run command in background"),
                ("Ctrl + Enter", "Run command in external terminal"),
            ], true),
            ("Calculator", vec![
                ("Enter", "Copy result to clipboard"),
            ], true),
            ("Files ('/', '~')", vec![
                ("↑ / ↓", "Navigate file list"),
                ("→", "Autocomplete / Enter folder"),
                ("Shift + ←", "Navigate to parent folder"),
                ("Enter", "Open file / Native folder manager"),
                ("Ctrl + Space", "Preview file (Quick Look)"),
            ], false),
            ("Global Search ('f ')", vec![
                ("f <file>", "Search files recursively in ~"),
            ], false),
            ("Web Search ('?')", vec![
                ("? <query>", "Search query on Google"),
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
