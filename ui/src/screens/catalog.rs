use grind_core::exercise::Exercise;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::fields::FigureField;
use crate::screens::lines;
use crate::tauri;
use crate::typed::figure_typed;

#[component]
fn ExercisesCatalog() -> impl IntoView {
    let catalog = RwSignal::new(Vec::<Exercise>::new());
    let name = RwSignal::new(String::new());
    let met = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        match tauri::exercises().await {
            Ok(known) => catalog.set(known),
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
    });
    let add = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let named = name.get_untracked();
        let met_typed = figure_typed(&met.get_untracked()).unwrap_or(f64::NAN);
        spawn_local(async move {
            match tauri::add_exercise(&named, met_typed).await {
                Ok(known) => {
                    catalog.set(known);
                    name.set(String::new());
                    met.set(String::new());
                    said.set(None);
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    let remove = move |named: String| {
        spawn_local(async move {
            match tauri::remove_exercise(&named).await {
                Ok(known) => catalog.set(known),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Exercises"</h2>
            <form class="exercise" on:submit=add>
                <div class="row">
                    <label class="field grow2">
                        "Name"
                        <input
                            name="exercise_name"
                            autocomplete="off"
                            prop:value=move || name.get()
                            on:input=move |event| name.set(event_target_value(&event))
                        />
                    </label>
                    <FigureField label="MET" name="exercise_met" value=met />
                </div>
                <button class="cta" type="submit">
                    "Add to the catalog"
                </button>
            </form>
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            {move || catalog.get().is_empty().then(|| view! { <p class="empty">{lines::NO_EXERCISE}</p> })}
            <table class="exercises">
                <tbody>
                    <For
                        each=move || catalog.get()
                        key=|known| format!("{known:?}")
                        children=move |known| {
                            let named = known.name.clone();
                            view! {
                                <tr>
                                    <td>{known.name.clone()}</td>
                                    <td>{format!("{} MET", known.met)}</td>
                                    <td>
                                        <button
                                            class="remove"
                                            type="button"
                                            aria-label="Remove this exercise"
                                            on:click=move |_| remove(named.clone())
                                        >
                                            "×"
                                        </button>
                                    </td>
                                </tr>
                            }
                        }
                    />
                </tbody>
            </table>
        </section>
    }
}

#[component]
pub fn Catalog() -> impl IntoView {
    view! { <ExercisesCatalog /> }
}
