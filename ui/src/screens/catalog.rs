use grind_core::drink::Drink;
use grind_core::exercise::Exercise;
use grind_core::figures::{Figure, shown};
use grind_core::food::Food;
use grind_core::for_cataloguing::PricedCircuit;
use grind_core::meal::{Meal, food_of};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::cells::nutrient_cells;
use crate::screens::circuits::{CircuitsList, NewCircuit};
use crate::screens::entries::{
    ExerciseFields, ExerciseFigures, FoodFields, FoodFigures, NameField, name_cell,
};
use crate::screens::hydrate::{DrinksList, NewDrink};
use crate::screens::{Recorded, lines, today};
use crate::tauri;
use crate::typed::figures_typed;

fn said_on_the_shelf(said: RwSignal<Option<String>>) -> impl IntoView {
    move || said.get().map(|line| view! { <p class="said">{line}</p> })
}

#[component]
fn NewExercise(exercises: RwSignal<Vec<Exercise>>, done: Callback<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let figures = ExerciseFigures::default();
    let said = RwSignal::new(None::<String>);
    let add = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let named = name.get_untracked();
        let Some([met_typed]) = figures_typed([&figures.met.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let met_typed = met_typed.unwrap_or(0.0);
        let description = figures.description.get_untracked();
        spawn_local(async move {
            match tauri::add_exercise(&named, met_typed, &description).await {
                Ok(known) => {
                    exercises.set(known);
                    name.set(String::new());
                    figures.cleared();
                    done.run(lines::added(named.trim()));
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <form class="exercise" on:submit=add>
            <div class="row">
                <NameField name="exercise_name" value=name />
            </div>
            <ExerciseFields prefix="exercise" figures=figures />
            <button class="cta" type="submit">
                "Add to the catalog"
            </button>
            {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
        </form>
    }
}

#[component]
fn ExercisesList(
    exercises: RwSignal<Vec<Exercise>>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let recorded = Recorded::here();
    let remove = move |named: String| {
        spawn_local(async move {
            match tauri::remove_exercise(&named).await {
                Ok(known) => {
                    exercises.set(known);
                    recorded.told();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Exercises"</h2>
            {move || exercises.get().is_empty().then(|| view! { <p class="empty">{lines::NO_EXERCISE}</p> })}
            <table class="exercises">
                <tbody>
                    <For
                        each=move || exercises.get()
                        key=|known| format!("{known:?}")
                        children=move |known| {
                            let named = known.name.clone();
                            view! {
                                <tr>
                                    {name_cell(known.name.clone(), known.description.clone())}
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
fn NewFood(foods: RwSignal<Vec<Food>>, done: Callback<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let figures = FoodFigures::default();
    let said = RwSignal::new(None::<String>);
    let add = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some(form) = untrack(|| figures.typed(&name.get())) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        spawn_local(async move {
            match tauri::add_food(&form).await {
                Ok(known) => {
                    foods.set(known);
                    name.set(String::new());
                    figures.cleared();
                    done.run(lines::added(&form.name));
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <form class="food" on:submit=add>
            <div class="row">
                <NameField name="food_name" value=name />
            </div>
            <FoodFields prefix="food" figures=figures />
            <button class="cta" type="submit">
                "Add to the catalog"
            </button>
            {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
        </form>
    }
}

fn figure_cells(food: &Food) -> impl IntoView + use<> {
    view! {
        <td>{shown(Figure::Amount(food.portion, food.unit))}</td>
        {nutrient_cells(food.nutrients)}
    }
}

fn figure_heads() -> impl IntoView {
    view! {
        <thead>
            <tr>
                <th>"Name"</th>
                <th>"Per"</th>
                <th>"kcal"</th>
                <th>"Prot (g)"</th>
                <th>"Fat (g)"</th>
                <th>"Fib (g)"</th>
                <th>"Carbs (g)"</th>
                <th></th>
            </tr>
        </thead>
    }
}

fn remove_button(remove: impl Fn() + 'static) -> impl IntoView {
    view! {
        <td>
            <button
                class="remove"
                type="button"
                aria-label="Remove from the catalog"
                on:click=move |event| {
                    event.stop_propagation();
                    remove();
                }
            >
                "×"
            </button>
        </td>
    }
}

#[component]
fn FoodsList(
    foods: RwSignal<Vec<Food>>,
    meals: RwSignal<Vec<Meal>>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let recorded = Recorded::here();
    let remove = move |named: String| {
        spawn_local(async move {
            match tauri::remove_food(&named).await {
                Ok(known) => {
                    foods.set(known);
                    recorded.told();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    let plain = move || {
        let cooked = meals.get();
        foods
            .get()
            .into_iter()
            .filter(|food| !cooked.iter().any(|meal| meal.name == food.name))
            .collect::<Vec<_>>()
    };
    view! {
        {move || plain().is_empty().then(|| view! { <p class="empty">{lines::NO_FOOD}</p> })}
            <div class="scroll">
                <table class="foods">
                    {figure_heads()}
                    <tbody>
                        {move || {
                            plain()
                                .into_iter()
                                .map(|food| {
                                    let named = food.name.clone();
                                    view! {
                                        <tr>
                                            {name_cell(food.name.clone(), food.description.clone())}
                                            {figure_cells(&food)}
                                            {remove_button(move || remove(named.clone()))}
                                        </tr>
                                    }
                                })
                                .collect_view()
                        }}
                    </tbody>
                </table>
            </div>
    }
}

#[component]
fn MealsList(meals: RwSignal<Vec<Meal>>, said: RwSignal<Option<String>>) -> impl IntoView {
    let opened = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let remove = move |named: String| {
        spawn_local(async move {
            match tauri::remove_meal(&named).await {
                Ok(kept) => {
                    meals.set(kept);
                    recorded.told();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        {move || meals.get().is_empty().then(|| view! { <p class="empty">{lines::NO_MEAL}</p> })}
            <div class="scroll">
                <table class="meals">
                    {figure_heads()}
                    <tbody>
                        {move || {
                            meals
                                .get()
                                .into_iter()
                                .map(|meal| {
                                    let named = meal.name.clone();
                                    let toggled = named.clone();
                                    let shown_name = named.clone();
                                    let detailed = meal.clone();
                                    view! {
                                        <tr
                                            class="meal"
                                            on:click=move |_| {
                                                opened.update(|shown| {
                                                    *shown = (shown.as_deref() != Some(toggled.as_str()))
                                                        .then(|| toggled.clone())
                                                })
                                            }
                                        >
                                            <td>{meal.name.clone()}</td>
                                            {figure_cells(&food_of(&meal))}
                                            {remove_button(move || remove(named.clone()))}
                                        </tr>
                                        {move || {
                                            (opened.get().as_deref() == Some(shown_name.as_str()))
                                                .then(|| view! { <MealDetails meal=detailed.clone() /> })
                                        }}
                                    }
                                })
                                .collect_view()
                        }}
                    </tbody>
                </table>
            </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Form {
    Exercise,
    Circuit,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Listed {
    Foods,
    Meals,
}

fn shelf_pills(chosen: RwSignal<Listed>) -> impl IntoView {
    view! {
        <div class="pills">
            {[(Listed::Foods, "Foods"), (Listed::Meals, "Meals")]
                .into_iter()
                .map(|(named, label)| {
                    view! {
                        <button
                            type="button"
                            class=format!("pill shelf-{named:?}")
                            class:chosen=move || chosen.get() == named
                            on:click=move |_| chosen.set(named)
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

fn form_pills(chosen: RwSignal<Form>, forms: [(Form, &'static str); 2]) -> impl IntoView {
    view! {
        <div class="pills">
            {forms
                .into_iter()
                .map(|(named, label)| {
                    view! {
                        <button
                            type="button"
                            class=format!("pill form-{named:?}")
                            class:chosen=move || chosen.get() == named
                            on:click=move |_| chosen.set(named)
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
pub fn FuelShelf() -> impl IntoView {
    let foods = RwSignal::new(Vec::<Food>::new());
    let meals = RwSignal::new(Vec::<Meal>::new());
    let listed = RwSignal::new(Listed::Foods);
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    Effect::new(move |_| {
        recorded.heard();
        spawn_local(async move {
            match tauri::foods().await {
                Ok(known) => foods.set(known),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
            if let Ok(kept) = tauri::meals().await {
                meals.set(kept);
            }
        });
    });
    let done = Callback::new(move |line: String| {
        said.set(Some(line));
        recorded.told();
    });
    view! {
        <section class="card">
            <h2>"Add to the catalog"</h2>
            <NewFood foods=foods done=done />
            {said_on_the_shelf(said)}
        </section>
        <section class="card">
            <h2>"On the shelf"</h2>
            {shelf_pills(listed)}
            {move || match listed.get() {
                Listed::Meals => view! { <MealsList meals=meals said=said /> }.into_any(),
                Listed::Foods => view! { <FoodsList foods=foods meals=meals said=said /> }.into_any(),
            }}
        </section>
    }
}

#[component]
pub fn HydrateShelf() -> impl IntoView {
    let drinks = RwSignal::new(Vec::<Drink>::new());
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    Effect::new(move |_| {
        recorded.heard();
        spawn_local(async move {
            match tauri::drinks().await {
                Ok(known) => drinks.set(known),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    });
    let done = Callback::new(move |line: String| {
        said.set(Some(line));
        recorded.told();
    });
    view! {
        <section class="card">
            <h2>"Add to the catalog"</h2>
            <NewDrink drinks=drinks done=done />
            {said_on_the_shelf(said)}
        </section>
        <section class="card">
            <h2>"On the shelf"</h2>
            <DrinksList drinks=drinks said=said />
        </section>
    }
}

#[component]
pub fn BurnShelf() -> impl IntoView {
    let exercises = RwSignal::new(Vec::<Exercise>::new());
    let priced = RwSignal::new(Vec::<PricedCircuit>::new());
    let body_mass = RwSignal::new(None::<f64>);
    let form = RwSignal::new(Form::Exercise);
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    Effect::new(move |_| {
        recorded.heard();
        spawn_local(async move {
            match tauri::exercises().await {
                Ok(known) => exercises.set(known),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
            if let Ok(known) = tauri::priced_circuits(&today()).await {
                priced.set(known);
            }
        });
    });
    spawn_local(async move {
        if let Ok(burn) = tauri::burn_day(&today()).await {
            body_mass.set(burn.body_mass_kg);
        }
    });
    let done = Callback::new(move |line: String| {
        said.set(Some(line));
        recorded.told();
    });
    view! {
        <section class="card">
            <h2>"Add to the catalog"</h2>
            {form_pills(form, [(Form::Exercise, "Exercise"), (Form::Circuit, "Circuit")])}
            {move || match form.get() {
                Form::Circuit => view! { <NewCircuit exercises=exercises body_mass=body_mass.into() done=done /> }.into_any(),
                _ => view! { <NewExercise exercises=exercises done=done /> }.into_any(),
            }}
            {said_on_the_shelf(said)}
        </section>
        {move || match form.get() {
            Form::Circuit => view! { <CircuitsList priced=priced said=said /> }.into_any(),
            _ => view! { <ExercisesList exercises=exercises said=said /> }.into_any(),
        }}
    }
}

#[component]
fn MealDetails(meal: Meal) -> impl IntoView {
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
