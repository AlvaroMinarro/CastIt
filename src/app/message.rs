use iced_layershell::to_layer_message;

#[to_layer_message]
#[derive(Debug, Clone)]
pub enum Message {
    QueryChanged(String),
    Submit,
    SubmitInTerminal,
    CommandFinished { command: String, result: Result<String, String> },
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    SelectEntry(usize),
    WindowFocused,
    ClearQuery,
    TogglePreview,
    ScrollPreviewUp,
    ScrollPreviewDown,
}
