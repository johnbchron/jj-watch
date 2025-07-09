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
