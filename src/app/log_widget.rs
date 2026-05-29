use std::{
  process::ExitStatus,
  sync::{
    Arc, RwLock,
    atomic::{AtomicU16, Ordering},
  },
};

use ansi_to_tui::IntoText;
use miette::{Context, IntoDiagnostic};
use ratatui::{
  layout::Rect,
  prelude::*,
  widgets::{
    Block, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
  },
};
use tokio::time::interval;

use crate::config::Config;

#[derive(Clone)]
pub struct JjLogWidget {
  state:         Arc<RwLock<JjLogState>>,
  reverse_lines: bool,
  padding:       Padding,
  scroll_offset: Arc<AtomicU16>,
}

impl Default for JjLogWidget {
  fn default() -> Self {
    JjLogWidget {
      state:         Default::default(),
      reverse_lines: true,
      padding:       Padding::uniform(1),
      scroll_offset: Arc::new(AtomicU16::new(0)),
    }
  }
}

impl JjLogWidget {
  pub(super) fn scroll_up(&self, lines: u16) {
    let current = self.scroll_offset.load(Ordering::Relaxed);
    self
      .scroll_offset
      .store(current.saturating_sub(lines), Ordering::Relaxed);
  }

  pub(super) fn scroll_down(&self, lines: u16) {
    self
      .scroll_offset
      .fetch_add(lines, Ordering::Relaxed);
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
  fn render(self, area: Rect, buf: &mut Buffer)
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
    let inner = block.inner(area);

    let line_count: u16 = content
      .lines
      .len()
      .try_into()
      .unwrap_or(u16::MAX);

    let scrollbar_area = inner;
    let mut content_area = inner;
    content_area.width = content_area.width.saturating_sub(1);

    let max_scroll = line_count.saturating_sub(content_area.height);
    let scroll_offset = self
      .scroll_offset
      .load(Ordering::Relaxed)
      .min(max_scroll);
    self
      .scroll_offset
      .store(scroll_offset, Ordering::Relaxed);

    if self.reverse_lines && line_count < content_area.height {
      let shift = content_area.height - line_count;
      content_area.y += shift;
      content_area.height = line_count;
    }

    Paragraph::new(content)
      .scroll((scroll_offset, 0))
      .render(content_area, buf);

    if max_scroll > 0 {
      let mut scrollbar_state =
        ScrollbarState::new((max_scroll + 1) as usize)
          .position(scroll_offset as usize)
          .viewport_content_length(content_area.height as usize);
      Scrollbar::new(ScrollbarOrientation::VerticalRight).render(
        scrollbar_area,
        buf,
        &mut scrollbar_state,
      );
    }
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
