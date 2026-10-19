use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use grind_core::burn::Burn;
use grind_core::circuit::Circuit;
use grind_core::drink::{Drink, DrinkKind};
use grind_core::exercise::Exercise;
use grind_core::failure::Failure;
use grind_core::food::Food;
use grind_core::for_burning::CircuitSessionForm;
use grind_core::for_burning::{self, OffTheClockForm, SessionForm, WorkoutForm};
use grind_core::for_cataloguing::CircuitForm;
use grind_core::for_cataloguing::{self, FoodForm, MealForm, PricedCircuit};
use grind_core::for_charting::{
    self, BandChart, BurnChart, BurnRow, FuelRow, GatherChart, GatherRow, HydrateRow, Insights,
    Month,
};
use grind_core::for_fuelling::{self, IntakeForm, PlateForm};
use grind_core::for_hydrating::{self, DrinkingForm};
use grind_core::for_importing::{self, Imported};
use grind_core::for_reviewing_progress::{self, Explored, GatherCard, Window};
use grind_core::for_setting_up_the_profile::{
    self, BurnTargetForm, FuelTargetsForm, GoalsForm, HydrateCountedForm, ProfileForm, UsualDayForm,
};
use grind_core::for_starting_over;
use grind_core::for_tracking;
use grind_core::for_tracking::GatherWeek;
use grind_core::fuel::Fuel;
use grind_core::fuel_records::FuelTargets;
use grind_core::fuel_week::FuelWeek;
use grind_core::goals::Goals;
use grind_core::hydrate::Hydrate;
use grind_core::meal::Meal;
use grind_core::profile::Profile;
use grind_core::recommendations::Recommendations;
use grind_core::stats::Measure;
use grind_core::streak::Streak;
use grind_core::volitional::Volitional;
use grind_draft::{ImportFailure, drafted};
use grind_sqlite::Sqlite;
use tauri::State;

pub struct Records(pub Mutex<Sqlite>);

pub struct DraftBackup(pub PathBuf);

#[tauri::command(rename_all = "snake_case")]
pub fn log_measurement(
    records: State<Records>,
    date: String,
    body_mass_kg: f64,
    waist_cm: Option<f64>,
) -> Result<GatherWeek, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_tracking::log_measurement(&*records, &date, body_mass_kg, waist_cm)
}

#[tauri::command(rename_all = "snake_case")]
pub fn gather_week(records: State<Records>, date: String) -> Result<GatherWeek, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_tracking::gather_week(&*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_measurement(records: State<Records>, date: String) -> Result<GatherWeek, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_tracking::remove_measurement(&*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_profile(records: State<Records>, form: ProfileForm) -> Result<Profile, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::set_profile(&*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn profile_on(records: State<Records>, date: String) -> Result<Option<Profile>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::profile_on(&*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_goals(records: State<Records>, form: GoalsForm) -> Result<Goals, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::set_goals(&*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn goals_on(records: State<Records>, date: String) -> Result<Option<Goals>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::goals_on(&*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn gather_card(
    records: State<Records>,
    today: String,
    window: Window,
) -> Result<GatherCard, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_reviewing_progress::gather_card(&*records, &today, window)
}

#[tauri::command(rename_all = "snake_case")]
pub fn import_draft(
    records: State<Records>,
    backup: State<DraftBackup>,
    today: String,
) -> Result<Imported, ImportFailure> {
    let backup = std::fs::read_to_string(&backup.0)
        .map_err(|_| ImportFailure::NoBackupFound(backup.0.display().to_string()))?;
    let drafted = drafted(&backup).map_err(ImportFailure::Draft)?;
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_importing::import_drafted(
        &*records, &*records, &*records, &*records, &*records, &drafted, &today,
    )
    .map_err(ImportFailure::Failed)
}

#[tauri::command(rename_all = "snake_case")]
pub fn burn_day(records: State<Records>, date: String) -> Result<Burn, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_burning::burn_day(&*records, &*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn log_workout(records: State<Records>, form: WorkoutForm) -> Result<Burn, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_burning::log_workout(&*records, &*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_workout(records: State<Records>, date: String, id: i64) -> Result<Burn, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_burning::remove_workout(&*records, &*records, &date, id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_day_off_the_clock(
    records: State<Records>,
    date: String,
    form: OffTheClockForm,
) -> Result<Burn, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_burning::set_day_off_the_clock(&*records, &*records, &date, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn exercises(records: State<Records>) -> Result<Vec<Exercise>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::exercises(&*records)
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_exercise(
    records: State<Records>,
    name: String,
    met: f64,
    description: Option<String>,
) -> Result<Vec<Exercise>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::add_exercise(&*records, &name, met, &description.unwrap_or_default())
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_exercise(records: State<Records>, name: String) -> Result<Vec<Exercise>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::remove_exercise(&*records, &name)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_usual_day(records: State<Records>, form: UsualDayForm) -> Result<Volitional, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::set_usual_day(&*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn usual_day_on(records: State<Records>, date: String) -> Result<Option<Volitional>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::usual_day_on(&*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn fuel_day(records: State<Records>, date: String) -> Result<Fuel, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_fuelling::fuel_day(&*records, &*records, &*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn fuel_week(
    records: State<Records>,
    date: String,
    today: String,
) -> Result<Option<FuelWeek>, Failure> {
    let _ = (records, date, today);
    Ok(None)
}

#[tauri::command(rename_all = "snake_case")]
pub fn log_intake(records: State<Records>, form: IntakeForm) -> Result<Fuel, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_fuelling::log_intake(&*records, &*records, &*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn log_plate(records: State<Records>, form: PlateForm) -> Result<Fuel, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_fuelling::log_plate(&*records, &*records, &*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_intake(records: State<Records>, date: String, id: i64) -> Result<Fuel, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_fuelling::remove_intake(&*records, &*records, &*records, &date, id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn foods(records: State<Records>) -> Result<Vec<Food>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::foods(&*records)
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_food(records: State<Records>, form: FoodForm) -> Result<Vec<Food>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::add_food(&*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_food(records: State<Records>, name: String) -> Result<Vec<Food>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::remove_food(&*records, &name)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_fuel_targets(
    records: State<Records>,
    form: FuelTargetsForm,
) -> Result<FuelTargets, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::set_fuel_targets(&*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn fuel_targets_on(records: State<Records>, date: String) -> Result<FuelTargets, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::fuel_targets_on(&*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn meals(records: State<Records>) -> Result<Vec<Meal>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::meals(&*records)
}

#[tauri::command(rename_all = "snake_case")]
pub fn cook_meal(records: State<Records>, form: MealForm) -> Result<Vec<Meal>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::cook_meal(&*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_meal(records: State<Records>, name: String) -> Result<Vec<Meal>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::remove_meal(&*records, &name)
}

#[tauri::command(rename_all = "snake_case")]
pub fn streak(records: State<Records>, today: String) -> Result<Streak, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_reviewing_progress::streak(&*records, &*records, &*records, &*records, &today)
}

#[tauri::command(rename_all = "snake_case")]
pub fn explore(
    records: State<Records>,
    today: String,
    window: Window,
    one: Measure,
    other: Measure,
) -> Result<Explored, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_reviewing_progress::explore(
        &*records,
        &*records,
        &*records,
        &today,
        window,
        (one, other),
    )
}

#[tauri::command(rename_all = "snake_case")]
pub fn circuits(records: State<Records>) -> Result<Vec<Circuit>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::circuits(&*records)
}

#[tauri::command(rename_all = "snake_case")]
pub fn priced_circuits(
    records: State<Records>,
    date: String,
) -> Result<Vec<PricedCircuit>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::priced_circuits(&*records, &*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn craft_circuit(records: State<Records>, form: CircuitForm) -> Result<Vec<Circuit>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::craft_circuit(&*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_circuit(records: State<Records>, name: String) -> Result<Vec<Circuit>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::remove_circuit(&*records, &name)
}

#[tauri::command(rename_all = "snake_case")]
pub fn log_circuit(records: State<Records>, form: CircuitSessionForm) -> Result<Burn, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_burning::log_circuit(&*records, &*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn log_session(records: State<Records>, form: SessionForm) -> Result<Burn, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_burning::log_session(&*records, &*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn hydrate_day(records: State<Records>, date: String) -> Result<Hydrate, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_hydrating::hydrate_day(&*records, &*records, &*records, &*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn log_drink(records: State<Records>, form: DrinkingForm) -> Result<Hydrate, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_hydrating::log_drink(&*records, &*records, &*records, &*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_drinking(records: State<Records>, date: String, id: i64) -> Result<Hydrate, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_hydrating::remove_drinking(&*records, &*records, &*records, &*records, &date, id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn drinks(records: State<Records>) -> Result<Vec<Drink>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::drinks(&*records)
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_drink(
    records: State<Records>,
    name: String,
    volume_ml: f64,
    kind: DrinkKind,
    description: Option<String>,
) -> Result<Vec<Drink>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::add_drink(
        &*records,
        &name,
        volume_ml,
        kind,
        &description.unwrap_or_default(),
    )
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_drink(records: State<Records>, name: String) -> Result<Vec<Drink>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::remove_drink(&*records, &name)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_burn_target(records: State<Records>, form: BurnTargetForm) -> Result<f64, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::set_burn_target(&*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn burn_target_on(records: State<Records>, date: String) -> Result<Option<f64>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::burn_target_on(&*records, &*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_hydrate_counted(
    records: State<Records>,
    form: HydrateCountedForm,
) -> Result<bool, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::set_hydrate_counted(&*records, &form)
}

#[tauri::command(rename_all = "snake_case")]
pub fn hydrate_counted_on(records: State<Records>, date: String) -> Result<bool, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::hydrate_counted_on(&*records, &date)
}

#[tauri::command(rename_all = "snake_case")]
pub fn gather_chart(
    records: State<Records>,
    today: String,
    window: Window,
) -> Result<GatherChart, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_charting::gather_chart(&*records, &today, window)
}

#[tauri::command(rename_all = "snake_case")]
pub fn fuel_chart(
    records: State<Records>,
    today: String,
    window: Window,
) -> Result<BandChart, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_charting::fuel_chart(&*records, &*records, &*records, &today, window)
}

#[tauri::command(rename_all = "snake_case")]
pub fn hydrate_chart(
    records: State<Records>,
    today: String,
    window: Window,
) -> Result<BandChart, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_charting::hydrate_chart(&*records, &*records, &*records, &*records, &today, window)
}

#[tauri::command(rename_all = "snake_case")]
pub fn burn_chart(
    records: State<Records>,
    today: String,
    window: Window,
) -> Result<BurnChart, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_charting::burn_chart(&*records, &*records, &today, window)
}

#[tauri::command(rename_all = "snake_case")]
pub fn gather_month(records: State<Records>, month: String) -> Result<Month<GatherRow>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_charting::gather_month(&*records, &month)
}

#[tauri::command(rename_all = "snake_case")]
pub fn fuel_month(records: State<Records>, month: String) -> Result<Month<FuelRow>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_charting::fuel_month(&*records, &month)
}

#[tauri::command(rename_all = "snake_case")]
pub fn hydrate_month(records: State<Records>, month: String) -> Result<Month<HydrateRow>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_charting::hydrate_month(&*records, &month)
}

#[tauri::command(rename_all = "snake_case")]
pub fn burn_month(records: State<Records>, month: String) -> Result<Month<BurnRow>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_charting::burn_month(&*records, &*records, &month)
}

#[tauri::command(rename_all = "snake_case")]
pub fn insights(
    records: State<Records>,
    today: String,
    window: Window,
) -> Result<Insights, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_charting::insights(&*records, &*records, &*records, &*records, &today, window)
}

#[tauri::command(rename_all = "snake_case")]
pub fn erase_everything(records: State<Records>) -> Result<(), Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_starting_over::erase_everything(&*records, &*records, &*records, &*records, &*records)
}

#[tauri::command(rename_all = "snake_case")]
pub fn recommended_on(records: State<Records>, date: String) -> Result<Recommendations, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_setting_up_the_profile::recommended_on(&*records, &date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};
    use tauri::ipc::{CallbackFn, InvokeBody};
    use tauri::test::{
        INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets,
    };
    use tauri::webview::InvokeRequest;
    use tauri::{App, WebviewWindow, WebviewWindowBuilder};

    fn launched_with(backup: PathBuf) -> (App<MockRuntime>, WebviewWindow<MockRuntime>) {
        let records = Sqlite::in_memory().expect("an in-memory database opens");
        let app = mock_builder()
            .manage(Records(Mutex::new(records)))
            .manage(DraftBackup(backup))
            .invoke_handler(tauri::generate_handler![
                log_measurement,
                gather_week,
                remove_measurement,
                set_profile,
                profile_on,
                set_goals,
                goals_on,
                gather_card,
                import_draft,
                burn_day,
                log_workout,
                remove_workout,
                set_day_off_the_clock,
                exercises,
                add_exercise,
                remove_exercise,
                set_usual_day,
                usual_day_on,
                fuel_day,
                fuel_week,
                log_intake,
                log_plate,
                remove_intake,
                foods,
                add_food,
                remove_food,
                set_fuel_targets,
                fuel_targets_on,
                meals,
                cook_meal,
                remove_meal,
                streak,
                explore,
                circuits,
                priced_circuits,
                craft_circuit,
                remove_circuit,
                log_circuit,
                log_session,
                hydrate_day,
                log_drink,
                remove_drinking,
                drinks,
                add_drink,
                remove_drink,
                set_hydrate_counted,
                hydrate_counted_on,
                set_burn_target,
                burn_target_on,
                gather_chart,
                fuel_chart,
                hydrate_chart,
                burn_chart,
                gather_month,
                fuel_month,
                hydrate_month,
                burn_month,
                insights,
                erase_everything,
                recommended_on,
            ])
            .build(mock_context(noop_assets()))
            .expect("the app builds on the mock runtime");
        let screen = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("the screen opens");
        (app, screen)
    }

    fn launched() -> (App<MockRuntime>, WebviewWindow<MockRuntime>) {
        launched_with(PathBuf::from("/nowhere/grindstone-draft.json"))
    }

    fn invoke(
        screen: &WebviewWindow<MockRuntime>,
        command: &str,
        arguments: Value,
    ) -> Result<Value, Value> {
        get_ipc_response(
            screen,
            InvokeRequest {
                cmd: command.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: "tauri://localhost".parse().expect("a valid origin"),
                body: InvokeBody::Json(arguments),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|answer| answer.deserialize::<Value>().expect("a JSON answer"))
    }

    #[test]
    fn an_impossible_mass_comes_back_as_a_refusal_the_screen_understands() {
        let (_app, screen) = launched();

        let answer = invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 0.0 }),
        );

        assert_eq!(answer, Err(json!({ "Refused": "NotAMass" })));
    }

    #[test]
    fn logging_a_measurement_answers_its_week_as_the_screen_reads_it() {
        let (_app, screen) = launched();

        let answer = invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 80.0, "waist_cm": 80.0 }),
        );

        let measured = json!({ "day": "2026-10-10", "body_mass_kg": 80.0, "waist_cm": 80.0, "composition": null, "age": null });
        assert_eq!(
            answer,
            Ok(json!({
                "day": measured, "days": [measured],
                "tendency": { "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null },
                "aims": null, "pace": null
            }))
        );
    }

    #[test]
    fn the_gather_week_starts_empty_on_a_first_launch() {
        let (_app, screen) = launched();

        assert_eq!(
            invoke(&screen, "gather_week", json!({ "date": "2026-10-10" })),
            Ok(json!({
                "day": null, "days": [],
                "tendency": { "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null },
                "aims": null, "pace": null
            }))
        );
    }

    #[test]
    fn removing_a_measured_day_answers_the_days_left() {
        let (_app, screen) = launched();
        for date in ["2026-10-09", "2026-10-10"] {
            invoke(
                &screen,
                "log_measurement",
                json!({ "date": date, "body_mass_kg": 80.0 }),
            )
            .expect("a measurement logs");
        }

        let answer = invoke(
            &screen,
            "remove_measurement",
            json!({ "date": "2026-10-10" }),
        );

        assert_eq!(
            answer.map(|week| week["days"].as_array().map(Vec::len)),
            Ok(Some(1))
        );
    }

    #[test]
    fn a_profile_set_from_the_screen_is_the_one_in_force_from_its_day() {
        let (_app, screen) = launched();
        let form = json!({
            "from": "2026-10-10", "birthday": "1985-04-12", "gender": "Male",
            "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null
        });

        let set = invoke(&screen, "set_profile", json!({ "form": form }));
        let in_force = invoke(&screen, "profile_on", json!({ "date": "2026-10-12" }));

        let expected = json!({ "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null });
        assert_eq!(set, Ok(expected.clone()));
        assert_eq!(in_force, Ok(expected));
    }

    #[test]
    fn goals_set_from_the_screen_are_the_ones_in_force_from_their_day() {
        let (_app, screen) = launched();
        let form = json!({ "from": "2026-10-10", "body_mass_kg": 70.0, "fat_percentage": 10.0 });

        let set = invoke(&screen, "set_goals", json!({ "form": form }));
        let in_force = invoke(&screen, "goals_on", json!({ "date": "2026-10-12" }));

        let expected = json!({ "body_mass_kg": 70.0, "fat_ratio": 0.1 });
        assert_eq!(set, Ok(expected.clone()));
        assert_eq!(in_force, Ok(expected));
    }

    #[test]
    fn the_gather_card_crosses_to_the_screen_with_its_masses_and_their_trend() {
        let (_app, screen) = launched();
        invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 80.0 }),
        )
        .expect("a measurement logs");

        let card = invoke(
            &screen,
            "gather_card",
            json!({ "today": "2026-10-10", "window": "Days30" }),
        );

        assert_eq!(
            card,
            Ok(json!({
                "masses": [["2026-10-10", 80.0]], "trend": [["2026-10-10", 80.0]], "target_kg": null,
                "progress": null, "fat_change_kg": null, "lean_change_kg": null
            }))
        );
    }

    #[test]
    fn importing_the_draft_backup_pushed_over_usb_tells_what_it_brought() {
        let backup =
            std::env::temp_dir().join(format!("grind-tauri-draft-{}.json", std::process::id()));
        std::fs::write(
            &backup,
            r#"{ "settings": { "version": "0.4.0" }, "measurement_bases": { "birthday": "1985-04-12" },
                 "measurement_logs": [{ "date": "2026-10-08", "gender": "male", "height": 180.0, "neck": 38.5,
                   "hip": null, "weight": 72.15, "waist": 80.5, "weight_target": 70.0, "fat_target": 10.0 }] }"#,
        )
        .expect("a backup pushed");
        let (_app, screen) = launched_with(backup.clone());

        let answer = invoke(&screen, "import_draft", json!({ "today": "2026-10-10" }));

        std::fs::remove_file(&backup).expect("the test backup is removed");
        assert_eq!(
            answer,
            Ok(
                json!({ "measurements": 1, "profiles": 1, "goals": 1, "exercises": 0, "workouts": 0, "foods": 0, "intakes": 0, "drinks": 2, "drinkings": 0 })
            )
        );
    }

    #[test]
    fn importing_with_no_backup_pushed_says_where_it_was_expected() {
        let (_app, screen) = launched();

        let answer = invoke(&screen, "import_draft", json!({ "today": "2026-10-10" }));

        assert_eq!(
            answer,
            Err(json!({ "NoBackupFound": "/nowhere/grindstone-draft.json" }))
        );
    }

    #[test]
    fn importing_a_backup_that_cannot_be_read_hands_the_screen_the_draft_refusal() {
        let backup = std::env::temp_dir().join(format!(
            "grind-tauri-unreadable-{}.json",
            std::process::id()
        ));
        std::fs::write(&backup, "{ not a backup").expect("a backup pushed");
        let (_app, screen) = launched_with(backup.clone());

        let answer = invoke(&screen, "import_draft", json!({ "today": "2026-10-10" }));

        std::fs::remove_file(&backup).expect("the test backup is removed");
        assert!(
            matches!(&answer, Err(refusal) if refusal["Draft"]["Unreadable"].is_string()),
            "{answer:?}"
        );
    }

    fn with_walking_in_the_catalog(screen: &WebviewWindow<MockRuntime>) {
        invoke(
            screen,
            "add_exercise",
            json!({ "name": "Walking", "met": 4.0 }),
        )
        .expect("an exercise adds");
    }

    fn walked(screen: &WebviewWindow<MockRuntime>, minutes: f64) -> Result<Value, Value> {
        invoke(
            screen,
            "log_workout",
            json!({ "form": { "date": "2026-10-10", "exercise": "Walking", "minutes": minutes,
                              "reps": null, "sets": 1, "load_kg": 0.0 } }),
        )
    }

    #[test]
    fn a_workout_logged_from_the_screen_comes_back_in_its_days_burn() {
        let (_app, screen) = launched();
        with_walking_in_the_catalog(&screen);
        invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 80.0 }),
        )
        .expect("a measurement logs");

        let burn = walked(&screen, 60.0).expect("a burn");
        let read_again = invoke(&screen, "burn_day", json!({ "date": "2026-10-10" }));

        assert!(
            burn["burnt_kcal"]
                .as_f64()
                .is_some_and(|kcal| (kcal - 252.0).abs() < 1e-9)
        );
        assert_eq!(read_again, Ok(burn));
    }

    #[test]
    fn removing_a_workout_answers_the_days_burn_without_it() {
        let (_app, screen) = launched();
        with_walking_in_the_catalog(&screen);
        let burn = walked(&screen, 30.0).expect("a burn");
        let id = burn["workouts"][0]["id"].clone();

        let answer = invoke(
            &screen,
            "remove_workout",
            json!({ "date": "2026-10-10", "id": id }),
        );

        assert_eq!(answer.map(|burn| burn["workouts"].clone()), Ok(json!([])));
    }

    #[test]
    fn a_day_off_the_clock_set_from_the_screen_is_that_days_own() {
        let (_app, screen) = launched();

        let answer = invoke(
            &screen,
            "set_day_off_the_clock",
            json!({ "date": "2026-10-10",
                    "form": { "sitting_min": 600.0, "standing_min": 60.0, "moving_km": 1.0 } }),
        );

        assert_eq!(
            answer.map(|burn| burn["off_the_clock_is_usual"].clone()),
            Ok(json!(false))
        );
    }

    #[test]
    fn exercises_added_and_removed_from_the_screen_answer_the_catalog() {
        let (_app, screen) = launched();
        with_walking_in_the_catalog(&screen);

        let listed = invoke(&screen, "exercises", json!({}));
        let removed = invoke(&screen, "remove_exercise", json!({ "name": "Walking" }));

        assert_eq!(
            listed,
            Ok(json!([{ "name": "Walking", "met": 4.0, "description": "" }]))
        );
        assert_eq!(removed, Ok(json!([])));
    }

    #[test]
    fn a_usual_day_set_from_the_screen_is_the_one_in_force_from_its_day() {
        let (_app, screen) = launched();
        let usual = json!({ "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 3.0 });

        let set = invoke(
            &screen,
            "set_usual_day",
            json!({ "form": { "from": "2026-03-01", "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 3.0 } }),
        );
        let in_force = invoke(&screen, "usual_day_on", json!({ "date": "2026-10-10" }));

        assert_eq!(set, Ok(usual.clone()));
        assert_eq!(in_force, Ok(usual));
    }

    fn with_skyr_in_the_catalog(screen: &WebviewWindow<MockRuntime>) -> Result<Value, Value> {
        invoke(
            screen,
            "add_food",
            json!({ "form": { "name": "Skyr", "unit": "Gram", "portion": 100.0,
                "nutrients": { "energy_kcal": 54.0, "protein_g": 9.5, "carbs_g": 4.0, "fat_g": 0.0, "fibre_g": 0.0 } } }),
        )
    }

    #[test]
    fn an_intake_logged_from_the_screen_comes_back_in_its_days_fuel() {
        let (_app, screen) = launched();
        with_skyr_in_the_catalog(&screen).expect("a food adds");

        let fuel = invoke(
            &screen,
            "log_intake",
            json!({ "form": { "date": "2026-10-10", "food": "Skyr", "quantity": 200.0 } }),
        )
        .expect("a fuel");
        let read_again = invoke(&screen, "fuel_day", json!({ "date": "2026-10-10" }));
        let id = fuel["intakes"][0][0].clone();
        let removed = invoke(
            &screen,
            "remove_intake",
            json!({ "date": "2026-10-10", "id": id }),
        );

        assert_eq!(fuel["eaten"]["energy_kcal"], json!(108.0));
        assert_eq!(read_again, Ok(fuel));
        assert_eq!(removed.map(|fuel| fuel["intakes"].clone()), Ok(json!([])));
    }

    #[test]
    fn a_plate_logged_from_the_screen_answers_the_days_fuel_with_an_intake_per_ingredient() {
        let (_app, screen) = launched();
        with_skyr_in_the_catalog(&screen).expect("a food adds");

        let fuel = invoke(
            &screen,
            "log_plate",
            json!({ "form": { "date": "2026-10-10", "ingredients": [
                { "food": "Skyr", "quantity": 200.0, "cooked_per_raw": null },
                { "food": "Skyr", "quantity": 50.0, "cooked_per_raw": null } ] } }),
        );

        assert_eq!(
            fuel.map(|fuel| fuel["eaten"]["energy_kcal"].clone()),
            Ok(json!(135.0))
        );
    }

    #[test]
    fn a_session_logged_from_the_screen_answers_the_days_burn_with_a_workout_per_step() {
        let (_app, screen) = launched();
        invoke(
            &screen,
            "add_exercise",
            json!({ "name": "Walking", "met": 4.0 }),
        )
        .expect("an exercise adds");

        let burnt = invoke(
            &screen,
            "log_session",
            json!({ "form": { "date": "2026-10-10", "sets": 2, "steps": [
                { "exercise": "Walking", "reps": null, "minutes": 10.0, "load_kg": 0.0 },
                { "exercise": "Walking", "reps": null, "minutes": 5.0, "load_kg": 0.0 } ] } }),
        );

        assert_eq!(
            burnt.map(|burn| burn["workouts"].as_array().map(Vec::len)),
            Ok(Some(2))
        );
    }

    #[test]
    fn foods_added_and_removed_from_the_screen_answer_the_catalog() {
        let (_app, screen) = launched();

        let added = with_skyr_in_the_catalog(&screen);
        let listed = invoke(&screen, "foods", json!({}));
        let removed = invoke(&screen, "remove_food", json!({ "name": "Skyr" }));

        assert_eq!(listed, added);
        assert_eq!(removed, Ok(json!([])));
    }

    #[test]
    fn fuel_targets_set_from_the_screen_are_the_ones_in_force_from_their_day() {
        let (_app, screen) = launched();
        let aims = json!({ "protein_g_per_kg": 2.025, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 });

        let set = invoke(
            &screen,
            "set_fuel_targets",
            json!({ "form": { "from": "2026-03-01", "protein_g_per_kg": 2.025, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 } }),
        );
        let in_force = invoke(&screen, "fuel_targets_on", json!({ "date": "2026-10-10" }));

        assert_eq!(set, Ok(aims.clone()));
        assert_eq!(in_force, Ok(aims));
    }

    #[test]
    fn a_meal_cooked_and_removed_from_the_screen_answers_as_the_screen_reads_it() {
        let (_app, screen) = launched();
        with_skyr_in_the_catalog(&screen).expect("a food adds");
        let form = json!({ "form": { "name": "Skyr bowl", "description": "Stir.",
            "ingredients": [{ "food": "Skyr", "quantity": 200.0, "cooked_per_raw": 1.0 }] } });

        let cooked = invoke(&screen, "cook_meal", form).expect("the meals");
        let listed = invoke(&screen, "meals", json!({}));
        let removed = invoke(&screen, "remove_meal", json!({ "name": "Skyr bowl" }));

        assert_eq!(cooked[0]["totals"]["energy_kcal"], json!(108.0));
        assert_eq!(cooked[0]["cooked_g"], json!(200.0));
        assert_eq!(listed, Ok(cooked));
        assert_eq!(removed, Ok(json!([])));
    }

    #[test]
    fn the_weeks_fuel_crosses_to_the_screen_with_its_logged_days_and_their_averages() {
        let (_app, screen) = launched();
        invoke(
            &screen,
            "set_profile",
            json!({ "form": { "from": "2026-10-01", "birthday": "1985-04-12", "gender": "Male",
                "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null } }),
        )
        .expect("a profile sets");
        invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-01", "body_mass_kg": 80.0, "waist_cm": 82.0 }),
        )
        .expect("a measurement logs");
        with_skyr_in_the_catalog(&screen).expect("a food adds");
        invoke(
            &screen,
            "log_intake",
            json!({ "form": { "date": "2026-10-09", "food": "Skyr", "quantity": 200.0 } }),
        )
        .expect("an intake logs");

        let answer = invoke(
            &screen,
            "fuel_week",
            json!({ "date": "2026-10-10", "today": "2026-10-10" }),
        );

        assert_eq!(
            answer.map(|week| (
                week["logged_days"].clone(),
                week["energy"]["per_day"].clone()
            )),
            Ok((json!(1), json!(108.0)))
        );
    }

    #[test]
    fn the_streak_crosses_to_the_screen_with_its_days_and_their_ticks() {
        let (_app, screen) = launched();
        invoke(
            &screen,
            "set_profile",
            json!({ "form": { "from": "2026-10-10", "birthday": "1985-04-12", "gender": "Male",
                "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null } }),
        )
        .expect("a profile sets");
        invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 80.0 }),
        )
        .expect("a measurement logs");

        let answer = invoke(&screen, "streak", json!({ "today": "2026-10-10" }));

        assert_eq!(
            answer,
            Ok(
                json!({ "days": 0, "longest": 0, "at_risk": false, "history": [{ "day": "2026-10-10",
                "ticks": { "weighed": true, "fuelled": false, "worked_out": false, "hydrated": false },
                "status": "Pending" }] })
            )
        );
    }

    #[test]
    fn two_measures_explored_from_the_screen_cross_with_their_series_and_pairs() {
        let (_app, screen) = launched();
        invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 80.0 }),
        )
        .expect("a measurement logs");

        let answer = invoke(
            &screen,
            "explore",
            json!({ "today": "2026-10-10", "window": "Days30", "one": "BodyMass", "other": "Eaten" }),
        );

        assert_eq!(
            answer,
            Ok(json!({ "one": [["2026-10-10", 80.0]], "other": [],
                "one_summary": { "first": 80.0, "last": 80.0, "mean": 80.0, "lowest": 80.0, "highest": 80.0 },
                "other_summary": null, "pairs": [], "correlation": null }))
        );
    }

    fn with_push_ups_and_burpees_in_the_catalog(screen: &WebviewWindow<MockRuntime>) {
        for (name, met) in [("Push up", 3.8), ("Burpee", 8.0)] {
            invoke(screen, "add_exercise", json!({ "name": name, "met": met }))
                .expect("an exercise adds");
        }
    }

    fn crafting_morning_rust(screen: &WebviewWindow<MockRuntime>) -> Result<Value, Value> {
        invoke(
            screen,
            "craft_circuit",
            json!({ "form": { "name": "Morning rust", "description": "Found on YouTube.", "steps": [
                { "exercise": "Push up", "reps": 12, "minutes": 0.5, "load_kg": 0.0 },
                { "exercise": "Burpee", "reps": null, "minutes": 1.0, "load_kg": 5.0 }
            ] } }),
        )
    }

    #[test]
    fn circuits_crafted_and_removed_from_the_screen_answer_the_lab_their_steps_frozen() {
        let (_app, screen) = launched();
        with_push_ups_and_burpees_in_the_catalog(&screen);

        let crafted = crafting_morning_rust(&screen);
        let listed = invoke(&screen, "circuits", json!({}));
        let removed = invoke(&screen, "remove_circuit", json!({ "name": "Morning rust" }));

        let morning_rust = json!([{ "name": "Morning rust", "description": "Found on YouTube.", "steps": [
            { "exercise": "Push up", "met": 3.8, "reps": 12, "minutes": 0.5, "load_kg": 0.0 },
            { "exercise": "Burpee", "met": 8.0, "reps": null, "minutes": 1.0, "load_kg": 5.0 }
        ] }]);
        assert_eq!(crafted, Ok(morning_rust.clone()));
        assert_eq!(listed, Ok(morning_rust));
        assert_eq!(removed, Ok(json!([])));
    }

    #[test]
    fn circuits_cross_to_the_screen_priced_per_set_once_weighed() {
        let (_app, screen) = launched();
        with_push_ups_and_burpees_in_the_catalog(&screen);
        crafting_morning_rust(&screen).expect("a circuit crafts");
        let on_the_day = json!({ "date": "2026-10-10" });
        let unweighed = invoke(&screen, "priced_circuits", on_the_day.clone());
        invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 80.0 }),
        )
        .expect("a measurement logs");

        let priced = invoke(&screen, "priced_circuits", on_the_day).expect("the circuits");

        assert_eq!(
            unweighed.map(|circuits| circuits[0]["burnt_per_set_kcal"].clone()),
            Ok(json!(null))
        );
        assert_eq!(priced[0]["circuit"]["name"], json!("Morning rust"));
        assert!(
            priced[0]["burnt_per_set_kcal"]
                .as_f64()
                .is_some_and(|kcal| kcal > 0.0)
        );
    }

    #[test]
    fn a_circuit_logged_from_the_screen_answers_the_days_burn_with_a_workout_per_step() {
        let (_app, screen) = launched();
        with_push_ups_and_burpees_in_the_catalog(&screen);
        crafting_morning_rust(&screen).expect("a circuit crafts");

        let burn = invoke(
            &screen,
            "log_circuit",
            json!({ "form": { "date": "2026-10-10", "circuit": "Morning rust", "sets": 3 } }),
        );

        assert_eq!(
            burn.map(|burn| {
                burn["workouts"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|done| {
                        (
                            done["workout"]["exercise"].clone(),
                            done["workout"]["sets"].clone(),
                            done["workout"]["circuit"].clone(),
                        )
                    })
                    .collect::<Vec<_>>()
            }),
            Ok(vec![
                (json!("Push up"), json!(3), json!("Morning rust")),
                (json!("Burpee"), json!(3), json!("Morning rust")),
            ])
        );
    }

    fn with_tea_in_the_catalog() -> (App<MockRuntime>, WebviewWindow<MockRuntime>) {
        let (app, screen) = launched();
        invoke(
            &screen,
            "add_drink",
            json!({ "name": "Tea", "volume_ml": 200.0, "kind": "Other" }),
        )
        .expect("a drink adds");
        (app, screen)
    }

    fn tea(volume_ml: f64) -> Value {
        json!({ "form": { "date": "2026-10-10", "drink": "Tea", "volume_ml": volume_ml } })
    }

    #[test]
    fn the_days_hydrate_starts_empty_against_the_default_base() {
        let (_app, screen) = launched();

        let answer = invoke(&screen, "hydrate_day", json!({ "date": "2026-10-10" }));

        assert_eq!(
            answer,
            Ok(json!({
                "day": "2026-10-10", "drinkings": [], "drunk_ml": 0.0, "target_ml": 2000.0, "floor_ml": 1600.0, "cap_ml": 2400.0, "verdict": "Low"
            }))
        );
    }

    #[test]
    fn a_drink_logged_from_the_screen_comes_back_in_its_days_hydrate() {
        let (_app, screen) = with_tea_in_the_catalog();

        let answer = invoke(&screen, "log_drink", tea(300.0));

        assert_eq!(
            answer,
            Ok(json!({
                "day": "2026-10-10",
                "drinkings": [[1, { "day": "2026-10-10", "drink": "Tea", "kind": "Other", "servings": 1.5, "volume_ml": 300.0 }]],
                "drunk_ml": 300.0, "target_ml": 2000.0, "floor_ml": 1600.0, "cap_ml": 2400.0, "verdict": "Low"
            }))
        );
    }

    #[test]
    fn removing_a_drinking_answers_the_days_hydrate_left() {
        let (_app, screen) = with_tea_in_the_catalog();
        invoke(&screen, "log_drink", tea(200.0)).expect("a drink logs");
        invoke(&screen, "log_drink", tea(400.0)).expect("a drink logs");

        let answer = invoke(
            &screen,
            "remove_drinking",
            json!({ "date": "2026-10-10", "id": 1 }),
        );

        assert_eq!(
            answer.map(|hydrate| hydrate["drunk_ml"].clone()),
            Ok(json!(400.0))
        );
    }

    #[test]
    fn a_drink_missing_from_the_catalog_comes_back_as_a_refusal_the_screen_understands() {
        let (_app, screen) = launched();

        let answer = invoke(&screen, "log_drink", tea(200.0));

        assert_eq!(answer, Err(json!({ "Refused": "NotADrink" })));
    }

    #[test]
    fn a_description_typed_on_the_screen_joins_its_food_drink_and_exercise() {
        let (_app, screen) = launched();

        let foods = invoke(
            &screen,
            "add_food",
            json!({ "form": { "name": "Skyr", "unit": "Gram", "portion": 100.0,
                "nutrients": { "energy_kcal": 54.0, "protein_g": 9.5, "carbs_g": 4.0, "fat_g": 0.0, "fibre_g": 0.0 },
                "description": "The plain 0% pot" } }),
        );
        let drinks = invoke(
            &screen,
            "add_drink",
            json!({ "name": "Kombucha", "volume_ml": 330.0, "kind": "Other", "description": "The ginger one" }),
        );
        let exercises = invoke(
            &screen,
            "add_exercise",
            json!({ "name": "Rowing", "met": 7.0, "description": "Concept2, damper on 5" }),
        );

        assert_eq!(
            [foods, drinks, exercises]
                .map(|answer| answer.map(|listed| listed[0]["description"].clone())),
            [
                Ok(json!("The plain 0% pot")),
                Ok(json!("The ginger one")),
                Ok(json!("Concept2, damper on 5"))
            ]
        );
    }

    #[test]
    fn the_drinks_catalog_crosses_to_the_screen_by_name() {
        let (_app, screen) = with_tea_in_the_catalog();
        invoke(
            &screen,
            "add_drink",
            json!({ "name": "Expresso", "volume_ml": 80.0, "kind": "Other" }),
        )
        .expect("a drink adds");

        let listed = invoke(&screen, "drinks", json!({}));
        let left = invoke(&screen, "remove_drink", json!({ "name": "Tea" }));

        assert_eq!(
            listed,
            Ok(json!([
                { "name": "Expresso", "volume_ml": 80.0, "kind": "Other", "description": "" },
                { "name": "Tea", "volume_ml": 200.0, "kind": "Other", "description": "" }
            ]))
        );
        assert_eq!(
            left,
            Ok(
                json!([{ "name": "Expresso", "volume_ml": 80.0, "kind": "Other", "description": "" }])
            )
        );
    }

    #[test]
    fn a_burn_target_kept_from_the_screen_stands_from_its_day_and_none_before_a_weigh_in() {
        let (_app, screen) = launched();

        let set = invoke(
            &screen,
            "set_burn_target",
            json!({ "form": { "from": "2026-10-15", "kcal": 150.0 } }),
        );
        let in_force = invoke(&screen, "burn_target_on", json!({ "date": "2026-10-20" }));
        let before = invoke(&screen, "burn_target_on", json!({ "date": "2026-10-10" }));

        assert_eq!(
            (set, in_force, before),
            (Ok(json!(150.0)), Ok(json!(150.0)), Ok(json!(null)))
        );
    }

    #[test]
    fn hydrate_left_out_from_the_screen_is_left_out_from_its_day() {
        let (_app, screen) = launched();
        let before = invoke(
            &screen,
            "hydrate_counted_on",
            json!({ "date": "2026-10-12" }),
        );

        let set = invoke(
            &screen,
            "set_hydrate_counted",
            json!({ "form": { "from": "2026-10-10", "counted": false } }),
        );
        let in_force = invoke(
            &screen,
            "hydrate_counted_on",
            json!({ "date": "2026-10-12" }),
        );

        assert_eq!(
            (before, set, in_force),
            (Ok(json!(true)), Ok(json!(false)), Ok(json!(false)))
        );
    }

    #[test]
    fn the_four_charts_cross_to_the_screen_over_their_window() {
        let (_app, screen) = launched();
        invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 80.0 }),
        )
        .expect("a measurement logs");
        let over_30_days = json!({ "today": "2026-10-10", "window": "Days30" });

        let gather = invoke(&screen, "gather_chart", over_30_days.clone());
        let charts: Vec<Result<Value, Value>> = ["fuel_chart", "hydrate_chart", "burn_chart"]
            .into_iter()
            .map(|chart| invoke(&screen, chart, over_30_days.clone()))
            .collect();

        assert_eq!(
            gather,
            Ok(
                json!({ "masses": [["2026-10-10", 80.0]], "lean_masses": [], "goal_kg": null, "goal_lean_kg": null })
            )
        );
        assert_eq!(
            charts,
            vec![
                Ok(json!({ "amounts": [], "floors": [], "caps": [] })),
                Ok(json!({ "amounts": [], "floors": [], "caps": [] })),
                Ok(json!({ "burnt": [], "targets": [] })),
            ]
        );
    }

    #[test]
    fn a_month_of_each_history_crosses_to_the_screen_with_the_month_before() {
        let (_app, screen) = launched();
        for date in ["2026-09-30", "2026-10-10"] {
            invoke(
                &screen,
                "log_measurement",
                json!({ "date": date, "body_mass_kg": 80.0 }),
            )
            .expect("a measurement logs");
        }
        let october = json!({ "month": "2026-10" });

        let gather = invoke(&screen, "gather_month", october.clone());
        let others: Vec<Result<Value, Value>> = ["fuel_month", "hydrate_month", "burn_month"]
            .into_iter()
            .map(|history| invoke(&screen, history, october.clone()))
            .collect();

        assert_eq!(
            gather,
            Ok(
                json!({ "month": "2026-10", "earlier": "2026-09", "rows": [{ "day": "2026-10-10",
                "body_mass_kg": 80.0, "waist_cm": null, "lean_mass_kg": null, "fat_mass_kg": null, "fat_ratio": null }] })
            )
        );
        assert!(
            others
                .iter()
                .all(|month| month.as_ref().is_ok_and(|month| month["rows"] == json!([])))
        );
    }

    #[test]
    fn insights_cross_to_the_screen_with_each_habits_tally() {
        let (_app, screen) = launched();
        invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 80.0 }),
        )
        .expect("a measurement logs");

        let answer = invoke(
            &screen,
            "insights",
            json!({ "today": "2026-10-10", "window": "Days30" }),
        );

        let nothing = json!({ "low": 0, "in_band": 0, "high": 0 });
        assert_eq!(
            answer,
            Ok(
                json!({ "days": 1, "weighed_days": 1, "fuelled_days": 0, "energy": nothing,
                "protein": nothing, "fat": nothing, "fibre": nothing, "carbs": nothing,
                "hydrated_days": 0, "hydrate": nothing, "active_days": 0, "burnt_per_active_day": null })
            )
        );
    }

    #[test]
    fn erasing_everything_from_the_screen_leaves_a_first_launch() {
        let (_app, screen) = launched();
        invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 80.0 }),
        )
        .expect("a measurement logs");

        let erased = invoke(&screen, "erase_everything", json!({}));
        let week_left = invoke(&screen, "gather_week", json!({ "date": "2026-10-10" }));

        assert_eq!(erased, Ok(json!(null)));
        assert_eq!(week_left.map(|week| week["days"].clone()), Ok(json!([])));
    }

    #[test]
    fn recommendations_cross_to_the_screen_for_the_profile_in_force() {
        let (_app, screen) = launched();
        invoke(
            &screen,
            "set_profile",
            json!({ "form": { "from": "2026-03-01", "birthday": "1985-04-12", "gender": "Male",
                "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null } }),
        )
        .expect("a profile set");

        let advised = invoke(&screen, "recommended_on", json!({ "date": "2026-10-11" }))
            .expect("recommendations");

        assert_eq!(advised["fuel"]["protein_g_per_kg"], json!(1.6));
        assert_eq!(
            advised["usual_day"],
            json!({ "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 6.0 })
        );
    }
}
