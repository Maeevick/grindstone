use grind_core::figures::{Figure, shown, typed_back};
use grind_core::food::{Food, Unit};
use grind_core::for_cataloguing::{IngredientForm, MealForm};
use grind_core::meal::{MASS_KEPT, Meal, food_of};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::chosen;
use crate::screens::fields::{Chooser, FigureField};
use crate::screens::lines;
use crate::tauri;
use crate::typed::figures_typed;

#[derive(Clone)]
struct InThePot {
    food: String,
    unit: Unit,
    quantity: f64,
    ratio: RwSignal<String>,
}

#[component]
pub fn NewMeal(
    foods: RwSignal<Vec<Food>>,
    meals: RwSignal<Vec<Meal>>,
    done: Callback<String>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let food = RwSignal::new(String::new());
    let quantity = RwSignal::new(String::new());
    let pot = RwSignal::new(Vec::<InThePot>::new());
    let said = RwSignal::new(None::<String>);
    let names = Signal::derive(move || {
        foods
            .get()
            .into_iter()
            .map(|known| known.name)
            .collect::<Vec<_>>()
    });
    let add_ingredient = move |_| {
        let Some(known) = chosen(&names.get_untracked(), &food.get_untracked()).and_then(|named| {
            foods
                .get_untracked()
                .into_iter()
                .find(|known| known.name == named)
        }) else {
            said.set(Some(lines::CHOOSE_FIRST.into()));
            return;
        };
        let Some([quantity_typed]) = figures_typed([&quantity.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        pot.update(|added| {
            added.push(InThePot {
                food: known.name,
                unit: known.unit,
                quantity: quantity_typed.unwrap_or(0.0),
                ratio: RwSignal::new(String::new()),
            })
        });
        food.set(String::new());
        quantity.set(String::new());
        said.set(None);
    };
    let cook = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let in_the_pot = pot.get_untracked();
        let Some(ratios) = in_the_pot
            .iter()
            .map(|added| figures_typed([&added.ratio.get_untracked()]).map(|[ratio]| ratio))
            .collect::<Option<Vec<_>>>()
        else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = MealForm {
            name: name.get_untracked(),
            description: description.get_untracked(),
            ingredients: in_the_pot
                .iter()
                .zip(ratios)
                .map(|(added, cooked_per_raw)| IngredientForm {
                    food: added.food.clone(),
                    quantity: added.quantity,
                    cooked_per_raw,
                })
                .collect(),
        };
        spawn_local(async move {
            match tauri::cook_meal(&form).await {
                Ok(kept) => {
                    meals.set(kept);
                    if let Ok(known) = tauri::foods().await {
                        foods.set(known);
                    }
                    for typed in [name, description, food, quantity] {
                        typed.set(String::new());
                    }
                    pot.set(Vec::new());
                    done.run(lines::cooked(form.name.trim()));
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <form class="meal" on:submit=cook>
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
                <Chooser label="Ingredient" name="ingredient" options=names typed=food />
                <FigureField label="Quantity" name="ingredient_quantity" value=quantity />
                <button class="cta ghost add-ingredient" type="button" on:click=add_ingredient>
                    "Add"
                </button>
            </div>
            <Show when=move || !pot.get().is_empty()>
            <table class="pot">
                <thead>
                    <tr>
                        <th>"Ingredient"</th>
                        <th>"Ratio"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        pot.get()
                            .into_iter()
                            .enumerate()
                            .map(|(at, added)| {
                                let ratio = added.ratio;
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
                                                on:input=move |event| ratio.set(event_target_value(&event))
                                            />
                                        </td>
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
            </table>
            </Show>
            <p class="hint">{lines::COOKING_RATIO_HOW}</p>
            <button class="cta" type="submit">
                "Cook"
            </button>
        </form>
        {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
    }
}

#[component]
fn KeptMeal(meal: Meal, remove: Callback<String>) -> impl IntoView {
    let named = meal.name.clone();
    let per_100_g = food_of(&meal).nutrients;
    view! {
        <tr>
            <td>
                <details>
                    <summary>{meal.name.clone()}</summary>
                    {(!meal.description.is_empty())
                        .then(|| view! { <p class="hint">{meal.description.clone()}</p> })}
                    <ul class="figures">
                        {meal
                            .ingredients
                            .iter()
                            .map(|ingredient| view! { <li>{lines::cooked_ingredient(ingredient)}</li> })
                            .collect_view()}
                    </ul>
                </details>
                <p class="hint">{lines::meal_totals(&meal)}</p>
                <p class="hint">{lines::per_100_g_cooked(&per_100_g)}</p>
            </td>
            <td>
                <button
                    class="remove"
                    type="button"
                    aria-label="Remove this meal"
                    on:click=move |_| remove.run(named.clone())
                >
                    "×"
                </button>
            </td>
        </tr>
    }
}

#[component]
pub fn MealsList(meals: RwSignal<Vec<Meal>>, said: RwSignal<Option<String>>) -> impl IntoView {
    let remove = Callback::new(move |named: String| {
        spawn_local(async move {
            match tauri::remove_meal(&named).await {
                Ok(kept) => meals.set(kept),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    });
    view! {
        <section class="card">
            <h2>"Meals"</h2>
            {move || meals.get().is_empty().then(|| view! { <p class="empty">{lines::NO_MEAL}</p> })}
            <table class="meals">
                <tbody>
                    <For
                        each=move || meals.get()
                        key=|meal| format!("{meal:?}")
                        children=move |meal| view! { <KeptMeal meal=meal remove=remove /> }
                    />
                </tbody>
            </table>
        </section>
    }
}
