use grind_core::for_reviewing_progress::Window;
use grind_core::stats::{MEASURES, Measure};
use leptos::prelude::*;

use crate::chart::{scatter_of, timeline_of};
use crate::screens::{lines, today};
use crate::tauri;

const WINDOWS: [(Window, &str); 3] = [
    (Window::Days30, "30 days"),
    (Window::Days90, "90 days"),
    (Window::Everything, "Everything"),
];

#[component]
fn MeasurePicker(name: &'static str, chosen: RwSignal<Measure>) -> impl IntoView {
    view! {
        <label class="field">
            {name}
            <select
                name=name
                on:change=move |event| {
                    let picked = event_target_value(&event);
                    if let Some(measure) = MEASURES.iter().find(|measure| format!("{measure:?}") == picked) {
                        chosen.set(*measure);
                    }
                }
            >
                {MEASURES
                    .iter()
                    .map(|measure| {
                        view! {
                            <option
                                value=format!("{measure:?}")
                                selected=move || chosen.get() == *measure
                            >
                                {lines::measure_name(*measure)}
                            </option>
                        }
                    })
                    .collect_view()}
            </select>
        </label>
    }
}

#[component]
pub fn Stats() -> impl IntoView {
    let window = RwSignal::new(Window::Days90);
    let one = RwSignal::new(Measure::BodyMass);
    let other = RwSignal::new(Measure::Eaten);
    let explored = LocalResource::new(move || {
        let (window, measures) = (window.get(), (one.get(), other.get()));
        async move { tauri::explore(&today(), window, measures).await }
    });
    view! {
        <section class="card">
            <h2>"Stats: Veil's hunting ground"</h2>
            <div class="row">
                <MeasurePicker name="One" chosen=one />
                <MeasurePicker name="Other" chosen=other />
            </div>
            <div class="pills">
                {WINDOWS
                    .into_iter()
                    .map(|(named, label)| {
                        view! {
                            <button
                                type="button"
                                class=format!("pill window-{named:?}")
                                class:chosen=move || window.get() == named
                                on:click=move |_| window.set(named)
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
                {move || {
                    explored
                        .get()
                        .map(|answer| match answer {
                            Ok(explored) => {
                                let days = explored.one.iter().chain(&explored.other).map(|(day, _)| *day);
                                let span = days
                                    .clone()
                                    .min()
                                    .zip(days.max());
                                let (one_measure, other_measure) = (one.get_untracked(), other.get_untracked());
                                view! {
                                    <svg class="timeline chart" viewBox="0 0 360 180" preserveAspectRatio="none">
                                        {span
                                            .map(|span| {
                                                view! {
                                                    <polyline class="one" points=timeline_of(&explored.one, span, 360.0, 180.0) />
                                                    <polyline class="other" points=timeline_of(&explored.other, span, 360.0, 180.0) />
                                                }
                                            })}
                                    </svg>
                                    <ul class="figures">
                                        <li class="one">
                                            {explored
                                                .one_summary
                                                .map(|summary| lines::summed_up(one_measure, &summary))
                                                .unwrap_or_else(|| lines::NOTHING_TO_EXPLORE.into())}
                                        </li>
                                        <li class="other">
                                            {explored
                                                .other_summary
                                                .map(|summary| lines::summed_up(other_measure, &summary))
                                                .unwrap_or_else(|| lines::NOTHING_TO_EXPLORE.into())}
                                        </li>
                                    </ul>
                                    <svg class="scatter chart" viewBox="0 0 360 180" preserveAspectRatio="none">
                                        {scatter_of(&explored.pairs, 360.0, 180.0)
                                            .into_iter()
                                            .map(|(x, y)| view! { <circle cx=x cy=y r="3" /> })
                                            .collect_view()}
                                    </svg>
                                    <p class="said">{lines::together(explored.correlation, explored.pairs.len())}</p>
                                }
                                    .into_any()
                            }
                            Err(failure) => {
                                view! { <p class="bad">{lines::failure(&failure)}</p> }.into_any()
                            }
                        })
                }}
            </Transition>
        </section>
    }
}
