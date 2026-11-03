use leptos::prelude::*;
use leptos_router::components::A;

use crate::screens::fields::DateField;

/// The day a tab shows and logs to, today when the tab opens.
#[component]
pub fn DayCard(day: RwSignal<String>) -> impl IntoView {
    view! {
        <section class="card day">
            <h2>"Pick the day"</h2>
            <DateField label="Day" name="day" value=day />
        </section>
    }
}

/// A form card's title row: the tab's name, and on its right the way to its grimoire, a page of its own.
#[component]
pub fn FormTitle(title: &'static str, grimoire: &'static str) -> impl IntoView {
    view! {
        <div class="card-title">
            <h2>{title}</h2>
            <A href=grimoire attr:class="grimoire">
                <span class="caption">"open the grimoire"</span>
                <span class="icon">"📚"</span>
            </A>
        </div>
    }
}
