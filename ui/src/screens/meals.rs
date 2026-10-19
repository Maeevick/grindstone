use grind_core::figures::{Figure, shown, typed_back};
use grind_core::food::{Food, Nutrients, Unit, nutrients_of, summed};
use grind_core::for_cataloguing::{IngredientForm, MealForm};
use grind_core::meal::{MASS_KEPT, Meal};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::chosen;
use crate::screens::cells::nutrient_cells;
use crate::screens::fields::{Chooser, FigureField, preview_line};
use crate::screens::lines;
use crate::tauri;
use crate::typed::figures_typed;

/// The meal on the lab's bench, kept while the app runs, so the cook can check the day's gaps in
/// another tab and come back to it.
#[derive(Clone, Copy)]
pub struct MealInProgress {
    name: RwSignal<String>,
    description: RwSignal<String>,
    food: RwSignal<String>,
    quantity: RwSignal<String>,
    pot: RwSignal<Vec<InThePot>>,
}

impl Default for MealInProgress {
    fn default() -> Self {
        Self {
            name: RwSignal::new(String::new()),
            description: RwSignal::new(String::new()),
            food: RwSignal::new(String::new()),
            quantity: RwSignal::new(String::new()),
            pot: RwSignal::new(Vec::new()),
        }
    }
}

impl MealInProgress {
    pub fn started(&self) -> bool {
        [self.name, self.description, self.food, self.quantity]
            .iter()
            .any(|typed| !typed.get_untracked().is_empty())
            || !self.pot.get_untracked().is_empty()
    }
}

#[derive(Clone)]
struct InThePot {
    food: String,
    unit: Unit,
    quantity: f64,
    nutrients: Nutrients,
    ratio: ArcRwSignal<String>,
}

#[component]
pub fn NewMeal(
    foods: RwSignal<Vec<Food>>,
    meals: RwSignal<Vec<Meal>>,
    done: Callback<String>,
) -> impl IntoView {
    let MealInProgress {
        name,
        description,
        food,
        quantity,
        pot,
    } = expect_context();
    let said = RwSignal::new(None::<String>);
    let names = Signal::derive(move || {
        foods
            .get()
            .into_iter()
            .map(|known| known.name)
            .collect::<Vec<_>>()
    });
    let chosen_food = move || {
        chosen(&names.get(), &food.get())
            .and_then(|named| foods.get().into_iter().find(|known| known.name == named))
    };
    let preview = move || {
        let known = chosen_food()?;
        let [typed] = figures_typed([&quantity.get()])?;
        typed.map(|typed| lines::figures_of(&nutrients_of(&known, typed)))
    };
    let add_ingredient = move |_| {
        let Some(known) = chosen_food() else {
            said.set(Some(lines::CHOOSE_FIRST.into()));
            return;
        };
        let Some([quantity_typed]) = figures_typed([&quantity.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let quantity_typed = quantity_typed.unwrap_or(0.0);
        pot.update(|added| {
            added.push(InThePot {
                nutrients: nutrients_of(&known, quantity_typed),
                food: known.name,
                unit: known.unit,
                quantity: quantity_typed,
                ratio: ArcRwSignal::new(String::new()),
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
            {preview_line(preview)}
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
            <p class="hint">{lines::COOKING_RATIO_HOW}</p>
            <button class="cta" type="submit">
                "Cook"
            </button>
        </form>
        {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
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
