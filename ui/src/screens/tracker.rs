use grind_core::figures::{Figure, shown};
use grind_core::ledger::Day;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::burn::BurnDay;
use crate::screens::fields::{DateField, FigureField};
use crate::screens::{lines, today};
use crate::tauri;
use crate::typed::figure_typed;

fn shown_or_blank(figure: Option<Figure>) -> String {
    figure.map(shown).unwrap_or_default()
}

#[component]
fn DayRow(day: Day, days: RwSignal<Vec<Day>>, said: RwSignal<Option<String>>) -> impl IntoView {
    let date = day.day.to_string();
    let remove = move |_| {
        let date = date.clone();
        spawn_local(async move {
            match tauri::remove_measurement(&date).await {
                Ok(recent) => days.set(recent),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <tr>
            <td>{day.day.to_string()}</td>
            <td>{shown(Figure::Mass(day.body_mass_kg))}</td>
            <td>{shown_or_blank(day.waist_cm.map(Figure::Length))}</td>
            <td>{shown_or_blank(day.composition.map(|split| Figure::Ratio(split.fat_ratio)))}</td>
            <td>{shown_or_blank(day.composition.map(|split| Figure::Mass(split.lean_mass_kg)))}</td>
            <td>
                <button class="remove" type="button" on:click=remove aria-label="Remove this day">
                    "×"
                </button>
            </td>
        </tr>
    }
}

#[component]
fn GatherDay(day: RwSignal<String>) -> impl IntoView {
    let mass = RwSignal::new(String::new());
    let waist = RwSignal::new(String::new());
    let days = RwSignal::new(Vec::<Day>::new());
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        match tauri::recent_days().await {
            Ok(recent) => days.set(recent),
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
    });
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let date = day.get_untracked();
        let body_mass_kg = figure_typed(&mass.get_untracked()).unwrap_or(f64::NAN);
        let waist_cm = figure_typed(&waist.get_untracked());
        spawn_local(async move {
            match tauri::log_measurement(&date, body_mass_kg, waist_cm).await {
                Ok(recent) => {
                    days.set(recent);
                    mass.set(String::new());
                    waist.set(String::new());
                    said.set(None);
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Gather"</h2>
            <form class="measurement" on:submit=log>
                <div class="row">
                    <FigureField label="Body mass (kg)" name="mass" value=mass />
                    <FigureField label="Waist (cm)" name="waist" value=waist />
                </div>
                <button class="cta" type="submit">
                    "Gather"
                </button>
                {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
            </form>
        </section>
        <section class="card">
            <h2>"The last days"</h2>
            <table>
                <thead>
                    <tr>
                        <th>"Day"</th>
                        <th>"Mass"</th>
                        <th>"Waist"</th>
                        <th>"Fat"</th>
                        <th>"Lean"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    <For
                        each=move || days.get()
                        key=|day| format!("{day:?}")
                        children=move |day| view! { <DayRow day=day days=days said=said /> }
                    />
                </tbody>
            </table>
        </section>
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Segment {
    Gather,
    Burn,
}

const SEGMENTS: [(Segment, &str); 2] = [(Segment::Gather, "Gather"), (Segment::Burn, "Burn")];

#[component]
pub fn Tracker() -> impl IntoView {
    let day = RwSignal::new(today());
    let segment = RwSignal::new(Segment::Gather);
    view! {
        <section class="card day">
            <DateField label="Day" name="day" value=day />
            <div class="pills">
                {SEGMENTS
                    .into_iter()
                    .map(|(named, label)| {
                        view! {
                            <button
                                type="button"
                                class=format!("pill segment-{named:?}")
                                class:chosen=move || segment.get() == named
                                on:click=move |_| segment.set(named)
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
        {move || match segment.get() {
            Segment::Gather => view! { <GatherDay day=day /> }.into_any(),
            Segment::Burn => view! { <BurnDay day=day /> }.into_any(),
        }}
    }
}
