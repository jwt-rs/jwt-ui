# JWT CLI - A command line UI for decoding/encoding JSON Web Tokens

![ci](https://github.com/jwt-rs/kdash/actions/workflows/ci.yml/badge.svg)
![cd](https://github.com/jwt-rs/kdash/actions/workflows/cd.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blueviolet.svg)
![LOC](https://tokei.rs/b1/github/jwt-rs/kdash?category=code)
[![crates.io link](https://img.shields.io/crates/v/kdash.svg)](https://crates.io/crates/kdash)
![Docker Release](https://img.shields.io/docker/v/deepu105/kdash?label=Docker%20version)
![Release](https://img.shields.io/github/v/release/jwt-rs/kdash?color=%23c694ff)
[![Coverage](https://coveralls.io/repos/github/jwt-rs/kdash/badge.svg?branch=main)](https://coveralls.io/github/jwt-rs/kdash?branch=main)
[![GitHub Downloads](https://img.shields.io/github/downloads/jwt-rs/kdash/total.svg?label=GitHub%20downloads)](https://github.com/jwt-rs/kdash/releases)
![Docker pulls](https://img.shields.io/docker/pulls/deepu105/kdash?label=Docker%20downloads)
![Crate.io downloads](https://img.shields.io/crates/d/kdash?label=Crate%20downloads)

[![Follow Deepu K Sasidharan (deepu105)](https://img.shields.io/twitter/follow/deepu105?label=Follow%20Deepu%20K%20Sasidharan%20%28deepu105%29&style=social)](https://twitter.com/intent/follow?screen_name=deepu105)

![logo](artwork/logo.png)

A command line UI for decoding/encoding JSON Web Tokens inspired by [JWT.io](https://jwt.io/) and [jwt-cli](https://github.com/mike-engel/jwt-cli)

![UI](screenshots/ui.gif)

## Installation

### Homebrew (Mac & Linux)

```bash
brew tap jwt-rs/jwt-cli
brew install jwt-cli

# If you need to be more specific, use:
brew install jwt-rs/jwt-cli/jwt-cli
```

To upgrade

```bash
brew upgrade jwt-cli
```

### Scoop (Windows)

```bash
scoop bucket add jwt-cli-bucket https://github.com/jwt-rs/scoop-jwt-cli

scoop install jwt-cli
```

### Install script

Run the below command to install the latest binary. Run with sudo if you don't have write access to `/usr/local/bin`. Else the script will install to the current directory

```sh
curl https://raw.githubusercontent.com/jwt-rs/jwt-cli/main/deployment/getLatest.sh | bash
```

### Manual

Binaries for macOS, Linux and Windows are available on the [releases](https://github.com/jwt-rs/jwt-cli/releases) page

1. Download the latest [binary](https://github.com/jwt-rs/jwt-cli/releases) for your OS.
1. For Linux/macOS:
   1. `cd` to the file you just downloaded and run `tar -C /usr/local/bin -xzf downloaded-file-name`. Use sudo if required.
   2. Run with `jwtd`
1. For Windows:
   1. Use 7-Zip or TarTool to unpack the tar file.
   2. Run the executable file `jwtd.exe`

### Cargo

If you have Cargo installed then you install KDash from crates.io

```bash
cargo install jwt-cli
```

> Note: On Debian/Ubuntu you might need to install `libxcb-xfixes0-dev` and `libxcb-shape0-dev`. On Fedora `libxcb` and `libxcb-devel` would be needed.

> Note: On Linux you might need to have package `xorg-dev` (Debian/Ubuntu) or `xorg-x11-server-devel` (Fedora) or equivalent installed for the copy to clipboard features to work

> Note: If you are getting compilation error from openSSL. Make sure perl and perl-core are installed for your OS.

You can also clone the repo and run `cargo run` or `make` to build and run the app

## USAGE:

```bash
jwtd

#or

jwtd <jwt-token>
```

Press `?` while running the app to see keybindings

## FLAGS:

- `-h, --help`: Prints help information
- `-V, --version`: Prints version information
- `-t, --tick-rate <tick-rate>`: Set the tick rate (milliseconds): the lower the number the higher the FPS.
- `-r, --raw`: Raw mode. Disables TUI and prints output to stdout.

## Limitations/Known issues

## Features

- Dark/Light themes
- Sensible keyboard shortcuts

## Screenshots

### Decoder screen

![UI](screenshots/overview.png)

### Stdout

![UI](screenshots/overview.png)

## Libraries used

- [ratatui](https://github.com/ratatui-org/ratatui)
- [crossterm](https://github.com/crossterm-rs/crossterm)
- [clap](https://github.com/clap-rs/clap)
- [tokio](https://github.com/tokio-rs/tokio)
- [duct.rs](https://github.com/oconnor663/duct.rs)
- [rust-clipboard](https://github.com/aweinstock314/rust-clipboard)

## Licence

MIT

## Creator

- [Deepu K Sasidharan](https://deepu.tech/)
