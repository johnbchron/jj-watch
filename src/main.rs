mod app;
mod args;
mod config;

use std::io::stdout;

use clap::Parser;
use crossterm::{
  event::{DisableMouseCapture, EnableMouseCapture},
  execute,
};
use miette::{Context, IntoDiagnostic, Result};

use self::{app::App, args::Args, config::Config};

#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();

  let config = Config::from_args_and_env(&args)
    .context("failed to build configuration")?;

  let app = App::new(config);

  let terminal = ratatui::init();
  execute!(stdout(), EnableMouseCapture)
    .into_diagnostic()
    .context("failed to enable mouse capture")?;
  let app_result = app.run(terminal).await;
  let _ = execute!(stdout(), DisableMouseCapture);
  ratatui::restore();
  app_result
}
