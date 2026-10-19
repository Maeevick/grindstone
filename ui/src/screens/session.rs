use grind_core::circuit::StepForm;
use grind_core::exercise::Exercise;
use grind_core::figures::{Figure, shown};
use grind_core::for_burning::DEFAULT_SETS;
use leptos::prelude::*;

use crate::screens::circuits::{asked_step, step_kcal};
use crate::screens::fields::FigureField;
use crate::screens::lines;
use crate::typed::count_typed;

/// The session in progress in Burn, kept while the app runs, so the athlete can check another tab
/// and come back to it.
#[derive(Clone, Copy)]
pub struct SessionInProgress {
    pub(crate) steps: RwSignal<Vec<StepForm>>,
    sets: RwSignal<String>,
}

impl Default for SessionInProgress {
    fn default() -> Self {
        Self {
            steps: RwSignal::new(Vec::new()),
            sets: RwSignal::new(String::new()),
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
) -> impl IntoView {
    let SessionInProgress { steps, sets } = session;
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
        </div>
    }
}
