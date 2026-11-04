use grind_core::bibliography::Topic;
use grind_core::burn::Burn;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, energy_apart, mass_apart, shown, typed_back};
use grind_core::food::{Nutrients, Unit};
use grind_core::for_copying_records::Counted;
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
    the band's floor) and the band's cap: protein from 0.83 g per kg of body mass and 10% of the energy to 35% of the \
    cap, fat from 20% to 35%, fibre from 10 g per 1000 kcal, carbs from 130 g. A target beyond them is yours to set; \
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
    (Ekelund 2016); standing 2 hours (Buckley 2015); moving the steps past which the risk of dying stops falling, \
    8000 a day under 60 and 6000 from 60 (Paluch 2022), at about 0.75 m a step.";

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

pub fn restored(copy: Counted) -> String {
    format!(
        "Restored {} from {}.",
        counted(copy.records, "record", "records"),
        counted(copy.tables, "table", "tables")
    )
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
        "Over 1 logged day.".into()
    } else {
        format!("Over {days} logged days.")
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
    let said = match (short.is_empty(), over.is_empty()) {
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
    };
    if week.has_goals() {
        said
    } else {
        format!("{said} {PAINT_YOUR_TARGETS}")
    }
}

const PAINT_YOUR_TARGETS: &str =
    "Paint your targets in Profile and I will tell how far each macro stands from them.";

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
    if active == 1 {
        format!("Over 1 active day in {days}.")
    } else {
        format!("Over {active} active days in {days}.")
    }
}

pub const NO_PACE_YET: &str =
    "Too few weigh-ins for a pace yet. Patience is half the hunt. The other half is also patience.";

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
