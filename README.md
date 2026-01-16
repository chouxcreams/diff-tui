# diff-tui

A terminal-based Git diff viewer. Provides an intuitive interface for browsing changed files and viewing diffs.

## Features

- **File Selection**: Browse changed files in your Git repository
- **Fuzzy Search**: Press `/` to filter files by name
- **Colored Diff**: Uses delta or git diff for syntax-highlighted diffs
- **Status Indicators**: M(modified), A(added), D(deleted), R(renamed), ?(untracked)

## Requirements

- Rust 1.70+
- Git

### Optional

- [delta](https://github.com/dandavison/delta) - Enhanced diff output (falls back to git diff if not installed)

## Installation

```bash
cargo install --path .
```

Or for development:

```bash
cargo build --release
./target/release/diff-tui
```

## Configuration

Create `~/.config/diff-tui/config.toml` to customize settings:

```toml
[diff]
# Diff tool to use (default: "auto")
# Options: "auto", "delta", "diff-so-fancy", "difftastic", "colordiff", "git"
# Or specify any custom command name
tool = "delta"

# Additional arguments to pass to the diff tool (optional)
args = ["--side-by-side"]
```

### Diff Tool Behavior

| Value | Behavior |
|-------|----------|
| `"auto"` | Try delta first, fall back to git diff (default) |
| `"delta"` | Use delta (falls back to git diff if not installed) |
| `"git"` | Use git diff directly |
| Other | Use specified command (falls back to git diff if not found) |

## Usage

Run inside a Git repository:

```bash
diff-tui
```

### Key Bindings

#### File Selection

| Key | Action |
|-----|--------|
| `j` / `↓` | Move to next file |
| `k` / `↑` | Move to previous file |
| `Enter` | View diff of selected file |
| `/` | Start search mode |
| `q` | Quit |

#### Search Mode

| Key | Action |
|-----|--------|
| Type | Add to search query |
| `Backspace` | Delete character |
| `Enter` | View diff of selected file |
| `Esc` | Cancel search |
| `↑` / `↓` | Navigate search results |

#### Diff View

| Key | Action |
|-----|--------|
| `j` / `↓` | Scroll down 1 line |
| `k` / `↑` | Scroll up 1 line |
| `d` / `PageDown` | Scroll down 20 lines |
| `u` / `PageUp` | Scroll up 20 lines |
| `g` / `Home` | Go to top |
| `G` / `End` | Go to bottom |
| `q` / `Esc` | Return to file selection |

## Tech Stack

- [Ratatui](https://ratatui.rs/) - TUI framework
- [git2](https://crates.io/crates/git2) - Git operations
- [nucleo](https://crates.io/crates/nucleo) - Fuzzy matching
- [ansi-to-tui](https://crates.io/crates/ansi-to-tui) - ANSI escape sequence parsing

## License

MIT
