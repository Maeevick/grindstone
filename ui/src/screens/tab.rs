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
pub fn ExploreInStats() -> impl IntoView {
    view! {
        <A href="/stats" attr:class="cta ghost explore">
            "Explore in Stats"
        </A>
    }
}
