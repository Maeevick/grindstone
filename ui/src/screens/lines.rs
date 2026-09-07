use grind_core::bibliography::Topic;
use grind_core::burn::Burn;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, shown};
use grind_core::food::{Nutrients, Unit};
use grind_core::for_charting::Insights;
use grind_core::for_importing::Imported;
use grind_core::for_reviewing_progress::GatherCard;
use grind_core::fuel::{Fuel, Verdict};
use grind_core::hydrate::Hydrate;
use grind_core::meal::Meal;
use grind_core::recommendations::{Recommendations, Rhythm};
use grind_core::refusal::Refusal;
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
        Failure::Refused(Refusal::NotAServing) => {
            "Servings above nothing, please: half a cup counts, no cup does not.".into()
        }
        Failure::Refused(Refusal::NotADrink) => {
            "That drink is not in the catalog yet. Grimble searched every shelf, twice.".into()
        }
        Failure::Broken(reason) => {
            format!("Something broke: {reason}. [A distant explosion.] Trix says it is data.")
        }
    }
}

pub const NO_BAND: &str = "No band yet: I need a profile and a weigh-in with a waist to price your day. \
    I do not guess bodies, not even goblin ones.";

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

pub fn verdict_word(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Low => "too low",
        Verdict::InBand => "in the band",
        Verdict::High => "too high",
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

pub fn weighed_up(meal: &Meal, per_100_g: &Nutrients) -> [String; 2] {
    [
        format!(
            "Before cooking, {} raw: {}.",
            shown(Figure::Amount(meal.raw_g, Unit::Gram)),
            figures_of(&meal.totals)
        ),
        format!(
            "After cooking, {} in the pot: {} per 100 g. [Trix: \"Fire in the hole!\"]",
            shown(Figure::Amount(meal.cooked_g, Unit::Gram)),
            figures_of(per_100_g)
        ),
    ]
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

fn share_of(part: usize, whole: usize) -> f64 {
    part as f64 / whole as f64
}

pub fn habits_to_sharpen(insights: &Insights) -> Vec<String> {
    let mut banded: Vec<(f64, String)> = [
        ("Energy", insights.energy, insights.fuelled_days),
        ("Protein", insights.protein, insights.fuelled_days),
        ("Fat", insights.fat, insights.fuelled_days),
        ("Fibre", insights.fibre, insights.fuelled_days),
        ("Carbs", insights.carbs, insights.fuelled_days),
        ("Hydrate", insights.hydrate, insights.hydrated_days),
    ]
    .into_iter()
    .filter(|(_, _, logged)| *logged > 0)
    .map(|(habit, tally, logged)| {
        let share = share_of(tally.in_band, logged);
        (
            share,
            format!(
                "{habit}: in its band on {} of {logged} logged days ({}), under it on {}, over it on {}.",
                tally.in_band,
                shown(Figure::Ratio(share)),
                tally.low,
                tally.high
            ),
        )
    })
    .collect();
    banded.sort_by(|(one, _), (other, _)| one.total_cmp(other));
    let mut lines: Vec<String> = banded.into_iter().map(|(_, line)| line).collect();
    if insights.days > 0 {
        lines.push(format!(
            "Workouts on {} of {} days, {} a week{}.",
            insights.active_days,
            insights.days,
            shown(Figure::Count(
                share_of(insights.active_days, insights.days) * 7.0
            )),
            insights
                .burnt_per_active_day
                .map(|kcal| format!(", {} burnt on an active day", shown(Figure::Energy(kcal))))
                .unwrap_or_default()
        ));
        lines.push(format!(
            "Weighed on {} of {} days.",
            insights.weighed_days, insights.days
        ));
    }
    lines
}

pub const SHARPEN: &str =
    "[Veil, without looking up:] the weakest habits first. A habit, not a verdict.";

pub const UNREADABLE: &str = "That is no number I can read. Digits, with a point or a comma, please. [Grimble squints at the ink.]";

pub const ERASE_WARNING: &str = "Erasing removes every record on this device: measurements, profile, goals, \
    foods, meals, intakes, drinks, workouts, circuits and settings. There is no undo. The app starts again \
    as on its first launch, and the draft can be imported again.";

pub const ERASED: &str = "Everything is erased. [Grimble closes an empty ledger and opens a fresh one.] Home starts over with the welcome.";

pub fn rhythm_name(rhythm: Rhythm) -> &'static str {
    match rhythm {
        Rhythm::Sedentary => "Sedentary",
        Rhythm::LightlyActive => "Lightly active",
        Rhythm::ModeratelyActive => "Moderately active",
        Rhythm::VeryActive => "Very active",
        Rhythm::ExtremelyActive => "Extremely active",
    }
}

pub fn rhythm_described(rhythm: Rhythm) -> &'static str {
    match rhythm {
        Rhythm::Sedentary => {
            "Desk or car most of the day, little walking, no regular training. Fewer than 5,000 steps a day."
        }
        Rhythm::LightlyActive => {
            "Desk work with errands or a short walk, light training one to three days a week. 5,000 to 7,500 steps a day."
        }
        Rhythm::ModeratelyActive => {
            "On your feet part of the day (teaching, a shop floor) or a daily brisk walk, training three to five days a week. 7,500 to 10,000 steps a day."
        }
        Rhythm::VeryActive => {
            "A job on your feet (care, service, a warehouse) or long walks, hard training six or seven days a week. 10,000 to 12,500 steps a day."
        }
        Rhythm::ExtremelyActive => {
            "Physical work (building, farming, delivering on foot) or training twice a day. More than 12,500 steps a day."
        }
    }
}

pub const RECOMMENDATIONS_DISCLAIMER: &str = "Recommendations for health, made for an average body: not medical advice, \
    and not a prescription. Grindstone takes no responsibility for them. Check them, adjust them, and keep adjusting \
    as your logs show what your body does: logging, monitoring, adjusting, enjoying. No activity multiplier inflates \
    the energy band: it only counts the workouts you log.";

pub const RECOMMENDATION_FILLED: &str =
    "Filled from the recommendation for your rhythm. Check it, adjust it, then keep it.";

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
