# Getting started

Run and test Grindstone on the Mac. Built and tested on macOS (Apple silicon) with Rust 1.99.0, Trunk 0.21.14, Tauri CLI 2.12.1 and cargo-llvm-cov 0.8.5.

## Install once

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cargo install tauri-cli --version "^2" --locked
```

For the tests and their gates:

```sh
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov --locked   # the coverage gate in make check
cargo install cargo-mutants --locked    # make mutants
cargo install wasm-pack --locked        # make test-ui: it fetches the matching wasm-bindgen test runner
```

`make test-ui` needs Google Chrome. It fetches, once per Chrome major version, the ChromeDriver matching the installed Chrome from Chrome for Testing into `~/Library/Caches/grindstone`, since the newest driver refuses a Chrome that has not updated yet.

## Run

```sh
make dev
```

The app opens in a window and rebuilds on every saved change. The first run compiles for a few minutes.

## Test

```sh
make test      # the Rust tests
make check     # formatting, lints, tests and no uncovered line outside the wiring, as before a commit
make test-ui   # the screen mounted in a headless Chrome, its commands answered by a stub
make mutants   # mutation testing of the core, at the end of a slice
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

To import the draft's records, copy one of its backups to `~/Documents/grindstone-draft.json`, then in the app go to Profile, the gear, **Import the draft**. macOS may first ask to let Grindstone read the Documents folder. The import replaces the measurements, profiles and goals only.

## Next

- The phone: [ANDROID_TOOLCHAIN.md](ANDROID_TOOLCHAIN.md), then [RUNBOOK.md](RUNBOOK.md).
- The commands: `make help`.
