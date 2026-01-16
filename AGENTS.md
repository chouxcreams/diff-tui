# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

diff-tui is a terminal-based Git diff viewer written in Rust using the Ratatui TUI framework.

## Build Commands

```bash
# Build for development
cargo build

# Build for release
cargo build --release

# Run
cargo run

# Run tests
cargo test

# Check code without building
cargo check
```

## Architecture

The application follows a simple state machine pattern with two screens:

- **FileList**: Displays changed files with status indicators and fuzzy search
- **DiffView**: Shows the diff output for a selected file

### Key Files

- `src/main.rs` - Entry point, terminal initialization
- `src/app.rs` - Application state, UI rendering, event handling
- `src/git/repository.rs` - Git operations using git2 crate
- `src/git/diff.rs` - Diff generation (delta with git diff fallback)
- `src/fuzzy.rs` - Fuzzy matching using nucleo

### Dependencies

- **ratatui** + **crossterm** - TUI framework and terminal backend
- **git2** - Native Git bindings
- **nucleo** - Fast fuzzy matching (same engine as Helix editor)
- **ansi-to-tui** - Parse ANSI escape codes into styled Ratatui text
- **which** - Check if delta is installed

## Code Patterns

### State Management

The `App` struct in `app.rs` holds all application state. Screen transitions are handled by changing `self.screen` and the corresponding state (e.g., `diff_content`, `diff_scroll`).

### Event Handling

Events are processed in `handle_events()` with screen-specific handlers (`handle_file_list_keys`, `handle_diff_view_keys`).

### Diff Pipeline

1. Check if delta is available via `which::which("delta")`
2. If available, pipe `git diff` output through delta
3. Otherwise, use `git diff --color=always` directly
4. Parse ANSI output with `ansi-to-tui` for colored display
