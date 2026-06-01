# Getting started

Run and test Grindstone on the Mac. Built and tested on macOS (Apple silicon) with Rust 1.98.1, Trunk 0.21.14 and Tauri CLI 2.12.1.

## Install once

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cargo install tauri-cli --version "^2" --locked
```

## Run

```sh
cd grind-leptos
make dev
```

The app opens in a window and rebuilds on every saved change. The first run compiles for a few minutes.

## Test

```sh
make test    # the Rust tests
make check   # formatting, lints and tests, as before a commit
```

## Install

```sh
make mac     # the Mac app, in /Applications
make android   # the phone over USB, once the Android toolchain is in place
```

## Data

The app keeps its SQLite file at:

```
~/Library/Application Support/com.maeevick.grindstone/grindstone.sqlite
```

To start from empty, quit the app and delete that file.

## Next

- The phone: [ANDROID_TOOLCHAIN.md](ANDROID_TOOLCHAIN.md), then [RUNBOOK.md](RUNBOOK.md).
- The commands: `make help`.
