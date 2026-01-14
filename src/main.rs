mod app;
mod fuzzy;
mod git;

use anyhow::Result;

fn main() -> Result<()> {
    let app = app::App::new()?;

    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();

    result
}
