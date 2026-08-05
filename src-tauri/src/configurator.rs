use std::sync::Mutex;

use grind_sqlite::Sqlite;
use tauri::Manager;

use crate::commands::{self, DraftBackup, Records};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let folder = app.path().app_data_dir()?;
            std::fs::create_dir_all(&folder)?;
            let records =
                Sqlite::open(&folder.join("grindstone.sqlite")).map_err(|broken| broken.0)?;
            app.manage(Records(Mutex::new(records)));
            let documents = app.path().document_dir()?;
            std::fs::create_dir_all(&documents)?;
            app.manage(DraftBackup(documents.join("grindstone-draft.json")));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::log_measurement,
            commands::recent_days,
            commands::remove_measurement,
            commands::set_profile,
            commands::profile_on,
            commands::set_goals,
            commands::goals_on,
            commands::gather_card,
            commands::import_draft,
            commands::burn_day,
            commands::log_workout,
            commands::remove_workout,
            commands::set_day_off_the_clock,
            commands::exercises,
            commands::add_exercise,
            commands::remove_exercise,
            commands::set_usual_day,
            commands::usual_day_on,
            commands::fuel_day,
            commands::log_intake,
            commands::remove_intake,
            commands::foods,
            commands::add_food,
            commands::remove_food,
            commands::set_fuel_targets,
            commands::fuel_targets_on,
            commands::meals,
            commands::preview_meal,
            commands::cook_meal,
            commands::remove_meal,
        ])
        .run(tauri::generate_context!())
        .expect("grindstone failed to start");
}
