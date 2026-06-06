use std::sync::Mutex;

use grind_core::failure::Failure;
use grind_core::for_storing_weigh_ins::ForStoringWeighIns;
use grind_core::weigh_in::{WeighIn, weigh_in};
use grind_sqlite::Sqlite;
use tauri::{Manager, State};

const RECENT_WEIGH_INS: u32 = 14;

struct Database(Mutex<Sqlite>);

fn broken(error: impl ToString) -> Failure {
    Failure::Broken(error.to_string())
}

#[tauri::command]
fn recent_weigh_ins(database: State<Database>) -> Result<Vec<WeighIn>, Failure> {
    let records = database.0.lock().map_err(broken)?;
    records
        .recent(RECENT_WEIGH_INS)
        .map_err(|failed| Failure::Broken(failed.0))
}

#[tauri::command]
fn log_weigh_in(database: State<Database>, date: String, kg: f64) -> Result<Vec<WeighIn>, Failure> {
    let weighed = weigh_in(&date, kg).map_err(Failure::Refused)?;
    let records = database.0.lock().map_err(broken)?;
    records
        .save(&weighed)
        .map_err(|failed| Failure::Broken(failed.0))?;
    records
        .recent(RECENT_WEIGH_INS)
        .map_err(|failed| Failure::Broken(failed.0))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let folder = app.path().app_data_dir()?;
            std::fs::create_dir_all(&folder)?;
            let records =
                Sqlite::open(&folder.join("grindstone.sqlite")).map_err(|failed| failed.0)?;
            app.manage(Database(Mutex::new(records)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![recent_weigh_ins, log_weigh_in])
        .run(tauri::generate_context!())
        .expect("grindstone failed to start");
}
