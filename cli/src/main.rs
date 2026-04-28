mod commands;
mod generator;
mod templates;
mod tui;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    commands::run()
}
