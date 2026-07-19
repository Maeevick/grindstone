use grind_core::failure::Failure;
use grind_core::refusal::Refusal;

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
        Failure::Broken(reason) => {
            format!("Something broke: {reason}. [A distant explosion.] Trix says it is data.")
        }
    }
}
