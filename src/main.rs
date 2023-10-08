#![warn(rust_2018_idioms)]
mod app;
mod banner;
mod event;
mod handlers;
mod ui;

use std::{
  io::{self, stdout, Stdout},
  panic::{self, PanicInfo},
  sync::Arc,
};

use anyhow::Result;
use app::{jwt::print_decoded_token, App};
use banner::BANNER;
use clap::Parser;
use crossterm::{
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::Key;
use ratatui::{
  backend::{Backend, CrosstermBackend},
  Terminal,
};
use tokio::sync::Mutex;

use crate::app::jwt::decode_jwt_token;

/// kdash CLI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, override_usage = "Press `?` while running the app to see keybindings", before_help = BANNER)]
pub struct Cli {
  /// whether the CLI should run in TUI mode or just print to stdout
  #[arg(short, long, value_parser)]
  pub token: Option<String>,
  /// whether the CLI should run in TUI mode or just print to stdout
  #[arg(short, long, value_parser, default_value_t = false)]
  pub stdout: bool,
  /// whether stdout should be formatted as JSON
  #[arg(short, long, value_parser, default_value_t = false)]
  pub json: bool,
  /// Set the tick rate (milliseconds): the lower the number the higher the FPS. Must be less than 1000.
  #[arg(short = 'r', long, value_parser, default_value_t = 250)]
  pub tick_rate: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
  openssl_probe::init_ssl_cert_env_vars();
  panic::set_hook(Box::new(|info| {
    panic_hook(info);
  }));

  // parse CLI arguments
  let cli = Cli::parse();

  if cli.tick_rate >= 1000 {
    panic!("Tick rate must be below 1000");
  }

  // Initialize app state
  let app = Arc::new(Mutex::new(App::new(cli.tick_rate, cli.token.clone())));

  if cli.stdout && cli.token.is_some() {
    // print decoded result to terminal
    let mut app = app.lock().await;
    decode_jwt_token(&mut app, cli.token.unwrap());
    if app.error.is_empty() && app.data.decoded_token.is_some() {
      print_decoded_token(app.data.decoded_token.as_ref().unwrap(), cli.json);
    } else {
      println!("{}", app.error);
    }
  } else {
    // Launch the UI asynchronously
    // The UI must run in the "main" thread
    start_ui(cli, &app).await?;
  }

  Ok(())
}

async fn start_ui(cli: Cli, app: &Arc<Mutex<App>>) -> Result<()> {
  // see https://docs.rs/crossterm/0.17.7/crossterm/terminal/#raw-mode
  enable_raw_mode()?;
  // Terminal initialization
  let mut stdout = stdout();
  // not capturing mouse to make text select/copy possible
  execute!(stdout, EnterAlternateScreen)?;
  // terminal backend for cross platform support
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.clear()?;
  terminal.hide_cursor()?;
  // custom events
  let events = event::Events::new(cli.tick_rate);
  let mut is_first_render = true;
  // main UI loop
  loop {
    let mut app = app.lock().await;
    // Get the size of the screen on each loop to account for resize event
    if let Ok(size) = terminal.backend().size() {
      // Reset the size if the terminal was resized
      if app.refresh || app.size != size {
        app.size = size;
      }
    };

    // draw the UI layout
    terminal.draw(|f| ui::draw(f, &mut app))?;

    // handle key events
    match events.next()? {
      event::Event::Input(key_event) => {
        // quit on CTRL + C
        let key = Key::from(key_event);

        if key == Key::Ctrl('c') {
          break;
        }
        // handle all other keys
        handlers::handle_key_events(key, key_event, &mut app).await
      }
      // handle mouse events
      event::Event::MouseInput(mouse) => handlers::handle_mouse_events(mouse, &mut app).await,
      // handle tick events
      event::Event::Tick => {
        app.on_tick(is_first_render).await;
      }
    }

    is_first_render = false;

    if app.should_quit {
      break;
    }
  }

  terminal.show_cursor()?;
  shutdown(terminal)?;

  Ok(())
}

// shutdown the CLI and show terminal
fn shutdown(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
  terminal.show_cursor()?;
  Ok(())
}

#[cfg(debug_assertions)]
fn panic_hook(info: &PanicInfo<'_>) {
  use backtrace::Backtrace;
  use crossterm::style::Print;

  let location = info.location().unwrap();

  let msg = match info.payload().downcast_ref::<&'static str>() {
    Some(s) => *s,
    None => match info.payload().downcast_ref::<String>() {
      Some(s) => &s[..],
      None => "Box<Any>",
    },
  };

  let stacktrace: String = format!("{:?}", Backtrace::new()).replace('\n', "\n\r");

  disable_raw_mode().unwrap();
  execute!(
    io::stdout(),
    LeaveAlternateScreen,
    Print(format!(
      "thread '<unnamed>' panicked at '{}', {}\n\r{}",
      msg, location, stacktrace
    )),
  )
  .unwrap();
}

#[cfg(not(debug_assertions))]
fn panic_hook(info: &PanicInfo<'_>) {
  use human_panic::{handle_dump, print_msg, Metadata};

  let meta = Metadata {
    version: env!("CARGO_PKG_VERSION").into(),
    name: env!("CARGO_PKG_NAME").into(),
    authors: env!("CARGO_PKG_AUTHORS").replace(":", ", ").into(),
    homepage: env!("CARGO_PKG_HOMEPAGE").into(),
  };
  let file_path = handle_dump(&meta, info);
  disable_raw_mode().unwrap();
  execute!(io::stdout(), LeaveAlternateScreen).unwrap();
  print_msg(file_path, &meta).expect("human-panic: printing error message to console failed");
}
