use std::sync::Mutex;

use grind_sqlite::Sqlite;
use tauri::Manager;

use crate::commands::{self, CopyFile, Records, RestoreFile};

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
            app.manage(CopyFile(documents.join("grindstone-export.json")));
            app.manage(RestoreFile(documents.join("grindstone-import.json")));
            Ok(())
        })
        .invoke_handler(commands::handlers())
        .run(tauri::generate_context!())
        .expect("grindstone failed to start");
}
