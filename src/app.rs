mod log_widget;

use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyCode};
use miette::{Context, IntoDiagnostic, Result};
use ratatui::{
  DefaultTerminal,
  layout::{Constraint, Layout},
  prelude::*,
};
use tokio_stream::StreamExt;

use self::{log_widget::JjLogWidget, status_line_widget::StatusLineWidget};
use crate::config::Config;

pub struct App {
  should_quit: bool,
  log_widget:  JjLogWidget,
  config:      Config,
}

impl App {
  const FRAMES_PER_SECOND: f32 = 60.0;

  pub fn new(config: Config) -> Self {
    App {
      should_quit: false,
      log_widget: Default::default(),
      config,
    }
  }

  pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
    self.log_widget.run(self.config.clone());

    let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
    let mut interval = tokio::time::interval(period);
    let mut events = EventStream::new();

    while !self.should_quit {
      tokio::select! {
          _ = interval.tick() => {
            terminal
              .draw(|frame| self.render(frame))
              .into_diagnostic()
              .context("failed to draw frame")?;
          },
          Some(Ok(event)) = events.next() => self.handle_event(&event),
      }
    }
    Ok(())
  }

  fn render(&self, frame: &mut Frame) {
    let page_splits = Layout::vertical([
      Constraint::Fill(1),
      Constraint::Length(1),
      Constraint::Length(1),
    ])
    .split(frame.area());
    frame.render_widget(&self.log_widget, page_splits[0]);

    frame.render_widget(StatusLineWidget, page_splits[2]);
  }

  fn handle_event(&mut self, event: &Event) {
    if let Some(key) = event.as_key_press_event() {
      match key.code {
        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
        _ => {}
      }
    }
  }
}

mod status_line_widget {
  use ratatui::{
    layout::{Constraint, Layout},
    prelude::*,
    widgets::{Block, Padding},
  };

  pub struct StatusLineWidget;

  impl Widget for StatusLineWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
      Self: Sized,
    {
      let block = Block::new()
        .bg(Color::Black)
        .padding(Padding::horizontal(1));
      (&block).render(area, buf);

      let quit_message = Span::from("`q` or `<esc>` to quit");
      let title = Line::from("jj-watch").bold().centered().fg(Color::Red);

      let status_line_splits = Layout::horizontal([
        Constraint::Length(quit_message.content.len() as _),
        Constraint::Fill(1),
        Constraint::Length(quit_message.content.len() as _),
      ])
      .split(block.inner(area));

      quit_message.render(status_line_splits[0], buf);
      title.render(status_line_splits[1], buf);
    }
  }
}
