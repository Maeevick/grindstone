use std::sync::{Mutex, PoisonError};

use grind_core::failure::Failure;
use grind_core::for_tracking;
use grind_core::ledger::Day;
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
                log_measurement
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
}
