use ratatui::{
  backend::Backend,
  layout::{Constraint, Rect},
  text::Text,
  widgets::{Block, Borders, Paragraph, Wrap},
  Frame,
};

use super::utils::{
  get_input_style, get_selectable_block, horizontal_chunks, render_input_widget, style_default,
  style_primary, vertical_chunks, vertical_chunks_with_margin,
};
use crate::app::{ActiveBlock, App, TextAreaInput};

pub fn draw_encoder<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  let chunks = horizontal_chunks(
    vec![Constraint::Percentage(50), Constraint::Percentage(50)],
    area,
  );
  draw_decoded_block(f, app, chunks[0]);
  draw_encoded_block(f, app, chunks[1]);
}

fn draw_decoded_block<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  let chunks = vertical_chunks(
    vec![Constraint::Percentage(40), Constraint::Percentage(60)],
    area,
  );

  draw_header_block(f, app, chunks[0]);
  draw_payload_block(f, app, chunks[1]);
}

fn draw_encoded_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let chunks = vertical_chunks(
    vec![Constraint::Percentage(30), Constraint::Percentage(70)],
    area,
  );

  draw_secret_block(f, app, chunks[0]);
  draw_token_block(f, app, chunks[1]);
}

fn draw_header_block<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  let block = get_selectable_block(
    "Header: Algorithm & Token Type",
    app.data.encoder.blocks.get_active_route(),
    ActiveBlock::EncoderHeader,
    Some(&app.data.encoder.header.input_mode),
    app.light_theme,
  );

  f.render_widget(block, area);

  render_text_area_widget(f, area, &mut app.data.encoder.header, app.light_theme);
}

fn draw_payload_block<B: Backend>(f: &mut Frame<'_, B>, app: &mut App, area: Rect) {
  let block = get_selectable_block(
    "Payload: Claims",
    app.data.encoder.blocks.get_active_route(),
    ActiveBlock::EncoderPayload,
    Some(&app.data.encoder.payload.input_mode),
    app.light_theme,
  );
  f.render_widget(block, area);

  render_text_area_widget(f, area, &mut app.data.encoder.payload, app.light_theme);
}

fn draw_secret_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let block = get_selectable_block(
    "Verify Signature",
    app.data.encoder.blocks.get_active_route(),
    ActiveBlock::EncoderSecret,
    Some(&app.data.encoder.secret.input_mode),
    app.light_theme,
  );

  f.render_widget(block, area);

  let chunks =
    vertical_chunks_with_margin(vec![Constraint::Length(1), Constraint::Min(2)], area, 1);

  let mut text = Text::from(
    "Prepend 'b64:' for base64 encoded secret. Prepend '@' for file path (.pem, .pk8, .der)",
  );
  text.patch_style(style_default(app.light_theme));
  let paragraph = Paragraph::new(text).block(Block::default());

  f.render_widget(paragraph, chunks[0]);

  render_input_widget(f, chunks[1], &app.data.encoder.secret, app.light_theme);
}

fn draw_token_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let block = get_selectable_block(
    "Encoded Token",
    app.data.encoder.blocks.get_active_route(),
    ActiveBlock::EncoderToken,
    None,
    app.light_theme,
  );

  f.render_widget(block, area);

  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);

  let encoded = app.data.encoder.encoded.get_txt();
  let mut txt = Text::from(encoded.clone());
  txt.patch_style(style_primary(app.light_theme));

  let paragraph = Paragraph::new(txt)
    .block(Block::default())
    .wrap(Wrap { trim: false })
    .scroll((app.data.encoder.encoded.offset, 0));
  f.render_widget(paragraph, chunks[0]);
}

// Utility methods
fn render_text_area_widget<B: Backend>(
  f: &mut Frame<'_, B>,
  area: Rect,
  text_input: &mut TextAreaInput<'_>,
  light_theme: bool,
) {
  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);
  let textarea = &mut text_input.input;
  textarea.set_block(
    Block::default()
      .borders(Borders::ALL)
      .style(get_input_style(&text_input.input_mode, light_theme)),
  );

  f.render_widget(textarea.widget(), chunks[0]);
}

#[cfg(test)]
mod tests {
  use ratatui::{
    backend::TestBackend,
    prelude::Buffer,
    style::{Modifier, Style},
    Terminal,
  };

  use super::*;
  use crate::{
    app::RouteId,
    ui::utils::{COLOR_CYAN, COLOR_WHITE, COLOR_YELLOW},
  };

  #[test]
  fn test_draw_encoder() {
    let mut app = App::new(250, None, "secret".into());

    app.data.encoder.payload.input = vec![
      "{",
      r#"  "sub": "1234567890","#,
      r#"  "name": "John Doe","#,
      r#"  "admin": true,"#,
      r#"  "iat": 1516239022"#,
      "}",
    ]
    .into();

    app.push_navigation_stack(RouteId::Encoder, ActiveBlock::EncoderHeader);

    app.on_tick();

    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
      .draw(|f| {
        draw_encoder(f, &mut app, f.size());
      })
      .unwrap();

    let mut expected = Buffer::with_lines(vec![
      r#"┌ Header: Algorithm & Token Type (<e> edit | <c> ┐┌ Verify Signature ──────────────────────────────┐"#,
      r#"│┌──────────────────────────────────────────────┐││Prepend 'b64:' for base64 encoded secret. Prepen│"#,
      r#"││{                                             │││┌──────────────────────────────────────────────┐│"#,
      r#"││  "alg": "HS256",                             ││││secret                                        ││"#,
      r#"││  "typ": "JWT"                                │││└──────────────────────────────────────────────┘│"#,
      r#"││}                                             ││└────────────────────────────────────────────────┘"#,
      r#"│└──────────────────────────────────────────────┘│┌ Encoded Token ─────────────────────────────────┐"#,
      r#"└────────────────────────────────────────────────┘│eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJhZG1pbiI│"#,
      r#"┌ Payload: Claims ───────────────────────────────┐│6dHJ1ZSwiaWF0IjoxNTE2MjM5MDIyLCJuYW1lIjoiSm9obiB│"#,
      r#"│┌──────────────────────────────────────────────┐││Eb2UiLCJzdWIiOiIxMjM0NTY3ODkwIn0.g7Ern-srhIi_7ZX│"#,
      r#"││{                                             │││qrl6uyey7xxWJjr-LTn4p2Nv-DOY                    │"#,
      r#"││  "sub": "1234567890",                        │││                                                │"#,
      r#"││  "name": "John Doe",                         │││                                                │"#,
      r#"││  "admin": true,                              │││                                                │"#,
      r#"││  "iat": 1516239022                           │││                                                │"#,
      r#"││}                                             │││                                                │"#,
      r#"││                                              │││                                                │"#,
      r#"││                                              │││                                                │"#,
      r#"│└──────────────────────────────────────────────┘││                                                │"#,
      r#"└────────────────────────────────────────────────┘└────────────────────────────────────────────────┘"#,
    ]);

    // set expected row styles
    for row in 0..=19 {
      for col in 0..=99 {
        match (col, row) {
          (2, 2 | 10) => {
            expected.get_mut(col, row).set_style(
              Style::default()
                .fg(COLOR_WHITE)
                .add_modifier(Modifier::REVERSED),
            );
          }
          (1..=32, 0) => {
            expected.get_mut(col, row).set_style(
              Style::default()
                .fg(COLOR_YELLOW)
                .add_modifier(Modifier::BOLD),
            );
          }
          (51..=68, 0) | (51..=65, 6) | (1..=17, 8) => {
            expected.get_mut(col, row).set_style(
              Style::default()
                .fg(COLOR_WHITE)
                .add_modifier(Modifier::BOLD),
            );
          }
          (0 | 16..=49, 0) | (0 | 49, 1..=7 | 20..=99) | (0..=49, 7) => {
            expected
              .get_mut(col, row)
              .set_style(Style::default().fg(COLOR_YELLOW));
          }

          (51, 9) | (51..=98, 7..=9) | (51..=78, 10) => {
            expected
              .get_mut(col, row)
              .set_style(Style::default().fg(COLOR_CYAN));
          }
          _ => {
            expected
              .get_mut(col, row)
              .set_style(Style::default().fg(COLOR_WHITE));
          }
        }
      }
    }

    terminal.backend().assert_buffer(&expected);
  }
}
