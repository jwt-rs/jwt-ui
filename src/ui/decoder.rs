use ratatui::{
  backend::Backend,
  layout::{Constraint, Rect},
  style::Style,
  text::{Line, Span, Text},
  widgets::{Block, Borders, Paragraph, Wrap},
  Frame,
};
use serde_json::to_string_pretty;

use super::utils::{
  horizontal_chunks, layout_block_default, style_default, style_secondary, vertical_chunks,
  vertical_chunks_with_margin,
};
use crate::app::{App, InputMode};

pub fn draw_decoder<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let chunks = horizontal_chunks(
    vec![Constraint::Percentage(50), Constraint::Percentage(50)],
    area,
  );
  draw_encoded_block(f, app, chunks[0]);
  draw_decoded_block(f, app, chunks[1]);
}

fn draw_encoded_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let block = layout_block_default(" Encoded Token ");

  f.render_widget(block, area);

  let mut text = Text::from(vec![Line::from(vec![Span::styled(
    match app.data.token_input.input_mode {
      InputMode::Normal => "Press <e> to start editing",
      InputMode::Editing => "Press <esc> to stop editing",
    },
    style_default(app.light_theme),
  )])]);

  text.patch_style(style_default(app.light_theme));

  let paragraph = Paragraph::new(text).block(Block::default());

  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2), Constraint::Min(2)], area, 1);
  f.render_widget(paragraph, chunks[0]);

  let width = chunks[1].width.max(3) - 3; // keep 2 for borders and 1 for cursor
  let scroll = app.data.token_input.input.visual_scroll(width as usize);
  let input = Paragraph::new(app.data.token_input.input.value())
    .wrap(Wrap { trim: false })
    .style(get_input_style(app))
    .scroll((0, scroll as u16))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(get_input_style(app)),
    );

  f.render_widget(input, chunks[1]);

  match app.data.token_input.input_mode {
    InputMode::Normal => {
      // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
    }

    InputMode::Editing => {
      // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
      f.set_cursor(
        // Put cursor past the end of the input text
        chunks[1].x
          + ((app.data.token_input.input.visual_cursor()).max(scroll) - scroll) as u16
          + 1,
        // Move one line down, from the border to the input line
        chunks[1].y + 1,
      )
    }
  }
}

fn draw_decoded_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let chunks = vertical_chunks(
    vec![
      Constraint::Percentage(20),
      Constraint::Percentage(40),
      Constraint::Percentage(40),
    ],
    area,
  );

  draw_header_block(f, app, chunks[0]);
  draw_payload_block(f, app, chunks[1]);
  draw_signature_block(f, app, chunks[2]);
}

fn draw_header_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);

  let block = layout_block_default(" Header: Algorithm & Token Type ");

  f.render_widget(block, area);

  if app.data.decoded_token.is_some() {
    let header = app.data.decoded_token.as_ref().unwrap().header.clone();

    let paragraph = Paragraph::new(to_string_pretty(&header).unwrap()).block(Block::default());
    f.render_widget(paragraph, chunks[0]);
  }
}

fn draw_payload_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);

  let block = layout_block_default(" Payload: Claims ");

  f.render_widget(block, area);

  if app.data.decoded_token.is_some() {
    let payload = app.data.decoded_token.as_ref().unwrap().claims.clone();
    let paragraph = Paragraph::new(to_string_pretty(&payload).unwrap()).block(Block::default());
    f.render_widget(paragraph, chunks[0]);
  }
}

fn draw_signature_block<B: Backend>(f: &mut Frame<'_, B>, _app: &App, area: Rect) {
  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);

  let block = layout_block_default(" Verify Signature ");

  f.render_widget(block, area);

  let paragraph = Paragraph::new("text").block(Block::default());
  f.render_widget(paragraph, chunks[0]);
}

// Utility methods

fn get_input_style(app: &App) -> Style {
  match app.data.token_input.input_mode {
    InputMode::Normal => style_default(app.light_theme),
    InputMode::Editing => style_secondary(app.light_theme),
  }
}

#[cfg(test)]
mod tests {}
