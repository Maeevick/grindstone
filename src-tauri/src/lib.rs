use std::sync::Mutex;

use grind_core::failure::Failure;
use grind_core::weigh_in::{WeighIn, weigh_in};
use grind_sqlite as storage;
use rusqlite::Connection;
use tauri::{Manager, State};

const RECENT_WEIGH_INS: u32 = 14;

struct Database(Mutex<Connection>);

fn broken(error: impl ToString) -> Failure {
    Failure::Broken(error.to_string())
}

#[tauri::command]
fn recent_weigh_ins(database: State<Database>) -> Result<Vec<WeighIn>, Failure> {
    let connection = database.0.lock().map_err(broken)?;
    storage::recent_weigh_ins(&connection, RECENT_WEIGH_INS).map_err(broken)
}

#[tauri::command]
fn log_weigh_in(database: State<Database>, date: String, kg: f64) -> Result<Vec<WeighIn>, Failure> {
    let weighed = weigh_in(&date, kg).map_err(Failure::Refused)?;
    let connection = database.0.lock().map_err(broken)?;
    storage::save_weigh_in(&connection, &weighed).map_err(broken)?;
    storage::recent_weigh_ins(&connection, RECENT_WEIGH_INS).map_err(broken)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let folder = app.path().app_data_dir()?;
            std::fs::create_dir_all(&folder)?;
            let connection = Connection::open(folder.join("grindstone.sqlite"))?;
            storage::migrate(&connection)?;
            app.manage(Database(Mutex::new(connection)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![recent_weigh_ins, log_weigh_in])
        .run(tauri::generate_context!())
        .expect("grindstone failed to start");
}
