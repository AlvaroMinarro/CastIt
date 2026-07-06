# CastIt

**CastIt** is a developer-focused, premium command palette and application launcher for Wayland-based Linux desktops. Built with Rust and powered by the Iced GUI library, it provides a fast, keyboard-driven interface with clean pill-based aesthetics inspired by Raycast.

---

## Features

- 🚀 **Application Launcher**: Fuzzy search desktop application names and descriptions instantly using the state-of-the-art `nucleo-matcher` fuzzy engine.
- 📋 **Quick Launch Dashboard**: When the search bar is empty, CastIt becomes a dashboard showing your bookmarked **Favorite Apps** (highlighted in gold) followed by your **top 5 most used apps** (highlighted in blue).
- 🐚 **Terminal Command Runner**: Type `>` followed by a command to execute it. Run commands in the background with real-time output preview, or run them in your preferred terminal emulator with `Ctrl+Enter` (spawns in detached process groups).
- 🔍 **Global File Search**: Type `f ` followed by a search query (e.g. `f resume`) to recursively search files and documents in your home directory (`~/`) using `fd` (or standard `find` fallback). Automatically displays filetypes with custom MIME icons.
- 📁 **File & Directory Browser**: Type `/` or `~` to explore your system directory tree. Press `Arrow Right` to navigate into folders, `Shift+Arrow Left` to backtrack, and `Enter` to open files or directories in your default file manager.
- 👁️ **macOS Quick Look Document Preview**: Press `Ctrl+Space` on any selected file to show a rich preview pane:
  - **Images** (`.png`, `.jpg`, `.webp`, `.svg`, etc.) are rendered directly inside the interface.
  - **Code & Text files** (supporting 30+ dev extensions: `.rs`, `.java`, `.kt`, `.py`, `.js`, etc.) render up to 300 lines of monospace code. Use `Shift+Arrow Up/Down` to scroll the file's preview context.
  - **Other files** display system-level file descriptors and metadata cards.
- 🌐 **Web Search Integration**: Type `?` followed by a search query (e.g. `? rust docs`) to execute search queries directly in your preferred web browser.
- 🧮 **Visual Calculator**: Type mathematical expressions (e.g. `10 * 9 - 8`) to automatically calculate results and preview them in real-time. Press `Enter` to copy results to the system clipboard.
- ⚙️ **Real-Time Settings Panel**: Type `..` to toggle settings. Customize theme, preferred terminal emulator, preferred web browser, window opacity, dimensions, and language (English / Spanish) reactively with auto-scan PATH discovery.
- 📋 **Shortcuts Cheatsheet**: Type `??` to view a balanced, two-column interactive cheatsheet of all keyboard shortcuts.

---

## Quick Start

### Build and Run

1. Clone the repository:
   ```bash
   git clone git@github.com:AlvaroMinarro/CastIt.git
   cd CastIt
   ```
2. Build and run using Cargo:
   ```bash
   cargo run --release
   ```

### System Dependencies

Ensure you have Wayland libraries installed on your distribution:
- **Ubuntu/Debian**: `sudo apt install libwayland-dev`
- **Arch Linux**: `sudo pacman -S wayland`
- **Fedora**: `sudo dnf install wayland-devel`

---

## Keyboard Shortcuts

### Global
| Shortcut | Action |
|:---|:---|
| `Esc` | Exit CastIt |
| `Shift + Delete` / `Backspace` | Clear search bar input |
| `..` | Open Settings |
| `??` | View Keyboard Shortcuts cheatsheet |

### Launcher / Application Mode
| Shortcut | Action |
|:---|:---|
| `↑ / ↓` | Navigate through search results (with auto-scroll) |
| `Enter` | Launch selected application |
| `Ctrl + D` | Toggle selected application as favorite (Dashboard) |

### Command Runner Mode (`>`)
| Shortcut | Action |
|:---|:---|
| `Enter` | Run command in background and preview output (`stdout`/`stderr`) |
| `Ctrl + Enter` | Run command in external preferred terminal emulator |

### File Browser / Global File Search Mode (`/`, `~`, `f `)
| Shortcut | Action |
|:---|:---|
| `f <query>` | Search files recursively in home directory (`~/`) |
| `↑ / ↓` | Navigate file list |
| `→` | Autocomplete / Navigate into selected folder (only in path browsing) |
| `Shift + ←` | Go back to parent folder (only in path browsing) |
| `Enter` | Open file (or native folder manager for directory) and close launcher |
| `Ctrl + Space` | Open/Close file preview (Quick Look) |

### Web Search & Calculator Mode (`?`, math expressions)
| Shortcut | Action |
|:---|:---|
| `Enter` (Web Search) | Open search query in preferred browser |
| `Enter` (Calculator) | Copy calculation result to system clipboard and close |

### Quick Look File Preview (While Open)
| Shortcut | Action |
|:---|:---|
| `Shift + ↑ / ↓` | Scroll through file text/code preview |
| `↑ / ↓` | Change preview to previous/next file in the background list |
| `Ctrl + Space` | Close file preview |

### Settings Mode (`..`)
| Shortcut | Action |
|:---|:---|
| `↑ / ↓` | Navigate settings items |
| `← / →` | Change selected value (Theme, Preferred Terminal, Opacity, Width, Height, Language, Preferred Browser) |

---

## Configuration

Settings are saved automatically upon change to `~/.config/castit/config.toml`. You can also configure it manually:

```toml
theme = "TokyoNight"
# preferred terminal (None for Auto, or specify "kitty", "alacritty", etc.)
terminal = "kitty"
opacity = 0.92
width = 800
height = 500
language = "ES" # "EN" or "ES"
```

## License

This project is licensed under the [MIT License](LICENSE).
