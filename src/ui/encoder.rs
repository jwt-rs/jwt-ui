use ratatui::{
  layout::{Constraint, Rect},
  text::Text,
  widgets::{Block, Borders, Paragraph, Wrap},
  Frame,
};

use super::utils::{
  get_input_style, get_selectable_block, horizontal_chunks, render_input_widget, style_default,
  style_primary, vertical_chunks, vertical_chunks_with_margin,
};
use crate::app::{ActiveBlock, App, Route, RouteId, TextAreaInput};

pub fn draw_encoder(f: &mut Frame<'_>, app: &mut App, area: Rect) {
  let chunks = horizontal_chunks(
    vec![Constraint::Percentage(50), Constraint::Percentage(50)],
    area,
  );
  draw_left_side(f, app, chunks[0]);
  draw_right_side(f, app, chunks[1]);
}

fn draw_left_side(f: &mut Frame<'_>, app: &mut App, area: Rect) {
  let chunks = vertical_chunks(
    vec![Constraint::Percentage(40), Constraint::Percentage(60)],
    area,
  );

  draw_header_block(f, app, chunks[0]);
  draw_payload_block(f, app, chunks[1]);
}

fn draw_right_side(f: &mut Frame<'_>, app: &mut App, area: Rect) {
  let chunks = vertical_chunks(
    vec![Constraint::Percentage(30), Constraint::Percentage(70)],
    area,
  );

  draw_secret_block(f, app, chunks[0]);
  draw_token_block(f, app, chunks[1]);
}

fn draw_header_block(f: &mut Frame<'_>, app: &mut App, area: Rect) {
  app.update_block_map(get_route(ActiveBlock::EncoderHeader), area);

  let block = get_selectable_block(
    "Header: Algorithm & Token Type",
    *app.data.encoder.blocks.get_active_block() == ActiveBlock::EncoderHeader,
    Some(&app.data.encoder.header.input_mode),
    app.light_theme,
  );

  f.render_widget(block, area);

  render_text_area_widget(f, area, &mut app.data.encoder.header, app.light_theme);
}

fn draw_payload_block(f: &mut Frame<'_>, app: &mut App, area: Rect) {
  app.update_block_map(get_route(ActiveBlock::EncoderPayload), area);

  let block = get_selectable_block(
    "Payload: Claims",
    *app.data.encoder.blocks.get_active_block() == ActiveBlock::EncoderPayload,
    Some(&app.data.encoder.payload.input_mode),
    app.light_theme,
  );
  f.render_widget(block, area);

  render_text_area_widget(f, area, &mut app.data.encoder.payload, app.light_theme);
}

fn draw_secret_block(f: &mut Frame<'_>, app: &mut App, area: Rect) {
  app.update_block_map(get_route(ActiveBlock::EncoderSecret), area);

  let block = get_selectable_block(
    "Signing Secret",
    *app.data.encoder.blocks.get_active_block() == ActiveBlock::EncoderSecret,
    Some(&app.data.encoder.secret.input_mode),
    app.light_theme,
  );

  f.render_widget(block, area);

  let chunks =
    vertical_chunks_with_margin(vec![Constraint::Length(1), Constraint::Min(2)], area, 1);

  let mut text = Text::from(
    "Prepend 'b64:' for base64 encoded secret. Prepend '@' for file path (.pem, .pk8, .der, .json)",
  );
  text = text.patch_style(style_default(app.light_theme));
  let paragraph = Paragraph::new(text).block(Block::default());

  f.render_widget(paragraph, chunks[0]);

  render_input_widget(f, chunks[1], &app.data.encoder.secret, app.light_theme);
}

fn draw_token_block(f: &mut Frame<'_>, app: &mut App, area: Rect) {
  app.update_block_map(get_route(ActiveBlock::EncoderToken), area);

  let block = get_selectable_block(
    "Encoded Token",
    *app.data.encoder.blocks.get_active_block() == ActiveBlock::EncoderToken,
    None,
    app.light_theme,
  );

  f.render_widget(block, area);

  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);

  let encoded = app.data.encoder.encoded.get_txt();
  let mut txt = Text::from(encoded.clone());
  txt = txt.patch_style(style_primary(app.light_theme));

  let paragraph = Paragraph::new(txt)
    .block(Block::default())
    .wrap(Wrap { trim: false })
    .scroll((app.data.encoder.encoded.offset, 0));
  f.render_widget(paragraph, chunks[0]);
}

// Utility methods
fn render_text_area_widget(
  f: &mut Frame<'_>,
  area: Rect,
  text_input: &mut TextAreaInput<'_>,
  light_theme: bool,
) {
  let chunks = vertical_chunks_with_margin(vec![Constraint::Min(2)], area, 1);
  let mut textarea = text_input.input.clone();
  textarea.set_block(
    Block::default()
      .borders(Borders::ALL)
      .style(get_input_style(&text_input.input_mode, light_theme)),
  );

  f.render_widget(&textarea, chunks[0]);
}

fn get_route(active_block: ActiveBlock) -> Route {
  Route {
    id: RouteId::Encoder,
    active_block,
  }
}

#[cfg(test)]
mod tests {
  use ratatui::{
    backend::TestBackend,
    layout::Position,
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
    let mut app = App::new(None, "secret".into());

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
        draw_encoder(f, &mut app, f.area());
      })
      .unwrap();

    let mut expected = Buffer::with_lines(vec![
      r#"┌ Header: Algorithm & Token Type (<enter> edit | ┐┌ Signing Secret ────────────────────────────────┐"#,
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
            expected
              .cell_mut(Position::new(col, row))
              .unwrap()
              .set_style(
                Style::default()
                  .fg(COLOR_WHITE)
                  .add_modifier(Modifier::REVERSED),
              );
          }
          (1..=32, 0) => {
            expected
              .cell_mut(Position::new(col, row))
              .unwrap()
              .set_style(
                Style::default()
                  .fg(COLOR_YELLOW)
                  .add_modifier(Modifier::BOLD),
              );
          }
          (51..=66, 0) | (51..=65, 6) | (1..=17, 8) => {
            expected
              .cell_mut(Position::new(col, row))
              .unwrap()
              .set_style(
                Style::default()
                  .fg(COLOR_WHITE)
                  .add_modifier(Modifier::BOLD),
              );
          }
          (0 | 16..=49, 0) | (0 | 49, 1..=6 | 20..=99) | (0..=49, 7) => {
            expected
              .cell_mut(Position::new(col, row))
              .unwrap()
              .set_style(Style::default().fg(COLOR_YELLOW));
          }

          (51, 9) | (51..=98, 7..=9) | (51..=78, 10) => {
            expected
              .cell_mut(Position::new(col, row))
              .unwrap()
              .set_style(Style::default().fg(COLOR_CYAN));
          }
          _ => {
            expected
              .cell_mut(Position::new(col, row))
              .unwrap()
              .set_style(Style::default().fg(COLOR_WHITE));
          }
        }
      }
    }

    terminal.backend().assert_buffer(&expected);
  }
}
