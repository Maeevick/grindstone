# Grindstone

A personal tracker for body mass, food and training, carried on an Android phone and usable offline, outdoors.

## Why

- The tracker must work where training happens: outdoors, on the phone, with or without a network.
- The data belongs on the device: SQLite, no account, no server for now.
- Two earlier attempts taught what to avoid. The Sycamore draft (`../grindstone_draft`) kept its data in the browser's storage. The SvelteKit spike (`../grind-svelte`) became too large before reaching the phone.
- This spike puts a usable app on the phone first, then grows one real need at a time.

## What

Version 0.1, the full scope, on four tabs.

- **Home:** Grimble's welcome until a profile exists. After that:
  - the streak: four ticks a day (a weigh-in; Fuel, meaning energy and the four macros in their bands; a workout; Hydrate), a joker for a day one tick short between two full ones, and an unfinished today that never breaks it;
  - today's fuel, burn and hydrate;
  - the Gather card over 30 days, 90 days or everything: masses, 7-day trend, target, weekly rate, the day the target is reached, the change in fat and lean mass;
  - a way into Stats.
- **Tracker:** one day at a time, today by default, in four segments:
  - **Gather:** mass and waist. The last 14 days show their fat ratio, fat mass and lean mass from the US Navy formula.
  - **Fuel:** intakes from the catalog, weighed. The day's energy is read against its band, the budget plus the burn capped above and floored by what the fat store can spare (Alpert) and by lean mass. Protein, fat, fibre and carbs are read against theirs.
  - **Hydrate:** calorie-free drinks against the day's target.
  - **Burn:** workouts priced on net METs at the day's body mass, circuits run with their sets, and the day off the clock (sitting, standing, moving).
- **Catalog:** foods with their figures per portion, meals cooked in Trix's lab (totals before and after cooking, per 100 g cooked), drinks, exercises with their MET, and circuits fixed at creation.
- **Profile:** birthday, gender, height, neck and hip, goals, fuel targets in g/kg beside the official recommendations and their sources, the hydrate base, and the usual day off the clock. Each is kept from a chosen day. The gear opens Settings, which imports the draft's backup pushed over USB.
- **Stats:** any two measures over a window, as timelines, summaries and a scatter of the days they share with their correlation.

Next, together: what v0.1 gets wrong, then export and import options, Health Connect steps, challenges, posture analysis.

## How

| Part | Technology | Role |
| --- | --- | --- |
| `ui` | Leptos 0.8, client-side, compiled to WebAssembly by Trunk | the screen |
| `src-tauri` | Tauri 2 | the app shell on Android and macOS: its commands and the configurator that wires everything at start |
| `core` | plain Rust | the application: its rules, what it offers (driving ports) and what it needs (driven ports) |
| `adapters/sqlite` | rusqlite, SQLite bundled | the storage, behind the core's storage port |
| `adapters/draft` | serde_json | reads the draft's backup into the core's records |

The screen calls the app's commands through Tauri. Each command hands over to one of the core's driving ports (`for_tracking`, `for_fuelling`, `for_hydrating`, `for_burning`, `for_cataloguing`, `for_setting_up_the_profile`, `for_reviewing_progress`, `for_importing`), which store through the SQLite adapter in the app's data folder.

## Documents

| Document | Use it to |
| --- | --- |
| [GETTING_STARTED.md](GETTING_STARTED.md) | run and test the app on the Mac |
| [ANDROID_TOOLCHAIN.md](ANDROID_TOOLCHAIN.md) | install what an Android build needs, and connect the phone over USB |
| [RUNBOOK.md](RUNBOOK.md) | go from nothing to the app installed on the phone, and update it |
