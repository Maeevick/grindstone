use grind_core::circuit::{CircuitStep, StepForm};
use grind_core::exercise::Exercise;
use grind_core::figures::{Figure, bare, shown};
use grind_core::for_cataloguing::PricedCircuit;
use grind_core::workout::activity_induced_energy_expenditure_from_compendium;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::{Recorded, lines};
use crate::tauri;

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

pub(crate) fn asked_step(step: &StepForm) -> String {
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

pub(crate) fn step_kcal(step: &StepForm, exercises: &[Exercise], body_mass_kg: f64) -> Option<f64> {
    exercises
        .iter()
        .find(|known| known.name == step.exercise)
        .map(|known| {
            activity_induced_energy_expenditure_from_compendium(
                body_mass_kg,
                known.met,
                step.minutes,
                step.load_kg,
            )
        })
}

#[component]
fn PricedRow(
    priced: PricedCircuit,
    opened: RwSignal<Option<String>>,
    remove: Callback<String>,
) -> impl IntoView {
    let named = priced.circuit.name.clone();
    let toggled = named.clone();
    let removed = named.clone();
    let shown_name = named.clone();
    let crafted = priced.circuit.clone();
    view! {
        <tr
            class="circuit"
            on:click=move |_| {
                opened.update(|shown| {
                    *shown = (shown.as_deref() != Some(toggled.as_str())).then(|| toggled.clone())
                })
            }
        >
            <td>{named.clone()}</td>
            <td>
                {priced
                    .burnt_per_set_kcal
                    .map_or_else(|| "-".to_string(), |kcal| bare(Figure::Energy(kcal)))}
            </td>
            <td>
                <button
                    class="remove"
                    type="button"
                    aria-label="Remove this circuit"
                    on:click=move |event| {
                        event.stop_propagation();
                        remove.run(removed.clone());
                    }
                >
                    "×"
                </button>
            </td>
        </tr>
        {move || {
            (opened.get().as_deref() == Some(shown_name.as_str()))
                .then(|| {
                    view! {
                        <tr class="details">
                            <td colspan="3">
                                {(!crafted.description.is_empty())
                                    .then(|| {
                                        view! { <p class="hint description">{crafted.description.clone()}</p> }
                                    })}
                                <ol class="figures">
                                    {crafted
                                        .steps
                                        .iter()
                                        .map(|step| view! { <li>{frozen_step(step)}</li> })
                                        .collect_view()}
                                </ol>
                            </td>
                        </tr>
                    }
                })
        }}
    }
}

#[component]
pub fn CircuitsList(
    priced: RwSignal<Vec<PricedCircuit>>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let opened = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let remove = Callback::new(move |named: String| {
        spawn_local(async move {
            match tauri::remove_circuit(&named).await {
                Ok(left) => {
                    priced.update(|kept| {
                        kept.retain(|priced| {
                            left.iter()
                                .any(|circuit| circuit.name == priced.circuit.name)
                        })
                    });
                    recorded.told();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    });
    view! {

            {move || {
                priced.get().is_empty().then(|| view! { <p class="empty">{lines::NO_CIRCUIT}</p> })
            }}
            <table class="circuits">
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"kcal / set"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        priced
                            .get()
                            .into_iter()
                            .map(|kept| view! { <PricedRow priced=kept opened=opened remove=remove /> })
                            .collect_view()
                    }}
                </tbody>
            </table>
    }
}
