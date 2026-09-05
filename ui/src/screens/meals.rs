use grind_core::figures::{Figure, shown};
use grind_core::food::{Food, Unit};
use grind_core::for_cataloguing::{IngredientForm, MealForm};
use grind_core::meal::{Cooking, Meal, food_of};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::fields::FigureField;
use crate::screens::lines;
use crate::tauri;
use crate::typed::figures_typed;

#[component]
pub fn MealCooker() -> impl IntoView {
    let catalog = RwSignal::new(Vec::<Food>::new());
    let cooked = RwSignal::new(Vec::<Meal>::new());
    let name = RwSignal::new(String::new());
    let recipe = RwSignal::new(String::new());
    let ingredients = RwSignal::new(Vec::<IngredientForm>::new());
    let food = RwSignal::new(String::new());
    let quantity = RwSignal::new(String::new());
    let by_ratio = RwSignal::new(false);
    let cooked_figure = RwSignal::new(String::new());
    let preview = RwSignal::new(None::<Meal>);
    let said = RwSignal::new(None::<String>);
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
        if let Ok(kept) = tauri::meals().await {
            cooked.set(kept);
        }
    });
    let add_ingredient = move |_| {
        let Some([quantity_typed]) = figures_typed([&quantity.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        ingredients.update(|added| {
            added.push(IngredientForm {
                food: food.get_untracked(),
                quantity: quantity_typed.unwrap_or(0.0),
            })
        });
        quantity.set(String::new());
    };
    let form = move || {
        let [figure] = figures_typed([&cooked_figure.get_untracked()])?;
        let figure = figure.unwrap_or(0.0);
        Some(MealForm {
            name: name.get_untracked(),
            recipe: recipe.get_untracked(),
            ingredients: ingredients.get_untracked(),
            cooking: if by_ratio.get_untracked() {
                Cooking::Ratio {
                    cooked_per_raw: figure / 100.0,
                }
            } else {
                Cooking::Weighed { cooked_g: figure }
            },
        })
    };
    let weigh_up = move |_| {
        let Some(asked) = form() else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        spawn_local(async move {
            match tauri::preview_meal(&asked).await {
                Ok(meal) => {
                    preview.set(Some(meal));
                    said.set(None);
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    let cook = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some(asked) = form() else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        spawn_local(async move {
            match tauri::cook_meal(&asked).await {
                Ok(kept) => {
                    cooked.set(kept);
                    said.set(Some(lines::cooked(&asked.name)));
                    for typed in [name, recipe, cooked_figure] {
                        typed.set(String::new());
                    }
                    ingredients.set(vec![]);
                    preview.set(None);
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    let remove = move |named: String| {
        spawn_local(async move {
            match tauri::remove_meal(&named).await {
                Ok(kept) => cooked.set(kept),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Trix's lab: the meal cooker"</h2>
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
                    "Recipe"
                    <textarea
                        name="recipe"
                        rows="3"
                        prop:value=move || recipe.get()
                        on:input=move |event| recipe.set(event_target_value(&event))
                    ></textarea>
                </label>
                <div class="row">
                    <label class="field grow2">
                        "Ingredient"
                        <select
                            name="ingredient"
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
                    <FigureField label="Quantity" name="ingredient_quantity" value=quantity />
                    <button class="cta ghost add-ingredient" type="button" on:click=add_ingredient>
                        "Add"
                    </button>
                </div>
                <ul class="ingredients">
                    {move || {
                        ingredients
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(at, ingredient)| {
                                let unit = catalog
                                    .get_untracked()
                                    .into_iter()
                                    .find(|known| known.name == ingredient.food)
                                    .map_or(Unit::Gram, |known| known.unit);
                                view! {
                                    <li>
                                        {format!(
                                            "{} {}",
                                            shown(Figure::Amount(ingredient.quantity, unit)),
                                            ingredient.food,
                                        )}
                                        <button
                                            class="remove"
                                            type="button"
                                            aria-label="Remove this ingredient"
                                            on:click=move |_| ingredients.update(|added| {
                                                added.remove(at);
                                            })
                                        >
                                            "×"
                                        </button>
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
                <div class="row">
                    <label class="field">
                        "After cooking"
                        <select
                            name="cooking"
                            on:change=move |event| by_ratio.set(event_target_value(&event) == "ratio")
                        >
                            <option value="weighed">"weighed (g)"</option>
                            <option value="ratio">"ratio of the raw mass (%)"</option>
                        </select>
                    </label>
                    <FigureField label="Cooked" name="cooked" value=cooked_figure />
                </div>
                <div class="row">
                    <button class="cta ghost weigh-up" type="button" on:click=weigh_up>
                        "Weigh it up"
                    </button>
                    <button class="cta" type="submit">
                        "Cook"
                    </button>
                </div>
            </form>
            {move || {
                preview
                    .get()
                    .map(|meal| {
                        let per_100_g = food_of(&meal).nutrients;
                        lines::weighed_up(&meal, &per_100_g)
                            .into_iter()
                            .map(|line| view! { <p class="figures">{line}</p> })
                            .collect_view()
                    })
            }}
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
        </section>
        <section class="card">
            <h2>"Meals"</h2>
            {move || cooked.get().is_empty().then(|| view! { <p class="empty">{lines::NO_MEAL}</p> })}
            <table class="meals">
                <tbody>
                    <For
                        each=move || cooked.get()
                        key=|meal| format!("{meal:?}")
                        children=move |meal| {
                            let named = meal.name.clone();
                            let per_100_g = food_of(&meal).nutrients;
                            view! {
                                <tr>
                                    <td>
                                        <details>
                                            <summary>{meal.name.clone()}</summary>
                                            <p class="hint">{meal.recipe.clone()}</p>
                                        </details>
                                    </td>
                                    <td>{format!("per 100 g: {}", lines::figures_of(&per_100_g))}</td>
                                    <td>
                                        <button
                                            class="remove"
                                            type="button"
                                            aria-label="Remove this meal"
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
