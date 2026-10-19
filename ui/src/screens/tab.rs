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

#[component]
pub fn ExploreTheNumbers() -> impl IntoView {
    view! {
        <A href="/stats" attr:class="cta ghost explore">
            "🎱 Explore the numbers"
        </A>
    }
}

/// Where a tab with a catalog ends: its grimoire, a page of its own, beside the numbers.
#[component]
pub fn GrimoireBesideTheNumbers(grimoire: &'static str) -> impl IntoView {
    view! {
        <div class="side-by-side">
            <A href=grimoire attr:class="cta ghost grimoire">
                "📚 Open the grimoire"
            </A>
            <ExploreTheNumbers />
        </div>
    }
}
