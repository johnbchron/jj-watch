use std::{
  process::ExitStatus,
  sync::{Arc, RwLock},
};

use ansi_to_tui::IntoText;
use miette::{Context, IntoDiagnostic};
use ratatui::{
  layout::Rect,
  prelude::*,
  widgets::{Block, Padding},
};
use tokio::time::interval;

use crate::config::Config;

#[derive(Clone)]
pub struct JjLogWidget {
  state:         Arc<RwLock<JjLogState>>,
  reverse_lines: bool,
  padding:       Padding,
}

impl Default for JjLogWidget {
  fn default() -> Self {
    JjLogWidget {
      state:         Default::default(),
      reverse_lines: true,
      padding:       Padding::uniform(1),
    }
  }
}

impl JjLogWidget {
  pub(super) fn run(&self, config: Config) {
    let this = self.clone();
    tokio::spawn(this.update_runner(config));
  }

  async fn update_runner(self, config: Config) {
    let mut interval = interval(config.log_command_duration());
    loop {
      interval.tick().await;
      self.clone().update(config.clone()).await;
    }
  }

  async fn update(self, config: Config) {
    let output = config
      .command()
      .output()
      .await
      .into_diagnostic()
      .context("failed to run `jj log` command")
      .unwrap();

    let stdout = text_from_ansi_bytes(&output.stdout);
    let stderr = text_from_ansi_bytes(&output.stderr);

    let result = match output.status.success() {
      true => JjLogState::Success { stdout },
      false => JjLogState::Failure {
        stdout,
        stderr,
        exit_status: output.status,
      },
    };

    {
      let mut lock = self
        .state
        .write()
        .expect("failed to lock `JjLogWidget` state");
      *lock = result;
    }
  }
}

impl Widget for &JjLogWidget {
  fn render(self, mut area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    let content = match &(*self
      .state
      .read()
      .expect("failed to lock `JjLogWidget` state"))
    {
      JjLogState::Success { stdout } => stdout.clone(),
      JjLogState::Failure { stderr, .. } => stderr.clone(),
      JjLogState::Unpopulated => Text::from(Span::raw("No data yet")),
    };

    let block = Block::new().padding(self.padding);
    area = block.inner(area);

    if self.reverse_lines {
      let line_count: u16 = content
        .lines
        .len()
        .try_into()
        .expect("failed to cast line count to u16");
      let line_count = line_count.min(area.height);

      area.y = area.y + area.height - line_count;
      area.height = line_count;
    }

    content.render(area, buf);
  }
}

#[derive(Default)]
enum JjLogState {
  Success {
    stdout: Text<'static>,
  },
  Failure {
    stdout:      Text<'static>,
    stderr:      Text<'static>,
    exit_status: ExitStatus,
  },
  #[default]
  Unpopulated,
}

fn text_from_ansi_bytes(data: &[u8]) -> Text<'static> {
  let string = data
    .utf8_chunks()
    .map(|c| c.valid())
    .filter(|c| !c.is_empty())
    .collect::<Vec<_>>()
    .join("");

  let text = string
    .into_text()
    .into_diagnostic()
    .context("failed to parse ANSI from `jj log` stdout")
    .unwrap();

  text
}
