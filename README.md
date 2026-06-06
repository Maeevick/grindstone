# Grindstone

A personal tracker for body mass, food and training, carried on an Android phone and usable offline, outdoors.

## Why

- The tracker must work where training happens: outdoors, on the phone, with or without a network.
- The data belongs on the device: SQLite, no account, no server for now.
- Two earlier attempts taught what to avoid. The Sycamore draft (`../grindstone_draft`) kept its data in the browser's storage. The SvelteKit spike (`../grind-svelte`) became too large before reaching the phone.
- This spike puts a usable app on the phone first, then grows one real need at a time.

## What

Today, a walking skeleton:

- one screen to log a weigh-in on a date and read the last 14;
- one weigh-in a day, a second one replacing the first;
- the data kept in SQLite on the device.

Next: the features you choose, rethought around how you train and eat, not ported from the draft.

## How

| Part | Technology | Role |
| --- | --- | --- |
| `ui` | Leptos 0.8, client-side, compiled to WebAssembly by Trunk | the screen |
| `src-tauri` | Tauri 2 | the app shell on Android and macOS, its commands, SQLite through rusqlite |
| `core` | plain Rust | the rules shared by the screen and the app, tested with `cargo test` |

The screen calls the app's commands (`recent_weigh_ins`, `log_weigh_in`) through Tauri. The app applies the rules from `core` and reads or writes SQLite in its data folder.

## Documents

| Document | Use it to |
| --- | --- |
| [GETTING_STARTED.md](GETTING_STARTED.md) | run and test the app on the Mac |
| [ANDROID_TOOLCHAIN.md](ANDROID_TOOLCHAIN.md) | install what an Android build needs, and connect the phone over USB |
| [RUNBOOK.md](RUNBOOK.md) | go from nothing to the app installed on the phone, and update it |
