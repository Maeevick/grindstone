use grind_core::figures::{Figure, shown, typed_back};
use grind_core::for_tracking::GatherWeek;
use grind_core::tendency::{Change, Heading};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::fields::FigureField;
use crate::screens::gauge::AimGauge;
use crate::screens::last_days::GatherLastDays;
use crate::screens::tab::{DayCard, ExploreInStats};
use crate::screens::{Recorded, lines, today};
use crate::tauri;
use crate::typed::figures_typed;

fn pace_line(
    name: &'static str,
    change: Option<Change>,
    figure: fn(f64) -> Figure,
) -> impl IntoView {
    change.map(|change| {
        let (class, moved) = match change.heading {
            Heading::Steady => ("steady", "steady".to_string()),
            heading => (
                match heading {
                    Heading::Toward => "toward",
                    Heading::Away => "away",
                    _ => "unaimed",
                },
                format!(
                    "{}{} a week",
                    if change.by > 0.0 { "+" } else { "" },
                    shown(figure(change.by))
                ),
            ),
        };
        view! {
            <li class=class>
                <span>{name}</span>
                <span>{moved}</span>
            </li>
        }
    })
}

#[component]
fn GatherDay(day: RwSignal<String>) -> impl IntoView {
    let mass = RwSignal::new(String::new());
    let waist = RwSignal::new(String::new());
    let week = RwSignal::new(None::<GatherWeek>);
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
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
    let remove = move |_| {
        let date = day.get_untracked();
        spawn_local(async move {
            match tauri::remove_measurement(&date).await {
                Ok(answer) => {
                    shown_week.run(answer);
                    recorded.told();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
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
                    recorded.told();
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
                {move || {
                    (!unrecorded.get())
                        .then(|| {
                            view! {
                                <button class="cta ghost remove-weigh-in" type="button" on:click=remove>
                                    "Remove this weigh-in"
                                </button>
                            }
                        })
                }}
                {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
            </form>
        </section>
        {move || {
            week.get()
                .and_then(|answer| {
                    let aims = answer.aims?;
                    let latest = answer.days.first().copied()?;
                    let lean_kg = answer
                        .days
                        .iter()
                        .find_map(|measured| measured.composition)
                        .map(|composition| composition.lean_mass_kg);
                    Some(view! {
                        <section class="card bands">
                            <h2>"Where you stand"</h2>
                            <AimGauge
                                label="Body mass"
                                amount=latest.body_mass_kg
                                aim=aims.body_mass_kg
                                recommended=aims.recommended
                            />
                            {lean_kg
                                .map(|lean_kg| {
                                    view! {
                                        <AimGauge
                                            label="Lean mass"
                                            amount=lean_kg
                                            aim=aims.lean_mass_kg
                                            recommended=aims.recommended
                                        />
                                    }
                                })}
                        </section>
                    })
                })
        }}
        {move || {
            week.get()
                .map(|answer| {
                    view! {
                        <section class="card pace">
                            <h2>"The week's pace"</h2>
                            {match answer.pace {
                                Some(pace) => {
                                    view! {
                                        <ul class="pace-lines">
                                            {pace_line("Body mass", Some(pace.body_mass), Figure::Mass)}
                                            {pace_line("Lean mass", pace.lean_mass, Figure::Mass)}
                                            {pace_line("Fat mass", pace.fat_mass, Figure::Mass)}
                                            {pace_line("Fat %", pace.fat_ratio, Figure::Ratio)}
                                        </ul>
                                        <p class=lines::pace_class(pace.judgement)>{lines::pace_said(&pace)}</p>
                                    }
                                        .into_any()
                                }
                                None => view! { <p class="empty">{lines::NO_PACE_YET}</p> }.into_any(),
                            }}
                        </section>
                    }
                })
        }}
    }
}

#[component]
pub fn GatherTab() -> impl IntoView {
    let day = RwSignal::new(today());
    view! {
        <DayCard day=day />
        <GatherDay day=day />
        <GatherLastDays day=day />
        <ExploreInStats />
    }
}
