use std::cmp::Reverse;

use grind_core::burn::{Burn, BurntWorkout};
use grind_core::exercise::{Exercise, exercise as exercise_of};
use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, shown, typed_back};
use grind_core::for_burning::{
    DEFAULT_LOAD_KG, DEFAULT_MINUTES, DEFAULT_REPS, DEFAULT_SETS, OffTheClockForm, WorkoutForm,
};
use grind_core::nutrition::Budget;
use grind_core::workout::{burnt_by, workout};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::{chosen, nothing_matches};
use crate::screens::catalog::BurnShelf;
use crate::screens::circuits::CircuitLog;
use crate::screens::fields::{Chooser, FigureField, preview_line};
use crate::screens::gauge::BurnGauge;
use crate::screens::tab::{DayCard, ExploreInStats};
use crate::screens::{Recorded, lines, today};
use crate::tauri;
use crate::typed::{count_typed, figures_typed};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Logging {
    Workout,
    Circuit,
}

const LOGGINGS: [(Logging, &str); 2] =
    [(Logging::Workout, "Workout"), (Logging::Circuit, "Circuit")];

#[component]
fn ExpenditureRow(
    done: BurntWorkout,
    day: RwSignal<String>,
    shown_burn: Callback<Burn>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let id = done.id;
    let recorded = Recorded::here();
    let remove = move |_| {
        let date = day.get_untracked();
        spawn_local(async move {
            match tauri::remove_workout(&date, id).await {
                Ok(burn) => {
                    shown_burn.run(burn);
                    recorded.told();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    let workout = done.workout;
    let sets = f64::from(workout.sets);
    view! {
        <tr>
            <td>
                {workout.exercise.clone()}
                {workout
                    .circuit
                    .clone()
                    .map(|circuit| view! { <span class="hint">{format!(" · {circuit}")}</span> })}
            </td>
            <td>{done.burnt_kcal.map_or_else(|| "-".to_string(), |kcal| bare(Figure::Energy(kcal)))}</td>
            <td>{bare(Figure::Duration(workout.minutes * sets))}</td>
            <td>
                {workout
                    .reps
                    .map_or_else(|| "-".to_string(), |reps| bare(Figure::Count(f64::from(reps) * sets)))}
            </td>
            <td>
                <button class="remove" type="button" on:click=remove aria-label="Remove this workout">
                    "×"
                </button>
            </td>
        </tr>
    }
}

fn neat_row(activity: &'static str, kcal: f64) -> impl IntoView {
    view! {
        <tr class="neat">
            <td>{activity}</td>
            <td>{bare(Figure::Energy(kcal))}</td>
            <td>"-"</td>
            <td>"-"</td>
            <td></td>
        </tr>
    }
}

/// What an exercise new to the catalog is typed with, its name aside.
#[derive(Clone, Copy)]
struct NewExerciseFigures {
    description: RwSignal<String>,
    met: RwSignal<String>,
}

impl NewExerciseFigures {
    fn new() -> Self {
        Self {
            description: RwSignal::new(String::new()),
            met: RwSignal::new(String::new()),
        }
    }

    fn cleared(self) {
        self.description.set(String::new());
        self.met.set(String::new());
    }
}

#[component]
fn NewExerciseFields(name: RwSignal<String>, figures: NewExerciseFigures) -> impl IntoView {
    view! {
        <p class="new">{move || format!("New {}", name.get().trim())}</p>
        <div class="row">
            <label class="field grow2">
                "Description"
                <input
                    name="new_exercise_description"
                    autocomplete="off"
                    placeholder="Optional"
                    prop:value=move || figures.description.get()
                    on:input=move |event| figures.description.set(event_target_value(&event))
                />
            </label>
            <FigureField label="MET" name="new_exercise_met" value=figures.met />
        </div>
    }
}

#[component]
pub fn BurnDay(day: RwSignal<String>) -> impl IntoView {
    let burn = RwSignal::new(None::<Burn>);
    let budget = RwSignal::new(None::<Budget>);
    let catalog = RwSignal::new(Vec::<Exercise>::new());
    let exercise = RwSignal::new(String::new());
    let minutes = RwSignal::new(String::new());
    let reps = RwSignal::new(String::new());
    let sets = RwSignal::new(String::new());
    let load = RwSignal::new(String::new());
    let sitting = RwSignal::new(String::new());
    let standing = RwSignal::new(String::new());
    let moving = RwSignal::new(String::new());
    let logging = RwSignal::new(Logging::Workout);
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let names = Signal::derive(move || {
        catalog
            .get()
            .into_iter()
            .map(|known| known.name)
            .collect::<Vec<_>>()
    });
    let shown_burn = Callback::new(move |answer: Burn| {
        if let Some(off) = answer.off_the_clock {
            sitting.set(off.sitting_min.to_string());
            standing.set(off.standing_min.to_string());
            moving.set(off.moving_km.to_string());
        }
        burn.set(Some(answer));
    });
    let logged_circuit = Callback::new(move |answer: Burn| {
        shown_burn.run(answer);
        recorded.told();
    });
    let body_mass = Signal::derive(move || burn.get().and_then(|answer| answer.body_mass_kg));
    let figures = NewExerciseFigures::new();
    let creating = Memo::new(move |_| nothing_matches(&names.get(), &exercise.get()));
    let new_exercise = move || {
        let [met] = figures_typed([&figures.met.get()])?;
        exercise_of(
            &exercise.get(),
            met.unwrap_or(0.0),
            &figures.description.get(),
        )
        .ok()
    };
    let chosen_exercise = move || {
        chosen(&names.get(), &exercise.get())
            .and_then(|named| catalog.get().into_iter().find(|known| known.name == named))
    };
    let preview = move || {
        let known = chosen_exercise().or_else(|| creating.get().then(new_exercise).flatten())?;
        let [minutes_typed, load_typed] = figures_typed([&minutes.get(), &load.get()])?;
        let done = workout(
            &day.get(),
            &known,
            minutes_typed.unwrap_or(DEFAULT_MINUTES),
            count_typed(&reps.get()),
            count_typed(&sets.get()).unwrap_or(DEFAULT_SETS),
            load_typed.unwrap_or(DEFAULT_LOAD_KG),
        )
        .ok()?;
        Some(shown(Figure::Energy(burnt_by(&done, body_mass.get()?))))
    };
    let budget_of = move |date: String| {
        spawn_local(async move {
            if let Ok(fuel) = tauri::fuel_day(&date).await {
                budget.set(fuel.budget);
            }
        });
    };
    spawn_local(async move {
        match tauri::exercises().await {
            Ok(known) => catalog.set(known),
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
    });
    Effect::new(move |_| {
        let date = day.get();
        budget_of(date.clone());
        spawn_local(async move {
            match tauri::burn_day(&date).await {
                Ok(answer) => shown_burn.run(answer),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    });
    let logged = move |answer: Result<Burn, Failure>| match answer {
        Ok(answer) => {
            shown_burn.run(answer);
            for typed in [exercise, minutes, reps, sets, load] {
                typed.set(String::new());
            }
            figures.cleared();
            said.set(None);
            recorded.told();
        }
        Err(failure) => said.set(Some(lines::failure(&failure))),
    };
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([minutes_typed, load_typed]) =
            figures_typed([&minutes.get_untracked(), &load.get_untracked()])
        else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = |name: String| WorkoutForm {
            date: day.get_untracked(),
            exercise: name,
            minutes: minutes_typed,
            reps: count_typed(&reps.get_untracked()),
            sets: count_typed(&sets.get_untracked()),
            load_kg: load_typed,
        };
        if creating.get_untracked() {
            let Some([met]) = figures_typed([&figures.met.get_untracked()]) else {
                said.set(Some(lines::UNREADABLE.into()));
                return;
            };
            let name = exercise.get_untracked().trim().to_string();
            let description = figures.description.get_untracked();
            let form = form(name.clone());
            spawn_local(async move {
                match tauri::add_exercise(&name, met.unwrap_or(0.0), &description).await {
                    Ok(known) => {
                        catalog.set(known);
                        logged(tauri::log_workout(&form).await);
                    }
                    Err(failure) => said.set(Some(lines::failure(&failure))),
                }
            });
            return;
        }
        let Some(name) = chosen(&names.get_untracked(), &exercise.get_untracked()) else {
            said.set(Some(lines::CHOOSE_FIRST.into()));
            return;
        };
        let form = form(name);
        spawn_local(async move { logged(tauri::log_workout(&form).await) });
    };
    let keep_off_the_clock = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let date = day.get_untracked();
        let Some([sitting_min, standing_min, moving_km]) = figures_typed([
            &sitting.get_untracked(),
            &standing.get_untracked(),
            &moving.get_untracked(),
        ]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = OffTheClockForm {
            sitting_min: sitting_min.unwrap_or(0.0),
            standing_min: standing_min.unwrap_or(0.0),
            moving_km: moving_km.unwrap_or(0.0),
        };
        spawn_local(async move {
            match tauri::set_day_off_the_clock(&date, &form).await {
                Ok(answer) => {
                    shown_burn.run(answer);
                    budget_of(date);
                    said.set(Some(lines::DAY_OFF_THE_CLOCK_KEPT.into()));
                    recorded.told();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Burn"</h2>
            <div class="pills">
                {LOGGINGS
                    .into_iter()
                    .map(|(named, label)| {
                        view! {
                            <button
                                type="button"
                                class=format!("pill burn-{named:?}")
                                class:chosen=move || logging.get() == named
                                on:click=move |_| logging.set(named)
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            {move || match logging.get() {
                Logging::Workout => {
                    view! {
                        <form class="workout" on:submit=log>
                            <div class="row">
                                <Chooser label="Exercise" name="exercise" options=names typed=exercise />
                                <FigureField
                                    label="Minutes a set"
                                    name="minutes"
                                    value=minutes
                                    placeholder=typed_back(DEFAULT_MINUTES)
                                />
                            </div>
                            {move || {
                                creating
                                    .get()
                                    .then(|| view! { <NewExerciseFields name=exercise figures=figures /> })
                            }}
                            <div class="row">
                                <FigureField
                                    label="Reps"
                                    name="reps"
                                    value=reps
                                    placeholder=DEFAULT_REPS.to_string()
                                />
                                <FigureField
                                    label="Sets"
                                    name="sets"
                                    value=sets
                                    placeholder=DEFAULT_SETS.to_string()
                                />
                                <FigureField
                                    label="Load (kg)"
                                    name="load"
                                    value=load
                                    placeholder=typed_back(DEFAULT_LOAD_KG)
                                />
                            </div>
                            {preview_line(preview)}
                            <button class="cta" type="submit">
                                {move || if creating.get() { "Add and log" } else { "Burn" }}
                            </button>
                        </form>
                    }
                        .into_any()
                }
                Logging::Circuit => view! { <CircuitLog day=day on_logged=logged_circuit body_mass=body_mass /> }.into_any(),
            }}
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
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
        {move || {
            burn.get()
                .map(|answer| {
                    let rested = answer.workouts.is_empty();
                    let burnt = answer.burnt_kcal.unwrap_or(0.0);
                    view! {
                        <section class="card bands">
                            <h2>"Where the day stands"</h2>
                            <p class="figures">{lines::burnt(&answer)}</p>
                            {answer
                                .target_kcal
                                .map(|target| view! { <BurnGauge burnt=burnt target=target /> })}
                            {rested.then(|| view! { <p class="empty">{lines::NO_WORKOUT}</p> })}
                        </section>
                    }
                })
        }}
        <section class="card">
            <h2>"The day's expenditures"</h2>
            <div class="scroll">
                <table class="expenditures">
                    <thead>
                        <tr>
                            <th>"Activity"</th>
                            <th>"kcal"</th>
                            <th>"Duration (min)"</th>
                            <th>"reps"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || {
                                let mut done = burn.get().map(|answer| answer.workouts).unwrap_or_default();
                                done.sort_by_key(|workout| Reverse(workout.id));
                                done
                            }
                            key=|done| done.id
                            children=move |done| {
                                view! { <ExpenditureRow done=done day=day shown_burn=shown_burn said=said /> }
                            }
                        />
                        {move || {
                            budget
                                .get()
                                .map(|spent| {
                                    view! {
                                        {neat_row("Off the clock (volitional)", spent.volitional_kcal)}
                                        {neat_row("Fidgeting (spontaneous)", spent.spontaneous_kcal)}
                                    }
                                })
                        }}
                    </tbody>
                </table>
            </div>
        </section>
    }
}

#[component]
pub fn BurnTab() -> impl IntoView {
    let day = RwSignal::new(today());
    view! {
        <DayCard day=day />
        <BurnDay day=day />
        <ExploreInStats />
        <BurnShelf />
    }
}
