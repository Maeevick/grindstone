use grind_core::figures::{Figure, shown, typed_back};
use grind_core::food::{Food, Nutrients, Unit, nutrients_of, summed};
use grind_core::for_cataloguing::{IngredientForm, MealForm};
use grind_core::for_fuelling::{IntakeForm, PlateForm};
use grind_core::fuel::Fuel;
use grind_core::meal::{MASS_KEPT, Meal};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::cells::nutrient_cells;
use crate::screens::fields::FigureField;
use crate::screens::{Recorded, lines};
use crate::tauri;
use crate::typed::figures_typed;

/// The meal on the lab's bench, kept while the app runs, so the cook can check the day's gaps in
/// another tab and come back to it.
#[derive(Clone, Copy)]
pub struct MealInProgress {
    name: RwSignal<String>,
    description: RwSignal<String>,
    pub(crate) pot: RwSignal<Vec<InThePot>>,
    keeping: RwSignal<bool>,
    eaten: RwSignal<String>,
}

impl Default for MealInProgress {
    fn default() -> Self {
        Self {
            name: RwSignal::new(String::new()),
            description: RwSignal::new(String::new()),
            pot: RwSignal::new(Vec::new()),
            keeping: RwSignal::new(false),
            eaten: RwSignal::new(String::new()),
        }
    }
}

#[derive(Clone)]
pub(crate) struct InThePot {
    food: String,
    unit: Unit,
    quantity: f64,
    pub(crate) nutrients: Nutrients,
    ratio: ArcRwSignal<String>,
}

impl InThePot {
    pub(crate) fn of(known: &Food, quantity: f64) -> Self {
        Self {
            food: known.name.clone(),
            unit: known.unit,
            quantity,
            nutrients: nutrients_of(known, quantity),
            ratio: ArcRwSignal::new(String::new()),
        }
    }
}

/// What the pot holds, each ingredient with its cooking ratio, and their total.
#[component]
fn PotTable(pot: RwSignal<Vec<InThePot>>) -> impl IntoView {
    view! {
        <Show when=move || !pot.get().is_empty()>
            <div class="scroll">
                <table class="pot">
                    <thead>
                        <tr>
                            <th>"Ingredient"</th>
                            <th>"Ratio"</th>
                            <th>"kcal"</th>
                            <th>"Prot (g)"</th>
                            <th>"Fat (g)"</th>
                            <th>"Fib (g)"</th>
                            <th>"Carbs (g)"</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || {
                            pot.get()
                                .into_iter()
                                .enumerate()
                                .map(|(at, added)| {
                                    let (ratio, typed) = (added.ratio.clone(), added.ratio);
                                    view! {
                                        <tr>
                                            <td>
                                                {format!(
                                                    "{} {}",
                                                    shown(Figure::Amount(added.quantity, added.unit)),
                                                    added.food,
                                                )}
                                            </td>
                                            <td>
                                                <input
                                                    class="ratio"
                                                    name=format!("ratio_{at}")
                                                    inputmode="decimal"
                                                    autocomplete="off"
                                                    placeholder=typed_back(MASS_KEPT)
                                                    prop:value=move || ratio.get()
                                                    on:input=move |event| typed.set(event_target_value(&event))
                                                />
                                            </td>
                                            {nutrient_cells(added.nutrients)}
                                            <td>
                                                <button
                                                    class="remove"
                                                    type="button"
                                                    aria-label="Remove this ingredient"
                                                    on:click=move |_| pot.update(|added| {
                                                        added.remove(at);
                                                    })
                                                >
                                                    "×"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                        }}
                    </tbody>
                    <tfoot>
                        <tr>
                            <td>"Total"</td>
                            <td></td>
                            {move || nutrient_cells(summed(pot.get().into_iter().map(|added| added.nutrients)))}
                            <td></td>
                        </tr>
                    </tfoot>
                </table>
            </div>
        </Show>
    }
}

/// Each ingredient in the pot as the core takes it, none when a ratio is no number.
fn ingredients_of(in_the_pot: &[InThePot]) -> Option<Vec<IngredientForm>> {
    in_the_pot
        .iter()
        .map(|added| {
            figures_typed([&added.ratio.get_untracked()]).map(|[cooked_per_raw]| IngredientForm {
                food: added.food.clone(),
                quantity: added.quantity,
                cooked_per_raw,
            })
        })
        .collect()
}

/// The foods put aside in Fuel before they are logged, on the meal in progress's pot.
#[component]
pub fn Plate(
    plate: MealInProgress,
    day: RwSignal<String>,
    shown_fuel: Callback<Fuel>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let MealInProgress {
        name,
        description,
        pot,
        keeping,
        eaten,
        ..
    } = plate;
    let recorded = Recorded::here();
    // Last, as it takes the plate off the screen.
    let cleared = move || {
        for typed in [name, description, eaten] {
            typed.set(String::new());
        }
        keeping.set(false);
        pot.set(Vec::new());
    };
    let log_the_plate = move |_| {
        let Some(ingredients) = ingredients_of(&pot.get_untracked()) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = PlateForm {
            date: day.get_untracked(),
            ingredients,
        };
        spawn_local(async move {
            match tauri::log_plate(&form).await {
                Ok(fuel) => {
                    shown_fuel.run(fuel);
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
        let Some(ingredients) = ingredients_of(&pot.get_untracked()) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let Some([eaten_typed]) = figures_typed([&eaten.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = MealForm {
            name: name.get_untracked(),
            description: description.get_untracked(),
            ingredients,
        };
        let date = day.get_untracked();
        spawn_local(async move {
            let kept = match tauri::cook_meal(&form).await {
                Ok(kept) => kept,
                Err(failure) => {
                    said.set(Some(lines::failure(&failure)));
                    return;
                }
            };
            let named = form.name.trim().to_string();
            let whole = kept
                .iter()
                .find(|meal| meal.name == named)
                .map(|meal| meal.cooked_g);
            match eaten_typed.or(whole).filter(|grams| *grams > 0.0) {
                Some(grams) => {
                    let intake = IntakeForm {
                        date,
                        food: named,
                        quantity: Some(grams),
                    };
                    match tauri::log_intake(&intake).await {
                        Ok(fuel) => {
                            shown_fuel.run(fuel);
                            said.set(None);
                        }
                        Err(failure) => said.set(Some(lines::failure(&failure))),
                    }
                }
                None => said.set(Some(lines::kept_for_later(&named))),
            }
            recorded.told();
            cleared();
        });
    };
    view! {
        <div class="plate">
            <p class="new">"On the plate"</p>
            <PotTable pot=pot />
            <button class="cta log-the-plate" type="button" on:click=log_the_plate>
                "Log the plate"
            </button>
            <button
                class="cta ghost keep-as-a-meal"
                type="button"
                on:click=move |_| keeping.update(|keeping| *keeping = !*keeping)
            >
                "Keep as a meal"
            </button>
            <Show when=move || keeping.get()>
                <form class="meal" on:submit=keep>
                    <label class="field grow2">
                        "Meal"
                        <input
                            name="meal_name"
                            autocomplete="off"
                            prop:value=move || name.get()
                            on:input=move |event| name.set(event_target_value(&event))
                        />
                    </label>
                    <label class="field grow2">
                        "Description"
                        <textarea
                            name="meal_description"
                            rows="3"
                            prop:value=move || description.get()
                            on:input=move |event| description.set(event_target_value(&event))
                        ></textarea>
                    </label>
                    <div class="row">
                        <FigureField
                            label="Eaten now (g)"
                            name="eaten"
                            value=eaten
                            placeholder="the whole plate"
                        />
                    </div>
                    <p class="hint">{lines::COOKING_RATIO_HOW}</p>
                    <p class="hint">{lines::EATEN_HOW}</p>
                    <button class="cta" type="submit">
                        "Keep the meal"
                    </button>
                </form>
            </Show>
        </div>
    }
}

#[component]
pub fn MealDetails(meal: Meal) -> impl IntoView {
    view! {
        <tr class="details">
            <td colspan="8">
                <div class="pinned">
                    {(!meal.description.is_empty())
                        .then(|| view! { <p class="hint description">{meal.description.clone()}</p> })}
                    <ul class="figures">
                        {meal
                            .ingredients
                            .iter()
                            .map(|ingredient| view! { <li>{lines::cooked_ingredient(ingredient)}</li> })
                            .collect_view()}
                    </ul>
                </div>
            </td>
        </tr>
    }
}
