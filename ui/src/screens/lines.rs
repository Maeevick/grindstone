use grind_core::bibliography::Topic;
use grind_core::burn::Burn;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, shown, signed};
use grind_core::food::{Nutrients, Unit};
use grind_core::for_importing::Imported;
use grind_core::fuel::{Fuel, Verdict};
use grind_core::hydrate::Hydrate;
use grind_core::meal::Ingredient;
use grind_core::pace::{Judgement, Pace};
use grind_core::recommendations::Recommendations;
use grind_core::refusal::Refusal;
use grind_core::sharpening::{BandNote, BurnNote, GatherNote, Habit, Leaning};
use grind_core::stats::{Measure, Summary, figure_of};
use grind_core::streak::{Status, Streak};
use grind_draft::{DraftRefusal, ImportFailure};

pub const WELCOME: &str = "Greetings, rusty one. From today I file your every gram. \
    Shall we begin with your profile, or fetch the old records from the draft first?";

pub const LOADING: &str = "Fetching the ledger…";

pub const LOST: &str = "Lost in the tunnels under the library. Try another tab.";

pub fn profile_kept(from: &str) -> String {
    format!("Filed. This profile is kept from {from}.")
}

pub fn goals_kept(from: &str) -> String {
    format!("Noted in the margin, in very small letters: these goals are kept from {from}.")
}

pub const IMPORT_HOW: &str = "Bring the draft's latest backup from the Mac with make import-draft-android, \
    the phone plugged in, then import it here. It replaces every record the draft had. \
    Run it again with a newer backup until you switch for good.";

fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

pub fn imported(imported: &Imported) -> String {
    format!(
        "Filed: {}, {}, {}, {}, {}, {}, {}, {} and {}.",
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
    format!("{name} joins the catalog.")
}

pub const NO_WORKOUT: &str = "No workout on this day. A brisk half-hour walk will usually do: fitness culture \
    calls it active rest, the streak counts it as the day's burn.";

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

pub const BURN_TARGET_HINT: &str = "Net kilocalories a day your workouts aim to burn, rest set aside. \
    Until you keep your own, WHO's 150 minutes a week of moderate activity, priced on your body.";

pub const NO_WEIGH_IN_TO_PRICE_A_WORKOUT: &str = "No weigh-in yet, so no kcal to price a workout on: \
    until then, a single workout holds the tick, a kcal will do.";

pub const NO_TARGET_TYPED: &str = "A target in kilocalories, please, or 0 for any workout at all.";

pub fn burn_target_kept(from: &str) -> String {
    format!("Filed: the day's burn target is kept from {from}.")
}

pub fn usual_day_kept(from: &str) -> String {
    format!("Filed: the usual day off the clock is kept from {from}.")
}

pub const DAY_OFF_THE_CLOCK_KEPT: &str = "Noted for this day only.";

pub fn import_failure(refused: &ImportFailure) -> String {
    match refused {
        ImportFailure::NoBackupFound(where_expected) => format!(
            "No backup in the tunnels. Bring one with make import-draft-android first; I looked in {where_expected}."
        ),
        ImportFailure::Draft(DraftRefusal::Unreadable(where_it_stopped)) => {
            format!(
                "This backup reads like lab notes: it stops making sense at {where_it_stopped}."
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
            "That circuit is not in the lab. Craft it in Catalog first.".into()
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

pub const NO_INTAKE: &str = "Nothing eaten on this day, or nothing filed yet.";

pub const NO_MEAL: &str = "No meal cooked yet. The lab is spotless, which worries everyone.";

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

pub fn cooked(name: &str) -> String {
    format!("{name} is in the food catalog, per 100 g cooked. Weigh your portions in Fuel.")
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

pub const STREAK_GRACE: &str = "Each tick holds within 5% of its band, a mercy the Tracker's gauges \
    never show. Aim for the gauges: the 5% is for the days that wobble.";

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
            "Correlation {r:.2} over {shared} shared days, and a correlation is not a cause."
        ),
        None => {
            format!("{shared} shared days: hunting a correlation takes at least three that move.")
        }
    }
}

pub const HUNTING: &str = "“shhhhh! I’m hunting...”";

pub const NOTHING_TO_EXPLORE: &str = "Nothing logged for this measure in the window.";

pub const CIRCUIT_HOW: &str = "Name it, stack its steps one by one, then craft. Once crafted it is \
    welded shut, like my last cauldron: to change it, remove it and craft another.";

pub const NO_CIRCUIT: &str = "No circuit in the lab yet. Stack a few exercises from the catalog \
    and craft one.";

pub const NO_CIRCUIT_TO_RUN: &str = "No circuit to run yet: the lab is in Catalog.";

pub fn a_set(kcal: f64) -> String {
    format!("{} a set", shown(Figure::Energy(kcal)))
}

pub fn circuit_crafted(name: &str) -> String {
    format!("{name} is crafted, its steps welded in place.")
}

pub fn circuit_logged(name: &str, sets: u32) -> String {
    format!(
        "Fire in the hole! {name}, {}, filed in the ledger.",
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

pub const TARGET_DRUNK: &str = "Target reached.";

pub const PAST_THE_CAP: &str = "Past the cap. Under normal activity, kidneys clear about a litre an \
    hour, so spread the cups through the day. They will not thank you aloud, but they will thank you.";

pub const NO_DRINKING: &str =
    "Not a drop filed on this day yet. The archive is drier than its oldest scroll.";

pub const HYDRATE_COUNTED_HINT: &str = "The band follows your lean mass and the day's burn, food \
    bringing a tenth of the water. Leave Hydrate out of the streak from a day your drinks went unfiled.";

pub fn hydrate_counted_from(from: &str) -> String {
    format!("Filed. Hydrate counts in the streak from {from}.")
}

pub fn hydrate_left_out_from(from: &str) -> String {
    format!("Filed. Hydrate stays out of the streak from {from}.")
}

pub fn month_name(month: &str) -> String {
    chrono::NaiveDate::parse_from_str(&format!("{month}-01"), "%Y-%m-%d")
        .map(|first| first.format("%B %Y").to_string())
        .unwrap_or_else(|_| month.to_string())
}

pub const NOTHING_TO_CHART: &str = "Nothing filed in this window yet. The parchment is ready.";

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
            "No weigh-in in this window. A hunt tracks footprints, and the scale has left none."
                .into()
        }
        GatherNote::Irregular { weighed_days, days } => format!(
            "Weighed on {weighed_days} of {days} days. A trend wants a footprint every other day at least; \
             fewer, and it is a rumour with a decimal point."
        ),
        GatherNote::TooSoon => "Too few weigh-ins for a trend yet. Patience is half the hunt. \
            The other half is also patience."
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
            "{}: on this trend the goal falls around {reached_on}. Keep the pace, not the pressure.",
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
            "Nothing eaten on the record in this window, and nobody hunts what was never filed."
                .into()
        }
        BandNote::Sharp {
            weakest,
            in_band,
            logged,
        } => format!(
            "Every habit held its band four days in five; even the weakest, {}, on {in_band} of {logged}. \
             Nothing to hunt, which is deeply suspicious.",
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
        } => format!("In the band on {in_band} of {logged} days."),
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
        BurnNote::Idle => {
            "No workout in this window. The joker forgives one idle day, not a window full of them; \
            one walk gets the grindstone turning again."
                .into()
        }
        BurnNote::Sparse { per_week } => format!(
            "{} active days a week. The WHO asks for strength on two; the grindstone turns either way, \
             but it turns better with a hand on it.",
            shown(Figure::Count(*per_week))
        ),
        BurnNote::Steady {
            per_week,
            burnt_per_active_day,
        } => format!(
            "{} active days a week{}. Steady as a goblin with a grudge.",
            shown(Figure::Count(*per_week)),
            burnt_per_active_day
                .map(|kcal| format!(", {} burnt on each", shown(Figure::Energy(kcal))))
                .unwrap_or_default()
        ),
    }
}

pub const UNREADABLE: &str =
    "That is no number I can read. Digits, with a point or a comma, please.";

pub const ERASE_WARNING: &str = "Erasing removes every record on this device: measurements, profile, goals, \
    foods, meals, intakes, drinks, workouts, circuits and settings. There is no undo. The app starts again \
    as on its first launch, and the draft can be imported again.";

pub const ERASED: &str = "Everything is erased. Home starts over with the welcome.";

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

pub const NO_PACE_YET: &str =
    "Too few weigh-ins for a pace yet. Patience is half the hunt. The other half is also patience.";

pub fn pace_said(pace: &Pace) -> String {
    let lost = shown(Figure::Mass(-pace.body_mass.by));
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
        Judgement::LosingTooFast => format!(
            "{lost} lost a week, more than your fat can spare ({}). Eat back up to the band's floor, keep \
             protein high, keep lifting: the rest is lean mass leaving.",
            shown(Figure::Mass(pace.safe_loss_kg))
        ),
        Judgement::UnderHealthyMass => {
            "Under the healthy mass for your height. Stop losing, eat to \
            maintenance, and see a doctor if it keeps dropping."
                .into()
        }
        Judgement::UnderHealthyFat => {
            "Under the healthy body fat for your age. Stop losing, eat to \
            maintenance, and see a doctor if it keeps dropping."
                .into()
        }
    }
}

pub fn pace_class(judgement: Judgement) -> &'static str {
    match judgement {
        Judgement::Holding | Judgement::VeryGood => "said good",
        Judgement::LosingTooFast | Judgement::UnderHealthyMass | Judgement::UnderHealthyFat => {
            "bad"
        }
        _ => "said",
    }
}
