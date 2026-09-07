pub mod burn;
pub mod catalog;
pub mod charts;
pub mod circuits;
pub mod fields;
pub mod fuel;
pub mod gauge;
pub mod home;
pub mod hydrate;
pub mod lines;
pub mod meals;
pub mod profile;
pub mod references;
pub mod settings;
pub mod stats;
pub mod tracker;

pub fn today() -> String {
    let now = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        now.get_full_year(),
        now.get_month() + 1,
        now.get_date()
    )
}
