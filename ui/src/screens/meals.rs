use grind_core::figures::{Figure, shown};
use grind_core::food::{Food, Unit};
use grind_core::for_cataloguing::{IngredientForm, MealForm};
use grind_core::meal::{Meal, food_of};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::chosen;
use crate::screens::fields::{Chooser, FigureField};
use crate::screens::lines;
use crate::tauri;
use crate::typed::figures_typed;

#[component]
pub fn NewMeal(
    foods: RwSignal<Vec<Food>>,
    meals: RwSignal<Vec<Meal>>,
    done: Callback<String>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let recipe = RwSignal::new(String::new());
    let ingredients = RwSignal::new(Vec::<IngredientForm>::new());
    let food = RwSignal::new(String::new());
    let quantity = RwSignal::new(String::new());
    let preview = RwSignal::new(None::<Meal>);
    let said = RwSignal::new(None::<String>);
    let names = Signal::derive(move || {
        foods
            .get()
            .into_iter()
            .map(|known| known.name)
            .collect::<Vec<_>>()
    });
    let add_ingredient = move |_| {
        let Some(chosen_food) = chosen(&names.get_untracked(), &food.get_untracked()) else {
            said.set(Some(lines::CHOOSE_FIRST.into()));
            return;
        };
        let Some([quantity_typed]) = figures_typed([&quantity.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        ingredients.update(|added| {
            added.push(IngredientForm {
                food: chosen_food,
                quantity: quantity_typed.unwrap_or(0.0),
                cooked_per_raw: None,
            })
        });
        food.set(String::new());
        quantity.set(String::new());
        said.set(None);
    };
    let form = move || {
        Some(MealForm {
            name: name.get_untracked(),
            recipe: recipe.get_untracked(),
            ingredients: ingredients.get_untracked(),
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
                    meals.set(kept);
                    if let Ok(known) = tauri::foods().await {
                        foods.set(known);
                    }
                    done.run(lines::cooked(&asked.name));
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
                "Recipe"
                <textarea
                    name="recipe"
                    rows="3"
                    prop:value=move || recipe.get()
                    on:input=move |event| recipe.set(event_target_value(&event))
                ></textarea>
            </label>
            <div class="row">
                <Chooser label="Ingredient" name="ingredient" options=names typed=food />
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
                            let unit = foods
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
        {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
    }
}

#[component]
pub fn MealsList(meals: RwSignal<Vec<Meal>>, said: RwSignal<Option<String>>) -> impl IntoView {
    let remove = move |named: String| {
        spawn_local(async move {
            match tauri::remove_meal(&named).await {
                Ok(kept) => meals.set(kept),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Meals"</h2>
            {move || meals.get().is_empty().then(|| view! { <p class="empty">{lines::NO_MEAL}</p> })}
            <table class="meals">
                <tbody>
                    <For
                        each=move || meals.get()
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
