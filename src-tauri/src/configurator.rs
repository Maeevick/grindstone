use std::sync::Mutex;

use grind_sqlite::Sqlite;
use tauri::Manager;

use crate::commands::{self, DraftBackup, Records};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
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
            commands::gather_week,
            commands::remove_measurement,
            commands::set_profile,
            commands::profile_on,
            commands::set_goals,
            commands::goals_on,
            commands::gather_card,
            commands::import_draft,
            commands::burn_day,
            commands::burn_week,
            commands::log_workout,
            commands::remove_workout,
            commands::set_day_off_the_clock,
            commands::exercises,
            commands::add_exercise,
            commands::remove_exercise,
            commands::set_usual_day,
            commands::usual_day_on,
            commands::fuel_day,
            commands::fuel_week,
            commands::log_intake,
            commands::log_plate,
            commands::remove_intake,
            commands::foods,
            commands::add_food,
            commands::remove_food,
            commands::set_fuel_targets,
            commands::fuel_targets_on,
            commands::meals,
            commands::cook_meal,
            commands::remove_meal,
            commands::streak,
            commands::explore,
            commands::gather_chart,
            commands::fuel_chart,
            commands::hydrate_chart,
            commands::burn_chart,
            commands::gather_month,
            commands::fuel_month,
            commands::hydrate_month,
            commands::burn_month,
            commands::insights,
            commands::erase_everything,
            commands::recommended_on,
            commands::circuits,
            commands::priced_circuits,
            commands::craft_circuit,
            commands::remove_circuit,
            commands::log_circuit,
            commands::log_session,
            commands::hydrate_day,
            commands::hydrate_week,
            commands::log_drink,
            commands::remove_drinking,
            commands::drinks,
            commands::add_drink,
            commands::remove_drink,
            commands::set_hydrate_target,
            commands::hydrate_target_on,
            commands::scope_on,
            commands::set_hydrate_counted,
            commands::hydrate_counted_on,
            commands::set_burn_target,
            commands::burn_target_on,
        ])
        .run(tauri::generate_context!())
        .expect("grindstone failed to start");
}
