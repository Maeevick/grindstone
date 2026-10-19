# Grindstone

A personal tracker for body mass, food and training, carried on an Android phone and usable offline, outdoors.

## Why

- The tracker must work where training happens: outdoors, on the phone, with or without a network.
- The data belongs on the device: SQLite, no account, no server for now.
- Two earlier attempts taught what to avoid. The Sycamore draft (`../grindstone_draft`) kept its data in the browser's storage. The SvelteKit spike (`../grind-svelte`) became too large before reaching the phone.
- This spike puts a usable app on the phone first, then grows one real need at a time.

## What

Version 0.1, the full scope, on five tabs under an icon bar (🏠 Home, ⚖️ Gather, ⚡ Fuel, 💧 Hydrate, 🔥 Burn).

- **Every screen** wears a header: the logo, the name and the tagline, a person opening the Profile at the right, then today in four tiles, ⚖️ the body mass weighed, ⚡ the energy eaten, 💧 the millilitres drunk, 🔥 the energy burnt, each green when its streak tick holds and red when it does not, each opening its tab. The tiles read again after every change to the records and every move between screens.
- **Home:** a welcome until a profile exists. After that:
  - the streak:
    - from the profile's first day, four ticks every day: a weigh-in; Fuel, meaning the day's energy inside its band, the macros left to Fuel's gauges; Hydrate inside its band, except on days Hydrate is left out of the streak in Profile, which count on the other three; Burn, meaning the day's workouts reach its burn target, any workout before a first weigh-in prices them. A tracker without a log fails its tick;
    - a 5% grace for the ticks: each bound of a band gives way by 5% of itself, and a burn holds at 95% of its target; the gauges keep the strict bands;
    - a joker for a day one tick short between two full ones, and an unfinished today that never breaks it;
    - one card: its count in gold beside its longest run, a remark, and a line on the 5% grace;
  - a chart behind four chips (Gather, Fuel, Hydrate, Burn) over 7 days by default, 30, 90 or ∞: Gather's body mass and lean mass on one axis, each with its goal dashed; Burn with its target as a green dashed line; round ticks over thin grid lines, the x axis dated from the oldest day to the newest, the current values and a legend;
  - beneath it, the unwanted tips: what is worth sharpening for the chosen chip, the weakest habit against its band, the course of the body mass towards its goal, or the workouts a week;
  - each chip's history, month by month, its units in the headers and each day's streak dot, a tap on a day showing its four ticks;
  - "🎱 Explore the numbers" at the bottom, into Stats.
- **Each tab** runs the same way: the day (today when the tab opens, shown as yyyy-mm-dd), the log form, where the day stands, its breakdown, the day's entries, the last days, then, at the bottom, "📚 Open the grimoire" and "🎱 Explore the numbers" side by side, the numbers alone on Gather, which has no grimoire.
  - **Search or create:** every chooser starts on "Select me" and narrows from three letters typed, starting matches first, then those containing them, ignoring case and accents. Fuel's offers its meals (💪) before its foods (🥦), Burn's its circuits (🔄) before its exercises (💪), Hydrate's its water (💧) before its other drinks (☕), each group in alphabetical order. A name nothing matches turns the form into "New {name}", with the fields of a new entry and an optional description, and "Add and log" adds it to the catalog and logs it in one tap.
  - **The last days:** the week up to the tab's day, the day in gold; "load more" takes them back to the first of the month, then a month at a time, the months without records skipped.
  - **The grimoire:** the tab's catalog on a page of its own, the tab's icon still lit: "Add to the catalog" (to add without logging, or to change an entry by adding it again under its name), then "On the shelf", its lists with each entry's description under its name.
- **Gather:** mass and waist, filled with the day's record when there is one, the body mass marked required when there is not, and the day's weigh-in removed from the same card.
  - Where you stand: the body mass and the lean mass each on a line, the goal ticked, the day in blue, the body mass green within 1% of its goal (where the pace says it stands at it), the lean mass green from its goal up, its goal being a floor; the goals in force, else WHO's recommendations (a body mass index of 21.7, Gallagher's healthy fat). The lean-mass goal is the goal mass less its goal fat.
  - The week's pace: the weekly change of the body mass, lean mass, fat mass and fat ratio (the slope of the 7-day average over the week, the rate Home's chart reads over its 7 days, from 4 weigh-ins in the week), green towards the goal, red away, steady under 0.15% a week (the lean mass's goal being a floor: rising is green wherever it stands, falling plain above the floor and red once at or under it), and one verdict on the body mass, health first: losing under the healthy mass or fat, or faster than the safe lane (the least of 1% of the body mass, 1 kg, and what the fat can spare in a week, at Alpert's 290 kJ per kilo of fat a day), is a warning, said with both figures to the hundredth and the limit that set it, lean mass named only when the tape sees it leave; then very good, good, stable, the wrong way; at the goal within 1%.
- **Fuel:** intakes from the catalog, one portion when no quantity is typed, the energy and macros to be recorded previewed under the fields.
  - The plate: "Add to the plate" puts foods aside, each with its cooking ratio (the cooked mass over the raw one, 1 when left empty), the plate adding up what each brings and drawn as a ghost on the energy band and the macros. "Log the plate" logs one intake per ingredient, all or none; "Keep as a meal" cooks it, per 100 g cooked, then logs the portion eaten, the whole plate when left empty, nothing for 0. The plate waits while the app runs.
  - Where the day stands: the energy on a gauge, the minimum (25 kcal per kilo of lean mass for a man, 30 for a woman, plus the burn) in red, Alpert's floor and maintenance in gold, the band between them in green, what was eaten in blue.
  - The four macros: their goal, what was eaten, the gap and their share of the energy, with a total.
  - The day's intakes, each food with its quantity, energy and macros; the grimoire lists foods or meals, a click on a meal showing its description and ingredients.
- **Hydrate:** drinks in millilitres, one serving when none are typed.
  - Where the day stands: a scale from min through optimal to max, food bringing a tenth of the water. The optimal is 90% of what the lean mass turns over in a day (7% of its 73.2% of water) plus 1.25 mL per kcal burnt, the min 90% of a millilitre per kcal the day spends (its budget and burn) and never above the optimal, the max a fifth above the optimal; before any lean mass is known, EFSA's base plus a millilitre per kcal burnt, 20% either side. Past the max, a word on the litre an hour kidneys clear.
  - The day's drinks; the grimoire lists drinks, water or other.
- **Burn:** two pills, a workout or the day off the clock (sitting, standing, moving). The workout's chooser holds exercises and circuits alike: an exercise's blank fields log a minute, one rep, one set and no load; a circuit asks its sets alone. What either will burn on the day's body mass is previewed under the fields.
  - The session: "Add to the session" puts exercises aside as steps (minutes, reps, load), a circuit bringing its own, priced a set at the day's body mass, run as many sets as the session, drawn as a ghost on the burn band. "Log the session" logs a workout per step, all or none; "Keep as a circuit" crafts it, then runs it by the session's sets, one when left empty, none for 0. The session waits while the app runs.
  - Where the day stands: the burn on a line from nothing to 200 kcal, stretched past the target, the burn or its ghost, the target ticked, green from 95% of it on.
  - The day's expenditures, the workouts newest first, then the day off the clock and fidgeting; the grimoire lists exercises or circuits, a click on a circuit showing its description and steps.
- **Profile:**
  - the rusty you behind the numbers (birthday, gender, height, neck, and the hip, open to a woman only), goals, fuel targets in g/kg, whether Hydrate counts in the streak, the day's burn target and the usual day off the clock, each kept from a chosen day;
  - on the goals, fuel targets, usual day and burn target, "Use the recommendation" above "Keep", filling the card from health recommendations to check and adjust: WHO's normal body mass index, Gallagher's healthy fat, Morton's 1.6 g of protein per kilo, EFSA's fat and fibre, a usual day of 8 hours sitting at most (Ekelund), 2 standing (Buckley) and the steps that pay off at the age (Paluch), and WHO's 150 minutes a week of moderate activity as the burn target on the body weighed, any workout holding the tick before a weigh-in;
  - from each card, a link to the scientific references, cited in full with their public pages, then the disclaimer;
  - past a divider, the settings close the screen: the draft's import, and a danger zone that erases everything.
- **Stats:** any two measures over a window, as timelines, summaries and a scatter of the days they share with their correlation.

Next, together: daily challenges, then the Stats rework, export and import options, Health Connect steps, posture analysis.

## How

| Part | Technology | Role |
| --- | --- | --- |
| `ui` | Leptos 0.8, client-side, compiled to WebAssembly by Trunk | the screen |
| `src-tauri` | Tauri 2 | the app shell on Android, and on the Mac under `make dev`: its commands and the configurator that wires everything at start |
| `core` | plain Rust | the application: its rules, what it offers (driving ports) and what it needs (driven ports) |
| `adapters/sqlite` | rusqlite, SQLite bundled | the storage, behind the core's storage port |
| `adapters/draft` | serde_json | reads the draft's backup into the core's records |

The screen calls the app's commands through Tauri. Each command hands over to one of the core's driving ports (`for_tracking`, `for_fuelling`, `for_hydrating`, `for_burning`, `for_cataloguing`, `for_setting_up_the_profile`, `for_reviewing_progress`, `for_charting`, `for_importing`), which store through the SQLite adapter in the app's data folder.

## Documents

| Document | Use it to |
| --- | --- |
| [GETTING_STARTED.md](GETTING_STARTED.md) | run and test the app on the Mac |
| [RUNBOOK.md](RUNBOOK.md) | go from nothing to the app installed on the phone, toolchain and USB connection included, and update it |
