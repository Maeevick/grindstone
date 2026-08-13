use grind_core::burn::Burn;
use leptos::prelude::*;

#[component]
pub fn CircuitCrafter() -> impl IntoView {}

#[component]
pub fn CircuitLog(day: RwSignal<String>, on_logged: Callback<Burn>) -> impl IntoView {
    let _ = (day, on_logged);
}
