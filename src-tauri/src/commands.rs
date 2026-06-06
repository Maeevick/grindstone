use std::sync::Mutex;

use grind_core::failure::Failure;
use grind_core::for_tracking;
use grind_core::weigh_in::WeighIn;
use grind_sqlite::Sqlite;
use tauri::State;

pub struct Records(pub Mutex<Sqlite>);

#[tauri::command]
pub fn recent_weigh_ins(records: State<Records>) -> Result<Vec<WeighIn>, Failure> {
    let records = records
        .0
        .lock()
        .map_err(|poisoned| Failure::Broken(poisoned.to_string()))?;
    for_tracking::recent_weigh_ins(&*records)
}

#[tauri::command]
pub fn log_weigh_in(
    records: State<Records>,
    date: String,
    kg: f64,
) -> Result<Vec<WeighIn>, Failure> {
    let records = records
        .0
        .lock()
        .map_err(|poisoned| Failure::Broken(poisoned.to_string()))?;
    for_tracking::log_weigh_in(&*records, &date, kg)
}
