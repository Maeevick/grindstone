use grind_core::figures::{Figure, bare, signed, typed_back};
use grind_core::for_tracking::GatherWeek;
use grind_core::ledger::Day;
use grind_core::tendency::{Change, Heading};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;

use crate::screens::burn::BurnDay;
use crate::screens::fields::{DateField, FigureField};
use crate::screens::fuel::FuelDay;
use crate::screens::hydrate::HydrateDay;
use crate::screens::{lines, today};
use crate::tauri;
use crate::typed::figures_typed;

fn bare_or_blank(figure: Option<Figure>) -> String {
    figure.map(bare).unwrap_or_default()
}

fn tendency_cell(change: Option<Change>, figure: fn(f64) -> Figure) -> impl IntoView {
    let (class, text) = match change {
        None => ("", String::new()),
        Some(change) => match change.heading {
            Heading::Steady => ("steady", "-".to_string()),
            Heading::Toward => ("toward", signed(figure(change.by))),
            Heading::Away => ("away", signed(figure(change.by))),
            Heading::Unaimed => ("unaimed", signed(figure(change.by))),
        },
    };
    view! { <td class=class>{text}</td> }
}

#[component]
fn DayRow(day: Day, chosen: bool, remove: Callback<String>) -> impl IntoView {
    let date = day.day.to_string();
    view! {
        <tr class:chosen=chosen>
            <td>{day.day.to_string()}</td>
            <td>{bare(Figure::Mass(day.body_mass_kg))}</td>
            <td>{bare_or_blank(day.waist_cm.map(Figure::Length))}</td>
            <td>{bare_or_blank(day.composition.map(|split| Figure::Ratio(split.fat_ratio)))}</td>
            <td>{bare_or_blank(day.composition.map(|split| Figure::Mass(split.lean_mass_kg)))}</td>
            <td>
                <button
                    class="remove"
                    type="button"
                    on:click=move |_| remove.run(date.clone())
                    aria-label="Remove this day"
                >
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
    let week = RwSignal::new(None::<GatherWeek>);
    let said = RwSignal::new(None::<String>);
    let shown_week = Callback::new(move |answer: GatherWeek| {
        mass.set(
            answer
                .day
                .map(|own| typed_back(own.body_mass_kg))
                .unwrap_or_default(),
        );
        waist.set(
            answer
                .day
                .and_then(|own| own.waist_cm)
                .map(typed_back)
                .unwrap_or_default(),
        );
        week.set(Some(answer));
    });
    let load = move |date: String| {
        spawn_local(async move {
            match tauri::gather_week(&date).await {
                Ok(answer) => shown_week.run(answer),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    Effect::new(move |_| load(day.get()));
    let remove = Callback::new(move |date: String| {
        let chosen = day.get_untracked();
        spawn_local(async move {
            match tauri::remove_measurement(&date).await {
                Ok(answer) if date == chosen => shown_week.run(answer),
                Ok(_) => load(chosen),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    });
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let date = day.get_untracked();
        let Some([body_mass_kg, waist_cm]) =
            figures_typed([&mass.get_untracked(), &waist.get_untracked()])
        else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let body_mass_kg = body_mass_kg.unwrap_or(0.0);
        spawn_local(async move {
            match tauri::log_measurement(&date, body_mass_kg, waist_cm).await {
                Ok(answer) => {
                    shown_week.run(answer);
                    said.set(None);
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    let unrecorded = Signal::derive(move || week.get().is_none_or(|answer| answer.day.is_none()));
    view! {
        <section class="card">
            <h2>"Gather"</h2>
            <form class="measurement" on:submit=log>
                <div class="row">
                    <FigureField label="Body mass (kg)" name="mass" value=mass required=unrecorded />
                    <FigureField label="Waist (cm)" name="waist" value=waist />
                </div>
                <button class="cta" type="submit">
                    "Gather"
                </button>
                {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
            </form>
        </section>
        <section class="card last-days">
            <h2>"The last days"</h2>
            <table>
                <thead>
                    <tr>
                        <th>"Day"</th>
                        <th>"Mass (kg)"</th>
                        <th>"Waist (cm)"</th>
                        <th>"Fat (%)"</th>
                        <th>"Lean (kg)"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let chosen = day.get();
                        week.get()
                            .map(|answer| answer.days)
                            .unwrap_or_default()
                            .into_iter()
                            .map(|measured| {
                                let is_chosen = measured.day.to_string() == chosen;
                                view! { <DayRow day=measured chosen=is_chosen remove=remove /> }
                            })
                            .collect_view()
                    }}
                </tbody>
                <tfoot>
                    {move || {
                        week.get()
                            .map(|answer| {
                                let tendency = answer.tendency;
                                view! {
                                    <tr class="tendency">
                                        <td>"Tendency"</td>
                                        {tendency_cell(tendency.body_mass, Figure::Mass)}
                                        {tendency_cell(tendency.waist, Figure::Length)}
                                        {tendency_cell(tendency.fat_ratio, Figure::Ratio)}
                                        {tendency_cell(tendency.lean_mass, Figure::Mass)}
                                        <td></td>
                                    </tr>
                                }
                            })
                    }}
                </tfoot>
            </table>
        </section>
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Segment {
    Gather,
    Fuel,
    Hydrate,
    Burn,
}

const SEGMENTS: [(Segment, &str); 4] = [
    (Segment::Gather, "Gather"),
    (Segment::Fuel, "Fuel"),
    (Segment::Hydrate, "Hydrate"),
    (Segment::Burn, "Burn"),
];

#[component]
pub fn Tracker() -> impl IntoView {
    let day = RwSignal::new(today());
    let named = use_query_map().get_untracked().get("segment");
    let segment = RwSignal::new(
        SEGMENTS
            .into_iter()
            .find(|(_, label)| named.as_deref() == Some(*label))
            .map_or(Segment::Gather, |(segment, _)| segment),
    );
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
            Segment::Fuel => view! { <FuelDay day=day /> }.into_any(),
            Segment::Hydrate => view! { <HydrateDay day=day /> }.into_any(),
            Segment::Burn => view! { <BurnDay day=day /> }.into_any(),
        }}
        <A href="/stats" attr:class="cta ghost explore">
            "Explore in Stats"
        </A>
    }
}
