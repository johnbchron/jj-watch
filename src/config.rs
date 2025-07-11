use std::{
  ops::Deref,
  sync::{Arc, RwLock},
  time::Duration,
};

use miette::Result;
use tokio::process::Command;

use crate::args::Args;

#[derive(Clone, Debug)]
pub struct Config {
  inner: Arc<RwLock<ConfigInner>>,
}

#[derive(Debug)]
struct ConfigInner {
  log_command_period: Duration,
  no_snapshot:        bool,
}

impl Config {
  pub fn from_args_and_env(args: &Args) -> Result<Self> {
    Ok(Config {
      inner: Arc::new(RwLock::new(ConfigInner {
        log_command_period: Duration::from_secs(2),
        no_snapshot:        args.no_snapshot,
      })),
    })
  }

  fn read(&self) -> impl Deref<Target = ConfigInner> {
    self
      .inner
      .read()
      .expect("failed to get read lock on config")
  }

  pub fn command(&self) -> Command {
    let mut command = Command::new("jj");

    let config = self.read();

    let mut args = vec!["log"];
    if config.no_snapshot {
      args.push("--ignore-working-copy");
    }
    args.extend(["--color", "always"]);
    args.push("--no-pager");
    args.push("--quiet");

    command.args(args);

    command
  }

  pub fn log_command_duration(&self) -> Duration {
    self.read().log_command_period
  }
}
