use grind_core::burn::{Burn, BurntWorkout};
use grind_core::exercise::Exercise;
use grind_core::figures::{Figure, shown};
use grind_core::for_burning::{OffTheClockForm, WorkoutForm};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::fields::FigureField;
use crate::screens::lines;
use crate::tauri;
use crate::typed::{count_typed, figure_typed};

fn how_it_went(done: &BurntWorkout) -> String {
    let workout = &done.workout;
    let sets = match workout.reps {
        Some(reps) => format!("{} × {reps}", workout.sets),
        None => format!("{} ×", workout.sets),
    };
    format!("{sets} · {}", shown(Figure::Duration(workout.minutes)))
}

#[component]
fn WorkoutRow(
    done: BurntWorkout,
    day: RwSignal<String>,
    shown_burn: Callback<Burn>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let id = done.id;
    let remove = move |_| {
        let date = day.get_untracked();
        spawn_local(async move {
            match tauri::remove_workout(&date, id).await {
                Ok(burn) => shown_burn.run(burn),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <tr>
            <td>{done.workout.exercise.clone()}</td>
            <td>{how_it_went(&done)}</td>
            <td>{done.burnt_kcal.map(|kcal| shown(Figure::Energy(kcal))).unwrap_or_default()}</td>
            <td>
                <button class="remove" type="button" on:click=remove aria-label="Remove this workout">
                    "×"
                </button>
            </td>
        </tr>
    }
}

#[component]
pub fn BurnDay(day: RwSignal<String>) -> impl IntoView {
    let burn = RwSignal::new(None::<Burn>);
    let catalog = RwSignal::new(Vec::<Exercise>::new());
    let exercise = RwSignal::new(String::new());
    let minutes = RwSignal::new(String::new());
    let reps = RwSignal::new(String::new());
    let sets = RwSignal::new("1".to_string());
    let load = RwSignal::new(String::new());
    let sitting = RwSignal::new(String::new());
    let standing = RwSignal::new(String::new());
    let moving = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    let shown_burn = Callback::new(move |answer: Burn| {
        if let Some(off) = answer.off_the_clock {
            sitting.set(off.sitting_min.to_string());
            standing.set(off.standing_min.to_string());
            moving.set(off.moving_km.to_string());
        }
        burn.set(Some(answer));
    });
    spawn_local(async move {
        match tauri::exercises().await {
            Ok(known) => {
                if let Some(first) = known.first() {
                    exercise.set(first.name.clone());
                }
                catalog.set(known);
            }
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
    });
    Effect::new(move |_| {
        let date = day.get();
        spawn_local(async move {
            match tauri::burn_day(&date).await {
                Ok(answer) => shown_burn.run(answer),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    });
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let form = WorkoutForm {
            date: day.get_untracked(),
            exercise: exercise.get_untracked(),
            minutes: figure_typed(&minutes.get_untracked()).unwrap_or(f64::NAN),
            reps: count_typed(&reps.get_untracked()),
            sets: count_typed(&sets.get_untracked()).unwrap_or(1),
            load_kg: figure_typed(&load.get_untracked()).unwrap_or(0.0),
        };
        spawn_local(async move {
            match tauri::log_workout(&form).await {
                Ok(answer) => {
                    shown_burn.run(answer);
                    minutes.set(String::new());
                    reps.set(String::new());
                    said.set(None);
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    let keep_off_the_clock = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let date = day.get_untracked();
        let form = OffTheClockForm {
            sitting_min: figure_typed(&sitting.get_untracked()).unwrap_or(0.0),
            standing_min: figure_typed(&standing.get_untracked()).unwrap_or(0.0),
            moving_km: figure_typed(&moving.get_untracked()).unwrap_or(0.0),
        };
        spawn_local(async move {
            match tauri::set_day_off_the_clock(&date, &form).await {
                Ok(answer) => {
                    shown_burn.run(answer);
                    said.set(Some(lines::DAY_OFF_THE_CLOCK_KEPT.into()));
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Burn"</h2>
            <form class="workout" on:submit=log>
                <div class="row">
                    <label class="field grow2">
                        "Exercise"
                        <select
                            name="exercise"
                            prop:value=move || exercise.get()
                            on:change=move |event| exercise.set(event_target_value(&event))
                        >
                            <For
                                each=move || catalog.get()
                                key=|known| known.name.clone()
                                children=move |known| {
                                    let label = known.name.clone();
                                    view! { <option value=known.name>{label}</option> }
                                }
                            />
                        </select>
                    </label>
                    <FigureField label="Minutes a set" name="minutes" value=minutes />
                </div>
                <div class="row">
                    <FigureField label="Reps" name="reps" value=reps />
                    <FigureField label="Sets" name="sets" value=sets />
                    <FigureField label="Load (kg)" name="load" value=load />
                </div>
                <button class="cta" type="submit">
                    "Burn"
                </button>
            </form>
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            {move || {
                burn.get()
                    .map(|answer| {
                        let rested = answer.workouts.is_empty();
                        view! {
                            <p class="figures">{lines::burnt(&answer)}</p>
                            {rested.then(|| view! { <p class="empty">{lines::NO_WORKOUT}</p> })}
                        }
                    })
            }}
            <table class="workouts">
                <tbody>
                    <For
                        each=move || burn.get().map(|answer| answer.workouts).unwrap_or_default()
                        key=|done| done.id
                        children=move |done| {
                            view! { <WorkoutRow done=done day=day shown_burn=shown_burn said=said /> }
                        }
                    />
                </tbody>
            </table>
        </section>
        <section class="card">
            <h2>"Off the clock"</h2>
            {move || burn.get().map(|answer| view! { <p class="hint">{lines::off_the_clock(&answer)}</p> })}
            <form class="off-the-clock" on:submit=keep_off_the_clock>
                <div class="row">
                    <FigureField label="Sitting (min)" name="sitting" value=sitting />
                    <FigureField label="Standing (min)" name="standing" value=standing />
                    <FigureField label="Moving (km)" name="moving" value=moving />
                </div>
                <button class="cta ghost" type="submit">
                    "Keep for this day"
                </button>
            </form>
        </section>
    }
}
