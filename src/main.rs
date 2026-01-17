mod app;
mod config;
mod fuzzy;
mod git;

use anyhow::Result;
use std::panic;

fn main() -> Result<()> {
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
