pub mod message;
pub mod state;
pub mod update;
pub mod view;
pub mod preview;
pub mod subscription;
pub mod views;

use iced_layershell::application;
use iced_layershell::reexport::{Anchor, KeyboardInteractivity, Layer};
use iced_layershell::settings::{LayerShellSettings, Settings, StartMode};

use crate::config;
use state::CastIt;

fn namespace() -> String {
    String::from("castit")
}

pub fn run() -> Result<(), iced_layershell::Error> {
    let config = config::Config::load();
    let width = config.width.unwrap_or(800) as u32;
    let height = config.height.unwrap_or(500) as u32;

    application(CastIt::new, namespace, update::update, view::view)
        .subscription(subscription::subscription)
        .theme(|state: &CastIt| {
            Some(view::resolve_iced_theme(state.config.theme.as_deref().unwrap_or("TokyoNight")))
        })
        .style(|_state, _theme| iced::theme::Style {
            background_color: iced::Color::TRANSPARENT,
            text_color: iced::Color::WHITE,
        })
        .settings(Settings {
            layer_settings: LayerShellSettings {
                anchor: Anchor::empty(),
                layer: Layer::Overlay,
                keyboard_interactivity: KeyboardInteractivity::Exclusive,
                exclusive_zone: -1,
                size: Some((width, height)),
                start_mode: StartMode::Active,
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
}
