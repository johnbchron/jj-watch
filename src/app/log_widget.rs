use std::sync::{Arc, RwLock};

use ansi_to_tui::IntoText;
use miette::{Context, IntoDiagnostic, Result};
use ratatui::{layout::Rect, prelude::*};
use tokio::time::{Duration, interval};

use crate::config::Config;

#[derive(Clone)]
pub struct JjLogWidget {
  state:        Arc<RwLock<JjLogState>>,
  bottom_align: bool,
}

impl Default for JjLogWidget {
  fn default() -> Self {
    JjLogWidget {
      state:        Default::default(),
      bottom_align: true,
    }
  }
}

impl JjLogWidget {
  pub(super) fn run(&self, config: Config) {
    let this = self.clone();
    tokio::spawn(this.update_runner(config));
  }

  async fn update_runner(self, config: Config) {
    let mut interval = interval(Duration::from_secs(2));
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

    let output_string = output
      .stdout
      .utf8_chunks()
      .map(|c| c.valid())
      .filter(|c| !c.is_empty())
      .collect::<Vec<_>>()
      .join("");

    let output_text = output_string
      .into_text()
      .into_diagnostic()
      .context("failed to parse ANSI from `jj log` stdout")
      .unwrap();

    let result = match output.status.success() {
      true => Ok(output_text),
      false => Err(output_text),
    };

    {
      let mut lock = self
        .state
        .write()
        .expect("failed to lock `JjLogWidget` state");
      *lock = JjLogState(Some(result));
    }
  }
}

impl Widget for &JjLogWidget {
  fn render(self, area: Rect, buf: &mut Buffer)
  where
    Self: Sized,
  {
    let content = match self
      .state
      .read()
      .expect("failed to lock `JjLogWidget` state")
      .0
    {
      Some(Ok(ref data)) => data.clone(),
      Some(Err(ref data)) => data.clone(),
      None => Text::from(Span::raw("No data yet")),
    };

    if self.bottom_align {
      let line_count: u16 = content
        .lines
        .len()
        .try_into()
        .expect("failed to cast line count to u16");
      let ba_area = Rect::new(
        area.x,
        area.y + area.height - line_count,
        area.width,
        line_count,
      );

      content.render(ba_area, buf);
    } else {
      content.render(area, buf);
    }
  }
}

#[derive(Default)]
struct JjLogState(Option<Result<Text<'static>, Text<'static>>>);
