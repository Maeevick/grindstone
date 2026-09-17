use grind_core::burn::Burn;
use grind_core::circuit::{Circuit, CircuitStep, StepForm};
use grind_core::exercise::Exercise;
use grind_core::figures::{Figure, shown};
use grind_core::for_burning::{CircuitSessionForm, DEFAULT_SETS};
use grind_core::for_cataloguing::CircuitForm;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::chosen;
use crate::screens::fields::{Chooser, FigureField};
use crate::screens::lines;
use crate::tauri;
use crate::typed::{count_typed, figures_typed};

fn how_a_step_goes(reps: Option<u32>, minutes: f64, load_kg: f64) -> String {
    let goal = reps
        .map(|reps| format!("{reps} reps · "))
        .unwrap_or_default();
    let load = if load_kg == 0.0 {
        String::new()
    } else {
        format!(" · {}", shown(Figure::Mass(load_kg)))
    };
    format!("{goal}{}{load}", shown(Figure::Duration(minutes)))
}

fn asked_step(step: &StepForm) -> String {
    format!(
        "{} · {}",
        step.exercise,
        how_a_step_goes(step.reps, step.minutes, step.load_kg)
    )
}

fn frozen_step(step: &CircuitStep) -> String {
    format!(
        "{} ({} MET) · {}",
        step.exercise,
        step.met,
        how_a_step_goes(step.reps, step.minutes, step.load_kg)
    )
}

fn said_line(said: RwSignal<Option<Result<String, String>>>) -> impl IntoView {
    move || {
        said.get().map(|line| match line {
            Ok(good) => view! { <p class="said">{good}</p> }.into_any(),
            Err(bad) => view! { <p class="bad">{bad}</p> }.into_any(),
        })
    }
}

#[component]
fn CraftedCircuit(crafted: Circuit, remove: Callback<String>) -> impl IntoView {
    let named = crafted.name.clone();
    view! {
        <tr>
            <td>
                <strong>{crafted.name.clone()}</strong>
                <ol class="figures">
                    {crafted
                        .steps
                        .iter()
                        .map(|step| view! { <li>{frozen_step(step)}</li> })
                        .collect_view()}
                </ol>
            </td>
            <td>
                <button
                    class="remove"
                    type="button"
                    aria-label="Remove this circuit"
                    on:click=move |_| remove.run(named.clone())
                >
                    "×"
                </button>
            </td>
        </tr>
    }
}

#[component]
pub fn CircuitCrafter() -> impl IntoView {
    let catalog = RwSignal::new(Vec::<Exercise>::new());
    let crafted = RwSignal::new(Vec::<Circuit>::new());
    let name = RwSignal::new(String::new());
    let exercise = RwSignal::new(String::new());
    let reps = RwSignal::new(String::new());
    let minutes = RwSignal::new(String::new());
    let load = RwSignal::new(String::new());
    let steps = RwSignal::new(Vec::<StepForm>::new());
    let said = RwSignal::new(None::<Result<String, String>>);
    spawn_local(async move {
        match tauri::exercises().await {
            Ok(known) => {
                if let Some(first) = known.first() {
                    exercise.set(first.name.clone());
                }
                catalog.set(known);
            }
            Err(failure) => said.set(Some(Err(lines::failure(&failure)))),
        }
    });
    spawn_local(async move {
        match tauri::circuits().await {
            Ok(known) => crafted.set(known),
            Err(failure) => said.set(Some(Err(lines::failure(&failure)))),
        }
    });
    let add_step = move |_| {
        let Some([minutes_typed, load_typed]) =
            figures_typed([&minutes.get_untracked(), &load.get_untracked()])
        else {
            said.set(Some(Err(lines::UNREADABLE.into())));
            return;
        };
        let step = StepForm {
            exercise: exercise.get_untracked(),
            reps: count_typed(&reps.get_untracked()),
            minutes: minutes_typed.unwrap_or(0.0),
            load_kg: load_typed.unwrap_or(0.0),
        };
        steps.update(|added| added.push(step));
        reps.set(String::new());
        minutes.set(String::new());
        load.set(String::new());
    };
    let craft = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let form = CircuitForm {
            name: name.get_untracked(),
            steps: steps.get_untracked(),
        };
        spawn_local(async move {
            match tauri::craft_circuit(&form).await {
                Ok(known) => {
                    crafted.set(known);
                    name.set(String::new());
                    steps.set(Vec::new());
                    said.set(Some(Ok(lines::circuit_crafted(form.name.trim()))));
                }
                Err(failure) => said.set(Some(Err(lines::failure(&failure)))),
            }
        });
    };
    let steps_in_order = move || {
        steps
            .get()
            .into_iter()
            .enumerate()
            .collect::<Vec<(usize, StepForm)>>()
    };
    let remove = Callback::new(move |named: String| {
        spawn_local(async move {
            match tauri::remove_circuit(&named).await {
                Ok(known) => crafted.set(known),
                Err(failure) => said.set(Some(Err(lines::failure(&failure)))),
            }
        });
    });
    view! {
        <section class="card">
            <h2>"Circuits"</h2>
            <p class="hint">{lines::CIRCUIT_HOW}</p>
            <form class="circuit" on:submit=craft>
                <div class="row">
                    <label class="field grow2">
                        "Name"
                        <input
                            name="circuit_name"
                            autocomplete="off"
                            prop:value=move || name.get()
                            on:input=move |event| name.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <div class="row">
                    <label class="field grow2">
                        "Exercise"
                        <select
                            name="step_exercise"
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
                    <FigureField label="Reps goal" name="step_reps" value=reps />
                </div>
                <div class="row">
                    <FigureField label="Minutes" name="step_minutes" value=minutes />
                    <FigureField label="Load (kg)" name="step_load" value=load />
                </div>
                <button class="cta ghost add-step" type="button" on:click=add_step>
                    "Add this step"
                </button>
                <ol class="figures steps">
                    <For
                        each=steps_in_order
                        key=|(position, step)| format!("{position} {step:?}")
                        children=move |(position, step)| {
                            view! {
                                <li>
                                    {asked_step(&step)}
                                    <button
                                        class="remove"
                                        type="button"
                                        aria-label="Remove this step"
                                        on:click=move |_| {
                                            steps
                                                .update(|added| {
                                                    added.remove(position);
                                                })
                                        }
                                    >
                                        "×"
                                    </button>
                                </li>
                            }
                        }
                    />
                </ol>
                <button class="cta" type="submit">
                    "Craft the circuit"
                </button>
            </form>
            {said_line(said)}
            {move || {
                crafted.get().is_empty().then(|| view! { <p class="empty">{lines::NO_CIRCUIT}</p> })
            }}
            <table class="circuits">
                <tbody>
                    <For
                        each=move || crafted.get()
                        key=|kept| format!("{kept:?}")
                        children=move |kept| view! { <CraftedCircuit crafted=kept remove=remove /> }
                    />
                </tbody>
            </table>
        </section>
    }
}

#[component]
pub fn CircuitLog(day: RwSignal<String>, on_logged: Callback<Burn>) -> impl IntoView {
    let crafted = RwSignal::new(Vec::<Circuit>::new());
    let circuit = RwSignal::new(String::new());
    let sets = RwSignal::new(String::new());
    let said = RwSignal::new(None::<Result<String, String>>);
    let names = Signal::derive(move || {
        crafted
            .get()
            .into_iter()
            .map(|known| known.name)
            .collect::<Vec<_>>()
    });
    spawn_local(async move {
        match tauri::circuits().await {
            Ok(known) => crafted.set(known),
            Err(failure) => said.set(Some(Err(lines::failure(&failure)))),
        }
    });
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some(name) = chosen(&names.get_untracked(), &circuit.get_untracked()) else {
            said.set(Some(Err(lines::CHOOSE_FIRST.into())));
            return;
        };
        let form = CircuitSessionForm {
            date: day.get_untracked(),
            circuit: name,
            sets: count_typed(&sets.get_untracked()),
        };
        spawn_local(async move {
            match tauri::log_circuit(&form).await {
                Ok(burn) => {
                    on_logged.run(burn);
                    circuit.set(String::new());
                    sets.set(String::new());
                    said.set(Some(Ok(lines::circuit_logged(
                        &form.circuit,
                        form.sets.unwrap_or(DEFAULT_SETS),
                    ))));
                }
                Err(failure) => said.set(Some(Err(lines::failure(&failure)))),
            }
        });
    };
    view! {
        {move || {
            crafted
                .get()
                .is_empty()
                .then(|| view! { <p class="empty">{lines::NO_CIRCUIT_TO_RUN}</p> })
        }}
        <form class="circuit-log" on:submit=log>
            <div class="row">
                <Chooser label="Circuit" name="circuit" options=names typed=circuit />
                <FigureField
                    label="Sets"
                    name="circuit_sets"
                    value=sets
                    placeholder=DEFAULT_SETS.to_string()
                />
            </div>
            <button class="cta" type="submit">
                "Run the circuit"
            </button>
        </form>
        {said_line(said)}
    }
}
