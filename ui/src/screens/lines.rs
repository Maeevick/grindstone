use grind_core::failure::Failure;
use grind_core::figures::{Figure, shown};
use grind_core::for_importing::Imported;
use grind_core::for_reviewing_progress::GatherCard;
use grind_core::refusal::Refusal;
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

pub const IMPORT_HOW: &str = "Push the draft's latest backup from the Mac with make import-draft, \
    the phone plugged in, then import it here. It replaces what Gather holds, so run it again with a newer backup \
    until you switch for good.";

fn counted(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}

pub fn imported(imported: &Imported) -> String {
    format!(
        "Filed: {}, {} and {}. [Grimble dusts off the old records.]",
        counted(imported.measurements, "day", "days"),
        counted(imported.profiles, "profile", "profiles"),
        counted(imported.goals, "goal", "goals")
    )
}

pub fn import_failure(refused: &ImportFailure) -> String {
    match refused {
        ImportFailure::NoBackupFound(where_expected) => format!(
            "No backup in the tunnels. Push one with make import-draft first; I looked in {where_expected}."
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
        Failure::Refused(Refusal::NotAnExercise) => {
            "That exercise is not in the catalog yet. Veil checked twice.".into()
        }
        Failure::Broken(reason) => {
            format!("Something broke: {reason}. [A distant explosion.] Trix says it is data.")
        }
    }
}
