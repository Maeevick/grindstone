use grind_core::burn::Burn;
use grind_core::circuit::Circuit;
use grind_core::drink::{Drink, DrinkKind};
use grind_core::exercise::Exercise;
use grind_core::failure::Failure;
use grind_core::food::Food;
use grind_core::for_burning::CircuitSessionForm;
use grind_core::for_burning::{OffTheClockForm, WorkoutForm};
use grind_core::for_cataloguing::CircuitForm;
use grind_core::for_cataloguing::{FoodForm, MealForm, PricedCircuit};
use grind_core::for_charting::{
    BandChart, BurnChart, BurnRow, FuelRow, GatherChart, GatherRow, HydrateRow, Insights, Month,
};
use grind_core::for_fuelling::IntakeForm;
use grind_core::for_hydrating::DrinkingForm;
use grind_core::for_importing::Imported;
use grind_core::for_reviewing_progress::{Explored, GatherCard, Window};
use grind_core::for_setting_up_the_profile::{
    BurnTargetForm, FuelTargetsForm, GoalsForm, HydrateCountedForm, ProfileForm, UsualDayForm,
};
use grind_core::for_tracking::GatherWeek;
use grind_core::fuel::Fuel;
use grind_core::fuel_records::FuelTargets;
use grind_core::goals::Goals;
use grind_core::hydrate::Hydrate;
use grind_core::meal::Meal;
use grind_core::profile::Profile;
use grind_core::recommendations::Recommendations;
use grind_core::stats::Measure;
use grind_core::streak::Streak;
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

pub async fn gather_week(date: &str) -> Result<GatherWeek, Failure> {
    call("gather_week", OnDay { date }, broken).await
}

pub async fn log_measurement(
    date: &str,
    body_mass_kg: f64,
    waist_cm: Option<f64>,
) -> Result<GatherWeek, Failure> {
    let measuring = Measuring {
        date,
        body_mass_kg,
        waist_cm,
    };
    call("log_measurement", measuring, broken).await
}

pub async fn remove_measurement(date: &str) -> Result<GatherWeek, Failure> {
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

pub async fn set_burn_target(form: &BurnTargetForm) -> Result<f64, Failure> {
    call("set_burn_target", Form { form }, broken).await
}

pub async fn burn_target_on(date: &str) -> Result<Option<f64>, Failure> {
    call("burn_target_on", OnDay { date }, broken).await
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

pub async fn cook_meal(form: &MealForm) -> Result<Vec<Meal>, Failure> {
    call("cook_meal", Form { form }, broken).await
}

pub async fn remove_meal(name: &str) -> Result<Vec<Meal>, Failure> {
    call("remove_meal", Naming { name }, broken).await
}

#[derive(Serialize)]
struct Today<'a> {
    today: &'a str,
}

pub async fn streak(today: &str) -> Result<Streak, Failure> {
    call("streak", Today { today }, broken).await
}

#[derive(Serialize)]
struct Exploring<'a> {
    today: &'a str,
    window: Window,
    one: Measure,
    other: Measure,
}

pub async fn explore(
    today: &str,
    window: Window,
    (one, other): (Measure, Measure),
) -> Result<Explored, Failure> {
    let exploring = Exploring {
        today,
        window,
        one,
        other,
    };
    call("explore", exploring, broken).await
}

#[derive(Serialize)]
struct InMonth<'a> {
    month: &'a str,
}

pub async fn gather_chart(today: &str, window: Window) -> Result<GatherChart, Failure> {
    call("gather_chart", Reviewing { today, window }, broken).await
}

pub async fn fuel_chart(today: &str, window: Window) -> Result<BandChart, Failure> {
    call("fuel_chart", Reviewing { today, window }, broken).await
}

pub async fn hydrate_chart(today: &str, window: Window) -> Result<BandChart, Failure> {
    call("hydrate_chart", Reviewing { today, window }, broken).await
}

pub async fn burn_chart(today: &str, window: Window) -> Result<BurnChart, Failure> {
    call("burn_chart", Reviewing { today, window }, broken).await
}

pub async fn gather_month(month: &str) -> Result<Month<GatherRow>, Failure> {
    call("gather_month", InMonth { month }, broken).await
}

pub async fn fuel_month(month: &str) -> Result<Month<FuelRow>, Failure> {
    call("fuel_month", InMonth { month }, broken).await
}

pub async fn hydrate_month(month: &str) -> Result<Month<HydrateRow>, Failure> {
    call("hydrate_month", InMonth { month }, broken).await
}

pub async fn burn_month(month: &str) -> Result<Month<BurnRow>, Failure> {
    call("burn_month", InMonth { month }, broken).await
}

pub async fn insights(today: &str, window: Window) -> Result<Insights, Failure> {
    call("insights", Reviewing { today, window }, broken).await
}

pub async fn erase_everything() -> Result<(), Failure> {
    call("erase_everything", Nothing {}, broken).await
}

pub async fn recommended_on(date: &str) -> Result<Recommendations, Failure> {
    call("recommended_on", OnDay { date }, broken).await
}

#[derive(Serialize)]
struct Opening<'a> {
    url: &'a str,
}

pub async fn open_page(url: &str) -> Result<(), Failure> {
    call("plugin:opener|open_url", Opening { url }, broken).await
}

pub async fn import_draft(today: &str) -> Result<Imported, ImportFailure> {
    call("import_draft", Today { today }, |reason| {
        ImportFailure::Failed(Failure::Broken(reason))
    })
    .await
}

pub async fn circuits() -> Result<Vec<Circuit>, Failure> {
    call("circuits", Nothing {}, broken).await
}

pub async fn priced_circuits(date: &str) -> Result<Vec<PricedCircuit>, Failure> {
    call("priced_circuits", OnDay { date }, broken).await
}

pub async fn craft_circuit(form: &CircuitForm) -> Result<Vec<Circuit>, Failure> {
    call("craft_circuit", Form { form }, broken).await
}

pub async fn remove_circuit(name: &str) -> Result<Vec<Circuit>, Failure> {
    call("remove_circuit", Naming { name }, broken).await
}

pub async fn log_circuit(form: &CircuitSessionForm) -> Result<Burn, Failure> {
    call("log_circuit", Form { form }, broken).await
}

#[derive(Serialize)]
struct Serving<'a> {
    name: &'a str,
    volume_ml: f64,
    kind: DrinkKind,
}

pub async fn hydrate_day(date: &str) -> Result<Hydrate, Failure> {
    call("hydrate_day", OnDay { date }, broken).await
}

pub async fn log_drink(form: &DrinkingForm) -> Result<Hydrate, Failure> {
    call("log_drink", Form { form }, broken).await
}

pub async fn remove_drinking(date: &str, id: i64) -> Result<Hydrate, Failure> {
    call("remove_drinking", Removing { date, id }, broken).await
}

pub async fn drinks() -> Result<Vec<Drink>, Failure> {
    call("drinks", Nothing {}, broken).await
}

pub async fn add_drink(name: &str, volume_ml: f64, kind: DrinkKind) -> Result<Vec<Drink>, Failure> {
    call(
        "add_drink",
        Serving {
            name,
            volume_ml,
            kind,
        },
        broken,
    )
    .await
}

pub async fn remove_drink(name: &str) -> Result<Vec<Drink>, Failure> {
    call("remove_drink", Naming { name }, broken).await
}

pub async fn set_hydrate_counted(form: &HydrateCountedForm) -> Result<bool, Failure> {
    call("set_hydrate_counted", Form { form }, broken).await
}

pub async fn hydrate_counted_on(date: &str) -> Result<bool, Failure> {
    call("hydrate_counted_on", OnDay { date }, broken).await
}
