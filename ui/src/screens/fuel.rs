use grind_core::figures::{Figure, shown};
use grind_core::food::Food;
use grind_core::for_fuelling::IntakeForm;
use grind_core::fuel::{Fuel, MacroRow};
use grind_core::intake::Intake;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::lines;
use crate::tauri;
use crate::typed::figure_typed;

#[component]
fn MacroLine(name: &'static str, row: MacroRow) -> impl IntoView {
    view! {
        <tr class=lines::verdict_class(row.verdict)>
            <td>{name}</td>
            <td>{shown(Figure::Grams(row.eaten_g))}</td>
            <td>
                {format!(
                    "{} to {}",
                    shown(Figure::Grams(row.floor_g)).trim_end_matches(" g"),
                    shown(Figure::Grams(row.ceiling_g)),
                )}
            </td>
            <td>{lines::verdict_word(row.verdict)}</td>
        </tr>
    }
}

#[component]
fn IntakeRow(
    id: i64,
    eaten: Intake,
    day: RwSignal<String>,
    shown_fuel: Callback<Fuel>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let remove = move |_| {
        let date = day.get_untracked();
        spawn_local(async move {
            match tauri::remove_intake(&date, id).await {
                Ok(fuel) => shown_fuel.run(fuel),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <tr>
            <td>{eaten.food.clone()}</td>
            <td>{shown(Figure::Amount(eaten.quantity, eaten.unit))}</td>
            <td>{shown(Figure::Energy(eaten.nutrients.energy_kcal))}</td>
            <td>
                <button class="remove" type="button" on:click=remove aria-label="Remove this intake">
                    "×"
                </button>
            </td>
        </tr>
    }
}

#[component]
pub fn FuelDay(day: RwSignal<String>) -> impl IntoView {
    let fuel = RwSignal::new(None::<Fuel>);
    let catalog = RwSignal::new(Vec::<Food>::new());
    let food = RwSignal::new(String::new());
    let quantity = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    let shown_fuel = Callback::new(move |answer: Fuel| fuel.set(Some(answer)));
    spawn_local(async move {
        match tauri::foods().await {
            Ok(known) => {
                if let Some(first) = known.first() {
                    food.set(first.name.clone());
                }
                catalog.set(known);
            }
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
    });
    Effect::new(move |_| {
        let date = day.get();
        spawn_local(async move {
            match tauri::fuel_day(&date).await {
                Ok(answer) => shown_fuel.run(answer),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    });
    let unit_of_the_food = move || {
        catalog
            .get()
            .into_iter()
            .find(|known| known.name == food.get())
            .map(|known| shown(Figure::Amount(known.portion, known.unit)))
            .map(|portion| format!("Quantity (figures per {portion})"))
            .unwrap_or_else(|| "Quantity".into())
    };
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let form = IntakeForm {
            date: day.get_untracked(),
            food: food.get_untracked(),
            quantity: figure_typed(&quantity.get_untracked()).unwrap_or(f64::NAN),
        };
        spawn_local(async move {
            match tauri::log_intake(&form).await {
                Ok(answer) => {
                    shown_fuel.run(answer);
                    quantity.set(String::new());
                    said.set(None);
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Fuel"</h2>
            <form class="intake" on:submit=log>
                <div class="row">
                    <label class="field grow2">
                        "Food"
                        <select
                            name="food"
                            prop:value=move || food.get()
                            on:change=move |event| food.set(event_target_value(&event))
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
                    <label class="field">
                        {unit_of_the_food}
                        <input
                            name="quantity"
                            inputmode="decimal"
                            autocomplete="off"
                            prop:value=move || quantity.get()
                            on:input=move |event| quantity.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <button class="cta" type="submit">
                    "Fuel"
                </button>
            </form>
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            {move || {
                fuel.get()
                    .map(|answer| {
                        let energy_class = answer.energy.map(lines::verdict_class).unwrap_or("hint");
                        view! {
                            <p class=format!("figures {energy_class}")>{lines::energy_line(&answer)}</p>
                            {answer.band.is_none().then(|| view! { <p class="hint">{lines::NO_BAND}</p> })}
                            {answer
                                .macros
                                .map(|macros| {
                                    view! {
                                        <table class="macros">
                                            <tbody>
                                                <MacroLine name="Protein" row=macros.protein />
                                                <MacroLine name="Fat" row=macros.fat />
                                                <MacroLine name="Fibre" row=macros.fibre />
                                                <MacroLine name="Carbs" row=macros.carbs />
                                            </tbody>
                                        </table>
                                    }
                                })}
                            {answer.intakes.is_empty().then(|| view! { <p class="empty">{lines::NO_INTAKE}</p> })}
                        }
                    })
            }}
            <table class="intakes">
                <tbody>
                    <For
                        each=move || fuel.get().map(|answer| answer.intakes).unwrap_or_default()
                        key=|(id, _)| *id
                        children=move |(id, eaten)| {
                            view! { <IntakeRow id=id eaten=eaten day=day shown_fuel=shown_fuel said=said /> }
                        }
                    />
                </tbody>
            </table>
        </section>
    }
}
