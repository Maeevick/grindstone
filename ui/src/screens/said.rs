use leptos::prelude::*;

/// What a screen says once something is done, in a line there before it is said, so a screen reader listening
/// to it announces the words when they come.
pub(crate) fn said_line(said: RwSignal<Option<String>>) -> impl IntoView {
    view! { <p class="said" role="status">{move || said.get()}</p> }
}

/// Why a screen refused, in a line there before it is said, so a screen reader announces the words at once.
pub(crate) fn refused_line(said: RwSignal<Option<String>>) -> impl IntoView {
    view! { <p class="bad" role="alert">{move || said.get()}</p> }
}
