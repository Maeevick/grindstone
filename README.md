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
    - four ticks a day: a weigh-in; Fuel, meaning the day's energy inside its band; a workout; Hydrate inside its band;
    - a joker for a day one tick short between two full ones, and an unfinished today that never breaks it;
    - its count in gold, its longest run and today's four ticks at the top, each day's dot in the history beneath the chart;
  - today in four tiles: ⚖️ the body mass weighed, ⚡ the energy eaten, 💧 the millilitres drunk, 🔥 the energy burnt;
  - a chart behind four chips (Gather, Fuel, Hydrate, Burn) over 7 days by default, 30, 90 or ∞, its y axis on round ticks over thin grid lines, its x axis dated from the oldest day to the newest (every day over a week), the current values and a legend;
  - beneath it, Veil's note on what is worth sharpening for the chosen chip: the weakest habit against its band, the course of the body mass towards its goal, or the workouts a week;
  - each chip's history beneath, month by month, its units in the headers and each day's streak dot;
  - a way into Stats at the bottom.
- **Tracker:** one day at a time, today by default, shown as yyyy-mm-dd, in four segments, each closed by a way into Stats. Every list of the catalog starts on "Select me" and narrows from three letters typed, starting matches first, then those containing them, ignoring case and accents.
  - **Gather:** mass and waist, filled with the day's record when there is one, the body mass marked required when there is not. The last 7 days show their mass, waist, fat ratio and lean mass under their units, the chosen day in gold, with a tendency row: green when a measure closes on its goal, red when it moves away, a dash when it holds steady.
  - **Fuel:** intakes from the catalog, one portion of the food when no quantity is typed. The day's energy sits on a gauge: the minimum (25 kcal per kilo of lean mass for a man, 30 for a woman, plus the burn) in red, Alpert's floor and maintenance in gold, the band between them in green, what was eaten in blue. The four macros show their goal, what was eaten, the gap and their share of the energy, with a total. The day's intakes list each food with its quantity, energy and macros.
  - **Hydrate:** drinks in millilitres, one serving when none are typed, on a scale from min through optimal to max, the band running 20% either side of the target.
  - **Burn:** a workout or a circuit, on two tabs; blank fields log a minute, one rep, one set and no load. The day off the clock (sitting, standing, moving). The day's expenditures list the workouts newest first, then the day off the clock and fidgeting, the two parts of non-exercise activity.
- **Catalog:** three shelves, each adding from a form card at its top, chips switching between its forms:
  - **Fuel:** foods or meals, each list on its chip, their figures per portion in columns. A meal is cooked in Trix's lab, each ingredient with its cooking ratio (the cooked mass over the raw one, 1 when left empty, so the hot pot needs no weighing), then logs in Fuel by the weighed portion, per 100 g cooked; a click on it shows its description and ingredients, each raw quantity × ratio → cooked mass;
  - **Hydrate:** drinks, water or other;
  - **Burn:** exercises with their MET, or circuits fixed at creation with their energy per set at today's body mass, a click showing a circuit's description and steps.
- **Profile:**
  - the rusty you behind the numbers (birthday, gender, height, neck, and the hip, open to a woman only), goals, fuel targets in g/kg, the hydrate base and the usual day off the clock, each kept from a chosen day;
  - on each card but the first, "Use the recommendation" above "Keep", filling the card from health recommendations to check and adjust: WHO's normal body mass index, Gallagher's healthy fat, Morton's 1.6 g of protein per kilo, EFSA's fat, fibre and water, and a usual day of 8 hours sitting at most (Ekelund), 2 standing (Buckley) and the steps that pay off at the age (Paluch);
  - from each card, a link to the scientific references, cited in full with their public pages; the disclaimer closes the screen;
  - the gear opens Settings: the draft's import, and a danger zone that erases everything.
- **Stats:** any two measures over a window, as timelines, summaries and a scatter of the days they share with their correlation.

Next, together: daily challenges, then the Stats rework, export and import options, Health Connect steps, posture analysis.

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
