use grind_core::burn_week::ExerciseWeek;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, shown};
use grind_core::for_reviewing_progress::Window;
use grind_core::tendency::{Change, Heading};
use leptos::prelude::*;

use crate::screens::cells::week_table;
use crate::screens::charts::bare_or_dash;
use crate::screens::lines;
use crate::tauri;

fn pace_line(
    name: &'static str,
    change: Option<Change>,
    figure: fn(f64) -> Figure,
) -> impl IntoView {
    change.map(|change| {
        let (class, moved, a_week) = match change.heading {
            Heading::Steady => ("steady", "steady".to_string(), ""),
            heading => (
                match heading {
                    Heading::Toward => "toward",
                    Heading::Away => "away",
                    _ => "unaimed",
                },
                format!(
                    "{}{}",
                    if change.by > 0.0 { "+" } else { "" },
                    shown(figure(change.by))
                ),
                " a week",
            ),
        };
        view! {
            <li>
                <span>{name}</span>
                <span>
                    <span class=class>{moved}</span>
                    {a_week}
                </span>
            </li>
        }
    })
}

fn failed(failure: &Failure) -> AnyView {
    view! { <p class="bad">{lines::failure(failure)}</p> }.into_any()
}

/// Gather over the window: how each measure moved a week, and the body mass's pace judged.
#[component]
pub fn PaceOver(today: String, window: RwSignal<Window>) -> impl IntoView {
    let pace = LocalResource::new(move || {
        let (today, window) = (today.clone(), window.get());
        async move { tauri::pace_over_the_window(&today, window).await }
    });
    view! {
        <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
            {move || {
                pace.get()
                    .map(|answer| match answer {
                        Ok(Some(pace)) => {
                            view! {
                                <ul class="pace-lines">
                                    {pace_line("Body mass", Some(pace.body_mass), Figure::Mass)}
                                    {pace_line("Lean mass", pace.lean_mass, Figure::Mass)}
                                    {pace_line("Fat mass", pace.fat_mass, Figure::Mass)}
                                    {pace_line("Fat %", pace.fat_ratio, Figure::Ratio)}
                                </ul>
                                <p class="tip">{lines::quoted(&lines::pace_said(&pace))}</p>
                            }
                                .into_any()
                        }
                        Ok(None) => view! { <p class="empty">{lines::NO_PACE_YET}</p> }.into_any(),
                        Err(failure) => failed(&failure),
                    })
            }}
        </Transition>
    }
}

/// Fuel over the window: a logged day on average against the bands, then what runs short and over.
#[component]
pub fn FuelOver(today: String, window: RwSignal<Window>) -> impl IntoView {
    let over = LocalResource::new(move || {
        let (today, window) = (today.clone(), window.get());
        async move { tauri::fuel_over_the_window(&today, window).await }
    });
    view! {
        <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
            {move || {
                over.get()
                    .map(|answer| match answer {
                        Ok(Some(over)) => {
                            view! {
                                {week_table(
                                    over.logged_days,
                                    vec![
                                        ("Energy (kcal)", over.energy, Figure::Energy),
                                        ("Protein (g)", over.protein, Figure::Grams),
                                        ("Fat (g)", over.fat, Figure::Grams),
                                        ("Fibre (g)", over.fibre, Figure::Grams),
                                        ("Carbs (g)", over.carbs, Figure::Grams),
                                    ],
                                )}
                                <p class="tip">{lines::quoted(&lines::fuel_week_said(&over))}</p>
                            }
                                .into_any()
                        }
                        Ok(None) => view! { <p class="empty">{lines::NO_WEEK_YET}</p> }.into_any(),
                        Err(failure) => failed(&failure),
                    })
            }}
        </Transition>
    }
}

/// Hydrate over the window: a logged day's drinks on average against the band.
#[component]
pub fn HydrateOver(today: String, window: RwSignal<Window>) -> impl IntoView {
    let over = LocalResource::new(move || {
        let (today, window) = (today.clone(), window.get());
        async move { tauri::hydrate_over_the_window(&today, window).await }
    });
    view! {
        <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
            {move || {
                over.get()
                    .map(|answer| match answer {
                        Ok(Some(over)) => {
                            view! {
                                {week_table(
                                    over.logged_days,
                                    vec![("Drinks (mL)", over.drunk, Figure::Volume)],
                                )}
                                <p class="tip">{lines::quoted(&lines::hydrate_week_said(&over))}</p>
                            }
                                .into_any()
                        }
                        Ok(None) => view! { <p class="empty">{lines::NO_WEEK_YET}</p> }.into_any(),
                        Err(failure) => failed(&failure),
                    })
            }}
        </Transition>
    }
}

fn in_all_and_a_time(
    total: Option<f64>,
    a_time: Option<f64>,
    figure: fn(f64) -> Figure,
) -> impl IntoView {
    view! {
        <td>
            <span class="total">{bare_or_dash(total, figure)}</span>
            <span class="a-time">{a_time.map(|a_time| bare(figure(a_time))).unwrap_or_default()}</span>
        </td>
    }
}

fn summed_up(exercise: ExerciseWeek) -> impl IntoView {
    view! {
        <tr>
            <td>{exercise.exercise.clone()}</td>
            <td>{exercise.times}</td>
            {in_all_and_a_time(Some(exercise.minutes), Some(exercise.minutes_a_time()), Figure::Duration)}
            {in_all_and_a_time(exercise.reps, exercise.reps_a_time(), Figure::Count)}
            {in_all_and_a_time(exercise.burnt_kcal, exercise.burnt_kcal_a_time(), Figure::Energy)}
        </tr>
    }
}

/// Burn over the window, today included: the burn against its days' targets, each exercise in all and a time.
#[component]
pub fn BurnOver(today: String, window: RwSignal<Window>) -> impl IntoView {
    let over = LocalResource::new(move || {
        let (today, window) = (today.clone(), window.get());
        async move { tauri::burn_over_the_window(&today, window).await }
    });
    view! {
        <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
            {move || {
                over.get()
                    .map(|answer| match answer {
                        Ok(Some(over)) => {
                            let held = match over.on_target() {
                                Some(true) => "good",
                                Some(false) => "low",
                                None => "",
                            };
                            view! {
                                <p class="week-burn">
                                    <span class=held>
                                        {over
                                            .burnt_kcal
                                            .map_or_else(|| "–".to_string(), |kcal| shown(Figure::Energy(kcal)))}
                                    </span>
                                    {lines::burnt_against(over.target_kcal, over.active_days)}
                                </p>
                                <table class="workouts">
                                    <thead>
                                        <tr>
                                            <th>"Exercise"</th>
                                            <th>"Times"</th>
                                            <th>"Min"</th>
                                            <th>"Reps"</th>
                                            <th>"kcal"</th>
                                        </tr>
                                    </thead>
                                    <tbody>{over.exercises.into_iter().map(summed_up).collect_view()}</tbody>
                                </table>
                                <p class="hint">{lines::A_TIME_BENEATH}</p>
                            }
                                .into_any()
                        }
                        Ok(None) => {
                            view! { <p class="empty">{lines::NO_WORKOUT_THIS_WEEK}</p> }.into_any()
                        }
                        Err(failure) => failed(&failure),
                    })
            }}
        </Transition>
    }
}
