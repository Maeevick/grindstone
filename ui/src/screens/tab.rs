use leptos::prelude::*;
use leptos_router::components::A;

use crate::screens::fields::DateField;

/// The day a tab shows and logs to, today when the tab opens.
#[component]
pub fn DayCard(day: RwSignal<String>) -> impl IntoView {
    view! {
        <section class="card day">
            <DateField label="Day" name="day" value=day />
        </section>
    }
}

/// Where a tab with a catalog ends: its grimoire, a page of its own.
#[component]
pub fn OpenTheGrimoire(grimoire: &'static str) -> impl IntoView {
    view! {
        <A href=grimoire attr:class="cta ghost grimoire">
            "📚 Open the grimoire"
        </A>
    }
}
