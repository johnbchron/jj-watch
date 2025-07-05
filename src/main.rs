mod app;
mod args;
mod config;

use clap::Parser;
use miette::{Context, Result};

use self::{app::App, args::Args, config::Config};

#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();

  let config = Config::from_args_and_env(&args)
    .context("failed to build configuration")?;

  // let output = config
  //   .command()
  //   .output()
  //   .into_diagnostic()
  //   .context("failed to execute `jj log`")?;

  // println!("got output: {len} bytes", len = output.stdout.len());

  // let output = output
  //   .stdout
  //   .utf8_chunks()
  //   .map(|c| c.valid())
  //   .filter(|c| !c.is_empty())
  //   .collect::<Vec<_>>()
  //   .join("");

  // println!("{output}");

  let app = App::new(config);

  let terminal = ratatui::init();
  let app_result = app.run(terminal).await;
  ratatui::restore();
  app_result
}
