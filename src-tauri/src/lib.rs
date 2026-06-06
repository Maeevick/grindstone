mod commands;
mod configurator;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    configurator::run();
}
