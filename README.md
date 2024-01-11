# JWT UI - A Terminal UI for decoding/encoding JSON Web Tokens

![ci](https://github.com/jwt-rs/jwt-ui/actions/workflows/ci.yml/badge.svg)
![cd](https://github.com/jwt-rs/jwt-ui/actions/workflows/cd.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blueviolet.svg)
![LOC](https://tokei.rs/b1/github/jwt-rs/jwt-ui?category=code)
[![crates.io link](https://img.shields.io/crates/v/jwt-ui.svg)](https://crates.io/crates/jwt-ui)
![Release](https://img.shields.io/github/v/release/jwt-rs/jwt-ui?color=%23c694ff)
[![Coverage](https://coveralls.io/repos/github/jwt-rs/jwt-ui/badge.svg?branch=main)](https://coveralls.io/github/jwt-rs/jwt-ui?branch=main)
[![GitHub Downloads](https://img.shields.io/github/downloads/jwt-rs/jwt-ui/total.svg?label=GitHub%20downloads)](https://github.com/jwt-rs/jwt-ui/releases)
![Crate.io downloads](https://img.shields.io/crates/d/jwt-ui?label=Crate%20downloads)

[![Follow Deepu K Sasidharan (deepu105)](https://img.shields.io/twitter/follow/deepu105?label=Follow%20Deepu%20K%20Sasidharan%20%28deepu105%29&style=social)](https://twitter.com/intent/follow?screen_name=deepu105)

```
     ____.__      _____________       ____ ___.___
    |    /  \    /  \__    ___/      |    |   \   |
    |    \   \/\/   / |    |  ______ |    |   /   |
/\__|    |\        /  |    | /_____/ |    |  /|   |
\________| \__/\  /   |____|         |______/ |___|
                \/
```

A terminal UI for decoding/encoding JSON Web Tokens inspired by [JWT.io](https://jwt.io/) and [jwt-cli](https://github.com/mike-engel/jwt-cli)

![UI](screenshots/decoder.png)

Crafted by

<img src="artwork/Auth0byOkta_logo.svg" alt="Auth0 by Okta" style="width:120px;"/>

## Installation

### Homebrew (Mac & Linux)

```bash
brew tap jwt-rs/jwt-ui
brew install jwt-ui

# If you need to be more specific, use:
brew install jwt-rs/jwt-ui/jwt-ui
```

To upgrade

```bash
brew upgrade jwt-ui
```

### Scoop (Windows)

```bash
scoop bucket add jwt-ui-bucket https://github.com/jwt-rs/scoop-jwt-ui

scoop install jwt-ui
```

### Cargo

If you have Cargo installed then you install JWT UI from crates.io

```bash
cargo install jwt-ui
```

You can also clone the repo and run `cargo run` or `make` to build and run the app

### Install script

Run the below command to install the latest binary. Run with sudo if you don't have write access to `/usr/local/bin`. Else the script will install to the current directory

```sh
curl https://raw.githubusercontent.com/jwt-rs/jwt-ui/main/deployment/getLatest.sh | bash
```

### Docker

Run KDash as a Docker container by mounting your `KUBECONFIG`. For example the below command for the default path

```bash
docker run --rm deepu105/jwt-ui
```

You can also clone this repo and run `make docker` to build a docker image locally and run it using the above command

### Manual

Binaries for macOS, Linux and Windows are available on the [releases](https://github.com/jwt-rs/jwt-ui/releases) page

1. Download the latest [binary](https://github.com/jwt-rs/jwt-ui/releases) for your OS.
1. For Linux/macOS:
   1. `cd` to the file you just downloaded and run `tar -C /usr/local/bin -xzf downloaded-file-name`. Use sudo if required.
   2. Run with `jwtui`
1. For Windows:
   1. Use 7-Zip or TarTool to unpack the tar file.
   2. Run the executable file `jwtui.exe`

## Troubleshooting

> Note: On Debian/Ubuntu you might need to install `libxcb-xfixes0-dev` and `libxcb-shape0-dev`. On Fedora `libxcb` and `libxcb-devel` would be needed.

> Note: On Linux you might need to have package `xorg-dev` (Debian/Ubuntu) or `xorg-x11-server-devel` (Fedora) or equivalent installed for the copy to clipboard features to work

## USAGE:

```bash
jwtui

#or

jwtui [OPTIONS] [TOKEN]
```

Press `?` while running the app to see keybindings

Arguments:
[TOKEN] JWT token to decode [mandatory for stdout mode, optional for TUI mode]

Options:
`-s, --stdout` whether the CLI should run in TUI mode or just print to stdout
`-j, --json` whether stdout should be formatted as JSON
`-t, --tick-rate <TICK_RATE>` Set the tick rate (milliseconds): the lower the number the higher the FPS. Must be less than 1000 [default: 250]
`-S, --secret <SECRET>` secret for validating the JWT [default: ]
`-h, --help` Print help
`-V, --version` Print version

## Limitations/Known issues

## Features

- Fully offline
- Dark/Light themes
- Sensible keyboard shortcuts

## Screenshots

### Decoder screen

![UI](screenshots/decoder.png)

### Encoder screen

![UI](screenshots/encoder.png)

### Stdout

![UI](screenshots/stdout.png)

## Libraries used

- [jsonwebtoken](https://github.com/Keats/jsonwebtoken)
- [ratatui](https://github.com/ratatui-org/ratatui)
- [crossterm](https://github.com/crossterm-rs/crossterm)
- [clap](https://github.com/clap-rs/clap)
- [rust-clipboard](https://github.com/aweinstock314/rust-clipboard)

## Licence

MIT

## Creator

- [Deepu K Sasidharan](https://deepu.tech/)
