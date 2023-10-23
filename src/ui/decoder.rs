use ratatui::{
  backend::Backend,
  layout::{Constraint, Rect},
  text::Text,
  widgets::{Block, Paragraph, Wrap},
  Frame,
};

use super::utils::{
  get_selectable_block, horizontal_chunks, render_input_widget, style_default, style_primary,
  vertical_chunks, vertical_chunks_with_margin,
};
use crate::app::{ActiveBlock, App};

pub fn draw_decoder<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let chunks = horizontal_chunks(
    vec![Constraint::Percentage(50), Constraint::Percentage(50)],
    area,
  );
  draw_encoded_block(f, app, chunks[0]);
  draw_decoded_block(f, app, chunks[1]);
}

fn draw_encoded_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let chunks = vertical_chunks(
    vec![Constraint::Percentage(70), Constraint::Percentage(30)],
    area,
  );

  draw_token_block(f, app, chunks[0]);
  draw_secret_block(f, app, chunks[1]);
}

fn draw_decoded_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let chunks = vertical_chunks(
    vec![Constraint::Percentage(40), Constraint::Percentage(60)],
    area,
  );

  draw_header_block(f, app, chunks[0]);
  draw_payload_block(f, app, chunks[1]);
}

fn draw_token_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let block = get_selectable_block(
    "Encoded Token",
    app.data.decoder.blocks.get_active_route(),
    ActiveBlock::DecoderToken,
    Some(&app.data.decoder.encoded.input_mode),
    app.light_theme,
  );

  f.render_widget(block, area);

  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);
  render_input_widget(f, chunks[0], &app.data.decoder.encoded, app.light_theme);
}

fn draw_secret_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let block = get_selectable_block(
    "Verify Signature",
    app.data.decoder.blocks.get_active_route(),
    ActiveBlock::DecoderSecret,
    Some(&app.data.decoder.secret.input_mode),
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

  render_input_widget(f, chunks[1], &app.data.decoder.secret, app.light_theme);
}

fn draw_header_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let block = get_selectable_block(
    "Header: Algorithm & Token Type",
    app.data.decoder.blocks.get_active_route(),
    ActiveBlock::DecoderHeader,
    None,
    app.light_theme,
  );

  f.render_widget(block, area);

  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);

  let header = app.data.decoder.header.get_txt();
  let mut txt = Text::from(header.clone());
  txt.patch_style(style_primary(app.light_theme));

  let paragraph = Paragraph::new(txt)
    .block(Block::default())
    .wrap(Wrap { trim: false })
    .scroll((app.data.decoder.header.offset, 0));
  f.render_widget(paragraph, chunks[0]);
}

fn draw_payload_block<B: Backend>(f: &mut Frame<'_, B>, app: &App, area: Rect) {
  let block = get_selectable_block(
    "Payload: Claims",
    app.data.decoder.blocks.get_active_route(),
    ActiveBlock::DecoderPayload,
    None,
    app.light_theme,
  );
  f.render_widget(block, area);

  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);

  let payload = app.data.decoder.payload.get_txt();
  let mut txt = Text::from(payload.clone());
  txt.patch_style(style_primary(app.light_theme));

  let paragraph = Paragraph::new(txt)
    .block(Block::default())
    .wrap(Wrap { trim: false })
    .scroll((app.data.decoder.payload.offset, 0));
  f.render_widget(paragraph, chunks[0]);
}

#[cfg(test)]
mod tests {
  use crate::ui::utils::{COLOR_CYAN, COLOR_WHITE, COLOR_YELLOW};

  use super::*;
  use ratatui::{
    backend::TestBackend,
    prelude::Buffer,
    style::{Modifier, Style},
    Terminal,
  };

  #[test]
  fn test_draw_decoder() {
    let mut app = App::new(
        250,
        Some("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.XbPfbIHMI6arZ3Y922BhjWgQzWXcXNrz0ogtVhfEd2o".into()), 
        "secret".into()
    );

    app.on_tick();

    let backend = TestBackend::new(100, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
      .draw(|f| {
        draw_decoder(f, &app, f.size());
      })
      .unwrap();

    let mut expected = Buffer::with_lines(vec![
      r#"┌ Encoded Token (<e> edit | <c> copy) ───────────┐┌ Header: Algorithm & Token Type ────────────────┐"#,
      r#"│┌──────────────────────────────────────────────┐││{                                               │"#,
      r#"││eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiO│││  "typ": "JWT",                                 │"#,
      r#"││iIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF│││  "alg": "HS256"                                │"#,
      r#"││0IjoxNTE2MjM5MDIyfQ.XbPfbIHMI6arZ3Y922BhjWgQzW│││}                                               │"#,
      r#"││XcXNrz0ogtVhfEd2o                             │││                                                │"#,
      r#"││                                              │││                                                │"#,
      r#"││                                              ││└────────────────────────────────────────────────┘"#,
      r#"││                                              ││┌ Payload: Claims ───────────────────────────────┐"#,
      r#"││                                              │││{                                               │"#,
      r#"││                                              │││  "iat": 1516239022,                            │"#,
      r#"││                                              │││  "name": "John Doe",                           │"#,
      r#"│└──────────────────────────────────────────────┘││  "sub": "1234567890"                           │"#,
      r#"└────────────────────────────────────────────────┘│}                                               │"#,
      r#"┌ Verify Signature ──────────────────────────────┐│                                                │"#,
      r#"│Prepend 'b64:' for base64 encoded secret. Prepen││                                                │"#,
      r#"│┌──────────────────────────────────────────────┐││                                                │"#,
      r#"││secret                                        │││                                                │"#,
      r#"│└──────────────────────────────────────────────┘││                                                │"#,
      r#"└────────────────────────────────────────────────┘└────────────────────────────────────────────────┘"#,
    ]);

    // set expected row styles
    for row in 0..=19 {
      for col in 0..=99 {
        match (col, row) {
          (1..=15, 0) => {
            expected.get_mut(col, row).set_style(
              Style::default()
                .fg(COLOR_YELLOW)
                .add_modifier(Modifier::BOLD),
            );
          }
          (51..=82, 0) | (51..=67, 8) | (1..=18, 14) => {
            expected.get_mut(col, row).set_style(
              Style::default()
                .fg(COLOR_WHITE)
                .add_modifier(Modifier::BOLD),
            );
          }
          (0 | 16..=49, 0) | (0..=49, 13) | (0 | 49, 1..=13 | 20..=99) => {
            expected
              .get_mut(col, row)
              .set_style(Style::default().fg(COLOR_YELLOW));
          }

          (51, 1 | 4 | 9 | 11 | 13)
          | (51..=65, 2)
          | (51..=66, 3)
          | (51..=70, 10 | 12)
          | (52..=71, 11 | 12) => {
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
