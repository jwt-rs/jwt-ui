use ratatui::{backend::Backend, layout::Rect, widgets::TableState, Frame};

use super::{ActiveBlock, App, Route};

pub trait AppResource {
  fn render<B: Backend>(block: ActiveBlock, f: &mut Frame<'_, B>, app: &mut App, area: Rect);
}

pub trait Scrollable {
  fn handle_scroll(&mut self, up: bool, page: bool) {
    // support page up/down
    let inc_or_dec = if page { 10 } else { 1 };
    if up {
      self.scroll_up(inc_or_dec);
    } else {
      self.scroll_down(inc_or_dec);
    }
  }
  fn scroll_down(&mut self, inc_or_dec: usize);
  fn scroll_up(&mut self, inc_or_dec: usize);
}

#[derive(Clone, Debug)]
pub struct StatefulTable<T> {
  pub state: TableState,
  pub items: Vec<T>,
}

impl<T> StatefulTable<T> {
  pub fn new() -> StatefulTable<T> {
    StatefulTable {
      state: TableState::default(),
      items: Vec::new(),
    }
  }

  pub fn with_items(items: Vec<T>) -> StatefulTable<T> {
    let mut table = StatefulTable::new();
    if !items.is_empty() {
      table.state.select(Some(0));
    }
    table.set_items(items);
    table
  }

  pub fn set_items(&mut self, items: Vec<T>) {
    let item_len = items.len();
    self.items = items;
    if !self.items.is_empty() {
      let i = self.state.selected().map_or(0, |i| {
        if i > 0 && i < item_len {
          i
        } else if i >= item_len {
          item_len - 1
        } else {
          0
        }
      });
      self.state.select(Some(i));
    }
  }
}

impl<T> Scrollable for StatefulTable<T> {
  fn scroll_down(&mut self, increment: usize) {
    if let Some(i) = self.state.selected() {
      if (i + increment) < self.items.len() {
        self.state.select(Some(i + increment));
      } else {
        self.state.select(Some(self.items.len().saturating_sub(1)));
      }
    }
  }

  fn scroll_up(&mut self, decrement: usize) {
    if let Some(i) = self.state.selected() {
      if i != 0 {
        self.state.select(Some(i.saturating_sub(decrement)));
      }
    }
  }
}

#[derive(Clone)]
pub struct TabRoute {
  pub title: String,
  pub route: Route,
}
#[derive(Default)]
pub struct TabsState {
  pub items: Vec<TabRoute>,
  pub index: usize,
}

impl TabsState {
  pub fn new(items: Vec<TabRoute>) -> TabsState {
    TabsState { items, index: 0 }
  }
  pub fn set_index(&mut self, index: usize) -> &TabRoute {
    self.index = index;
    &self.items[self.index]
  }
  pub fn get_active_route(&self) -> &Route {
    &self.items[self.index].route
  }

  pub fn next(&mut self) {
    self.index = (self.index + 1) % self.items.len();
  }
  pub fn previous(&mut self) {
    if self.index > 0 {
      self.index -= 1;
    } else {
      self.index = self.items.len() - 1;
    }
  }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct ScrollableTxt {
  items: Vec<String>,
  pub offset: u16,
}

impl ScrollableTxt {
  pub fn new(item: String) -> ScrollableTxt {
    let items: Vec<&str> = item.split('\n').collect();
    let items: Vec<String> = items.iter().map(|it| it.to_string()).collect();
    ScrollableTxt { items, offset: 0 }
  }

  pub fn get_txt(&self) -> String {
    self.items.join("\n")
  }
}

impl Scrollable for ScrollableTxt {
  fn scroll_down(&mut self, increment: usize) {
    // scroll only if offset is less than total lines in text
    // we subtract increment + 2 to keep the text in view. Its just an arbitrary number that works
    if self.offset < self.items.len().saturating_sub(increment + 2) as u16 {
      self.offset += increment as u16;
    }
  }
  fn scroll_up(&mut self, decrement: usize) {
    // scroll up and avoid going negative
    if self.offset > 0 {
      self.offset = self.offset.saturating_sub(decrement as u16);
    }
  }
}

#[cfg(test)]
mod tests {

  use crate::app::RouteId;

  use super::*;

  #[test]
  fn test_stateful_table() {
    let mut sft: StatefulTable<String> = StatefulTable::new();

    assert_eq!(sft.items.len(), 0);
    assert_eq!(sft.state.selected(), None);
    // check default selection on set
    sft.set_items(vec![String::from("test"), String::from("testw")]);
    assert_eq!(sft.items.len(), 2);
    assert_eq!(sft.state.selected(), Some(0));
    // check selection retain on set
    sft.state.select(Some(1));
    sft.set_items(vec![
      String::from("test"),
      String::from("test2"),
      String::from("test3"),
    ]);
    assert_eq!(sft.items.len(), 3);
    assert_eq!(sft.state.selected(), Some(1));
    // check selection overflow prevention
    sft.state.select(Some(2));
    sft.set_items(vec![String::from("test"), String::from("test")]);
    assert_eq!(sft.items.len(), 2);
    assert_eq!(sft.state.selected(), Some(1));
    // check scroll down
    sft.state.select(Some(0));
    assert_eq!(sft.state.selected(), Some(0));
    sft.scroll_down(1);
    assert_eq!(sft.state.selected(), Some(1));
    // check scroll overflow
    sft.scroll_down(1);
    assert_eq!(sft.state.selected(), Some(1));
    sft.scroll_up(1);
    assert_eq!(sft.state.selected(), Some(0));
    // check scroll overflow
    sft.scroll_up(1);
    assert_eq!(sft.state.selected(), Some(0));
    // check increment
    sft.scroll_down(10);
    assert_eq!(sft.state.selected(), Some(1));

    let sft2 = StatefulTable::with_items(vec![String::from("test"), String::from("test")]);
    assert_eq!(sft2.state.selected(), Some(0));
  }

  #[test]
  fn test_handle_table_scroll() {
    let mut item: StatefulTable<&str> = StatefulTable::new();
    item.set_items(vec!["A", "B", "C"]);

    assert_eq!(item.state.selected(), Some(0));

    item.handle_scroll(false, false);
    assert_eq!(item.state.selected(), Some(1));

    item.handle_scroll(false, false);
    assert_eq!(item.state.selected(), Some(2));

    item.handle_scroll(false, false);
    assert_eq!(item.state.selected(), Some(2));
    // previous
    item.handle_scroll(true, false);
    assert_eq!(item.state.selected(), Some(1));
    // page down
    item.handle_scroll(false, true);
    assert_eq!(item.state.selected(), Some(2));
    // page up
    item.handle_scroll(true, true);
    assert_eq!(item.state.selected(), Some(0));
  }

  #[test]
  fn test_stateful_tab() {
    let mut tab = TabsState::new(vec![
      TabRoute {
        title: "Hello".into(),
        route: Route {
          active_block: ActiveBlock::Help,
          id: RouteId::Help,
        },
      },
      TabRoute {
        title: "Test".into(),
        route: Route {
          active_block: ActiveBlock::DecoderToken,
          id: RouteId::Decoder,
        },
      },
    ]);

    assert_eq!(tab.index, 0);
    assert_eq!(tab.get_active_route().active_block, ActiveBlock::Help);
    tab.next();
    assert_eq!(tab.index, 1);
    assert_eq!(
      tab.get_active_route().active_block,
      ActiveBlock::DecoderToken
    );
    tab.next();
    assert_eq!(tab.index, 0);
    assert_eq!(tab.get_active_route().active_block, ActiveBlock::Help);
    tab.previous();
    assert_eq!(tab.index, 1);
    assert_eq!(
      tab.get_active_route().active_block,
      ActiveBlock::DecoderToken
    );
    tab.previous();
    assert_eq!(tab.index, 0);
    assert_eq!(tab.get_active_route().active_block, ActiveBlock::Help);
  }

  #[test]
  fn test_scrollable_txt() {
    let mut stxt = ScrollableTxt::new("test\n multiline\n string".into());

    assert_eq!(stxt.offset, 0);
    assert_eq!(stxt.items.len(), 3);

    assert_eq!(stxt.get_txt(), "test\n multiline\n string");

    stxt.scroll_down(1);
    assert_eq!(stxt.offset, 0);

    let mut stxt2 = ScrollableTxt::new("te\nst\nmul\ntil\ni\nne\nstr\ni\nn\ng".into());
    assert_eq!(stxt2.items.len(), 10);
    stxt2.scroll_down(1);
    assert_eq!(stxt2.offset, 1);
    stxt2.scroll_down(1);
    assert_eq!(stxt2.offset, 2);
    stxt2.scroll_down(5);
    assert_eq!(stxt2.offset, 7);
    stxt2.scroll_down(1);
    // no overflow past (len - 2)
    assert_eq!(stxt2.offset, 7);
    stxt2.scroll_up(1);
    assert_eq!(stxt2.offset, 6);
    stxt2.scroll_up(6);
    assert_eq!(stxt2.offset, 0);
    stxt2.scroll_up(1);
    // no overflow past (0)
    assert_eq!(stxt2.offset, 0);
  }
}
