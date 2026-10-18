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
make test         # the Rust tests
make check        # formatting, lints, tests and no uncovered line outside the wiring, as before a commit
make test-ui      # the screen mounted in a headless Chrome, its commands answered by a stub
make mutants      # mutation testing of the core lines not pushed yet, after a big change: minutes
make mutants-all  # mutation testing of the whole core, once a day: about a quarter of an hour
```

On GitHub, every push to `main` and every pull request runs `make check`, `make test-ui` and a build of the APK signed with a key made for the run ([ci.yml](.github/workflows/ci.yml)).

## Install

```sh
make android   # the phone over USB, once the Android toolchain is in place
```

## Data

`make dev` keeps the app's SQLite file at:

```
~/Library/Application Support/com.maeevick.grindstone/grindstone.sqlite
```

To start from empty, quit the app and delete that file.

The draft's records are imported on the phone: [RUNBOOK.md](RUNBOOK.md), step 11.

## Next

- The phone: [RUNBOOK.md](RUNBOOK.md), from its toolchain to the app installed.
- The commands: `make help`.
