pub mod burn;
pub mod catalog;
pub mod cells;
pub mod charts;
pub mod circuits;
pub mod entries;
pub mod fields;
pub mod fuel;
pub mod gather;
pub mod gauge;
pub mod header;
pub mod home;
pub mod hydrate;
pub mod lines;
pub mod meals;
pub mod profile;
pub mod references;
pub mod settings;
pub mod stats;
pub mod tab;

use leptos::prelude::{Get, RwSignal, Update, use_context};

/// Told of every change to the records, so what reads today's reads them again.
#[derive(Clone, Copy, Default)]
pub struct Recorded(RwSignal<u32>);

impl Recorded {
    /// The app's, or a lone one nobody listens to on a screen mounted alone.
    pub fn here() -> Self {
        use_context::<Self>().unwrap_or_default()
    }

    pub fn told(self) {
        self.0.update(|changes| *changes += 1);
    }

    pub fn heard(self) -> u32 {
        self.0.get()
    }
}

pub fn today() -> String {
    let now = js_sys::Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        now.get_full_year(),
        now.get_month() + 1,
        now.get_date()
    )
}
