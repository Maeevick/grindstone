use grind_core::for_reviewing_progress::Explored;
use grind_core::for_reviewing_progress::Window;
use grind_core::stats::{MEASURES, Measure, figure_of};
use leptos::prelude::*;

use crate::chart::{axis_of, days_along, dots_of, line_of};
use crate::screens::charts::{HEIGHT, Line, Plot, Plotted, WIDTH, span_of, ticks_on};
use crate::screens::{lines, today};
use crate::tauri;

const WINDOWS: [(Window, &str); 3] = [
    (Window::Days30, "30 days"),
    (Window::Days90, "90 days"),
    (Window::Everything, "∞"),
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

fn timeline_plot(explored: &Explored, (one, other): (Measure, Measure)) -> Option<Plotted> {
    let span = span_of(explored.one.iter().chain(&explored.other))?;
    let axis_over =
        |points: &[(chrono::NaiveDate, f64)]| axis_of(points.iter().map(|(_, value)| *value));
    let (one_axis, other_axis) = (axis_over(&explored.one), axis_over(&explored.other));
    Some(Plotted {
        lines: [
            one_axis.map(|axis| Line {
                points: line_of(&explored.one, span, axis.scale(), WIDTH, HEIGHT),
                class: "one",
            }),
            other_axis.map(|axis| Line {
                points: line_of(&explored.other, span, axis.scale(), WIDTH, HEIGHT),
                class: "other",
            }),
        ]
        .into_iter()
        .flatten()
        .collect(),
        ticks: one_axis
            .map(|axis| ticks_on(axis, |value| figure_of(one, value), "one"))
            .unwrap_or_default(),
        right_ticks: other_axis
            .map(|axis| ticks_on(axis, |value| figure_of(other, value), "other"))
            .unwrap_or_default(),
        days: days_along(span, WIDTH),
        legend: vec![
            ("one", lines::measure_name(one)),
            ("other", lines::measure_name(other)),
        ],
        ..Plotted::default()
    })
}

fn scatter_plot(pairs: &[(f64, f64)], other: Measure) -> Option<Plotted> {
    let xs = axis_of(pairs.iter().map(|(x, _)| *x))?;
    let ys = axis_of(pairs.iter().map(|(_, y)| *y))?;
    Some(Plotted {
        dots: dots_of(pairs, xs.scale(), ys.scale(), WIDTH, HEIGHT),
        ticks: ticks_on(ys, |value| figure_of(other, value), "other"),
        ..Plotted::default()
    })
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
            <h2>"Stats: the hunting ground"</h2>
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
                                let (one_measure, other_measure) = (one.get_untracked(), other.get_untracked());
                                view! {
                                    <div class="timeline">
                                        <Plot plotted=timeline_plot(&explored, (one_measure, other_measure)) />
                                    </div>
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
                                    <div class="scatter">
                                        <Plot plotted=scatter_plot(&explored.pairs, other_measure) />
                                    </div>
                                    <p class="said">{lines::together(explored.correlation, explored.pairs.len())}</p>
                                    {explored
                                        .correlation
                                        .is_some()
                                        .then(|| view! { <p class="quote">{lines::HUNTING}</p> })}
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
