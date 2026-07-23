# Grindstone

A personal tracker for body mass, food and training, carried on an Android phone and usable offline, outdoors.

## Why

- The tracker must work where training happens: outdoors, on the phone, with or without a network.
- The data belongs on the device: SQLite, no account, no server for now.
- Two earlier attempts taught what to avoid. The Sycamore draft (`../grindstone_draft`) kept its data in the browser's storage. The SvelteKit spike (`../grind-svelte`) became too large before reaching the phone.
- This spike puts a usable app on the phone first, then grows one real need at a time.

## What

Today, slice 1: the profile and Gather.

- **Profile:** birthday, gender, height, neck, and hip for a woman, then goals (a target mass and fat percentage). Each is kept from a chosen day, so changing it leaves the days before as they were.
- **Tracker:** the day's mass and waist, one measurement a day, a second replacing the first. The last 14 days show their fat ratio, fat mass and lean mass from the US Navy formula, a day without a waist carrying the last lean mass.
- **Home:** a welcome until a profile exists, then the Gather card over 30 days, 90 days or everything: the masses, their 7-day trend, the target line, the weekly rate, the day the target is reached at that rate, and the change in fat and lean mass.
- **Settings**, behind the gear on Profile: the draft's measurements, profiles and goals imported from a backup pushed over USB.

Next, one slice at a time: Burn, Fuel, the meal cooker, Hydrate, circuits, then Stats.

## How

| Part | Technology | Role |
| --- | --- | --- |
| `ui` | Leptos 0.8, client-side, compiled to WebAssembly by Trunk | the screen |
| `src-tauri` | Tauri 2 | the app shell on Android and macOS: its commands and the configurator that wires everything at start |
| `core` | plain Rust | the application: its rules, what it offers (driving ports) and what it needs (driven ports) |
| `adapters/sqlite` | rusqlite, SQLite bundled | the storage, behind the core's storage port |
| `adapters/draft` | serde_json | reads the draft's backup into the core's records |

The screen calls the app's commands through Tauri. Each command hands over to one of the core's driving ports (`for_tracking`, `for_setting_up_the_profile`, `for_reviewing_progress`, `for_importing`), which store through the SQLite adapter in the app's data folder.

## Documents

| Document | Use it to |
| --- | --- |
| [GETTING_STARTED.md](GETTING_STARTED.md) | run and test the app on the Mac |
| [ANDROID_TOOLCHAIN.md](ANDROID_TOOLCHAIN.md) | install what an Android build needs, and connect the phone over USB |
| [RUNBOOK.md](RUNBOOK.md) | go from nothing to the app installed on the phone, and update it |
