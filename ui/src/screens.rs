pub mod burn;
pub mod catalog;
pub mod fields;
pub mod fuel;
pub mod home;
pub mod lines;
pub mod meals;
pub mod profile;
pub mod settings;
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
