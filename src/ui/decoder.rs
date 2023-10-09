use ratatui::{
  backend::Backend,
  layout::{Constraint, Rect},
  style::Style,
  text::{Line, Span, Text},
  widgets::{Block, Borders, Paragraph, Wrap},
  Frame,
};
use serde_json::to_string_pretty;
use tui_input::Input;

use super::utils::{
  horizontal_chunks, layout_block, layout_block_default, layout_block_line, style_default,
  style_secondary, title_with_dual_style, vertical_chunks, vertical_chunks_with_margin,
};
use crate::app::{App, InputMode, TextInput};

pub fn draw_decoder<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let chunks = horizontal_chunks(
    vec![Constraint::Percentage(50), Constraint::Percentage(50)],
    area,
  );
  draw_encoded_block(f, app, chunks[0]);
  draw_decoded_block(f, app, chunks[1]);
}

fn draw_encoded_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let title_hint = get_edit_hint(&app.data.decoder.encoded.input_mode);

  let block = layout_block_line(title_with_dual_style(
    " Encoded Token ".into(),
    title_hint.into(),
    app.light_theme,
  ));

  f.render_widget(block, area);

  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);

  render_input_widget(f, chunks[0], &app.data.decoder.encoded, app.light_theme);
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

  if app.data.decoder.decoded.is_some() {
    let header = app.data.decoder.decoded.as_ref().unwrap().header.clone();

    let paragraph = Paragraph::new(to_string_pretty(&header).unwrap()).block(Block::default());
    f.render_widget(paragraph, chunks[0]);
  }
}

fn draw_payload_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);

  let block = layout_block_default(" Payload: Claims ");

  f.render_widget(block, area);

  if app.data.decoder.decoded.is_some() {
    let payload = app.data.decoder.decoded.as_ref().unwrap().claims.clone();
    let paragraph = Paragraph::new(to_string_pretty(&payload).unwrap()).block(Block::default());
    f.render_widget(paragraph, chunks[0]);
  }
}

fn draw_signature_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let title_hint = get_edit_hint(&app.data.decoder.secret.input_mode);

  let block = layout_block_line(title_with_dual_style(
    " Verify Signature ".into(),
    title_hint.into(),
    app.light_theme,
  ));

  f.render_widget(block, area);

  let chunks =
    vertical_chunks_with_margin(vec![Constraint::Length(1), Constraint::Min(2)], area, 1);

  let mut text = Text::from(
    "Prepend 'b64:' for base64 encoded secret. Prepend '@' for file path (.pem, .pk8, .der)",
  );

  text.patch_style(style_default(app.light_theme));

  let paragraph = Paragraph::new(text).block(Block::default());

  f.render_widget(paragraph, chunks[0]);

  render_input_widget(f, chunks[1], &app.data.decoder.secret, app.light_theme);
}

fn render_input_widget<B: Backend>(
  f: &mut Frame<'_, B>,
  chunk: Rect,
  text_input: &TextInput,
  light_theme: bool,
) {
  let width = chunk.width.max(3) - 3;
  // keep 2 for borders and 1 for cursor
  let scroll = text_input.input.visual_scroll(width as usize);
  let input = Paragraph::new(text_input.input.value())
    .wrap(Wrap { trim: false })
    .style(get_input_style(&text_input.input_mode, light_theme))
    .scroll((0, scroll as u16))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .style(get_input_style(&text_input.input_mode, light_theme)),
    );

  f.render_widget(input, chunk);

  match text_input.input_mode {
    InputMode::Normal => {
      // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
    }

    InputMode::Editing => {
      // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
      f.set_cursor(
        // Put cursor past the end of the input text
        chunk.x + ((text_input.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
        // Move one line down, from the border to the input line
        chunk.y + 1,
      )
    }
  }
}

// Utility methods
fn get_edit_hint(input_mode: &InputMode) -> &str {
  match input_mode {
    InputMode::Normal => "(Press <e> to edit) ",
    InputMode::Editing => "(Press <esc> to stop) ",
  }
}

fn get_input_style(input_mode: &InputMode, light: bool) -> Style {
  match input_mode {
    InputMode::Normal => style_default(light),
    InputMode::Editing => style_secondary(light),
  }
}

#[cfg(test)]
mod tests {}
