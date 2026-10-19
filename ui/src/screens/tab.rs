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

/// Where the tab keeps its catalog, on a page of its own.
#[component]
pub fn OpenTheGrimoire(href: &'static str) -> impl IntoView {
    view! {
        <A href=href attr:class="cta ghost grimoire">
            "Open the grimoire"
        </A>
    }
}

#[component]
pub fn ExploreInStats() -> impl IntoView {
    view! {
        <A href="/stats" attr:class="cta ghost explore">
            "Explore the numbers"
        </A>
    }
}
