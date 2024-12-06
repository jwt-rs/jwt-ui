#![warn(rust_2018_idioms)]
mod app;
mod banner;
mod event;
mod handlers;
mod ui;

use std::{
  error::Error,
  io::{self, stdout, Stdout, Write},
  panic::{self, PanicHookInfo},
};

use app::{jwt_decoder::print_decoded_token, App};
use banner::BANNER;
use clap::Parser;
use crossterm::{
  event::DisableMouseCapture,
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::Key;
use ratatui::{
  backend::{Backend, CrosstermBackend},
  Terminal,
};

use crate::app::jwt_decoder::decode_jwt_token;

/// JWT UI
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, before_help = BANNER)]
pub struct Cli {
  /// JWT token to decode [mandatory for stdout mode, optional for TUI mode].
  #[clap(index = 1)]
  #[clap(value_parser)]
  pub token: Option<String>,
  /// Secret for validating the JWT. Can be text, file path (beginning with @) or base64 encoded string (beginning with b64:).
  #[arg(short = 'S', long, value_parser, default_value = "")]
  pub secret: String,
  /// Print to STDOUT instead of starting the CLI in TUI mode.
  #[arg(short, long, value_parser, default_value_t = false)]
  pub stdout: bool,
  /// Do not validate the signature of the JWT when printing to STDOUT.
  #[arg(short, long, value_parser, default_value_t = false)]
  pub no_verify: bool,
  /// Print to STDOUT as JSON.
  #[arg(short, long, value_parser, default_value_t = false)]
  pub json: bool,
  /// Set the tick rate (milliseconds): the lower the number the higher the FPS. Must be less than 1000.
  #[arg(short, long, value_parser, default_value_t = 250)]
  pub tick_rate: u64,
  /// Disable mouse capture in order to copy individual text.
  #[arg(short, long, value_parser, default_value_t = false)]
  pub disable_mouse_capture: bool,
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
  panic::set_hook(Box::new(|info| {
    panic_hook(info);
  }));

  // parse CLI arguments
  let cli = Cli::parse();

  if cli.tick_rate >= 1000 {
    panic!("Tick rate must be below 1000");
  }

  if (cli.stdout || cli.json) && cli.token.is_some() {
    to_stdout(cli);
  } else {
    // The UI must run in the "main" thread
    start_ui(cli)?;
  }

  Ok(())
}

fn to_stdout(cli: Cli) {
  let mut app = App::new(cli.token.clone(), cli.secret.clone());
  // print decoded result to stdout
  decode_jwt_token(&mut app, cli.no_verify);
  if app.data.error.is_empty() && app.data.decoder.is_decoded() {
    print_decoded_token(app.data.decoder.get_decoded().as_ref().unwrap(), cli.json);
  } else {
    println!("{}", app.data.error);
  }
}

/// Enable mouse capture, but don't enable capture of all the mouse movements, doing so will improve performance, and is part of the fix for the weird mouse event output bug
pub fn enable_mouse_capture() -> Result<()> {
  Ok(
    io::stdout().write_all(
      concat!(
        crossterm::csi!("?1000h"),
        crossterm::csi!("?1015h"),
        crossterm::csi!("?1006h"),
      )
      .as_bytes(),
    )?,
  )
}

fn start_ui(cli: Cli) -> Result<()> {
  // see https://docs.rs/crossterm/0.17.7/crossterm/terminal/#raw-mode
  enable_raw_mode()?;
  // Terminal initialization
  let mut stdout = stdout();
  // not capturing mouse to make text select/copy possible
  execute!(stdout, EnterAlternateScreen)?;
  if !cli.disable_mouse_capture {
    enable_mouse_capture()?;
  }
  // terminal backend for cross platform support
  let backend = CrosstermBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;
  terminal.clear()?;
  terminal.hide_cursor()?;
  // custom events
  let events = event::Events::new(cli.tick_rate);

  let mut app = App::new(cli.token.clone(), cli.secret.clone());
  // main UI loop
  loop {
    // Get the size of the screen on each loop to account for resize event
    if let Ok(size) = terminal.backend().size() {
      // Reset the size if the terminal was resized
      if app.size.as_size() != size {
        app.size.width = size.width;
        app.size.height = size.height;
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
        handlers::handle_key_events(key, key_event, &mut app);
      }
      // handle mouse events
      event::Event::MouseInput(mouse) => handlers::handle_mouse_events(mouse, &mut app),
      // handle tick events
      event::Event::Tick => {
        app.on_tick();
      }
    }
    if app.should_quit {
      break;
    }
  }

  terminal.show_cursor()?;
  shutdown(terminal)?;

  Ok(())
}

// shutdown the CLI and show terminal
fn shutdown(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
  disable_raw_mode()?;
  execute!(
    terminal.backend_mut(),
    LeaveAlternateScreen,
    DisableMouseCapture
  )?;
  terminal.show_cursor()?;
  Ok(())
}

#[cfg(debug_assertions)]
fn panic_hook(info: &PanicHookInfo<'_>) {
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
    DisableMouseCapture,
    Print(format!(
      "thread '<unnamed>' panicked at '{}', {}\n\r{}",
      msg, location, stacktrace
    )),
  )
  .unwrap();
}

#[cfg(not(debug_assertions))]
fn panic_hook(info: &PanicHookInfo<'_>) {
  use human_panic::{handle_dump, print_msg, Metadata};

  let meta = Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    .authors(env!("CARGO_PKG_AUTHORS").replace(':', ", "))
    .homepage(env!("CARGO_PKG_HOMEPAGE"));

  let file_path = handle_dump(&meta, info);
  disable_raw_mode().unwrap();
  execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
  print_msg(file_path, &meta).expect("human-panic: printing error message to console failed");
}

