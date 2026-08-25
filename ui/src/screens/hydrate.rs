use leptos::prelude::*;

#[component]
pub fn HydrateDay(day: RwSignal<String>) -> impl IntoView {
    let _ = day;
    view! { <section class="card"></section> }
}

#[component]
pub fn DrinksCatalog() -> impl IntoView {
    view! { <section class="card"></section> }
}

#[component]
pub fn HydrateBaseCard() -> impl IntoView {
    view! { <section class="card"></section> }
}
