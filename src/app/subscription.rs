use iced::keyboard;
use iced::window;
use iced::Event;
use iced::Subscription;

use super::message::Message;
use super::state::{CastIt, Mode};

pub fn subscription(state: &CastIt) -> Subscription<Message> {
    match state.mode {
        Mode::Settings => {
            iced::event::listen_with(|event, _status, _id| match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    match key {
                        keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::Escape),
                        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(Message::ArrowDown),
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::ArrowUp),
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) => Some(Message::ArrowLeft),
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => Some(Message::ArrowRight),
                        keyboard::Key::Named(keyboard::key::Named::Delete) if modifiers.shift() => Some(Message::ClearQuery),
                        keyboard::Key::Named(keyboard::key::Named::Backspace) if modifiers.shift() => Some(Message::ClearQuery),
                        _ => None,
                    }
                }
                Event::Window(window::Event::Focused) => Some(Message::WindowFocused),
                _ => None,
            })
        }
        Mode::FileBrowser => {
            iced::event::listen_with(|event, _status, _id| match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    match key {
                        keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::Escape),
                        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => {
                            if modifiers.shift() {
                                Some(Message::ScrollPreviewDown)
                            } else {
                                Some(Message::ArrowDown)
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => {
                            if modifiers.shift() {
                                Some(Message::ScrollPreviewUp)
                            } else {
                                Some(Message::ArrowUp)
                            }
                        }
                        keyboard::Key::Named(keyboard::key::Named::ArrowLeft) if modifiers.shift() => Some(Message::ArrowLeft),
                        keyboard::Key::Named(keyboard::key::Named::ArrowRight) => Some(Message::ArrowRight),
                        keyboard::Key::Named(keyboard::key::Named::Space) if modifiers.control() => Some(Message::TogglePreview),
                        keyboard::Key::Named(keyboard::key::Named::Delete) if modifiers.shift() => Some(Message::ClearQuery),
                        keyboard::Key::Named(keyboard::key::Named::Backspace) if modifiers.shift() => Some(Message::ClearQuery),
                        _ => None,
                    }
                }
                Event::Window(window::Event::Focused) => Some(Message::WindowFocused),
                _ => None,
            })
        }
        _ => {
            iced::event::listen_with(|event, _status, _id| match event {
                Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                    match key {
                        keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::Escape),
                        keyboard::Key::Named(keyboard::key::Named::ArrowDown) => Some(Message::ArrowDown),
                        keyboard::Key::Named(keyboard::key::Named::ArrowUp) => Some(Message::ArrowUp),
                        keyboard::Key::Named(keyboard::key::Named::Delete) if modifiers.shift() => Some(Message::ClearQuery),
                        keyboard::Key::Named(keyboard::key::Named::Backspace) if modifiers.shift() => Some(Message::ClearQuery),
                        keyboard::Key::Named(keyboard::key::Named::Enter) if modifiers.control() => Some(Message::SubmitInTerminal),
                        _ => None,
                    }
                }
                Event::Window(window::Event::Focused) => Some(Message::WindowFocused),
                _ => None,
            })
        }
    }
}
