use grind_core::burn::Burn;
use grind_core::exercise::Exercise;
use grind_core::failure::Failure;
use grind_core::food::Food;
use grind_core::for_burning::{OffTheClockForm, WorkoutForm};
use grind_core::for_cataloguing::{FoodForm, MealForm};
use grind_core::for_fuelling::IntakeForm;
use grind_core::for_importing::Imported;
use grind_core::for_reviewing_progress::{GatherCard, Window};
use grind_core::for_setting_up_the_profile::{
    FuelTargetsForm, GoalsForm, ProfileForm, UsualDayForm,
};
use grind_core::fuel::Fuel;
use grind_core::fuel_records::FuelTargets;
use grind_core::goals::Goals;
use grind_core::ledger::Day;
use grind_core::meal::Meal;
use grind_core::profile::Profile;
use grind_core::volitional::Volitional;
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

#[derive(Serialize)]
struct Removing<'a> {
    date: &'a str,
    id: i64,
}

#[derive(Serialize)]
struct OnDayWith<'a, Filled> {
    date: &'a str,
    form: &'a Filled,
}

#[derive(Serialize)]
struct Naming<'a> {
    name: &'a str,
}

#[derive(Serialize)]
struct Cataloguing<'a> {
    name: &'a str,
    met: f64,
}

pub async fn burn_day(date: &str) -> Result<Burn, Failure> {
    call("burn_day", OnDay { date }, broken).await
}

pub async fn log_workout(form: &WorkoutForm) -> Result<Burn, Failure> {
    call("log_workout", Form { form }, broken).await
}

pub async fn remove_workout(date: &str, id: i64) -> Result<Burn, Failure> {
    call("remove_workout", Removing { date, id }, broken).await
}

pub async fn set_day_off_the_clock(date: &str, form: &OffTheClockForm) -> Result<Burn, Failure> {
    call("set_day_off_the_clock", OnDayWith { date, form }, broken).await
}

pub async fn exercises() -> Result<Vec<Exercise>, Failure> {
    call("exercises", Nothing {}, broken).await
}

pub async fn add_exercise(name: &str, met: f64) -> Result<Vec<Exercise>, Failure> {
    call("add_exercise", Cataloguing { name, met }, broken).await
}

pub async fn remove_exercise(name: &str) -> Result<Vec<Exercise>, Failure> {
    call("remove_exercise", Naming { name }, broken).await
}

pub async fn set_usual_day(form: &UsualDayForm) -> Result<Volitional, Failure> {
    call("set_usual_day", Form { form }, broken).await
}

pub async fn usual_day_on(date: &str) -> Result<Option<Volitional>, Failure> {
    call("usual_day_on", OnDay { date }, broken).await
}

pub async fn fuel_day(date: &str) -> Result<Fuel, Failure> {
    call("fuel_day", OnDay { date }, broken).await
}

pub async fn log_intake(form: &IntakeForm) -> Result<Fuel, Failure> {
    call("log_intake", Form { form }, broken).await
}

pub async fn remove_intake(date: &str, id: i64) -> Result<Fuel, Failure> {
    call("remove_intake", Removing { date, id }, broken).await
}

pub async fn foods() -> Result<Vec<Food>, Failure> {
    call("foods", Nothing {}, broken).await
}

pub async fn add_food(form: &FoodForm) -> Result<Vec<Food>, Failure> {
    call("add_food", Form { form }, broken).await
}

pub async fn remove_food(name: &str) -> Result<Vec<Food>, Failure> {
    call("remove_food", Naming { name }, broken).await
}

pub async fn set_fuel_targets(form: &FuelTargetsForm) -> Result<FuelTargets, Failure> {
    call("set_fuel_targets", Form { form }, broken).await
}

pub async fn fuel_targets_on(date: &str) -> Result<FuelTargets, Failure> {
    call("fuel_targets_on", OnDay { date }, broken).await
}

pub async fn meals() -> Result<Vec<Meal>, Failure> {
    call("meals", Nothing {}, broken).await
}

pub async fn preview_meal(form: &MealForm) -> Result<Meal, Failure> {
    call("preview_meal", Form { form }, broken).await
}

pub async fn cook_meal(form: &MealForm) -> Result<Vec<Meal>, Failure> {
    call("cook_meal", Form { form }, broken).await
}

pub async fn remove_meal(name: &str) -> Result<Vec<Meal>, Failure> {
    call("remove_meal", Naming { name }, broken).await
}

pub async fn import_draft() -> Result<Imported, ImportFailure> {
    call("import_draft", Nothing {}, |reason| {
        ImportFailure::Failed(Failure::Broken(reason))
    })
    .await
}
