use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use grind_core::burn::Burn;
use grind_core::exercise::Exercise;
use grind_core::failure::Failure;
use grind_core::for_burning::{self, OffTheClockForm, WorkoutForm};
use grind_core::for_cataloguing;
use grind_core::for_importing::{self, Imported};
use grind_core::for_reviewing_progress::{self, GatherCard, Window};
use grind_core::for_setting_up_the_profile::{self, GoalsForm, ProfileForm, UsualDayForm};
use grind_core::for_tracking;
use grind_core::goals::Goals;
use grind_core::ledger::Day;
use grind_core::profile::Profile;
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
) -> Result<Vec<Day>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_tracking::log_measurement(&*records, &date, body_mass_kg, waist_cm)
}

#[tauri::command(rename_all = "snake_case")]
pub fn recent_days(records: State<Records>) -> Result<Vec<Day>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_tracking::recent_days(&*records)
}

#[tauri::command(rename_all = "snake_case")]
pub fn remove_measurement(records: State<Records>, date: String) -> Result<Vec<Day>, Failure> {
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
) -> Result<Imported, ImportFailure> {
    let backup = std::fs::read_to_string(&backup.0)
        .map_err(|_| ImportFailure::NoBackupFound(backup.0.display().to_string()))?;
    let drafted = drafted(&backup).map_err(ImportFailure::Draft)?;
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_importing::import_drafted(&*records, &*records, &*records, &drafted)
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
) -> Result<Vec<Exercise>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_cataloguing::add_exercise(&*records, &name, met)
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
                recent_days,
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
    fn logging_a_measurement_answers_the_recent_days_as_the_screen_reads_them() {
        let (_app, screen) = launched();

        let answer = invoke(
            &screen,
            "log_measurement",
            json!({ "date": "2026-10-10", "body_mass_kg": 80.0, "waist_cm": 80.0 }),
        );

        assert_eq!(
            answer,
            Ok(
                json!([{ "day": "2026-10-10", "body_mass_kg": 80.0, "waist_cm": 80.0, "composition": null, "age": null }])
            )
        );
    }

    #[test]
    fn the_recent_days_start_empty_on_a_first_launch() {
        let (_app, screen) = launched();

        assert_eq!(invoke(&screen, "recent_days", json!({})), Ok(json!([])));
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
            answer.map(|days| days.as_array().map(Vec::len)),
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

        let answer = invoke(&screen, "import_draft", json!({}));

        std::fs::remove_file(&backup).expect("the test backup is removed");
        assert_eq!(
            answer,
            Ok(
                json!({ "measurements": 1, "profiles": 1, "goals": 1, "exercises": 0, "workouts": 0, "foods": 0, "intakes": 0 })
            )
        );
    }

    #[test]
    fn importing_with_no_backup_pushed_says_where_it_was_expected() {
        let (_app, screen) = launched();

        let answer = invoke(&screen, "import_draft", json!({}));

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

        let answer = invoke(&screen, "import_draft", json!({}));

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

        assert_eq!(listed, Ok(json!([{ "name": "Walking", "met": 4.0 }])));
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
}
