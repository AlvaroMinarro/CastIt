/// Represents a launchable application parsed from a `.desktop` file.
#[derive(Debug, Clone)]
pub struct AppEntry {
    /// Display name (e.g. "Firefox")
    pub name: String,
    /// Exec command with field codes stripped (e.g. "firefox")
    pub exec: String,
    /// Icon name or path (e.g. "firefox")
    pub icon: Option<String>,
    /// Resolved absolute path to the icon file (png, svg)
    pub icon_path: Option<String>,
    /// Short description (Comment field)
    pub description: Option<String>,
}
