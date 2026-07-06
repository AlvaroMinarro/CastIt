use iced::widget::{container, scrollable, text, text_input, Column, Id};
use iced::{Color, Element, Length, Padding};

use super::message::Message;
use super::state::{CastIt, Mode};

pub fn translate<'a>(key: &'a str, lang: &str) -> &'a str {
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
            "setting_browser" => "Navegador Web Preferido",
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
            "setting_browser" => "Preferred Web Browser",
            "launch_tag" => "⏎ Launch",
            _ => key,
        },
    }
}

pub fn resolve_iced_theme(name: &str) -> iced::Theme {
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

pub fn view(state: &CastIt) -> Element<'_, Message> {
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
                        let is_favorite = state.config.favorites.as_ref().map_or(false, |f| f.contains(&entry.exec));
                        let is_recent = state.config.history.as_ref().map_or(false, |h| h.contains_key(&entry.exec));
                        results = results.push(super::views::launcher::result_row(
                            entry,
                            is_selected,
                            is_favorite,
                            is_recent,
                            i,
                            palette,
                            lang,
                        ));
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
                        super::preview::preview_pane(entry, palette, lang)
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
                        results = results.push(super::views::file_browser::file_row(entry, is_selected, i, palette, lang));
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
            Mode::CommandRunner => super::views::command_runner::command_runner_view(state, palette, lang),
            Mode::Settings => super::views::settings::settings_view(state, palette, lang),
            Mode::Help => super::views::help::help_view(palette, lang),
            Mode::WebSearch => super::views::web_search::web_search_view(state, palette, lang),
            Mode::Calculator => super::views::calculator::calculator_view(state, palette, lang),
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
