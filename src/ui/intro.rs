use ratatui::{
  backend::Backend,
  layout::{Constraint, Rect},
  Frame,
};

use super::utils::horizontal_chunks;
use crate::app::App;

pub fn draw_intro<B: Backend>(_f: &mut Frame<'_, B>, _app: &mut App, area: Rect) {
  let _chunks = horizontal_chunks(
    vec![Constraint::Percentage(50), Constraint::Percentage(50)],
    area,
  );
}

#[cfg(test)]
mod tests {}
