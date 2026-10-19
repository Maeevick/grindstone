use grind_core::burn::Burn;
use grind_core::circuit::StepForm;
use grind_core::exercise::Exercise;
use grind_core::figures::{Figure, shown};
use grind_core::for_burning::{CircuitSessionForm, DEFAULT_SETS, SessionForm};
use grind_core::for_cataloguing::CircuitForm;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::circuits::{asked_step, step_kcal};
use crate::screens::fields::FigureField;
use crate::screens::{Recorded, lines};
use crate::tauri;
use crate::typed::count_typed;

/// The session in progress in Burn, kept while the app runs, so the athlete can check another tab
/// and come back to it.
#[derive(Clone, Copy)]
pub struct SessionInProgress {
    pub(crate) steps: RwSignal<Vec<StepForm>>,
    sets: RwSignal<String>,
    name: RwSignal<String>,
    description: RwSignal<String>,
    keeping: RwSignal<bool>,
}

impl Default for SessionInProgress {
    fn default() -> Self {
        Self {
            steps: RwSignal::new(Vec::new()),
            sets: RwSignal::new(String::new()),
            name: RwSignal::new(String::new()),
            description: RwSignal::new(String::new()),
            keeping: RwSignal::new(false),
        }
    }
}

impl SessionInProgress {
    /// What one set of the session burns at the body mass given, each step priced on the catalog.
    pub(crate) fn per_set(self, exercises: &[Exercise], body_mass_kg: f64) -> f64 {
        self.steps
            .get()
            .iter()
            .filter_map(|step| step_kcal(step, exercises, body_mass_kg))
            .sum()
    }

    /// The sets typed, one when none are.
    pub(crate) fn sets(self) -> u32 {
        count_typed(&self.sets.get()).unwrap_or(DEFAULT_SETS)
    }
}

/// The steps put aside in Burn before they are logged, priced a set at the day's body mass.
#[component]
pub fn Session(
    session: SessionInProgress,
    exercises: Signal<Vec<Exercise>>,
    body_mass: Signal<Option<f64>>,
    day: RwSignal<String>,
    shown_burn: Callback<Burn>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let SessionInProgress {
        steps,
        sets,
        name,
        description,
        keeping,
    } = session;
    let recorded = Recorded::here();
    // Last, as it takes the session off the screen.
    let cleared = move || {
        for typed in [sets, name, description] {
            typed.set(String::new());
        }
        keeping.set(false);
        steps.set(Vec::new());
    };
    let log_the_session = move |_| {
        let form = SessionForm {
            date: day.get_untracked(),
            steps: steps.get_untracked(),
            sets: count_typed(&sets.get_untracked()),
        };
        spawn_local(async move {
            match tauri::log_session(&form).await {
                Ok(burn) => {
                    shown_burn.run(burn);
                    said.set(None);
                    recorded.told();
                    cleared();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    let keep = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let form = CircuitForm {
            name: name.get_untracked(),
            description: description.get_untracked(),
            steps: steps.get_untracked(),
        };
        let times = count_typed(&sets.get_untracked()).unwrap_or(DEFAULT_SETS);
        let date = day.get_untracked();
        spawn_local(async move {
            if let Err(failure) = tauri::craft_circuit(&form).await {
                said.set(Some(lines::failure(&failure)));
                return;
            }
            let named = form.name.trim().to_string();
            if times == 0 {
                said.set(Some(lines::circuit_kept_for_later(&named)));
            } else {
                let run = CircuitSessionForm {
                    date,
                    circuit: named,
                    sets: Some(times),
                };
                match tauri::log_circuit(&run).await {
                    Ok(burn) => {
                        shown_burn.run(burn);
                        said.set(None);
                    }
                    Err(failure) => said.set(Some(lines::failure(&failure))),
                }
            }
            recorded.told();
            cleared();
        });
    };
    let steps_in_order = move || {
        steps
            .get()
            .into_iter()
            .enumerate()
            .collect::<Vec<(usize, StepForm)>>()
    };
    let per_set = move || {
        body_mass
            .get()
            .map(|body_mass_kg| lines::a_set(session.per_set(&exercises.get(), body_mass_kg)))
    };
    view! {
        <div class="session">
            <p class="new">"In the session"</p>
            <ol class="figures steps">
                <For
                    each=steps_in_order
                    key=|(position, step)| format!("{position} {step:?}")
                    children=move |(position, step)| {
                        let priced = body_mass
                            .get()
                            .and_then(|body_mass_kg| step_kcal(&step, &exercises.get(), body_mass_kg));
                        view! {
                            <li>
                                {asked_step(&step)}
                                {priced
                                    .map(|kcal| {
                                        view! {
                                            " · "
                                            <span class="kcal">{shown(Figure::Energy(kcal))}</span>
                                        }
                                    })}
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
            <div class="row">
                <FigureField
                    label="Sets"
                    name="session_sets"
                    value=sets
                    placeholder=DEFAULT_SETS.to_string()
                />
            </div>
            {move || per_set().map(|line| view! { <p class="hint per-set">{line}</p> })}
            <button class="cta log-the-session" type="button" on:click=log_the_session>
                "Log the session"
            </button>
            <button
                class="cta ghost keep-as-a-circuit"
                type="button"
                on:click=move |_| keeping.update(|keeping| *keeping = !*keeping)
            >
                "Keep as a circuit"
            </button>
            <Show when=move || keeping.get()>
                <form class="circuit" on:submit=keep>
                    <label class="field grow2">
                        "Circuit"
                        <input
                            name="circuit_name"
                            autocomplete="off"
                            prop:value=move || name.get()
                            on:input=move |event| name.set(event_target_value(&event))
                        />
                    </label>
                    <label class="field grow2">
                        "Description"
                        <textarea
                            name="circuit_description"
                            rows="2"
                            placeholder="Where it comes from, what to mind"
                            prop:value=move || description.get()
                            on:input=move |event| description.set(event_target_value(&event))
                        ></textarea>
                    </label>
                    <p class="hint">{lines::KEEP_CIRCUIT_HOW}</p>
                    <button class="cta" type="submit">
                        "Keep the circuit"
                    </button>
                </form>
            </Show>
        </div>
    }
}
