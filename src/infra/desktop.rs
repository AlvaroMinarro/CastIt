use crate::domain::models::AppEntry;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Scans all XDG data directories for `.desktop` files and returns
/// a deduplicated list of launchable applications.
pub fn scan_desktop_entries() -> Vec<AppEntry> {
    let dirs = get_application_dirs();
    let mut entries: HashMap<String, AppEntry> = HashMap::new();

    for dir in dirs {
        if let Ok(read_dir) = fs::read_dir(&dir) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("desktop") {
                    if let Some(app) = parse_desktop_file(&path) {
                        // Deduplicate by name — user-local overrides system
                        entries.entry(app.name.clone()).or_insert(app);
                    }
                }
            }
        }
    }

    let mut result: Vec<AppEntry> = entries.into_values().collect();
    result.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    result
}

/// Returns the list of directories containing `.desktop` files,
/// ordered by priority (user-local first).
fn get_application_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // User-local applications (highest priority)
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(format!("{}/.local/share/applications", home)));
    }

    // XDG_DATA_DIRS (system directories)
    let data_dirs = std::env::var("XDG_DATA_DIRS")
        .unwrap_or_else(|_| String::from("/usr/local/share:/usr/share"));

    for dir in data_dirs.split(':') {
        dirs.push(PathBuf::from(format!("{}/applications", dir)));
    }

    dirs
}

/// Parses a single `.desktop` file into an `AppEntry`.
/// Returns `None` if the entry should be hidden or isn't an application.
fn parse_desktop_file(path: &PathBuf) -> Option<AppEntry> {
    let content = fs::read_to_string(path).ok()?;

    let mut entry_type = None;
    let mut name = None;
    let mut exec = None;
    let mut icon = None;
    let mut comment = None;
    let mut no_display = false;
    let mut hidden = false;
    let mut in_desktop_entry = false;

    for line in content.lines() {
        let line = line.trim();

        if line == "[Desktop Entry]" {
            in_desktop_entry = true;
            continue;
        }

        // Stop parsing if we hit another section
        if line.starts_with('[') && in_desktop_entry {
            break;
        }

        if !in_desktop_entry {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Type" => entry_type = Some(value.to_string()),
                // Only take non-localized Name (skip Name[es], Name[de], etc.)
                "Name" => name = Some(value.to_string()),
                "Exec" => exec = Some(strip_field_codes(value)),
                "Icon" => icon = Some(value.to_string()),
                "Comment" => comment = Some(value.to_string()),
                "NoDisplay" => no_display = value.eq_ignore_ascii_case("true"),
                "Hidden" => hidden = value.eq_ignore_ascii_case("true"),
                _ => {}
            }
        }
    }

    // Only include visible Application entries with a name and exec
    if entry_type.as_deref() != Some("Application") || no_display || hidden {
        return None;
    }

    Some(AppEntry {
        name: name?,
        exec: exec?,
        icon,
        icon_path: None,
        description: comment,
    })
}

/// Strips XDG field codes (%u, %U, %f, %F, %i, %c, %k, etc.) from an Exec value.
fn strip_field_codes(exec: &str) -> String {
    let mut result = String::with_capacity(exec.len());
    let mut chars = exec.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '%' {
            // Skip the field code character
            chars.next();
        } else {
            result.push(ch);
        }
    }

    result.trim().to_string()
}
