use std::env;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn get_available_terminals() -> Vec<String> {
    let mut available = vec!["Auto".to_string()];
    let terminals = [
        "ghostty",
        "kitty",
        "alacritty",
        "wezterm",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "xterm",
    ];

    if let Ok(path_var) = env::var("PATH") {
        for term in &terminals {
            for p in path_var.split(':') {
                let bin_path = Path::new(p).join(term);
                if bin_path.exists() {
                    available.push(term.to_string());
                    break;
                }
            }
        }
    }
    available
}

pub fn get_available_browsers() -> Vec<String> {
    let mut available = vec!["Auto".to_string()];
    let browsers = [
        "zen-browser",
        "zen",
        "firefox",
        "google-chrome",
        "brave-browser",
        "brave",
        "chromium",
        "epiphany",
        "opera",
        "vivaldi",
        "librewolf",
        "waterfox",
    ];

    if let Ok(path_var) = env::var("PATH") {
        for browser in &browsers {
            for p in path_var.split(':') {
                let bin_path = Path::new(p).join(browser);
                if bin_path.exists() {
                    available.push(browser.to_string());
                    break;
                }
            }
        }
    }
    available
}

fn find_terminal() -> Option<String> {
    get_available_terminals()
        .into_iter()
        .find(|t| t != "Auto")
}

/// Runs a command in the background, capturing stdout and stderr combined.
pub fn run_in_background(command_str: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command_str)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to spawn process: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        if stdout.is_empty() && !stderr.is_empty() {
            Ok(stderr)
        } else {
            Ok(stdout)
        }
    } else {
        let err_msg = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("Process exited with status: {}", output.status)
        };
        Err(err_msg)
    }
}

/// Spawns a terminal emulator to run an interactive command.
pub fn run_in_terminal(command_str: &str, preferred_terminal: Option<&str>) -> Result<(), String> {
    let term = if let Some(pref) = preferred_terminal {
        pref.to_string()
    } else {
        find_terminal().ok_or_else(|| {
            "No supported terminal emulator (kitty, alacritty, wezterm, gnome-terminal, konsole, xfce4-terminal, xterm) found in PATH.".to_string()
        })?
    };

    // We chain a read command to keep the terminal window open after completion
    let shell_cmd = format!(
        "{}; echo; echo '--- Process finished. Press Enter to close ---'; read -r _",
        command_str
    );

    let mut cmd = Command::new(&term);
    cmd.process_group(0);
    match term.as_str() {
        "wezterm" => {
            cmd.args(["start", "--", "sh", "-c", &shell_cmd]);
        }
        "gnome-terminal" => {
            cmd.args(["--", "sh", "-c", &shell_cmd]);
        }
        _ => {
            // kitty, alacritty, konsole, xfce4-terminal, xterm
            cmd.args(["-e", "sh", "-c", &shell_cmd]);
        }
    }

    cmd.spawn()
        .map_err(|e| format!("Failed to spawn terminal '{}': {}", term, e))?;

    Ok(())
}

pub fn global_file_search(term: String) -> Result<Vec<crate::app::state::FileEntry>, String> {
    let fd_exists = if let Ok(path_var) = env::var("PATH") {
        path_var.split(':').any(|p| Path::new(p).join("fd").exists())
    } else {
        false
    };

    let output = if fd_exists {
        Command::new("sh")
            .arg("-c")
            .arg("fd --type f --max-results 30 \"$1\" ~")
            .arg("--")
            .arg(&term)
            .output()
            .map_err(|e| format!("Failed to execute fd: {}", e))?
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("find ~/ -maxdepth 5 -type f -iname \"*$1*\" | head -n 30")
            .arg("--")
            .arg(&term)
            .output()
            .map_err(|e| format!("Failed to execute find: {}", e))?
    };

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let mut results = Vec::new();

    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let path = Path::new(trimmed);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(trimmed)
            .to_string();

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let icon_name = match ext.as_str() {
            "pdf" => "document-pdf",
            "jpg" | "jpeg" | "png" | "gif" | "svg" | "bmp" | "webp" => "image-x-generic",
            "zip" | "tar" | "gz" | "xz" | "bz2" | "7z" | "rar" | "deb" | "rpm" => "package-x-generic",
            "mp3" | "ogg" | "wav" | "flac" | "m4a" | "wma" => "audio-x-generic",
            "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" | "webm" => "video-x-generic",
            _ => "text-x-generic",
        };

        results.push(crate::app::state::FileEntry {
            name,
            path: trimmed.to_string(),
            is_dir: false,
            icon_path: Some(icon_name.to_string()),
        });
    }

    Ok(results)
}
