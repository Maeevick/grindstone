use grind_core::circuit::Circuit;
use grind_core::drink::Drink;
use grind_core::exercise::Exercise;
use grind_core::figures::{Figure, shown};
use grind_core::food::{Food, Nutrients, Unit};
use grind_core::for_cataloguing::FoodForm;
use grind_core::meal::Meal;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::circuits::{CircuitsList, NewCircuit};
use crate::screens::fields::FigureField;
use crate::screens::hydrate::{DrinksList, NewDrink};
use crate::screens::lines;
use crate::screens::meals::{MealsList, NewMeal};
use crate::tauri;
use crate::typed::figures_typed;

fn said_on_the_shelf(said: RwSignal<Option<String>>) -> impl IntoView {
    move || said.get().map(|line| view! { <p class="said">{line}</p> })
}

#[component]
fn NewExercise(exercises: RwSignal<Vec<Exercise>>, done: Callback<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let met = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    let add = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let named = name.get_untracked();
        let Some([met_typed]) = figures_typed([&met.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let met_typed = met_typed.unwrap_or(0.0);
        spawn_local(async move {
            match tauri::add_exercise(&named, met_typed).await {
                Ok(known) => {
                    exercises.set(known);
                    name.set(String::new());
                    met.set(String::new());
                    done.run(lines::added(named.trim()));
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
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
            {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
        </form>
    }
}

#[component]
fn ExercisesList(
    exercises: RwSignal<Vec<Exercise>>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let remove = move |named: String| {
        spawn_local(async move {
            match tauri::remove_exercise(&named).await {
                Ok(known) => exercises.set(known),
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

fn unit_named(word: &str) -> Unit {
    match word {
        "Millilitre" => Unit::Millilitre,
        "Piece" => Unit::Piece,
        _ => Unit::Gram,
    }
}

#[component]
fn NewFood(foods: RwSignal<Vec<Food>>, done: Callback<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let unit = RwSignal::new("Gram".to_string());
    let portion = RwSignal::new("100".to_string());
    let energy = RwSignal::new(String::new());
    let protein = RwSignal::new(String::new());
    let carbs = RwSignal::new(String::new());
    let fat = RwSignal::new(String::new());
    let fibre = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    let add = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some(
            [
                portion_typed,
                energy_kcal,
                protein_g,
                carbs_g,
                fat_g,
                fibre_g,
            ],
        ) = figures_typed([
            &portion.get_untracked(),
            &energy.get_untracked(),
            &protein.get_untracked(),
            &carbs.get_untracked(),
            &fat.get_untracked(),
            &fibre.get_untracked(),
        ])
        else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = FoodForm {
            name: name.get_untracked(),
            unit: unit_named(&unit.get_untracked()),
            portion: portion_typed.unwrap_or(0.0),
            nutrients: Nutrients {
                energy_kcal: energy_kcal.unwrap_or(0.0),
                protein_g: protein_g.unwrap_or(0.0),
                carbs_g: carbs_g.unwrap_or(0.0),
                fat_g: fat_g.unwrap_or(0.0),
                fibre_g: fibre_g.unwrap_or(0.0),
            },
        };
        spawn_local(async move {
            match tauri::add_food(&form).await {
                Ok(known) => {
                    foods.set(known);
                    for typed in [name, energy, protein, carbs, fat, fibre] {
                        typed.set(String::new());
                    }
                    done.run(lines::added(form.name.trim()));
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <form class="food" on:submit=add>
            <div class="row">
                <label class="field grow2">
                    "Name"
                    <input
                        name="food_name"
                        autocomplete="off"
                        prop:value=move || name.get()
                        on:input=move |event| name.set(event_target_value(&event))
                    />
                </label>
                <label class="field">
                    "Unit"
                    <select
                        name="food_unit"
                        prop:value=move || unit.get()
                        on:change=move |event| unit.set(event_target_value(&event))
                    >
                        <option value="Gram">"g"</option>
                        <option value="Millilitre">"mL"</option>
                        <option value="Piece">"piece"</option>
                    </select>
                </label>
                <FigureField label="Per" name="food_portion" value=portion />
            </div>
            <div class="row">
                <FigureField label="kcal" name="food_energy" value=energy />
                <FigureField label="Protein (g)" name="food_protein" value=protein />
                <FigureField label="Carbs (g)" name="food_carbs" value=carbs />
                <FigureField label="Fat (g)" name="food_fat" value=fat />
                <FigureField label="Fibre (g)" name="food_fibre" value=fibre />
            </div>
            <button class="cta" type="submit">
                "Add to the catalog"
            </button>
            {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
        </form>
    }
}

#[component]
fn FoodsList(foods: RwSignal<Vec<Food>>, said: RwSignal<Option<String>>) -> impl IntoView {
    let remove = move |named: String| {
        spawn_local(async move {
            match tauri::remove_food(&named).await {
                Ok(known) => foods.set(known),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Foods"</h2>
            {move || foods.get().is_empty().then(|| view! { <p class="empty">{lines::NO_FOOD}</p> })}
            <table class="foods">
                <tbody>
                    <For
                        each=move || foods.get()
                        key=|known| format!("{known:?}")
                        children=move |known| {
                            let named = known.name.clone();
                            let figures = known.nutrients;
                            view! {
                                <tr>
                                    <td>{known.name.clone()}</td>
                                    <td>{format!("per {}", shown(Figure::Amount(known.portion, known.unit)))}</td>
                                    <td>{shown(Figure::Energy(figures.energy_kcal))}</td>
                                    <td>
                                        {format!(
                                            "P {} · C {} · F {} · Fib {}",
                                            shown(Figure::Grams(figures.protein_g)),
                                            shown(Figure::Grams(figures.carbs_g)),
                                            shown(Figure::Grams(figures.fat_g)),
                                            shown(Figure::Grams(figures.fibre_g)),
                                        )}
                                    </td>
                                    <td>
                                        <button
                                            class="remove"
                                            type="button"
                                            aria-label="Remove this food"
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Form {
    Food,
    Meal,
    Exercise,
    Circuit,
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
fn FuelShelf() -> impl IntoView {
    let foods = RwSignal::new(Vec::<Food>::new());
    let meals = RwSignal::new(Vec::<Meal>::new());
    let form = RwSignal::new(Form::Food);
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        match tauri::foods().await {
            Ok(known) => foods.set(known),
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
        if let Ok(kept) = tauri::meals().await {
            meals.set(kept);
        }
    });
    let done = Callback::new(move |line: String| said.set(Some(line)));
    view! {
        <section class="card">
            <h2>"Add to the catalog"</h2>
            {form_pills(form, [(Form::Food, "Food"), (Form::Meal, "Meal")])}
            {move || match form.get() {
                Form::Meal => view! { <NewMeal foods=foods meals=meals done=done /> }.into_any(),
                _ => view! { <NewFood foods=foods done=done /> }.into_any(),
            }}
            {said_on_the_shelf(said)}
        </section>
        <MealsList meals=meals said=said />
        <FoodsList foods=foods said=said />
    }
}

#[component]
fn HydrateShelf() -> impl IntoView {
    let drinks = RwSignal::new(Vec::<Drink>::new());
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        match tauri::drinks().await {
            Ok(known) => drinks.set(known),
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
    });
    let done = Callback::new(move |line: String| said.set(Some(line)));
    view! {
        <section class="card">
            <h2>"Add to the catalog"</h2>
            <NewDrink drinks=drinks done=done />
            {said_on_the_shelf(said)}
        </section>
        <DrinksList drinks=drinks said=said />
    }
}

#[component]
fn BurnShelf() -> impl IntoView {
    let exercises = RwSignal::new(Vec::<Exercise>::new());
    let circuits = RwSignal::new(Vec::<Circuit>::new());
    let form = RwSignal::new(Form::Exercise);
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        match tauri::exercises().await {
            Ok(known) => exercises.set(known),
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
        if let Ok(known) = tauri::circuits().await {
            circuits.set(known);
        }
    });
    let done = Callback::new(move |line: String| said.set(Some(line)));
    view! {
        <section class="card">
            <h2>"Add to the catalog"</h2>
            {form_pills(form, [(Form::Exercise, "Exercise"), (Form::Circuit, "Circuit")])}
            {move || match form.get() {
                Form::Circuit => {
                    view! { <NewCircuit exercises=exercises circuits=circuits done=done /> }.into_any()
                }
                _ => view! { <NewExercise exercises=exercises done=done /> }.into_any(),
            }}
            {said_on_the_shelf(said)}
        </section>
        <CircuitsList circuits=circuits said=said />
        <ExercisesList exercises=exercises said=said />
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Shelf {
    Fuel,
    Hydrate,
    Burn,
}

const SHELVES: [(Shelf, &str); 3] = [
    (Shelf::Fuel, "Fuel"),
    (Shelf::Hydrate, "Hydrate"),
    (Shelf::Burn, "Burn"),
];

#[component]
pub fn Catalog() -> impl IntoView {
    let shelf = RwSignal::new(Shelf::Fuel);
    view! {
        <section class="card">
            <div class="pills">
                {SHELVES
                    .into_iter()
                    .map(|(named, label)| {
                        view! {
                            <button
                                type="button"
                                class=format!("pill shelf-{named:?}")
                                class:chosen=move || shelf.get() == named
                                on:click=move |_| shelf.set(named)
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
        {move || match shelf.get() {
            Shelf::Fuel => view! { <FuelShelf /> }.into_any(),
            Shelf::Hydrate => view! { <HydrateShelf /> }.into_any(),
            Shelf::Burn => view! { <BurnShelf /> }.into_any(),
        }}
    }
}
