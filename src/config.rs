use miette::Result;
use tokio::process::Command;

use crate::args::Args;

#[derive(Clone, Debug)]
pub struct Config {}

impl Config {
  pub fn from_args_and_env(args: &Args) -> Result<Self> { Ok(Config {}) }

  pub fn command(&self) -> Command {
    let mut command = Command::new("jj");
    command.args([
      "log",
      "--ignore-working-copy",
      "--color",
      "always",
      "--no-pager",
      "--quiet",
    ]);

    command
  }
}
