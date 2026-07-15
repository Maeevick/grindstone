use std::sync::Mutex;

use grind_sqlite::Sqlite;
use tauri::Manager;

use crate::commands::{self, Records};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let folder = app.path().app_data_dir()?;
            std::fs::create_dir_all(&folder)?;
            let records =
                Sqlite::open(&folder.join("grindstone.sqlite")).map_err(|broken| broken.0)?;
            app.manage(Records(Mutex::new(records)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::recent_weigh_ins,
            commands::log_weigh_in,
            commands::log_measurement,
            commands::recent_days,
            commands::remove_measurement,
            commands::set_profile,
            commands::profile_on,
            commands::set_goals,
            commands::goals_on,
            commands::gather_card,
            commands::import_draft_gather,
        ])
        .run(tauri::generate_context!())
        .expect("grindstone failed to start");
}
