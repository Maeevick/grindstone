use std::fmt::Debug;

use leptos::prelude::*;

/// The pills choosing one of a few, each classed by its group's prefix and its choice, the chosen one marked.
pub(crate) fn pill_buttons<T>(
    prefix: &'static str,
    choices: impl IntoIterator<Item = (T, &'static str)>,
    chosen: RwSignal<T>,
) -> impl IntoView
where
    T: Copy + PartialEq + Debug + Send + Sync + 'static,
{
    choices
        .into_iter()
        .map(|(named, label)| {
            view! {
                <button
                    type="button"
                    class=format!("pill {prefix}-{named:?}")
                    class:chosen=move || chosen.get() == named
                    on:click=move |_| chosen.set(named)
                >
                    {label}
                </button>
            }
        })
        .collect_view()
}
