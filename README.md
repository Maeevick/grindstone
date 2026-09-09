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
  - the streak:
    - four ticks a day: a weigh-in; Fuel, meaning energy and the four macros in their bands; a workout; Hydrate inside its band;
    - a joker for a day one tick short between two full ones, and an unfinished today that never breaks it;
    - laid out as weeks from Monday to Sunday, five at a time, with the earlier weeks on demand;
  - today's fuel, hydrate and burn;
  - a chart behind four chips (Gather, Fuel, Hydrate, Burn) over 30 days, 90 days or everything, with its axes' values, the current values and a legend;
  - each chip's history beneath, month by month;
  - a way into Stats.
- **Tracker:** one day at a time, today by default, in four segments:
  - **Gather:** mass and waist. The last 14 days show their fat ratio, fat mass and lean mass from the US Navy formula.
  - **Fuel:** intakes from the catalog, weighed. The day's energy is read against its band, the budget plus the burn capped above and floored by what the fat store can spare (Alpert) and by lean mass. Protein, fat, fibre and carbs are read against theirs.
  - **Hydrate:** calorie-free drinks on a scale from min through optimal to max, the band running 10% either side of the target.
  - **Burn:** workouts priced on net METs at the day's body mass, circuits run with their sets, and the day off the clock (sitting, standing, moving).
- **Catalog:** foods with their figures per portion, meals cooked in Trix's lab (totals before and after cooking, per 100 g cooked), drinks, exercises with their MET, and circuits fixed at creation.
- **Profile:**
  - the rusty you behind the numbers (birthday, gender, height, neck and hip), goals, fuel targets in g/kg, the hydrate base and the usual day off the clock, each kept from a chosen day;
  - the rhythm of the week, from sedentary to extremely active, to fill every card but the first from health recommendations (WHO, Gallagher, the ISSN, EFSA, Tudor-Locke), to check and adjust;
  - from each card, a link to the scientific references, cited in full with their public pages;
  - the gear opens Settings: the draft's import, and a danger zone that erases everything.
- **Stats:** the habits to sharpen, the weakest first, then any two measures over a window, as timelines, summaries and a scatter of the days they share with their correlation.

Next, together: what v0.1 gets wrong, then export and import options, Health Connect steps, challenges, posture analysis.

## How

| Part | Technology | Role |
| --- | --- | --- |
| `ui` | Leptos 0.8, client-side, compiled to WebAssembly by Trunk | the screen |
| `src-tauri` | Tauri 2 | the app shell on Android and macOS: its commands and the configurator that wires everything at start |
| `core` | plain Rust | the application: its rules, what it offers (driving ports) and what it needs (driven ports) |
| `adapters/sqlite` | rusqlite, SQLite bundled | the storage, behind the core's storage port |
| `adapters/draft` | serde_json | reads the draft's backup into the core's records |

The screen calls the app's commands through Tauri. Each command hands over to one of the core's driving ports (`for_tracking`, `for_fuelling`, `for_hydrating`, `for_burning`, `for_cataloguing`, `for_setting_up_the_profile`, `for_reviewing_progress`, `for_charting`, `for_importing`), which store through the SQLite adapter in the app's data folder.

## Documents

| Document | Use it to |
| --- | --- |
| [GETTING_STARTED.md](GETTING_STARTED.md) | run and test the app on the Mac |
| [ANDROID_TOOLCHAIN.md](ANDROID_TOOLCHAIN.md) | install what an Android build needs, and connect the phone over USB |
| [RUNBOOK.md](RUNBOOK.md) | go from nothing to the app installed on the phone, and update it |
