use std::cmp::Reverse;

use grind_core::burn::{Burn, BurntWorkout};
use grind_core::burn_week::ExerciseWeek;
use grind_core::circuit::{Circuit, StepForm, burnt_per_set};
use grind_core::exercise::{Exercise, exercise as exercise_of};
use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, shown, typed_back};
use grind_core::for_burning::{
    CircuitSessionForm, DEFAULT_LOAD_KG, DEFAULT_MINUTES, DEFAULT_REPS, DEFAULT_SETS,
    OffTheClockForm, WorkoutForm,
};
use grind_core::nutrition::Budget;
use grind_core::workout::{burnt_by, workout};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::{chosen, grouped, nothing_matches};
use crate::screens::entries::{ExerciseFields, ExerciseFigures, NewEntry};
use crate::screens::fields::{Chooser, FigureField, preview_line};
use crate::screens::gauge::BurnGauge;
use crate::screens::last_days::BurnLastDays;
use crate::screens::session::{Session, SessionInProgress};
use crate::screens::tab::{DayCard, GrimoireBesideTheNumbers};
use crate::screens::{Recorded, lines, today};
use crate::tauri;
use crate::typed::{count_typed, figures_typed};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Logging {
    Workout,
    OffTheClock,
}

const LOGGINGS: [(Logging, &str); 2] = [
    (Logging::Workout, "Workout"),
    (Logging::OffTheClock, "Off the clock"),
];

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

#[component]
pub fn BurnDay(day: RwSignal<String>) -> impl IntoView {
    let burn = RwSignal::new(None::<Burn>);
    let budget = RwSignal::new(None::<Budget>);
    let catalog = RwSignal::new(Vec::<Exercise>::new());
    let crafted = RwSignal::new(Vec::<Circuit>::new());
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
    let options = Signal::derive(move || {
        grouped(
            (
                lines::CIRCUIT_ICON,
                crafted.get().into_iter().map(|known| known.name).collect(),
            ),
            (
                lines::EXERCISE_ICON,
                catalog.get().into_iter().map(|known| known.name).collect(),
            ),
        )
    });
    let names = Signal::derive(move || {
        options
            .get()
            .into_iter()
            .map(|choice| choice.name)
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
    let body_mass = Signal::derive(move || burn.get().and_then(|answer| answer.body_mass_kg));
    let figures = ExerciseFigures::default();
    let session = use_context::<SessionInProgress>().unwrap_or_default();
    let on_the_session = Memo::new(move |_| !session.steps.get().is_empty());
    let exercises = Signal::derive(move || catalog.get());
    let sessioned = Signal::derive(move || {
        let body_mass_kg = body_mass.get()?;
        on_the_session
            .get()
            .then(|| session.per_set(&catalog.get(), body_mass_kg) * f64::from(session.sets()))
    });
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
    let chosen_circuit = move || {
        chosen(&names.get(), &exercise.get())
            .filter(|_| chosen_exercise().is_none())
            .and_then(|named| crafted.get().into_iter().find(|known| known.name == named))
    };
    let running_a_circuit = Memo::new(move |_| chosen_circuit().is_some());
    let preview = move || {
        if let Some(circuit) = chosen_circuit() {
            let times = count_typed(&sets.get()).unwrap_or(DEFAULT_SETS);
            let kcal = burnt_per_set(&circuit, body_mass.get()?) * f64::from(times);
            return Some(shown(Figure::Energy(kcal)));
        }
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
    Effect::new(move |_| {
        recorded.heard();
        spawn_local(async move {
            match tauri::exercises().await {
                Ok(known) => catalog.set(known),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
            if let Ok(known) = tauri::circuits().await {
                crafted.set(known);
            }
        });
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
    let stepped_now = move |stepped: Vec<StepForm>| {
        session.steps.update(|steps| steps.extend(stepped));
        for typed in [exercise, minutes, reps, sets, load] {
            typed.set(String::new());
        }
        figures.cleared();
        said.set(None);
    };
    // The circuit chosen, run by its sets or its steps put in the session.
    let run = move |circuit: Circuit, into_the_session: bool| {
        if into_the_session {
            stepped_now(
                circuit
                    .steps
                    .into_iter()
                    .map(|step| StepForm {
                        exercise: step.exercise,
                        reps: step.reps,
                        minutes: step.minutes,
                        load_kg: step.load_kg,
                    })
                    .collect(),
            );
            return;
        }
        let form = CircuitSessionForm {
            date: day.get_untracked(),
            circuit: circuit.name,
            sets: count_typed(&sets.get_untracked()),
        };
        spawn_local(async move { logged(tauri::log_circuit(&form).await) });
    };
    // The exercise typed, added to the catalog first when new, then logged or put in the session.
    let served = move |into_the_session: bool| {
        if let Some(circuit) = untrack(chosen_circuit) {
            run(circuit, into_the_session);
            return;
        }
        let Some([minutes_typed, load_typed]) =
            figures_typed([&minutes.get_untracked(), &load.get_untracked()])
        else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let added = if creating.get_untracked() {
            let Some([met]) = figures_typed([&figures.met.get_untracked()]) else {
                said.set(Some(lines::UNREADABLE.into()));
                return;
            };
            Some((
                exercise.get_untracked().trim().to_string(),
                met.unwrap_or(0.0),
                figures.description.get_untracked(),
            ))
        } else {
            None
        };
        let known = chosen(&names.get_untracked(), &exercise.get_untracked());
        if added.is_none() && known.is_none() {
            said.set(Some(lines::CHOOSE_FIRST.into()));
            return;
        }
        let reps_typed = count_typed(&reps.get_untracked());
        let form = WorkoutForm {
            date: day.get_untracked(),
            exercise: String::new(),
            minutes: minutes_typed,
            reps: reps_typed,
            sets: count_typed(&sets.get_untracked()),
            load_kg: load_typed,
        };
        spawn_local(async move {
            let name = match added {
                Some((name, met, description)) => {
                    match tauri::add_exercise(&name, met, &description).await {
                        Ok(listed) => {
                            catalog.set(listed);
                            recorded.told();
                            name
                        }
                        Err(failure) => {
                            said.set(Some(lines::failure(&failure)));
                            return;
                        }
                    }
                }
                None => known.unwrap_or_default(),
            };
            if into_the_session {
                stepped_now(vec![StepForm {
                    exercise: name,
                    reps: reps_typed,
                    minutes: minutes_typed.unwrap_or(DEFAULT_MINUTES),
                    load_kg: load_typed.unwrap_or(DEFAULT_LOAD_KG),
                }]);
                return;
            }
            let form = WorkoutForm {
                exercise: name,
                ..form
            };
            logged(tauri::log_workout(&form).await);
        });
    };
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        served(on_the_session.get_untracked());
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
                                <Chooser label="Exercise or circuit" name="exercise" options=options typed=exercise />
                                {move || {
                                    if !running_a_circuit.get() {
                                        view! {
                                            <FigureField
                                                label="Minutes a set"
                                                name="minutes"
                                                value=minutes
                                                placeholder=typed_back(DEFAULT_MINUTES)
                                            />
                                        }
                                            .into_any()
                                    } else if on_the_session.get() {
                                        ().into_any()
                                    } else {
                                        view! {
                                            <FigureField
                                                label="Sets"
                                                name="sets"
                                                value=sets
                                                placeholder=DEFAULT_SETS.to_string()
                                            />
                                        }
                                            .into_any()
                                    }
                                }}
                            </div>
                            {move || {
                                creating
                                    .get()
                                    .then(|| {
                                        view! {
                                            <NewEntry name=exercise />
                                            <ExerciseFields prefix="new_exercise" figures=figures />
                                        }
                                    })
                            }}
                            {move || {
                                (!running_a_circuit.get())
                                    .then(|| {
                                        view! {
                                            <div class="row">
                                                <FigureField
                                                    label="Reps"
                                                    name="reps"
                                                    value=reps
                                                    placeholder=DEFAULT_REPS.to_string()
                                                />
                                                {move || {
                                                    (!on_the_session.get())
                                                        .then(|| {
                                                            view! {
                                                                <FigureField
                                                                    label="Sets"
                                                                    name="sets"
                                                                    value=sets
                                                                    placeholder=DEFAULT_SETS.to_string()
                                                                />
                                                            }
                                                        })
                                                }}
                                                <FigureField
                                                    label="Load (kg)"
                                                    name="load"
                                                    value=load
                                                    placeholder=typed_back(DEFAULT_LOAD_KG)
                                                />
                                            </div>
                                        }
                                    })
                            }}
                            {preview_line(preview)}
                            <button
                                class=move || if on_the_session.get() { "cta to-the-session" } else { "cta" }
                                type="submit"
                            >
                                {move || {
                                    if on_the_session.get() {
                                        "Add to the session"
                                    } else if creating.get() {
                                        "Add and log"
                                    } else {
                                        "Burn"
                                    }
                                }}
                            </button>
                            {move || {
                                (!on_the_session.get())
                                    .then(|| {
                                        view! {
                                            <button
                                                class="cta ghost to-the-session"
                                                type="button"
                                                on:click=move |_| served(true)
                                            >
                                                "Add to the session"
                                            </button>
                                        }
                                    })
                            }}
                        </form>
                        {move || {
                            on_the_session
                                .get()
                                .then(|| {
                                    view! {
                                        <Session
                                            session=session
                                            exercises=exercises
                                            body_mass=body_mass
                                            day=day
                                            shown_burn=shown_burn
                                            said=said
                                        />
                                    }
                                })
                        }}
                    }
                        .into_any()
                }
                Logging::OffTheClock => {
                    view! {
                        {move || {
                            burn.get()
                                .map(|answer| view! { <p class="hint">{lines::off_the_clock(&answer)}</p> })
                        }}
                        <form class="off-the-clock" on:submit=keep_off_the_clock>
                            <div class="row">
                                <FigureField label="Sitting (min)" name="sitting" value=sitting />
                                <FigureField label="Standing (min)" name="standing" value=standing />
                                <FigureField label="Moving (km)" name="moving" value=moving />
                            </div>
                            <button class="cta" type="submit">
                                "Keep for this day"
                            </button>
                        </form>
                    }
                        .into_any()
                }
            }}
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
        </section>
        {move || {
            burn.get()
                .map(|answer| {
                    let rested = answer.workouts.is_empty();
                    let burnt = answer.burnt_kcal.unwrap_or(0.0);
                    let with_the_session = sessioned.get().map(|kcal| burnt + kcal);
                    view! {
                        <section class="card bands">
                            <h2>"Where the day stands"</h2>
                            <p class="figures">{lines::burnt(&answer)}</p>
                            {with_the_session
                                .map(|kcal| {
                                    view! { <p class="hint with-the-session">{lines::with_the_session(kcal)}</p> }
                                })}
                            {answer
                                .target_kcal
                                .map(|target| {
                                    view! { <BurnGauge burnt=burnt target=target ghost=with_the_session /> }
                                })}
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

fn in_all_and_a_time(
    total: Option<f64>,
    a_time: Option<f64>,
    figure: fn(f64) -> Figure,
) -> impl IntoView {
    view! {
        <td>
            <span class="total">{total.map_or_else(|| "–".to_string(), |total| bare(figure(total)))}</span>
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

/// The seven days up to the tab's day summed up, the day included: each exercise in all and a time.
#[component]
pub fn BurnWeekCard(day: RwSignal<String>) -> impl IntoView {
    let recorded = Recorded::here();
    let week = LocalResource::new(move || {
        recorded.heard();
        let date = day.get();
        async move { tauri::burn_week(&date).await }
    });
    view! {
        <section class="card week">
            <h2>"The week's sum-up"</h2>
            <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
                {move || {
                    week.get()
                        .map(|answer| match answer {
                            Ok(Some(week)) => {
                                let held = match week.on_target() {
                                    Some(true) => "good",
                                    Some(false) => "low",
                                    None => "",
                                };
                                view! {
                                    <p class="week-burn">
                                        <span class=held>
                                            {week
                                                .burnt_kcal
                                                .map_or_else(|| "–".to_string(), |kcal| shown(Figure::Energy(kcal)))}
                                        </span>
                                        {lines::burnt_against(week.target_kcal, week.active_days)}
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
                                        <tbody>{week.exercises.into_iter().map(summed_up).collect_view()}</tbody>
                                    </table>
                                    <p class="hint">{lines::A_TIME_BENEATH}</p>
                                }
                                    .into_any()
                            }
                            Ok(None) => {
                                view! { <p class="empty">{lines::NO_WORKOUT_THIS_WEEK}</p> }.into_any()
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

#[component]
pub fn BurnTab() -> impl IntoView {
    let day = RwSignal::new(today());
    view! {
        <DayCard day=day />
        <BurnDay day=day />
        <BurnWeekCard day=day />
        <BurnLastDays day=day />
        <GrimoireBesideTheNumbers grimoire="/burn/grimoire" />
    }
}
