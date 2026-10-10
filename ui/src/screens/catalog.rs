use grind_core::drink::Drink;
use grind_core::exercise::Exercise;
use grind_core::figures::{Figure, shown};
use grind_core::food::{Food, Nutrients, Unit};
use grind_core::for_cataloguing::{FoodForm, PricedCircuit};
use grind_core::meal::{Meal, food_of};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::cells::nutrient_cells;
use crate::screens::circuits::{CircuitsList, NewCircuit};
use crate::screens::fields::FigureField;
use crate::screens::hydrate::{DrinksList, NewDrink};
use crate::screens::meals::{MealDetails, NewMeal};
use crate::screens::{lines, today};
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
                <label class="field">
                    "Name"
                    <input
                        name="food_name"
                        autocomplete="off"
                        prop:value=move || name.get()
                        on:input=move |event| name.set(event_target_value(&event))
                    />
                </label>
            </div>
            <div class="row thirds">
                <FigureField label="Per" name="food_portion" value=portion />
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
                <FigureField label="kcal" name="food_energy" value=energy />
            </div>
            <div class="row quarters">
                <FigureField label="Protein (g)" name="food_protein" value=protein />
                <FigureField label="Fat (g)" name="food_fat" value=fat />
                <FigureField label="Fibre (g)" name="food_fibre" value=fibre />
                <FigureField label="Carbs (g)" name="food_carbs" value=carbs />
            </div>
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
    let remove = move |named: String| {
        spawn_local(async move {
            match tauri::remove_food(&named).await {
                Ok(known) => foods.set(known),
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
        <section class="card">
            <h2>"Foods"</h2>
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
                                            <td>{food.name.clone()}</td>
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
        </section>
    }
}

#[component]
fn MealsList(
    foods: RwSignal<Vec<Food>>,
    meals: RwSignal<Vec<Meal>>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let opened = RwSignal::new(None::<String>);
    let remove = move |named: String| {
        spawn_local(async move {
            match tauri::remove_meal(&named).await {
                Ok(kept) => {
                    meals.set(kept);
                    if let Ok(known) = tauri::foods().await {
                        foods.set(known);
                    }
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Meals"</h2>
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
        {move || match form.get() {
            Form::Meal => view! { <MealsList foods=foods meals=meals said=said /> }.into_any(),
            _ => view! { <FoodsList foods=foods meals=meals said=said /> }.into_any(),
        }}
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
    let priced = RwSignal::new(Vec::<PricedCircuit>::new());
    let form = RwSignal::new(Form::Exercise);
    let said = RwSignal::new(None::<String>);
    let price = move || {
        spawn_local(async move {
            if let Ok(known) = tauri::priced_circuits(&today()).await {
                priced.set(known);
            }
        });
    };
    spawn_local(async move {
        match tauri::exercises().await {
            Ok(known) => exercises.set(known),
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
    });
    price();
    let done = Callback::new(move |line: String| {
        said.set(Some(line));
        price();
    });
    view! {
        <section class="card">
            <h2>"Add to the catalog"</h2>
            {form_pills(form, [(Form::Exercise, "Exercise"), (Form::Circuit, "Circuit")])}
            {move || match form.get() {
                Form::Circuit => view! { <NewCircuit exercises=exercises done=done /> }.into_any(),
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
