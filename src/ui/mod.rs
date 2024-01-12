mod decoder;
mod encoder;
mod help;
pub mod utils;

use ratatui::{
  layout::{Alignment, Constraint, Rect},
  text::{Line, Span, Text},
  widgets::{Block, Borders, Paragraph, Tabs, Wrap},
  Frame,
};

use self::{
  decoder::draw_decoder,
  encoder::draw_encoder,
  help::draw_help,
  utils::{
    horizontal_chunks_with_margin, layout_block, style_default, style_failure, style_help,
    style_logo, style_main_background, style_primary, style_secondary, title_style_logo,
    vertical_chunks,
  },
};
use crate::app::{App, RouteId};

pub static HIGHLIGHT: &str = "=> ";

pub fn draw(f: &mut Frame<'_>, app: &mut App) {
  let block = Block::default().style(style_main_background(app.light_theme));
  f.render_widget(block, f.size());

  let chunks = if !app.data.error.is_empty() {
    vertical_chunks(
      vec![
        Constraint::Length(3), // header
        Constraint::Length(3), // error
        Constraint::Min(1),    // main area
        Constraint::Length(3), // footer
      ],
      f.size(),
    )
  } else {
    vertical_chunks(
      vec![
        Constraint::Length(3), // header
        Constraint::Min(1),    // main area
        Constraint::Length(3), // footer
      ],
      f.size(),
    )
  };

  draw_app_header(f, app, chunks[0]);

  if !app.data.error.is_empty() {
    draw_app_error(f, app, chunks[1]);
  }

  let main_chunk = chunks[chunks.len() - 2];

  match app.get_current_route().id {
    RouteId::Help => {
      draw_help(f, app, main_chunk);
    }
    RouteId::Decoder => {
      draw_decoder(f, app, main_chunk);
    }
    RouteId::Encoder => {
      draw_encoder(f, app, main_chunk);
    }
  }

  draw_app_footer(f, app, chunks[chunks.len() - 1]);
}

fn draw_app_header(f: &mut Frame<'_>, app: &App, area: Rect) {
  let chunks =
    horizontal_chunks_with_margin(vec![Constraint::Length(50), Constraint::Min(0)], area, 1);

  let titles = app
    .main_tabs
    .items
    .iter()
    .map(|t| Line::from(Span::styled(&t.title, style_default(app.light_theme))))
    .collect();
  let tabs = Tabs::new(titles)
    .block(layout_block(title_style_logo(app.title, app.light_theme)))
    .highlight_style(style_secondary(app.light_theme))
    .select(app.main_tabs.index);

  f.render_widget(tabs, area);
  draw_header_text(f, app, chunks[1]);
}

fn draw_app_footer(f: &mut Frame<'_>, app: &App, area: Rect) {
  // Footer text with correct styling
  let text = format!(
    "JWT UI v{} with ♥ in Rust | Crafted by Auth0 by Okta",
    env!("CARGO_PKG_VERSION"),
  );
  let mut text = Text::from(text);
  text.patch_style(style_logo(app.light_theme));

  let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::ALL));
  f.render_widget(paragraph, area);
}

fn draw_header_text(f: &mut Frame<'_>, app: &App, area: Rect) {
  let text: Vec<Line<'_>> = match app.get_current_route().id {
    RouteId::Decoder => vec![Line::from(
      "<?> help | <tab> switch tabs | <←→> select block | <u> toggle UTC dates | <↑↓> scroll ",
    )],
    RouteId::Encoder => vec![Line::from(
      "<?> help | <tab> switch tabs | <←→> select block | <↑↓> scroll ",
    )],
    RouteId::Help => vec![],
  };
  let paragraph = Paragraph::new(text)
    .style(style_help(app.light_theme))
    .block(Block::default())
    .alignment(Alignment::Right);
  f.render_widget(paragraph, area);
}

fn draw_app_error(f: &mut Frame<'_>, app: &App, size: Rect) {
  let block = Block::default()
    .title(" Error ")
    .style(style_failure(app.light_theme))
    .borders(Borders::ALL);

  let mut text = Text::from(app.data.error.clone());
  text.patch_style(style_failure(app.light_theme));

  let paragraph = Paragraph::new(text)
    .style(style_primary(app.light_theme))
    .block(block)
    .wrap(Wrap { trim: true });
  f.render_widget(paragraph, size);
}
