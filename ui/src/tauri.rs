use grind_core::failure::Failure;
use grind_core::for_importing::Imported;
use grind_core::for_reviewing_progress::{GatherCard, Window};
use grind_core::for_setting_up_the_profile::{GoalsForm, ProfileForm};
use grind_core::goals::Goals;
use grind_core::ledger::Day;
use grind_core::profile::Profile;
use grind_draft::ImportFailure;
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(command: &str, arguments: JsValue) -> Result<JsValue, JsValue>;
}

async fn call<Output: DeserializeOwned, Refused: DeserializeOwned>(
    command: &str,
    arguments: impl Serialize,
    broken: fn(String) -> Refused,
) -> Result<Output, Refused> {
    let arguments =
        serde_wasm_bindgen::to_value(&arguments).map_err(|error| broken(error.to_string()))?;
    match invoke(command, arguments).await {
        Ok(answer) => {
            serde_wasm_bindgen::from_value(answer).map_err(|error| broken(error.to_string()))
        }
        Err(refusal) => Err(serde_wasm_bindgen::from_value(refusal)
            .unwrap_or_else(|error| broken(error.to_string()))),
    }
}

fn broken(reason: String) -> Failure {
    Failure::Broken(reason)
}

#[derive(Serialize)]
struct Nothing {}

#[derive(Serialize)]
struct OnDay<'a> {
    date: &'a str,
}

#[derive(Serialize)]
struct Measuring<'a> {
    date: &'a str,
    body_mass_kg: f64,
    waist_cm: Option<f64>,
}

#[derive(Serialize)]
struct Form<'a, Filled> {
    form: &'a Filled,
}

#[derive(Serialize)]
struct Reviewing<'a> {
    today: &'a str,
    window: Window,
}

pub async fn recent_days() -> Result<Vec<Day>, Failure> {
    call("recent_days", Nothing {}, broken).await
}

pub async fn log_measurement(
    date: &str,
    body_mass_kg: f64,
    waist_cm: Option<f64>,
) -> Result<Vec<Day>, Failure> {
    let measuring = Measuring {
        date,
        body_mass_kg,
        waist_cm,
    };
    call("log_measurement", measuring, broken).await
}

pub async fn remove_measurement(date: &str) -> Result<Vec<Day>, Failure> {
    call("remove_measurement", OnDay { date }, broken).await
}

pub async fn profile_on(date: &str) -> Result<Option<Profile>, Failure> {
    call("profile_on", OnDay { date }, broken).await
}

pub async fn set_profile(form: &ProfileForm) -> Result<Profile, Failure> {
    call("set_profile", Form { form }, broken).await
}

pub async fn goals_on(date: &str) -> Result<Option<Goals>, Failure> {
    call("goals_on", OnDay { date }, broken).await
}

pub async fn set_goals(form: &GoalsForm) -> Result<Goals, Failure> {
    call("set_goals", Form { form }, broken).await
}

pub async fn gather_card(today: &str, window: Window) -> Result<GatherCard, Failure> {
    call("gather_card", Reviewing { today, window }, broken).await
}

pub async fn import_draft_gather() -> Result<Imported, ImportFailure> {
    call("import_draft_gather", Nothing {}, |reason| {
        ImportFailure::Failed(Failure::Broken(reason))
    })
    .await
}
