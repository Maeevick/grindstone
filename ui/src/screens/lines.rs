use grind_core::bibliography::Topic;
use grind_core::burn::Burn;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, energy_apart, mass_apart, shown, typed_back};
use grind_core::food::{Nutrients, Unit};
use grind_core::for_copying_records::{Counted, Restored};
use grind_core::fuel::{Fuel, Verdict};
use grind_core::fuel_records::FuelTargets;
use grind_core::fuel_week::Habit;
use grind_core::fuel_week::{FuelWeek, Gap};
use grind_core::gender::Gender;
use grind_core::goals::Goals;
use grind_core::heading::Aim;
use grind_core::hydrate::Hydrate;
use grind_core::hydrate_week::HydrateWeek;
use grind_core::judgement::{AT_THE_AIM_SHARE, Judgement};
use grind_core::meal::Ingredient;
use grind_core::pace::{Lean, Pace, SafeLoss};
use grind_core::profile::Profile;
use grind_core::recommendations::Recommendations;
use grind_core::refusal::Refusal;
use grind_core::streak::{Status, Streak};
use grind_core::volitional::Volitional;

pub const WELCOME: &str = "Greetings, rusty one. From today I file your every gram. \
    Shall we begin with your profile, or restore a copy of your records first?";

pub const LOADING: &str = "Fetching the ledger…";

pub const LOST: &str = "Lost in the tunnels under the library. Try another tab.";

pub fn profile_kept(from: &str) -> String {
    format!("Filed. This profile is kept from {from}.")
}

pub fn goals_kept(from: &str) -> String {
    format!("Noted in the margin, in very small letters: these goals are kept from {from}.")
}

fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

pub fn added(name: &str) -> String {
    format!("{name} joins the catalog.")
}

pub const NO_WORKOUT: &str = "No workout on this day. A brisk half-hour walk will usually do: fitness culture \
    calls it active rest, the streak counts it as the day's burn.";

pub const NO_EXERCISE: &str = "The catalog is empty. Add an exercise and its MET \
    (the 2024 Compendium of Physical Activities lists them).";

/// The workouts' burn, and how far it stands from the day's target: what is left to reach it, or how far past.
pub fn burnt(burn: &Burn) -> String {
    let energy = |kcal| shown(Figure::Energy(kcal));
    match (burn.burnt_kcal, burn.target_kcal) {
        (Some(kcal), Some(target)) => format!(
            "{} burnt in workouts, {} {} the {} target.",
            energy(kcal),
            shown(energy_apart(kcal, target)),
            if burn.verdict == Some(Verdict::Low) {
                "to reach"
            } else {
                "past"
            },
            energy(target)
        ),
        (Some(kcal), None) => format!("{} burnt in workouts.", energy(kcal)),
        (None, _) => {
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

pub const NO_WEIGH_IN_TO_PRICE_A_WORKOUT: &str = "No weigh-in yet, so no kcal to price a workout on: \
    until then, a single workout holds the tick, a kcal will do.";

pub fn burn_target_kept(from: &str) -> String {
    format!("Filed: the day's burn target is kept from {from}.")
}

pub fn usual_day_kept(from: &str) -> String {
    format!("Filed: the usual day off the clock is kept from {from}.")
}

pub const DAY_OFF_THE_CLOCK_KEPT: &str = "Noted for this day only.";

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
            "Even the lab names its explosions. A name, please.".into()
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
            "Energy and macros of nothing or more. Negative protein is a lab experiment.".into()
        }
        Failure::Refused(Refusal::NotAQuantity) => "A quantity above nothing, please.".into(),
        Failure::Refused(Refusal::NotAFood) => {
            "That food is not in the catalog yet. Add it there first.".into()
        }
        Failure::Refused(Refusal::NotAnExercise) => {
            "That exercise is not in the catalog yet. I checked twice.".into()
        }
        Failure::Refused(Refusal::NotASingleStep) => {
            "A circuit needs one step at least. Even my experiments have one: the bang.".into()
        }
        Failure::Refused(Refusal::NotACircuit) => {
            "That circuit is not in the lab. Craft it first, at the bottom of Burn.".into()
        }
        Failure::Refused(Refusal::NotAVolume) => {
            "Some millilitres above nothing, please. An empty cup files as nothing at all.".into()
        }
        Failure::Refused(Refusal::NotADrink) => {
            "That drink is not in the catalog yet. I searched every shelf, twice.".into()
        }
        Failure::Refused(Refusal::NotACookingRatio) => {
            "A cooking ratio above nothing, please: 2.5 for pasta that drinks its water, 0.66 for cream \
             that boils down, 1 for what keeps its mass."
                .into()
        }
        Failure::Refused(Refusal::NotAnEnergy) => {
            "That is no amount of sweat I can file: kilocalories, from nothing up.".into()
        }
        Failure::Refused(Refusal::NotACopy) => {
            "That file is no copy of mine: I only restore what Copy the records wrote. Nothing changed.".into()
        }
        Failure::Refused(Refusal::NoCopyPushed) => {
            "No copy waits on the phone yet: plug it in and run make import-android \
             BACKUP=exports/<file>.json first."
                .into()
        }
        Failure::Refused(Refusal::AnotherSchema { copy, store }) => format!(
            "That copy holds version {copy} of the records, and this phone keeps version {store}. I restore \
             only a copy of the same version: nothing changed."
        ),
        Failure::Refused(Refusal::NotWeighed) => {
            "That ingredient is counted in pieces, and the lab cooks by mass: add it in grams or \
             millilitres for now."
                .into()
        }
        Failure::Broken(reason) => {
            format!("Something broke: {reason}. [A distant explosion.] The lab calls it data.")
        }
    }
}

pub const NO_BAND: &str = "No band yet: I need a profile and a weigh-in with a waist to price your day. \
    I do not guess bodies, not even goblin ones.";

pub const CHOOSE_FIRST: &str = "Choose from the list first: three letters narrow it down.";

pub const NO_MEAL: &str = "No meal cooked yet. The lab is spotless, which worries everyone.";

pub const NO_FOOD: &str = "The catalog has no food yet. Add one with its figures per portion.";

pub fn energy_line(fuel: &Fuel) -> String {
    let eaten = shown(Figure::Energy(fuel.eaten.energy_kcal));
    match fuel.band {
        Some(band) => format!(
            "{eaten} eaten, the band runs from {} to {}.",
            bare(Figure::Energy(band.floor_kcal)),
            shown(Figure::Energy(band.cap_kcal))
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
        "Filed: these targets are kept from {from}, and every band is recalculated from that day."
    )
}

pub fn figures_of(nutrients: &Nutrients) -> String {
    format!(
        "{} · P {} · F {} · Fib {} · C {}",
        shown(Figure::Energy(nutrients.energy_kcal)),
        shown(Figure::Grams(nutrients.protein_g)),
        shown(Figure::Grams(nutrients.fat_g)),
        shown(Figure::Grams(nutrients.fibre_g)),
        shown(Figure::Grams(nutrients.carbs_g)),
    )
}

pub const COOKING_RATIO_HOW: &str = "Ratio: the cooked mass over the raw one. 2.5 for pasta that drinks \
    its water, 0.66 for cream that boils down, 1 when left empty. No need to weigh the pot, it is hot.";

pub fn cooked_ingredient(ingredient: &Ingredient) -> String {
    format!(
        "{} {} {} → {}",
        shown(Figure::Amount(ingredient.quantity, ingredient.unit)),
        ingredient.food,
        shown(Figure::Times(ingredient.cooked_per_raw)),
        shown(Figure::Amount(ingredient.cooked_g(), Unit::Gram))
    )
}

pub const EATEN_HOW: &str =
    "Eaten now: left empty, the whole plate; 0 keeps it all for another day.";

pub fn kept_for_later(name: &str) -> String {
    format!("{name} waits in the food catalog, per 100 g cooked, for the day you eat it.")
}

fn days_counted(days: u32) -> String {
    match days {
        1 => "1 day".to_string(),
        many => format!("{many} days"),
    }
}

pub fn streak_count(days: u32) -> String {
    match days {
        0 => "No streak yet".to_string(),
        days => days_counted(days),
    }
}

pub fn longest_streak(longest: u32) -> String {
    format!("(longest: {})", days_counted(longest))
}

pub const KEEP_CIRCUIT_HOW: &str = "Once kept, a circuit is welded shut, like my last cauldron: to change \
    it, remove it and keep another. Sets: left empty, one; 0 keeps it for another day.";

pub fn circuit_kept_for_later(name: &str) -> String {
    format!("{name} waits in the lab, crafted, for the day you run it.")
}

pub const MEAL_ICON: &str = "💪";
pub const FOOD_ICON: &str = "🥦";
pub const CIRCUIT_ICON: &str = "🔄";
pub const EXERCISE_ICON: &str = "💪";
pub const WATER_ICON: &str = "💧";
pub const OTHER_DRINK_ICON: &str = "☕\u{fe0f}";

/// What is about to be burnt: the workout chosen, the session in progress, or the session with it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AboutToBurn {
    Workout,
    Session,
    SessionAndWorkout,
}

/// The day previewed with what is about to be burnt.
pub fn with_what_is_about_to_be_burnt(what: AboutToBurn, kcal: f64) -> String {
    let named = match what {
        AboutToBurn::Workout => "the workout",
        AboutToBurn::Session => "the session",
        AboutToBurn::SessionAndWorkout => "the session and the workout",
    };
    format!("With {named}: {}.", shown(Figure::Energy(kcal)))
}

/// The day previewed with the drink chosen.
pub fn with_the_drink(ml: f64) -> String {
    format!("With the drink: {}.", shown(Figure::Volume(ml)))
}

/// What is about to be eaten: the food chosen, the plate in progress, or both.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AboutToEat {
    Food,
    Plate,
    PlateAndFood,
}

/// The day previewed with what is about to be eaten, and what digesting it would spend.
pub fn with_what_is_about_to_be_eaten(
    what: AboutToEat,
    kcal: f64,
    digesting_kcal: Option<f64>,
) -> String {
    let (named, it) = match what {
        AboutToEat::Food => ("the food", "it"),
        AboutToEat::Plate => ("the plate", "it"),
        AboutToEat::PlateAndFood => ("the plate and the food", "them"),
    };
    let total = shown(Figure::Energy(kcal));
    match digesting_kcal {
        Some(digesting) => format!(
            "With {named}: {total}, {} of them spent digesting {it}.",
            bare(Figure::Energy(digesting))
        ),
        None => format!("With {named}: {total}."),
    }
}

pub fn streak_remark(streak: &Streak) -> &'static str {
    if streak.at_risk {
        "Yesterday was a joker: today must be complete, or the streak goes back to the archive."
    } else {
        "The streak is there to build habits, not to flatter."
    }
}

pub fn tick_mark(held: Option<bool>) -> &'static str {
    match held {
        Some(true) => "✓",
        Some(false) => "✗",
        None => "–",
    }
}

pub fn tick_said(name: &str, held: Option<bool>) -> String {
    match held {
        Some(true) => format!("{name} held"),
        Some(false) => format!("{name} missed"),
        None => format!("{name} left out"),
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

pub const NO_CIRCUIT: &str = "No circuit in the lab yet. Put a few exercises in a session, then \
    keep it as a circuit.";

pub fn a_set(kcal: f64) -> String {
    format!("{} a set", shown(Figure::Energy(kcal)))
}

pub fn circuit_logged(name: &str, sets: u32) -> String {
    format!(
        "Fire in the hole! {name}, {}, filed in the ledger.",
        counted(sets as usize, "set", "sets")
    )
}

/// A mass against its aim: one to reach tells its band, a floor how far the mass stands from it.
pub fn aim_line(label: &str, amount: f64, aim: Aim, recommended: bool) -> String {
    let mass = |kg| shown(Figure::Mass(kg));
    match aim {
        Aim::Reach(value) => format!(
            "{label}: {}, the band runs from {} to {}.",
            mass(amount),
            bare(Figure::Mass((1.0 - AT_THE_AIM_SHARE) * value)),
            mass((1.0 + AT_THE_AIM_SHARE) * value)
        ),
        Aim::Floor(value) => {
            let floor = if recommended {
                format!("the healthy {}", mass(value))
            } else {
                format!("the {} goal", mass(value))
            };
            let standing = if amount < value { "to reach" } else { "above" };
            format!(
                "{label}: {}, {} {standing} {floor}.",
                mass(amount),
                shown(mass_apart(amount, value))
            )
        }
    }
}

pub fn drunk_line(hydrate: &Hydrate) -> String {
    format!(
        "{} drunk, the band runs from {} to {}.",
        shown(Figure::Volume(hydrate.drunk_ml)),
        bare(Figure::Volume(hydrate.floor_ml)),
        shown(Figure::Volume(hydrate.cap_ml))
    )
}

pub const PAST_THE_CAP: &str = "Past the cap. Under normal activity, kidneys clear about a litre an \
    hour, so spread the cups through the day. They will not thank you aloud, but they will thank you.";

pub fn month_name(month: &str) -> String {
    chrono::NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
        .map(|first| first.format("%B %Y").to_string())
        .unwrap_or_else(|_| month.to_string())
}

pub const NOTHING_TO_CHART: &str = "Nothing filed in this window yet. The parchment is ready.";

pub const NOTHING_THIS_MONTH: &str = "Nothing filed this month yet.";

pub const NOTHING_THESE_DAYS: &str = "Nothing filed these days yet.";

pub fn load_the_rest_of(month: &str) -> String {
    format!("Load the rest of {}", month_name(month))
}

pub fn load_the_month(month: &str) -> String {
    format!("Load {}", month_name(month))
}

fn habit_name(habit: Habit) -> &'static str {
    match habit {
        Habit::Energy => "Energy",
        Habit::Protein => "Protein",
        Habit::Fat => "Fat",
        Habit::Fibre => "Fibre",
        Habit::Carbs => "Carbs",
    }
}

pub const PROFILE_HOW: &str = "Who the numbers belong to: the birthday, the gender, and in centimetres the height, \
    the neck just below the larynx and, for a woman, the hip at its widest. They price every band.";

pub const PROFILE_WHY: &str = "The US Navy formula reads the neck, the waist logged with each weigh-in and, for a \
    woman, the hip, to tell the lean mass from the fat.";

pub const GOALS_HOW: &str = "The body mass to reach, in kg, and its fat, in %. Type one alone and the other keeps \
    its recommendation, shown in grey; leave both empty to follow the recommendation as your body changes.";

pub const FUEL_TARGETS_HOW: &str = "Your targets, in grams a day per kilo of body mass. An empty one aims at its \
    health floor, today's shown in grey.";

pub const FUEL_TARGETS_WHY: &str = "Healthy, whatever they are, on the day's energy (what is eaten, never less than \
    the band's floor) and the band's cap: protein from 1.6 g per kg of body mass and 10% of the energy to 35% of the \
    cap, fat from 20% to 35%, fibre from 14 g per 1000 kcal, carbs from 130 g. A target beyond them is yours to set; \
    if you are not sure, ask a doctor or a coach first.";

pub const WATER_HOW: &str = "Your water a day, in mL, before the burn: the day's band adds what your workouts \
    sweat out. Leave it empty to follow the recommendation, shown in grey.";

pub const WATER_WHY: &str = "Recommended: once the tapes read a lean mass, nine tenths of the water it turns over \
    in a day, food bringing the rest; weighed without one, 35 mL per kilo of body mass; before a weigh-in, EFSA's \
    2.5 L a day for a man and 2 L for a woman, four fifths of it drunk.";

pub const BURN_TARGET_HOW: &str = "Net kilocalories a day your workouts aim to burn, rest set aside. Leave it \
    empty to follow the recommendation, shown in grey.";

pub const BURN_TARGET_WHY: &str =
    "Recommended: WHO's 150 minutes a week of moderate activity, priced on your body.";

pub const USUAL_DAY_HOW: &str = "Outside workouts: the minutes you usually sit and stand, the kilometres you \
    usually move. It prices the day's NEAT. Type some alone and the others keep their recommendation; leave all \
    empty to follow it.";

pub const USUAL_DAY_WHY: &str = "Recommended: sitting 8 hours at most, past which the least active die sooner \
    (Ekelund 2016); standing 2 hours (Buckley 2015); moving 2500 steps, the most of basal activity (Tudor-Locke \
    2011), at about 0.75 m a step: the workouts you log count the rest.";

pub const UNREADABLE: &str =
    "That is no number I can read. Digits, with a point or a comma, please.";

pub const COPY_HOW: &str = "Every record, written to grindstone-export.json in the app's Documents folder. \
    make export-android brings it to the Mac over USB.";

pub fn copied(copy: Counted) -> String {
    format!(
        "Copied {} from {}. Run make export-android now.",
        counted(copy.records, "record", "records"),
        counted(copy.tables, "table", "tables")
    )
}

pub const RESTORE_WARNING: &str = "Restoring replaces every record on this phone with the copy pushed by \
    make import-android BACKUP=exports/<file>.json. There is no undo: copy the records first.";

pub fn restored(copy: Restored) -> String {
    let brought_back = format!(
        "Restored {} from {}.",
        counted(copy.counted.records, "record", "records"),
        counted(copy.counted.tables, "table", "tables")
    );
    match copy.left_behind {
        None => brought_back,
        Some(path) => format!(
            "{brought_back} An error occurred: the copy could not be deleted. You can clean it up by hand \
             in {path}, as I do with the boxes in my cave."
        ),
    }
}

pub const ERASE_WARNING: &str = "Erasing removes every record on this device: measurements, profile, goals, \
    foods, meals, intakes, drinks, workouts, circuits and settings. There is no undo. The app starts again \
    as on its first launch, and a copy can be restored.";

pub const ERASED: &str = "Everything is erased. Home starts over with the welcome.";

pub const RECOMMENDATIONS_DISCLAIMER: &str = "Recommendations for health, made for an average body: not medical advice, \
    and not a prescription. Grindstone takes no responsibility for them. Check them, adjust them, and keep adjusting \
    as your logs show what your body does: logging, monitoring, adjusting, enjoying. No activity multiplier inflates \
    the energy band: it only counts the workouts you log.";

pub const THE_RECOMMENDATION: &str = "the recommendation";

pub const HELD: &str = "held";

pub const MISSED: &str = "missed";

/// What a card held before a choice was kept, recalled so a slip can be typed back.
pub fn before(held: &str) -> String {
    format!("Before: {held}.")
}

pub const NOTHING_HELD: &str = "nothing yet";

pub fn profile_held(profile: &Profile) -> String {
    let gender = match profile.gender {
        Gender::Male => "male",
        Gender::Female => "female",
    };
    let hip = profile
        .hip_cm
        .map(|cm| format!(", hip {}", shown(Figure::Length(cm))))
        .unwrap_or_default();
    format!(
        "born {}, {gender}, {} tall, neck {}{hip}",
        profile.birthday,
        shown(Figure::Length(profile.height_cm)),
        shown(Figure::Length(profile.neck_cm))
    )
}

pub fn goals_held(goals: &Goals) -> String {
    format!(
        "{} and {}",
        shown(Figure::Mass(goals.body_mass_kg)),
        shown(Figure::Ratio(goals.fat_ratio))
    )
}

pub fn fuel_targets_held(targets: &FuelTargets) -> String {
    let held: Vec<String> = [
        ("protein", targets.protein_g_per_kg),
        ("fat", targets.fat_g_per_kg),
        ("fibre", targets.fibre_g_per_kg),
        ("carbs", targets.carbs_g_per_kg),
    ]
    .into_iter()
    .filter_map(|(macro_name, target)| {
        target.map(|g_per_kg| format!("{macro_name} {} g/kg", typed_back(g_per_kg)))
    })
    .collect();
    if held.is_empty() {
        "no target".into()
    } else {
        held.join(", ")
    }
}

pub fn water_target_held(ml: &f64) -> String {
    shown(Figure::Volume(*ml))
}

pub fn burn_target_held(kcal: &f64) -> String {
    shown(Figure::Energy(*kcal))
}

pub fn usual_day_held(usual: &Volitional) -> String {
    format!(
        "{} sitting, {} standing, {} moving",
        shown(Figure::Duration(usual.sitting_min)),
        shown(Figure::Duration(usual.standing_min)),
        shown(Figure::Distance(usual.moving_km))
    )
}

pub fn recommendation_kept(from: &str) -> String {
    format!("Filed: the recommendation is followed from {from}, as your body changes.")
}

pub const NOTHING_TO_RECOMMEND_FOR: &str =
    "Filed, though I have nothing to recommend for yet: tell me about the rusty you first.";

pub fn water_target_kept(from: &str) -> String {
    format!("Filed: this water target is kept from {from}, each kcal burnt adding a millilitre.")
}

pub fn healthy_ranges(advised: &Recommendations) -> String {
    let (lightest, heaviest) = advised.healthy_mass_kg;
    let (leanest, fattest) = advised.healthy_fat_ratio;
    format!(
        "Healthy at your height and age: {} to {} (a body mass index of 18.5 to 24.9) and {} to {} body fat. \
        A goal beyond them is yours to set; if you are not sure, ask a doctor or a coach first. \
        I only have a quill, and it has never seen a patient.",
        shown(Figure::Mass(lightest)),
        shown(Figure::Mass(heaviest)),
        shown(Figure::Ratio(leanest)),
        shown(Figure::Ratio(fattest))
    )
}

pub const REFERENCES_INTRO: &str = "The studies and guidelines behind Grindstone's figures, as a bibliography. \
    Each public page shows the abstract; reading a study in full may cost money, the reference itself never should.";

pub const READING: &str = "“shhhhh! I’m reading...”";

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

pub const NO_WEEK_YET: &str = "No finished day of the window is logged yet. I only average what is over: it spares me the rubbing out.";

pub fn over_logged_days(days: usize) -> String {
    if days == 1 {
        "Average over 1 logged day, today left out.".into()
    } else {
        format!("Average over {days} logged days, today left out.")
    }
}

fn gaps_said(gaps: &[Gap], a_day_first: bool) -> String {
    let said: Vec<String> = gaps
        .iter()
        .enumerate()
        .map(|(at, gap)| {
            let figure = match gap.habit {
                Habit::Energy => Figure::Energy(gap.per_day),
                _ => Figure::Grams(gap.per_day),
            };
            format!(
                "{} ({}{}{})",
                habit_name(gap.habit).to_lowercase(),
                if gap.per_day > 0.0 { "+" } else { "" },
                shown(figure),
                if at == 0 && a_day_first { " a day" } else { "" }
            )
        })
        .collect();
    match said.as_slice() {
        [rest @ .., last] if !rest.is_empty() => format!("{} and {last}", rest.join(", ")),
        _ => said.concat(),
    }
}

/// What the window's average runs short of, then over, said plainly before a word from the ledger: the energy
/// against its band, each macro against its goal; without a goal, an invitation to paint one.
pub fn fuel_week_said(week: &FuelWeek) -> String {
    let (short, over) = (week.short(), week.over());
    match (short.is_empty(), over.is_empty()) {
        (true, true) if week.has_goals() => {
            "Every average sits on its goal. I checked twice, which for me counts as restraint."
                .to_string()
        }
        (true, true) => "The energy sits in its band.".to_string(),
        (false, true) => format!(
            "Short on {}. A little more every day and the gap closes on its own; I only have to watch, my favourite kind of work.",
            gaps_said(&short, true)
        ),
        (true, false) => format!(
            "Over on {}. A few swaps sand that down; no need to grind the whole stone.",
            gaps_said(&over, true)
        ),
        (false, false) => format!(
            "Short on {}, over on {}. Trade a little of the one for the other: the energy barely moves, the days do.",
            gaps_said(&short, true),
            gaps_said(&over, false)
        ),
    }
}

/// How far the window's drinks run from the band, then how much of them was water, said plainly before a word
/// from the ledger.
pub fn hydrate_week_said(week: &HydrateWeek) -> String {
    let gap = week.drunk.gap().unwrap_or_default();
    let drunk = match week.drunk.verdict() {
        Verdict::Low => format!(
            "Short of the band by {} a day. A glass more by mid-afternoon closes it; I measured, twice.",
            shown(Figure::Volume(-gap))
        ),
        Verdict::High => format!(
            "Past the band by {} a day. Kidneys clear about a litre an hour, so nothing grave; a glass less will do.",
            shown(Figure::Volume(gap))
        ),
        Verdict::InBand => {
            "A day's drinks sit in the band on average. Your kidneys say nothing, which from kidneys is high praise."
                .into()
        }
    };
    let water = match week.water.verdict() {
        Verdict::Low => "A little more water than the others would do.",
        _ => "Keep going: water is the best one.",
    };
    format!("{drunk} {water}")
}

pub const A_TIME_BENEATH: &str = "Totals, with a time's average beneath.";

/// What the window's burn stands against, after its figure: the sum of its days' targets when known.
pub fn over_active_days(active: usize, days: usize) -> String {
    if days == 1 {
        format!("Average over 1 day, {active} active, today left out.")
    } else {
        format!("Average over {days} days, {active} active, today left out.")
    }
}

pub const NO_PACE_YET: &str =
    "Too few weigh-ins for a pace yet. Patience is half the hunt. The other half is also patience.";
pub fn paced_over(weigh_ins: usize) -> String {
    if weigh_ins == 1 {
        "Pace: the slope of the 7-day average over 1 weigh-in.".into()
    } else {
        format!("Pace: the slope of the 7-day average over {weigh_ins} weigh-ins.")
    }
}

pub fn averaged_over(weigh_ins: usize) -> String {
    if weigh_ins == 1 {
        "Average over 1 weigh-in, today included.".into()
    } else {
        format!("Average over {weigh_ins} weigh-ins, today included.")
    }
}

pub const NO_WEIGH_IN_YET: &str =
    "No weigh-in in these 7 days yet. The scale is patient: it has nowhere else to be.";

/// Past the safe loss: said plainly first, figures to the hundredth, lean mass named only when the tape
/// sees it leave.
fn losing_too_fast(pace: &Pace) -> String {
    let moved = shown(Figure::WeeklyMass(pace.body_mass.by));
    let safe = shown(Figure::WeeklyMass(pace.safe_loss_kg));
    let lean = pace.lean();
    let past = match (pace.safe_loss_by, lean) {
        (SafeLoss::FatStore, Lean::Leaving(_)) => format!("past the {safe} your fat can spare"),
        (SafeLoss::FatStore, _) => format!("past the {safe} your fat can spare on paper"),
        (SafeLoss::MassShare, _) => format!("past the {safe} that 1% of your body mass allows"),
        (SafeLoss::AKilo, _) => format!("past the {safe} the guidelines allow"),
    };
    match lean {
        Lean::Leaving(by) => format!(
            "{moved} a week, {past}: your lean mass is going too ({} a week). Ease the deficit towards \
             the top of your band, keep protein high, keep lifting. Steel ground too hot loses its \
             temper; so do goblins, I have the burnt eyebrows to prove it.",
            shown(Figure::WeeklyMass(by))
        ),
        Lean::Holding => format!(
            "{moved} a week, {past}, though your lean mass holds so far. Ease the deficit a little and \
             watch the next weigh-ins. The paper is mine, so it is right about 97.3% of the time."
        ),
        Lean::Unmeasured => format!(
            "{moved} a week, {past}. Ease the deficit a little and watch the next weigh-ins. Without a \
             tape round the waist I cannot tell what is leaving, and I squint badly."
        ),
    }
}

pub fn pace_said(pace: &Pace) -> String {
    match pace.judgement {
        Judgement::Holding => {
            "Holding the line at the aim. Steady is the hardest pace there is: keep it.".into()
        }
        Judgement::VeryGood => format!(
            "{} a week, inside the safe lane. Keep the pace: the grindstone likes a steady hand.",
            shown(Figure::Mass(pace.body_mass.by))
        ),
        Judgement::Good => {
            "The right way, slower than the lane allows. Slow still sharpens.".into()
        }
        Judgement::Stable => {
            "Steady this week. Plateaus are where scales go to think; give it another week.".into()
        }
        Judgement::WrongWay => {
            "The week drifted from the aim. One week is a data point, not a verdict.".into()
        }
        Judgement::GainingTooFast => {
            "Gaining faster than muscle can follow: past half a percent a week, \
            the rest is stored. Trim the surplus a little."
                .into()
        }
        Judgement::LosingTooFast => losing_too_fast(pace),
    }
}

/// A tip, said between quotation marks.
pub fn quoted(line: &str) -> String {
    format!("“{line}”")
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use grind_core::fuel::Band;
    use grind_core::tendency::{Change, Heading};
    use grind_core::week::Averaged;

    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn burn(burnt_kcal: Option<f64>, target_kcal: Option<f64>, verdict: Option<Verdict>) -> Burn {
        Burn {
            day: day("2026-11-14"),
            workouts: vec![],
            burnt_kcal,
            off_the_clock: None,
            off_the_clock_is_usual: false,
            body_mass_kg: Some(72.0),
            target_kcal,
            verdict,
        }
    }

    fn averaged(per_day: f64, floor: f64, ceiling: Option<f64>, goal: Option<f64>) -> Averaged {
        Averaged {
            per_day,
            floor,
            ceiling,
            goal,
            in_band: 0,
        }
    }

    /// A week whose energy sits in its band and every macro on its goal, or on its floor without one.
    fn a_week_on_its_goals() -> FuelWeek {
        FuelWeek {
            logged_days: 6,
            energy: averaged(2000.0, 1800.0, Some(2400.0), Some(1900.0)),
            protein: averaged(130.0, 60.0, None, Some(130.0)),
            fat: averaged(70.0, 50.0, None, Some(70.0)),
            fibre: averaged(30.0, 25.0, None, None),
            carbs: averaged(250.0, 130.0, None, None),
        }
    }

    fn hydrate_week(drunk_ml: f64, water_percent: f64) -> HydrateWeek {
        HydrateWeek {
            logged_days: 6,
            drunk: averaged(drunk_ml, 2000.0, Some(3500.0), None),
            water: averaged(water_percent, 80.0, Some(100.0), None),
        }
    }

    fn pace(judgement: Judgement, by: f64) -> Pace {
        Pace {
            body_mass: Change {
                by,
                heading: Heading::Toward,
            },
            lean_mass: None,
            fat_mass: None,
            fat_ratio: None,
            judgement,
            safe_loss_kg: 0.8456,
            safe_loss_by: SafeLoss::FatStore,
            weigh_ins: 7,
        }
    }

    fn losing_too_fast_by(safe_loss_by: SafeLoss, lean_by: Option<f64>) -> Pace {
        Pace {
            lean_mass: lean_by.map(|by| Change {
                by,
                heading: if by < 0.0 {
                    Heading::Away
                } else {
                    Heading::Toward
                },
            }),
            safe_loss_by,
            ..pace(Judgement::LosingTooFast, -1.234)
        }
    }

    #[test]
    fn a_choice_kept_tells_the_day_it_is_kept_from() {
        let from = "2026-11-14";

        let said = [
            profile_kept(from),
            goals_kept(from),
            burn_target_kept(from),
            usual_day_kept(from),
            fuel_targets_kept(from),
            recommendation_kept(from),
            water_target_kept(from),
        ];

        assert_eq!(
            said,
            [
                "Filed. This profile is kept from 2026-11-14.",
                "Noted in the margin, in very small letters: these goals are kept from 2026-11-14.",
                "Filed: the day's burn target is kept from 2026-11-14.",
                "Filed: the usual day off the clock is kept from 2026-11-14.",
                "Filed: these targets are kept from 2026-11-14, and every band is recalculated from that day.",
                "Filed: the recommendation is followed from 2026-11-14, as your body changes.",
                "Filed: this water target is kept from 2026-11-14, each kcal burnt adding a millilitre.",
            ]
        );
    }

    #[test]
    fn what_joins_the_catalog_or_waits_for_its_day_is_said_by_its_name() {
        let said = [
            added("Skyr"),
            kept_for_later("Chili sin carne"),
            circuit_kept_for_later("Kettlebell trio"),
        ];

        assert_eq!(
            said,
            [
                "Skyr joins the catalog.",
                "Chili sin carne waits in the food catalog, per 100 g cooked, for the day you eat it.",
                "Kettlebell trio waits in the lab, crafted, for the day you run it.",
            ]
        );
    }

    #[test]
    fn workouts_short_of_the_burn_target_tell_what_is_left_to_reach_it() {
        let short = burn(Some(312.4), Some(400.0), Some(Verdict::Low));

        let said = burnt(&short);

        assert_eq!(
            said,
            "312 kcal burnt in workouts, 88 kcal to reach the 400 kcal target."
        );
    }

    #[test]
    fn workouts_at_or_past_the_burn_target_tell_how_far_past_it() {
        let reached = burn(Some(450.0), Some(400.0), Some(Verdict::InBand));
        let passed = burn(Some(612.0), Some(400.0), Some(Verdict::High));

        let said = [burnt(&reached), burnt(&passed)];

        assert_eq!(
            said,
            [
                "450 kcal burnt in workouts, 50 kcal past the 400 kcal target.",
                "612 kcal burnt in workouts, 212 kcal past the 400 kcal target.",
            ]
        );
    }

    #[test]
    fn workouts_without_a_burn_target_tell_their_burn_alone() {
        let untargeted = burn(Some(312.4), None, None);

        let said = burnt(&untargeted);

        assert_eq!(said, "312 kcal burnt in workouts.");
    }

    #[test]
    fn workouts_without_a_weigh_in_ask_for_one_first() {
        let unpriced = burn(None, Some(400.0), None);

        let said = burnt(&unpriced);

        assert_eq!(
            said,
            "Weigh in first: I price a workout on the body that did it, not on a stand-in."
        );
    }

    #[test]
    fn the_day_off_the_clock_says_whether_it_is_the_usual_one_or_the_days_own() {
        let usual = Burn {
            off_the_clock_is_usual: true,
            ..burn(None, None, None)
        };
        let own = burn(None, None, None);

        let said = [off_the_clock(&usual), off_the_clock(&own)];

        assert_eq!(
            said,
            [
                "The usual day off the clock, as set in Profile. Change it for this day only:",
                "This day's own day off the clock:",
            ]
        );
    }

    #[test]
    fn a_body_refused_is_said_by_what_it_lacks() {
        let refused = [
            Refusal::NotADate,
            Refusal::NotAMass,
            Refusal::NotAWaist,
            Refusal::NotAHeight,
            Refusal::NotANeck,
            Refusal::NotAHip,
            Refusal::NotARatio,
            Refusal::NotAProfile,
        ];

        let said = refused.map(|refusal| failure(&Failure::Refused(refusal)));

        assert_eq!(
            said,
            [
                "That date does not exist. I have checked every calendar in the library.",
                "A body of nothing at all? I file many things, but not ghosts.",
                "A waist above nothing, please.",
                "A height above nothing, please.",
                "A neck above nothing, please.",
                "Her Navy formula reads the hip: a hip above nothing, please.",
                "A fat target between 0 and 100%, the goblin way or any other.",
                "Recommendations need the rusty you first: birthday, gender and height.",
            ]
        );
    }

    #[test]
    fn a_workout_refused_is_said_by_what_it_lacks() {
        let refused = [
            Refusal::NotAName,
            Refusal::NotAMet,
            Refusal::NotADuration,
            Refusal::NotACount,
            Refusal::NotALoad,
            Refusal::NotADistance,
            Refusal::NotAnExercise,
            Refusal::NotASingleStep,
            Refusal::NotACircuit,
            Refusal::NotAnEnergy,
        ];

        let said = refused.map(|refusal| failure(&Failure::Refused(refusal)));

        assert_eq!(
            said,
            [
                "Even the lab names its explosions. A name, please.",
                "A MET above nothing: resting already counts as one.",
                "Some minutes above nothing, please.",
                "At least one set, rusty one.",
                "A load in kilos, a band counting below zero.",
                "Minutes and kilometres of nothing or more, and no more minutes than a day holds.",
                "That exercise is not in the catalog yet. I checked twice.",
                "A circuit needs one step at least. Even my experiments have one: the bang.",
                "That circuit is not in the lab. Craft it first, at the bottom of Burn.",
                "That is no amount of sweat I can file: kilocalories, from nothing up.",
            ]
        );
    }

    #[test]
    fn a_food_or_a_drink_refused_is_said_by_what_it_lacks() {
        let refused = [
            Refusal::NotAPortion,
            Refusal::NotANutrient,
            Refusal::NotAQuantity,
            Refusal::NotAFood,
            Refusal::NotAVolume,
            Refusal::NotADrink,
            Refusal::NotACookingRatio,
            Refusal::NotWeighed,
        ];

        let said = refused.map(|refusal| failure(&Failure::Refused(refusal)));

        assert_eq!(
            said,
            [
                "A portion above nothing: per 100 g, per 250 mL or per piece.",
                "Energy and macros of nothing or more. Negative protein is a lab experiment.",
                "A quantity above nothing, please.",
                "That food is not in the catalog yet. Add it there first.",
                "Some millilitres above nothing, please. An empty cup files as nothing at all.",
                "That drink is not in the catalog yet. I searched every shelf, twice.",
                "A cooking ratio above nothing, please: 2.5 for pasta that drinks its water, 0.66 for cream \
                 that boils down, 1 for what keeps its mass.",
                "That ingredient is counted in pieces, and the lab cooks by mass: add it in grams or \
                 millilitres for now.",
            ]
        );
    }

    #[test]
    fn a_copy_refused_says_why_nothing_changed() {
        let refused = [
            Refusal::NotACopy,
            Refusal::NoCopyPushed,
            Refusal::AnotherSchema { copy: 3, store: 5 },
        ];

        let said = refused.map(|refusal| failure(&Failure::Refused(refusal)));

        assert_eq!(
            said,
            [
                "That file is no copy of mine: I only restore what Copy the records wrote. Nothing changed.",
                "No copy waits on the phone yet: plug it in and run make import-android \
                 BACKUP=exports/<file>.json first.",
                "That copy holds version 3 of the records, and this phone keeps version 5. I restore \
                 only a copy of the same version: nothing changed.",
            ]
        );
    }

    #[test]
    fn something_broken_is_said_with_its_reason() {
        let broken = Failure::Broken("the disk is full".into());

        let said = failure(&broken);

        assert_eq!(
            said,
            "Something broke: the disk is full. [A distant explosion.] The lab calls it data."
        );
    }

    #[test]
    fn the_energy_eaten_is_said_against_its_band_when_one_is_priced() {
        let fuel = |band| Fuel {
            day: day("2026-11-14"),
            intakes: vec![],
            eaten: Nutrients {
                energy_kcal: 1834.6,
                ..Nutrients::default()
            },
            band,
            energy: None,
            macros: None,
            budget: None,
        };
        let band = Band {
            minimum_kcal_per_kg_of_lean_mass: 22.0,
            minimum_kcal: 1300.0,
            floor_kcal: 1800.0,
            cap_kcal: 2300.4,
            goal_kcal: 1950.0,
            maintenance_kcal: 2500.0,
        };

        let said = [energy_line(&fuel(Some(band))), energy_line(&fuel(None))];

        assert_eq!(
            said,
            [
                "1835 kcal eaten, the band runs from 1800 to 2300 kcal.",
                "1835 kcal eaten.",
            ]
        );
    }

    #[test]
    fn a_verdict_is_classed_low_good_or_high() {
        let verdicts = [Verdict::Low, Verdict::InBand, Verdict::High];

        let classes = verdicts.map(verdict_class);

        assert_eq!(classes, ["low", "good", "high"]);
    }

    #[test]
    fn a_portions_figures_read_energy_then_protein_fat_fibre_and_carbs() {
        let skyr = Nutrients {
            energy_kcal: 512.3,
            protein_g: 31.6,
            carbs_g: 54.4,
            fat_g: 18.2,
            fibre_g: 7.5,
        };

        let said = figures_of(&skyr);

        assert_eq!(said, "512 kcal · P 32 g · F 18 g · Fib 8 g · C 54 g");
    }

    #[test]
    fn a_cooked_ingredient_reads_raw_times_its_ratio_into_its_cooked_mass() {
        let penne = Ingredient {
            food: "Penne".into(),
            unit: Unit::Gram,
            quantity: 80.0,
            cooked_per_raw: 2.5,
            nutrients: Nutrients::default(),
        };

        let said = cooked_ingredient(&penne);

        assert_eq!(said, "80 g Penne × 2.5 → 200 g");
    }

    #[test]
    fn a_streak_counts_its_days_none_said_as_no_streak_yet() {
        let said = [
            streak_count(0),
            streak_count(1),
            streak_count(12),
            longest_streak(1),
            longest_streak(30),
        ];

        assert_eq!(
            said,
            [
                "No streak yet",
                "1 day",
                "12 days",
                "(longest: 1 day)",
                "(longest: 30 days)",
            ]
        );
    }

    #[test]
    fn a_streak_at_risk_warns_today_must_be_complete() {
        let streak = |at_risk| Streak {
            days: 12,
            longest: 30,
            at_risk,
            history: vec![],
        };

        let said = [streak_remark(&streak(true)), streak_remark(&streak(false))];

        assert_eq!(
            said,
            [
                "Yesterday was a joker: today must be complete, or the streak goes back to the archive.",
                "The streak is there to build habits, not to flatter.",
            ]
        );
    }

    #[test]
    fn a_tick_is_marked_and_said_held_missed_or_left_out() {
        let ticks = [Some(true), Some(false), None];

        let marked = ticks.map(tick_mark);
        let said = ticks.map(|held| tick_said("Weighed", held));

        assert_eq!(marked, ["✓", "✗", "–"]);
        assert_eq!(said, ["Weighed held", "Weighed missed", "Weighed left out"]);
    }

    #[test]
    fn a_streak_day_is_said_by_its_status() {
        let statuses = [
            Status::Success,
            Status::Joker,
            Status::Break,
            Status::Pending,
        ];

        let said = statuses.map(status_word);

        assert_eq!(said, ["every tick", "a joker", "a break", "under way"]);
    }

    #[test]
    fn the_day_previewed_names_what_is_about_to_be_burnt() {
        let about_to_burn = [
            AboutToBurn::Workout,
            AboutToBurn::Session,
            AboutToBurn::SessionAndWorkout,
        ];

        let said = about_to_burn.map(|what| with_what_is_about_to_be_burnt(what, 254.6));

        assert_eq!(
            said,
            [
                "With the workout: 255 kcal.",
                "With the session: 255 kcal.",
                "With the session and the workout: 255 kcal.",
            ]
        );
    }

    #[test]
    fn the_day_previewed_with_a_drink_tells_its_volume() {
        let said = with_the_drink(330.0);

        assert_eq!(said, "With the drink: 330 mL.");
    }

    #[test]
    fn the_day_previewed_names_what_is_about_to_be_eaten_and_what_digesting_it_spends() {
        let about_to_eat = [
            AboutToEat::Food,
            AboutToEat::Plate,
            AboutToEat::PlateAndFood,
        ];

        let said =
            about_to_eat.map(|what| with_what_is_about_to_be_eaten(what, 2140.0, Some(214.3)));

        assert_eq!(
            said,
            [
                "With the food: 2140 kcal, 214 of them spent digesting it.",
                "With the plate: 2140 kcal, 214 of them spent digesting it.",
                "With the plate and the food: 2140 kcal, 214 of them spent digesting them.",
            ]
        );
    }

    #[test]
    fn the_day_previewed_without_a_body_to_digest_tells_the_energy_alone() {
        let said = with_what_is_about_to_be_eaten(AboutToEat::Food, 2140.0, None);

        assert_eq!(said, "With the food: 2140 kcal.");
    }

    #[test]
    fn a_circuit_set_is_priced_in_kilocalories() {
        let said = a_set(37.6);

        assert_eq!(said, "38 kcal a set");
    }

    #[test]
    fn a_circuit_logged_counts_its_sets() {
        let said = [
            circuit_logged("Kettlebell trio", 1),
            circuit_logged("Kettlebell trio", 3),
        ];

        assert_eq!(
            said,
            [
                "Fire in the hole! Kettlebell trio, 1 set, filed in the ledger.",
                "Fire in the hole! Kettlebell trio, 3 sets, filed in the ledger.",
            ]
        );
    }

    #[test]
    fn a_mass_to_reach_tells_the_band_around_its_aim() {
        let said = aim_line("Body mass", 71.24, Aim::Reach(70.0), true);

        assert_eq!(
            said,
            "Body mass: 71.2 kg, the band runs from 69.3 to 70.7 kg."
        );
    }

    #[test]
    fn a_mass_under_its_floor_tells_how_far_it_is_to_reach() {
        let healthy = aim_line("Lean mass", 52.36, Aim::Floor(55.0), true);
        let goal = aim_line("Lean mass", 52.36, Aim::Floor(55.0), false);

        assert_eq!(
            [healthy, goal],
            [
                "Lean mass: 52.4 kg, 2.6 kg to reach the healthy 55.0 kg.",
                "Lean mass: 52.4 kg, 2.6 kg to reach the 55.0 kg goal.",
            ]
        );
    }

    #[test]
    fn a_mass_on_or_above_its_floor_tells_how_far_above_it_stands() {
        let on = aim_line("Lean mass", 55.0, Aim::Floor(55.0), false);
        let above = aim_line("Lean mass", 57.0, Aim::Floor(55.0), true);

        assert_eq!(
            [on, above],
            [
                "Lean mass: 55.0 kg, 0.0 kg above the 55.0 kg goal.",
                "Lean mass: 57.0 kg, 2.0 kg above the healthy 55.0 kg.",
            ]
        );
    }

    #[test]
    fn the_day_drunk_is_said_against_its_band() {
        let hydrate = Hydrate {
            day: day("2026-11-14"),
            drinkings: vec![],
            drunk_ml: 1840.4,
            target_ml: 2500.0,
            floor_ml: 2000.0,
            cap_ml: 3500.0,
            verdict: Verdict::Low,
        };

        let said = drunk_line(&hydrate);

        assert_eq!(said, "1840 mL drunk, the band runs from 2000 to 3500 mL.");
    }

    #[test]
    fn a_month_is_named_in_full_with_its_year_and_left_as_typed_when_unreadable() {
        let said = [
            month_name("2026-10"),
            month_name("Brumaire"),
            load_the_rest_of("2026-10"),
            load_the_month("2026-09"),
        ];

        assert_eq!(
            said,
            [
                "October 2026",
                "Brumaire",
                "Load the rest of October 2026",
                "Load September 2026",
            ]
        );
    }

    #[test]
    fn a_copy_counts_its_records_and_tables() {
        let many = Counted {
            tables: 9,
            records: 1234,
        };
        let one = Counted {
            tables: 1,
            records: 1,
        };

        let kept = |counted| Restored {
            counted,
            left_behind: None,
        };

        let said = [
            copied(many),
            copied(one),
            restored(kept(many)),
            restored(kept(one)),
        ];

        assert_eq!(
            said,
            [
                "Copied 1234 records from 9 tables. Run make export-android now.",
                "Copied 1 record from 1 table. Run make export-android now.",
                "Restored 1234 records from 9 tables.",
                "Restored 1 record from 1 table.",
            ]
        );
    }

    #[test]
    fn a_restored_copy_the_phone_kept_names_where_to_clean_it_up() {
        let left = Restored {
            counted: Counted {
                tables: 9,
                records: 1234,
            },
            left_behind: Some("/storage/emulated/0/Documents/grindstone-copy.json".into()),
        };

        let said = restored(left);

        assert_eq!(
            said,
            "Restored 1234 records from 9 tables. An error occurred: the copy could not be deleted. You can clean \
             it up by hand in /storage/emulated/0/Documents/grindstone-copy.json, as I do with the boxes in my cave."
        );
    }

    #[test]
    fn what_a_card_held_before_is_recalled() {
        let said = before("72.0 kg and 15.0%");

        assert_eq!(said, "Before: 72.0 kg and 15.0%.");
    }

    #[test]
    fn a_profile_held_reads_its_birthday_gender_and_tapes_the_hip_only_when_taken() {
        let his = Profile {
            birthday: day("1985-04-12"),
            gender: Gender::Male,
            height_cm: 178.0,
            neck_cm: 38.5,
            hip_cm: None,
        };
        let hers = Profile {
            birthday: day("1990-07-03"),
            gender: Gender::Female,
            height_cm: 165.0,
            neck_cm: 32.0,
            hip_cm: Some(98.2),
        };

        let said = [profile_held(&his), profile_held(&hers)];

        assert_eq!(
            said,
            [
                "born 1985-04-12, male, 178.0 cm tall, neck 38.5 cm",
                "born 1990-07-03, female, 165.0 cm tall, neck 32.0 cm, hip 98.2 cm",
            ]
        );
    }

    #[test]
    fn goals_held_read_the_mass_and_the_fat() {
        let goals = Goals {
            body_mass_kg: 72.0,
            fat_ratio: 0.15,
        };

        let said = goals_held(&goals);

        assert_eq!(said, "72.0 kg and 15.0%");
    }

    #[test]
    fn fuel_targets_held_read_only_the_macros_targeted() {
        let none = FuelTargets {
            protein_g_per_kg: None,
            fat_g_per_kg: None,
            fibre_g_per_kg: None,
            carbs_g_per_kg: None,
        };
        let some = FuelTargets {
            protein_g_per_kg: Some(1.6),
            fibre_g_per_kg: Some(0.35),
            ..none
        };
        let all = FuelTargets {
            protein_g_per_kg: Some(2.0),
            fat_g_per_kg: Some(0.8),
            fibre_g_per_kg: Some(0.4),
            carbs_g_per_kg: Some(3.0),
        };

        let said = [
            fuel_targets_held(&none),
            fuel_targets_held(&some),
            fuel_targets_held(&all),
        ];

        assert_eq!(
            said,
            [
                "no target",
                "protein 1.6 g/kg, fibre 0.35 g/kg",
                "protein 2 g/kg, fat 0.8 g/kg, fibre 0.4 g/kg, carbs 3 g/kg",
            ]
        );
    }

    #[test]
    fn a_water_or_burn_target_held_reads_its_figure() {
        let said = [water_target_held(&2500.0), burn_target_held(&350.4)];

        assert_eq!(said, ["2500 mL", "350 kcal"]);
    }

    #[test]
    fn a_usual_day_held_reads_sitting_standing_and_moving() {
        let usual = Volitional {
            sitting_min: 480.0,
            standing_min: 120.0,
            moving_km: 6.0,
        };

        let said = usual_day_held(&usual);

        assert_eq!(said, "480 min sitting, 120 min standing, 6 km moving");
    }

    #[test]
    fn the_healthy_ranges_read_the_mass_and_the_fat_at_the_height_and_age() {
        let advised = Recommendations {
            healthy_mass_kg: (58.6, 78.9),
            goal_mass_kg: 70.0,
            healthy_fat_ratio: (0.08, 0.19),
            goal_fat_ratio: 0.15,
            usual_day: Volitional {
                sitting_min: 480.0,
                standing_min: 120.0,
                moving_km: 6.0,
            },
            burn_target_kcal: None,
            fuel_floors_g_per_kg: FuelTargets {
                protein_g_per_kg: None,
                fat_g_per_kg: None,
                fibre_g_per_kg: None,
                carbs_g_per_kg: None,
            },
            water_ml: None,
        };

        let said = healthy_ranges(&advised);

        assert_eq!(
            said,
            "Healthy at your height and age: 58.6 kg to 78.9 kg (a body mass index of 18.5 to 24.9) and 8.0% \
             to 19.0% body fat. A goal beyond them is yours to set; if you are not sure, ask a doctor or a coach \
             first. I only have a quill, and it has never seen a patient."
        );
    }

    #[test]
    fn a_reference_topic_is_named() {
        let topics = [
            Topic::BodyComposition,
            Topic::HealthyRanges,
            Topic::Energy,
            Topic::Activity,
            Topic::Macronutrients,
            Topic::Water,
        ];

        let named = topics.map(topic_name);

        assert_eq!(
            named,
            [
                "Body composition",
                "Healthy ranges",
                "Energy",
                "Activity",
                "Macronutrients",
                "Water",
            ]
        );
    }

    #[test]
    fn a_windows_average_tells_the_days_it_reads_today_left_out() {
        let said = [
            over_logged_days(1),
            over_logged_days(5),
            over_active_days(1, 1),
            over_active_days(3, 6),
        ];

        assert_eq!(
            said,
            [
                "Average over 1 logged day, today left out.",
                "Average over 5 logged days, today left out.",
                "Average over 1 day, 1 active, today left out.",
                "Average over 6 days, 3 active, today left out.",
            ]
        );
    }

    #[test]
    fn the_pace_and_the_weeks_average_tell_the_weigh_ins_they_read() {
        let said = [
            paced_over(1),
            paced_over(5),
            averaged_over(1),
            averaged_over(4),
        ];

        assert_eq!(
            said,
            [
                "Pace: the slope of the 7-day average over 1 weigh-in.",
                "Pace: the slope of the 7-day average over 5 weigh-ins.",
                "Average over 1 weigh-in, today included.",
                "Average over 4 weigh-ins, today included.",
            ]
        );
    }

    #[test]
    fn a_week_on_its_goals_says_so_and_one_without_goals_says_its_energy_sits_in_its_band() {
        let on_its_goals = a_week_on_its_goals();
        let without_goals = FuelWeek {
            protein: averaged(130.0, 60.0, None, None),
            fat: averaged(70.0, 50.0, None, None),
            ..on_its_goals
        };

        let said = [
            fuel_week_said(&on_its_goals),
            fuel_week_said(&without_goals),
        ];

        assert_eq!(
            said,
            [
                "Every average sits on its goal. I checked twice, which for me counts as restraint.",
                "The energy sits in its band.",
            ]
        );
    }

    #[test]
    fn a_week_short_of_its_goals_says_each_gap_furthest_first_the_first_a_day() {
        let short = FuelWeek {
            energy: averaged(1650.0, 1800.0, Some(2400.0), Some(1900.0)),
            protein: averaged(90.0, 60.0, None, Some(130.0)),
            fibre: averaged(20.0, 30.0, None, None),
            ..a_week_on_its_goals()
        };

        let said = fuel_week_said(&short);

        assert_eq!(
            said,
            "Short on fibre (-10 g a day), protein (-40 g) and energy (-150 kcal). A little more every day and \
             the gap closes on its own; I only have to watch, my favourite kind of work."
        );
    }

    #[test]
    fn a_week_over_its_goals_says_each_gap_with_its_sign() {
        let over = FuelWeek {
            energy: averaged(2600.0, 1800.0, Some(2400.0), Some(1900.0)),
            fat: averaged(95.0, 50.0, None, Some(70.0)),
            ..a_week_on_its_goals()
        };

        let said = fuel_week_said(&over);

        assert_eq!(
            said,
            "Over on fat (+25 g a day) and energy (+200 kcal). A few swaps sand that down; no need to grind the \
             whole stone."
        );
    }

    #[test]
    fn a_week_short_on_one_and_over_on_another_says_both_and_offers_a_trade() {
        let both = FuelWeek {
            protein: averaged(90.0, 60.0, None, Some(130.0)),
            carbs: averaged(310.0, 130.0, None, Some(250.0)),
            ..a_week_on_its_goals()
        };

        let said = fuel_week_said(&both);

        assert_eq!(
            said,
            "Short on protein (-40 g a day), over on carbs (+60 g). Trade a little of the one for the other: \
             the energy barely moves, the days do."
        );
    }

    #[test]
    fn a_week_short_of_drinks_tells_the_gap_a_day_and_asks_for_more_water() {
        let short = hydrate_week(1650.0, 60.0);

        let said = hydrate_week_said(&short);

        assert_eq!(
            said,
            "Short of the band by 350 mL a day. A glass more by mid-afternoon closes it; I measured, twice. \
             A little more water than the others would do."
        );
    }

    #[test]
    fn a_week_past_its_drinks_band_tells_the_gap_a_day() {
        let past = hydrate_week(3900.0, 92.0);

        let said = hydrate_week_said(&past);

        assert_eq!(
            said,
            "Past the band by 400 mL a day. Kidneys clear about a litre an hour, so nothing grave; a glass less \
             will do. Keep going: water is the best one."
        );
    }

    #[test]
    fn a_week_of_drinks_in_its_band_is_praised_by_the_kidneys() {
        let in_band = hydrate_week(2600.0, 100.0);

        let said = hydrate_week_said(&in_band);

        assert_eq!(
            said,
            "A day's drinks sit in the band on average. Your kidneys say nothing, which from kidneys is high \
             praise. Keep going: water is the best one."
        );
    }

    #[test]
    fn a_pace_in_or_short_of_the_safe_lane_is_said_plainly() {
        let judged = [
            pace(Judgement::Holding, 0.02),
            pace(Judgement::VeryGood, -0.54),
            pace(Judgement::Good, -0.21),
            pace(Judgement::Stable, 0.03),
            pace(Judgement::WrongWay, 0.31),
            pace(Judgement::GainingTooFast, 0.72),
        ];

        let said = judged.map(|judged| pace_said(&judged));

        assert_eq!(
            said,
            [
                "Holding the line at the aim. Steady is the hardest pace there is: keep it.",
                "-0.5 kg a week, inside the safe lane. Keep the pace: the grindstone likes a steady hand.",
                "The right way, slower than the lane allows. Slow still sharpens.",
                "Steady this week. Plateaus are where scales go to think; give it another week.",
                "The week drifted from the aim. One week is a data point, not a verdict.",
                "Gaining faster than muscle can follow: past half a percent a week, the rest is stored. Trim \
                 the surplus a little.",
            ]
        );
    }

    #[test]
    fn a_pace_losing_lean_mass_past_what_the_fat_can_spare_is_said_with_both_figures_and_its_limit()
    {
        let leaving = losing_too_fast_by(SafeLoss::FatStore, Some(-0.312));

        let said = pace_said(&leaving);

        assert_eq!(
            said,
            "-1.23 kg a week, past the 0.85 kg your fat can spare: your lean mass is going too (-0.31 kg a \
             week). Ease the deficit towards the top of your band, keep protein high, keep lifting. Steel \
             ground too hot loses its temper; so do goblins, I have the burnt eyebrows to prove it."
        );
    }

    #[test]
    fn a_pace_losing_too_fast_while_the_lean_mass_holds_is_past_the_fat_store_on_paper() {
        let holding = losing_too_fast_by(SafeLoss::FatStore, Some(0.05));

        let said = pace_said(&holding);

        assert_eq!(
            said,
            "-1.23 kg a week, past the 0.85 kg your fat can spare on paper, though your lean mass holds so \
             far. Ease the deficit a little and watch the next weigh-ins. The paper is mine, so it is right \
             about 97.3% of the time."
        );
    }

    #[test]
    fn a_pace_losing_too_fast_without_a_tape_names_the_limit_it_passes() {
        let by_the_mass_share = losing_too_fast_by(SafeLoss::MassShare, None);
        let by_a_kilo = Pace {
            safe_loss_kg: 1.0,
            ..losing_too_fast_by(SafeLoss::AKilo, None)
        };

        let said = [pace_said(&by_the_mass_share), pace_said(&by_a_kilo)];

        assert_eq!(
            said,
            [
                "-1.23 kg a week, past the 0.85 kg that 1% of your body mass allows. Ease the deficit a little \
                 and watch the next weigh-ins. Without a tape round the waist I cannot tell what is leaving, \
                 and I squint badly.",
                "-1.23 kg a week, past the 1.00 kg the guidelines allow. Ease the deficit a little and watch \
                 the next weigh-ins. Without a tape round the waist I cannot tell what is leaving, and I \
                 squint badly.",
            ]
        );
    }

    #[test]
    fn a_tip_is_said_between_quotation_marks() {
        let said = quoted("Patience is half the hunt.");

        assert_eq!(said, "“Patience is half the hunt.”");
    }
}
