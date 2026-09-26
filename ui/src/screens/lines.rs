use grind_core::bibliography::Topic;
use grind_core::burn::Burn;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, shown, signed};
use grind_core::food::{Nutrients, Unit};
use grind_core::for_importing::Imported;
use grind_core::for_reviewing_progress::GatherCard;
use grind_core::fuel::{Fuel, Verdict};
use grind_core::hydrate::Hydrate;
use grind_core::meal::{Ingredient, Meal};
use grind_core::recommendations::Recommendations;
use grind_core::refusal::Refusal;
use grind_core::sharpening::{BandNote, BurnNote, GatherNote, Habit, Leaning};
use grind_core::stats::{Measure, Summary, figure_of};
use grind_core::streak::{Status, Streak, Ticks};
use grind_draft::{DraftRefusal, ImportFailure};

pub const WELCOME: &str = "Greetings, rusty one. I am Grimble, archivist of the Great Library of Cogsburg. \
    From today I file your every gram. [He adjusts his glasses.] Shall we begin with your profile, \
    or fetch the old records from the draft first?";

pub const LOADING: &str = "Grimble is fetching the ledger…";

pub const LOST: &str = "Lost in the tunnels under the library. Try another tab.";

pub fn profile_kept(from: &str) -> String {
    format!("Filed. [Grimble licks a finger and turns a page.] This profile is kept from {from}.")
}

pub fn goals_kept(from: &str) -> String {
    format!("Noted in the margin, in very small letters: these goals are kept from {from}.")
}

pub fn gather_figures(card: &GatherCard) -> Vec<String> {
    let rate = card.progress.map(|progress| {
        format!(
            "{} a week ({}), by my meticulous observations.",
            shown(Figure::Mass(progress.per_week)),
            shown(Figure::Ratio(progress.share_per_week))
        )
    });
    let changes = card
        .fat_change_kg
        .zip(card.lean_change_kg)
        .map(|(fat, lean)| {
            format!(
                "Fat {}, lean {} over the window.",
                shown(Figure::Mass(fat)),
                shown(Figure::Mass(lean))
            )
        });
    let target =
        card.target_kg.map(
            |target| match card.progress.and_then(|progress| progress.reached_on) {
                Some(day) => format!(
                    "{} reached around {day}, give or take a goblin.",
                    shown(Figure::Mass(target))
                ),
                None => format!(
                    "{}: the trend is not heading there yet.",
                    shown(Figure::Mass(target))
                ),
            },
        );
    [rate, changes, target].into_iter().flatten().collect()
}

pub const IMPORT_HOW: &str = "Bring the draft's latest backup from the Mac with make import-draft-android, \
    the phone plugged in, or make import-draft-mac for the Mac app, then import it here. It replaces every record the draft had, and from its day the streak \
    asks for Hydrate and the four macros too. Run it again with a newer backup until you switch for good.";

fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

pub fn imported(imported: &Imported) -> String {
    format!(
        "Filed: {}, {}, {}, {}, {}, {}, {}, {} and {}. [Grimble dusts off the old records.]",
        counted(imported.measurements, "day", "days"),
        counted(imported.profiles, "profile", "profiles"),
        counted(imported.goals, "goal", "goals"),
        counted(imported.exercises, "exercise", "exercises"),
        counted(imported.workouts, "workout", "workouts"),
        counted(imported.foods, "food", "foods"),
        counted(imported.intakes, "intake", "intakes"),
        counted(imported.drinks, "drink", "drinks"),
        counted(imported.drinkings, "drinking", "drinkings")
    )
}

pub fn added(name: &str) -> String {
    format!("{name} joins the catalog. [Grimble files it under its own letter.]")
}

pub const NO_WORKOUT: &str = "No workout on this day. A rest day counts as work: \
    the point is to have decided, not to have moved.";

pub const NO_EXERCISE: &str = "The catalog is empty. Add an exercise and its MET \
    (the 2024 Compendium of Physical Activities lists them), or import the draft's.";

pub fn burnt(burn: &Burn) -> String {
    match burn.burnt_kcal {
        Some(kcal) => format!(
            "{} burnt beyond the day off the clock.",
            shown(Figure::Energy(kcal))
        ),
        None => {
            "Weigh in first: I price a workout on the body that did it, not on a stand-in.".into()
        }
    }
}

pub fn off_the_clock(burn: &Burn) -> &'static str {
    if burn.off_the_clock_is_usual {
        "The usual day off the clock, as set in Profile. Change it for this day only:"
    } else {
        "This day's own day off the clock:"
    }
}

pub fn usual_day_kept(from: &str) -> String {
    format!("Filed: the usual day off the clock is kept from {from}.")
}

pub const DAY_OFF_THE_CLOCK_KEPT: &str = "Noted for this day only. [Grimble underlines it twice.]";

pub fn import_failure(refused: &ImportFailure) -> String {
    match refused {
        ImportFailure::NoBackupFound(where_expected) => format!(
            "No backup in the tunnels. Bring one with make import-draft-android or make import-draft-mac first; I looked in {where_expected}."
        ),
        ImportFailure::Draft(DraftRefusal::Unreadable(where_it_stopped)) => {
            format!(
                "This backup reads like Trix's notes: it stops making sense at {where_it_stopped}."
            )
        }
        ImportFailure::Draft(DraftRefusal::NotAVersionWeRead(version)) => {
            format!("A backup of version {version}: I only read the draft's 0.4.0.")
        }
        ImportFailure::Draft(DraftRefusal::NotAMeasurement { date, refusal }) => {
            format!(
                "The day {date} would not be gathered: {}",
                failure_of_refusal(*refusal)
            )
        }
        ImportFailure::Draft(DraftRefusal::NotAnExercise { name, refusal }) => {
            format!(
                "The activity {name} would not join the catalog: {}",
                failure_of_refusal(*refusal)
            )
        }
        ImportFailure::Draft(DraftRefusal::NotAWorkout { date, refusal }) => {
            format!(
                "A workout of {date} would not be filed: {}",
                failure_of_refusal(*refusal)
            )
        }
        ImportFailure::Draft(DraftRefusal::NotAFood { name, refusal }) => {
            format!(
                "The food {name} would not join the catalog: {}",
                failure_of_refusal(*refusal)
            )
        }
        ImportFailure::Draft(DraftRefusal::NotADrinking { date, refusal }) => {
            format!(
                "A drink of {date} would not be filed: {}",
                failure_of_refusal(*refusal)
            )
        }
        ImportFailure::Failed(failed) => failure(failed),
    }
}

fn failure_of_refusal(refusal: grind_core::refusal::Refusal) -> String {
    failure(&Failure::Refused(refusal))
}

pub fn failure(failure: &Failure) -> String {
    match failure {
        Failure::Refused(Refusal::NotADate) => {
            "That date does not exist. I have checked every calendar in the library.".into()
        }
        Failure::Refused(Refusal::NotAMass) => {
            "A body of nothing at all? I file many things, but not ghosts.".into()
        }
        Failure::Refused(Refusal::NotAWaist) => "A waist above nothing, please.".into(),
        Failure::Refused(Refusal::NotAHeight) => "A height above nothing, please.".into(),
        Failure::Refused(Refusal::NotANeck) => "A neck above nothing, please.".into(),
        Failure::Refused(Refusal::NotAHip) => {
            "Her Navy formula reads the hip: a hip above nothing, please.".into()
        }
        Failure::Refused(Refusal::NotARatio) => {
            "A fat target between 0 and 100%, the goblin way or any other.".into()
        }
        Failure::Refused(Refusal::NotAProfile) => {
            "Recommendations need the rusty you first: birthday, gender and height.".into()
        }
        Failure::Refused(Refusal::NotAName) => {
            "Even Trix names her explosions. A name, please.".into()
        }
        Failure::Refused(Refusal::NotAMet) => {
            "A MET above nothing: resting already counts as one.".into()
        }
        Failure::Refused(Refusal::NotADuration) => "Some minutes above nothing, please.".into(),
        Failure::Refused(Refusal::NotACount) => "At least one set, rusty one.".into(),
        Failure::Refused(Refusal::NotALoad) => {
            "A load in kilos, a band counting below zero.".into()
        }
        Failure::Refused(Refusal::NotADistance) => {
            "Minutes and kilometres of nothing or more, and no more minutes than a day holds."
                .into()
        }
        Failure::Refused(Refusal::NotAPortion) => {
            "A portion above nothing: per 100 g, per 250 mL or per piece.".into()
        }
        Failure::Refused(Refusal::NotANutrient) => {
            "Energy and macros of nothing or more. Negative protein is a Trix experiment.".into()
        }
        Failure::Refused(Refusal::NotAQuantity) => "A quantity above nothing, please.".into(),
        Failure::Refused(Refusal::NotAFood) => {
            "That food is not in the catalog yet. Add it there first.".into()
        }
        Failure::Refused(Refusal::NotAnExercise) => {
            "That exercise is not in the catalog yet. Veil checked twice.".into()
        }
        Failure::Refused(Refusal::NotASingleStep) => {
            "A circuit needs one step at least. Even my experiments have one: the bang. ~ Trix 🧪💥"
                .into()
        }
        Failure::Refused(Refusal::NotACircuit) => {
            "That circuit is not in the lab. Craft it in Catalog first. ~ Trix 🧪💥".into()
        }
        Failure::Refused(Refusal::NotAVolume) => {
            "Some millilitres above nothing, please. An empty cup files as nothing at all.".into()
        }
        Failure::Refused(Refusal::NotADrink) => {
            "That drink is not in the catalog yet. Grimble searched every shelf, twice.".into()
        }
        Failure::Refused(Refusal::NotACookingRatio) => {
            "A cooking ratio above nothing, please: 2.5 for pasta that drinks its water, 0.66 for cream \
             that boils down, 1 for what keeps its mass. ~ Trix 🧪💥"
                .into()
        }
        Failure::Refused(Refusal::NotWeighed) => {
            "That ingredient is counted in pieces, and the lab cooks by mass: add it in grams or \
             millilitres for now. ~ Trix 🧪💥"
                .into()
        }
        Failure::Broken(reason) => {
            format!("Something broke: {reason}. [A distant explosion.] Trix says it is data.")
        }
    }
}

pub const NO_BAND: &str = "No band yet: I need a profile and a weigh-in with a waist to price your day. \
    I do not guess bodies, not even goblin ones.";

pub const CHOOSE_FIRST: &str = "Choose from the list first: three letters narrow it down. [Grimble taps the empty line twice.]";

pub const NO_INTAKE: &str = "Nothing eaten on this day, or nothing filed yet.";

pub const NO_FOOD: &str =
    "The catalog has no food yet. Add one with its figures per portion, or import the draft's.";

pub fn energy_line(fuel: &Fuel) -> String {
    let eaten = shown(Figure::Energy(fuel.eaten.energy_kcal));
    match fuel.band {
        Some(band) => format!(
            "{eaten} eaten, the band runs from {} to {} kcal.",
            shown(Figure::Energy(band.floor_kcal)).trim_end_matches(" kcal"),
            shown(Figure::Energy(band.cap_kcal)).trim_end_matches(" kcal")
        ),
        None => format!("{eaten} eaten."),
    }
}

pub fn verdict_class(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Low => "low",
        Verdict::InBand => "good",
        Verdict::High => "high",
    }
}

pub fn fuel_targets_kept(from: &str) -> String {
    format!(
        "Filed: these targets are kept from {from}. Grimble recalculates every band from that day."
    )
}

pub const NO_MEAL: &str = "No meal cooked yet. Trix's lab is spotless, which worries everyone.";

pub fn figures_of(nutrients: &Nutrients) -> String {
    format!(
        "{} · P {} · C {} · F {} · Fib {}",
        shown(Figure::Energy(nutrients.energy_kcal)),
        shown(Figure::Grams(nutrients.protein_g)),
        shown(Figure::Grams(nutrients.carbs_g)),
        shown(Figure::Grams(nutrients.fat_g)),
        shown(Figure::Grams(nutrients.fibre_g)),
    )
}

pub const COOKING_RATIO_HOW: &str = "Ratio: the cooked mass over the raw one. 2.5 for pasta that drinks \
    its water, 0.66 for cream that boils down, 1 when left empty. No need to weigh the pot, it is hot. ~ Trix 🧪💥";

pub fn cooked_ingredient(ingredient: &Ingredient) -> String {
    format!(
        "{} {} {} → {}",
        shown(Figure::Amount(ingredient.quantity, ingredient.unit)),
        ingredient.food,
        shown(Figure::Times(ingredient.cooked_per_raw)),
        shown(Figure::Amount(ingredient.cooked_g(), Unit::Gram))
    )
}

pub fn meal_totals(meal: &Meal) -> String {
    format!(
        "{} cooked from {} raw: {}.",
        shown(Figure::Amount(meal.cooked_g, Unit::Gram)),
        shown(Figure::Amount(meal.raw_g, Unit::Gram)),
        figures_of(&meal.totals)
    )
}

pub fn per_100_g_cooked(per_100_g: &Nutrients) -> String {
    format!("Per 100 g cooked: {}.", figures_of(per_100_g))
}

pub fn cooked(name: &str) -> String {
    format!("{name} is in the food catalog, per 100 g cooked. Weigh your portions in Fuel.")
}

pub fn streak_line(streak: &Streak) -> String {
    let days = match streak.days {
        0 => "No streak yet".to_string(),
        1 => "1 day".to_string(),
        many => format!("{many} days"),
    };
    if streak.at_risk {
        format!(
            "{days}. Yesterday was a joker: today must be complete, or the streak goes back to the archive."
        )
    } else {
        format!("{days}. The streak is there to build habits, not to flatter.")
    }
}

pub fn status_word(status: Status) -> &'static str {
    match status {
        Status::Success => "every tick",
        Status::Joker => "a joker",
        Status::Break => "a break",
        Status::Pending => "under way",
    }
}

pub fn ticks_of(ticks: &Ticks) -> Vec<(&'static str, bool)> {
    [
        ("Weigh-in", ticks.weighed),
        ("Fuel in band", ticks.fuelled),
        ("Workout", ticks.worked_out),
        ("Hydrate", ticks.hydrated),
    ]
    .into_iter()
    .filter_map(|(name, held)| held.map(|held| (name, held)))
    .collect()
}

pub fn measure_name(measure: Measure) -> &'static str {
    match measure {
        Measure::BodyMass => "Body mass",
        Measure::Waist => "Waist",
        Measure::FatRatio => "Fat ratio",
        Measure::LeanMass => "Lean mass",
        Measure::Eaten => "Energy eaten",
        Measure::Balance => "Eaten minus the cap",
        Measure::Protein => "Protein",
        Measure::Carbs => "Carbs",
        Measure::Fat => "Fat",
        Measure::Fibre => "Fibre",
        Measure::Burnt => "Burnt in workouts",
    }
}

pub fn summed_up(measure: Measure, summary: &Summary) -> String {
    let shown_as = |value| shown(figure_of(measure, value));
    format!(
        "{}: from {} to {}, {} on average, between {} and {}.",
        measure_name(measure),
        shown_as(summary.first),
        shown_as(summary.last),
        shown_as(summary.mean),
        shown_as(summary.lowest),
        shown_as(summary.highest)
    )
}

pub fn together(correlation: Option<f64>, shared: usize) -> String {
    match correlation {
        Some(r) => format!(
            "Correlation {r:.2} over {shared} shared days. [Veil, from behind a bush: \"shhhhh! I'm hunting...\" and a correlation is not a cause.]"
        ),
        None => format!(
            "{shared} shared days: Veil needs at least three that move to hunt a correlation."
        ),
    }
}

pub const NOTHING_TO_EXPLORE: &str = "Nothing logged for this measure in the window.";

pub const CIRCUIT_HOW: &str = "Name it, stack its steps one by one, then craft. Once crafted it is \
    welded shut, like my last cauldron: to change it, remove it and craft another. ~ Trix 🧪💥";

pub const NO_CIRCUIT: &str = "No circuit in the lab yet. Stack a few exercises from the catalog \
    and craft one. ~ Trix 🧪💥";

pub const NO_CIRCUIT_TO_RUN: &str = "No circuit to run yet: the lab is in Catalog. ~ Trix 🧪💥";

pub fn circuit_crafted(name: &str) -> String {
    format!("{name} is crafted, its steps welded in place. ~ Trix 🧪💥")
}

pub fn circuit_logged(name: &str, sets: u32) -> String {
    format!(
        "Fire in the hole! {name}, {}, filed in the ledger. ~ Trix 🧪💥",
        counted(sets as usize, "set", "sets")
    )
}

pub fn drunk_against_target(hydrate: &Hydrate) -> String {
    format!(
        "{} of {}",
        shown(Figure::Volume(hydrate.drunk_ml)),
        shown(Figure::Volume(hydrate.target_ml))
    )
}

pub const TARGET_DRUNK: &str =
    "Target reached. [Grimble files the cup beside the 213 espressos, purely for science.]";

pub const NO_DRINKING: &str =
    "Not a drop filed on this day yet. The archive is drier than its oldest scroll.";

pub fn hydrate_base_kept(from: &str) -> String {
    format!("Filed. [Grimble dips his quill.] This base is kept from {from}.")
}

pub fn default_hydrate_base(base_ml: f64) -> String {
    format!(
        "No base set yet, so EFSA's default holds: {} from drinks, four fifths of the adequate \
         intake of total water, food bringing the rest.",
        shown(Figure::Volume(base_ml))
    )
}

pub fn month_name(month: &str) -> String {
    chrono::NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
        .map(|first| first.format("%B %Y").to_string())
        .unwrap_or_else(|_| month.to_string())
}

pub const NOTHING_TO_CHART: &str =
    "Nothing filed in this window yet. Grimble keeps the parchment ready.";

pub const NOTHING_THIS_MONTH: &str = "Nothing filed this month yet.";

fn habit_name(habit: Habit) -> &'static str {
    match habit {
        Habit::Energy => "Energy",
        Habit::Protein => "Protein",
        Habit::Fat => "Fat",
        Habit::Fibre => "Fibre",
        Habit::Carbs => "Carbs",
        Habit::Hydrate => "Water",
    }
}

fn a_week(per_week_kg: f64) -> String {
    format!("{} kg a week", signed(Figure::Mass(per_week_kg)))
}

pub fn gather_note(note: &GatherNote) -> String {
    match note {
        GatherNote::Unweighed => {
            "No weigh-in in this window. Veil tracks footprints, and the scale has left none. \
            [Veil waits. Veil is good at waiting.]"
                .into()
        }
        GatherNote::Irregular { weighed_days, days } => format!(
            "Weighed on {weighed_days} of {days} days. A trend wants a footprint every other day at least; \
             fewer, and it is a rumour with a decimal point."
        ),
        GatherNote::TooSoon => "Too few weigh-ins for a trend yet. Patience is half the hunt. \
            The other half, Veil insists, is also patience."
            .into(),
        GatherNote::Aimless { per_week_kg } => format!(
            "{}, heading nowhere in particular: no goal is set. A hunt without a quarry is just a walk \
             in the woods. Set one in Profile.",
            a_week(*per_week_kg)
        ),
        GatherNote::OnCourse {
            per_week_kg,
            reached_on,
        } => format!(
            "{}: on this trend the goal falls around {reached_on}. Keep the pace, not the pressure.* \
             (*Veil does not do pressure. Veil does patience, and arrows.)",
            a_week(*per_week_kg)
        ),
        GatherNote::OffCourse { per_week_kg } => format!(
            "{}, and the goal lies elsewhere. Not a verdict: trends turn when habits do, and habits \
             are the one thing down here that sharpens.",
            a_week(*per_week_kg)
        ),
    }
}

pub fn fuel_note(note: &BandNote) -> String {
    match note {
        BandNote::Unlogged => {
            "Nothing eaten on the record in this window. Veil cannot hunt what was never filed. \
            [Grimble looks wounded.]"
                .into()
        }
        BandNote::Sharp {
            weakest,
            in_band,
            logged,
        } => format!(
            "Every habit held its band four days in five; even the weakest, {}, on {in_band} of {logged}. \
             Nothing to hunt. Veil finds this deeply suspicious.",
            habit_name(*weakest)
        ),
        BandNote::Blunt {
            habit,
            in_band,
            logged,
            leaning: Leaning::Under,
        } => format!(
            "{} reached its band on {in_band} of {logged} days and fell short on the rest. \
             The quarry is small, slow, and frankly asking to be caught.",
            habit_name(*habit)
        ),
        BandNote::Blunt {
            habit,
            in_band,
            logged,
            leaning: Leaning::Over,
        } => format!(
            "{} went past its band on most days: in it on {in_band} of {logged}. \
             Generosity is a virtue, except in a ledger.",
            habit_name(*habit)
        ),
    }
}

pub fn hydrate_note(note: &BandNote) -> String {
    match note {
        BandNote::Unlogged => {
            "Not a drop on the record in this window. The archive is drier than its oldest scroll, \
            and that one is a fire hazard."
                .into()
        }
        BandNote::Sharp {
            in_band, logged, ..
        } => format!(
            "In the band on {in_band} of {logged} days. [Veil nods once, which for Veil is a standing ovation.]"
        ),
        BandNote::Blunt {
            in_band,
            logged,
            leaning: Leaning::Under,
            ..
        } => format!(
            "Short of water on most days: in the band on {in_band} of {logged}. A cup within reach does \
             most of the work; remembering it is there does the rest."
        ),
        BandNote::Blunt {
            in_band,
            logged,
            leaning: Leaning::Over,
            ..
        } => format!(
            "Past the band on most days: in it on {in_band} of {logged}. More is not better, only wetter."
        ),
    }
}

pub fn burn_note(note: &BurnNote) -> String {
    match note {
        BurnNote::Idle => "No workout in this window. A rest day counts as work; a rest month is \
            a different hunt altogether."
            .into(),
        BurnNote::Sparse { per_week } => format!(
            "{} active days a week. The WHO asks for strength on two; the grindstone turns either way, \
             but it turns better with a hand on it.",
            shown(Figure::Count(*per_week))
        ),
        BurnNote::Steady {
            per_week,
            burnt_per_active_day,
        } => format!(
            "{} active days a week{}. Steady as a goblin with a grudge. [Veil marks the tree and moves on.]",
            shown(Figure::Count(*per_week)),
            burnt_per_active_day
                .map(|kcal| format!(", {} burnt on each", shown(Figure::Energy(kcal))))
                .unwrap_or_default()
        ),
    }
}

pub const UNREADABLE: &str = "That is no number I can read. Digits, with a point or a comma, please. [Grimble squints at the ink.]";

pub const ERASE_WARNING: &str = "Erasing removes every record on this device: measurements, profile, goals, \
    foods, meals, intakes, drinks, workouts, circuits and settings. There is no undo. The app starts again \
    as on its first launch, and the draft can be imported again.";

pub const ERASED: &str = "Everything is erased. [Grimble closes an empty ledger and opens a fresh one.] Home starts over with the welcome.";

pub const RECOMMENDATIONS_DISCLAIMER: &str = "Recommendations for health, made for an average body: not medical advice, \
    and not a prescription. Grindstone takes no responsibility for them. Check them, adjust them, and keep adjusting \
    as your logs show what your body does: logging, monitoring, adjusting, enjoying. No activity multiplier inflates \
    the energy band: it only counts the workouts you log.";

pub const RECOMMENDATION_FILLED: &str =
    "Filled from the recommendation. Check it, adjust it, then keep it.";

pub fn healthy_ranges(advised: &Recommendations) -> String {
    let (lightest, heaviest) = advised.healthy_mass_kg;
    let (leanest, fattest) = advised.healthy_fat_ratio;
    format!(
        "Healthy at your height and age: {} to {} (a body mass index of 18.5 to 24.9), {} to {} body fat.",
        shown(Figure::Mass(lightest)),
        shown(Figure::Mass(heaviest)),
        shown(Figure::Ratio(leanest)),
        shown(Figure::Ratio(fattest))
    )
}

pub const REFERENCES_INTRO: &str = "The studies and guidelines behind Grindstone's figures, as a bibliography. \
    Each public page shows the abstract; reading a study in full may cost money, the reference itself never should. \
    [Veil, from the shelves: \"shhhhh! I'm reading...\"]";

pub fn topic_name(topic: Topic) -> &'static str {
    match topic {
        Topic::BodyComposition => "Body composition",
        Topic::HealthyRanges => "Healthy ranges",
        Topic::Energy => "Energy",
        Topic::Activity => "Activity",
        Topic::Macronutrients => "Macronutrients",
        Topic::Water => "Water",
    }
}
