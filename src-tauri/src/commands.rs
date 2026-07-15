use std::sync::{Mutex, PoisonError};

use grind_core::failure::Failure;
use grind_core::for_setting_up_the_profile::{self, GoalsForm, ProfileForm};
use grind_core::for_tracking;
use grind_core::goals::Goals;
use grind_core::ledger::Day;
use grind_core::profile::Profile;
use grind_core::weigh_in::WeighIn;
use grind_sqlite::Sqlite;
use tauri::State;

pub struct Records(pub Mutex<Sqlite>);

#[tauri::command]
pub fn recent_weigh_ins(records: State<Records>) -> Result<Vec<WeighIn>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_tracking::recent_weigh_ins(&*records)
}

#[tauri::command]
pub fn log_weigh_in(
    records: State<Records>,
    date: String,
    kg: f64,
) -> Result<Vec<WeighIn>, Failure> {
    let records = records.0.lock().unwrap_or_else(PoisonError::into_inner);
    for_tracking::log_weigh_in(&*records, &date, kg)
}

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
    Err(Failure::Broken("not wired yet".into()))
}

#[tauri::command(rename_all = "snake_case")]
pub fn goals_on(records: State<Records>, date: String) -> Result<Option<Goals>, Failure> {
    Err(Failure::Broken("not wired yet".into()))
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

    fn launched() -> (App<MockRuntime>, WebviewWindow<MockRuntime>) {
        let records = Sqlite::in_memory().expect("an in-memory database opens");
        let app = mock_builder()
            .manage(Records(Mutex::new(records)))
            .invoke_handler(tauri::generate_handler![
                recent_weigh_ins,
                log_weigh_in,
                log_measurement,
                recent_days,
                remove_measurement,
                set_profile,
                profile_on,
                set_goals,
                goals_on,
            ])
            .build(mock_context(noop_assets()))
            .expect("the app builds on the mock runtime");
        let screen = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("the screen opens");
        (app, screen)
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
    fn a_first_launch_answers_no_weigh_in() {
        let (_app, screen) = launched();

        assert_eq!(
            invoke(&screen, "recent_weigh_ins", json!({})),
            Ok(json!([]))
        );
    }

    #[test]
    fn logging_a_weigh_in_answers_the_recent_ones_as_the_screen_reads_them() {
        let (_app, screen) = launched();

        let answer = invoke(
            &screen,
            "log_weigh_in",
            json!({ "date": "2026-10-03", "kg": 72.149 }),
        );

        assert_eq!(answer, Ok(json!([{ "date": "2026-10-03", "kg": 72.15 }])));
    }

    #[test]
    fn an_impossible_mass_comes_back_as_a_refusal_the_screen_understands() {
        let (_app, screen) = launched();

        let answer = invoke(
            &screen,
            "log_weigh_in",
            json!({ "date": "2026-10-03", "kg": 0.0 }),
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
}
