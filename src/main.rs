mod app;
mod config;
mod fuzzy;
mod git;

use anyhow::Result;
use clap::{ArgAction, Parser};
use std::panic;

/// A terminal-based Git diff viewer with fuzzy search
#[derive(Parser)]
#[command(version, about)]
#[command(help_template = "\
{name} {version}
{about}

{usage-heading} {usage}

{all-args}

KEYBINDINGS:
    File List:
        j/Down    Move to next file
        k/Up      Move to previous file
        Enter     View diff of selected file
        e         Open file in editor
        /         Start search mode
        q         Quit

    Diff View:
        j/Down    Scroll down
        k/Up      Scroll up
        d/PgDn    Scroll down 20 lines
        u/PgUp    Scroll up 20 lines
        g/Home    Go to top
        G/End     Go to bottom
        e         Open file in editor
        Esc       Return to file list
        q         Quit
")]
struct Cli {
    /// Also show help (alias for -h)
    #[arg(short = 'H', long = "Help", hide = true, action = ArgAction::Help)]
    help_alias: (),
}

fn main() -> Result<()> {
    let _cli = Cli::parse();
    // パニックハンドラーを設定して、パニック時にターミナルを復元する
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();
        original_hook(panic_info);
    }));

    let app = app::App::new()?;

    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();

    result
}
