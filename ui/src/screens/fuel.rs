use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, shown, signed};
use grind_core::food::{Food, Nutrients, Unit, food as food_of, nutrients_of};
use grind_core::for_cataloguing::FoodForm;
use grind_core::for_fuelling::IntakeForm;
use grind_core::fuel::{EnergyShares, Fuel, MacroRow, Macros, energy_shares};
use grind_core::intake::Intake;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::{chosen, nothing_matches};
use crate::screens::catalog::{FuelShelf, unit_named};
use crate::screens::cells::nutrient_cells;
use crate::screens::fields::{Chooser, FigureField, preview_line};
use crate::screens::gauge::FuelGauge;
use crate::screens::tab::{DayCard, ExploreInStats};
use crate::screens::{Recorded, lines, today};
use crate::tauri;
use crate::typed::figures_typed;

#[component]
fn MacroLine(
    name: &'static str,
    key: &'static str,
    row: MacroRow,
    share: Option<f64>,
) -> impl IntoView {
    let verdict = lines::verdict_class(row.verdict);
    view! {
        <tr class=key>
            <td>{name}</td>
            <td>{bare(Figure::Grams(row.floor_g))}</td>
            <td class=verdict>{bare(Figure::Grams(row.eaten_g))}</td>
            <td class=verdict>{signed(Figure::Grams(row.gap_g()))}</td>
            <td class="hint">{share.map_or_else(|| "-".to_string(), |share| bare(Figure::Ratio(share)))}</td>
        </tr>
    }
}

#[component]
fn MacrosCard(macros: Macros, eaten: Nutrients) -> impl IntoView {
    let shares = energy_shares(&eaten);
    let share = move |of: fn(&EnergyShares) -> f64| shares.as_ref().map(of);
    view! {
        <section class="card">
            <h2>"The four macros"</h2>
            <table class="macros">
                <thead>
                    <tr>
                        <th></th>
                        <th>"Goal (g)"</th>
                        <th>"Current (g)"</th>
                        <th>"Δ (g)"</th>
                        <th>"% cal"</th>
                    </tr>
                </thead>
                <tbody>
                    <MacroLine name="Protein" key="protein" row=macros.protein share=share(|of| of.protein) />
                    <MacroLine name="Fat" key="fat" row=macros.fat share=share(|of| of.fat) />
                    <MacroLine name="Fibre" key="fibre" row=macros.fibre share=share(|of| of.fibre) />
                    <MacroLine name="Carbs" key="carbs" row=macros.carbs share=share(|of| of.carbs) />
                </tbody>
                <tfoot>
                    <tr>
                        <td>"Total"</td>
                        <td></td>
                        <td></td>
                        <td></td>
                        <td>
                            {share(|of| of.total)
                                .map_or_else(|| "-".to_string(), |total| bare(Figure::Ratio(total)))}
                        </td>
                    </tr>
                </tfoot>
            </table>
        </section>
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
    let recorded = Recorded::here();
    let remove = move |_| {
        let date = day.get_untracked();
        spawn_local(async move {
            match tauri::remove_intake(&date, id).await {
                Ok(fuel) => {
                    shown_fuel.run(fuel);
                    recorded.told();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <tr>
            <td>{eaten.food.clone()}</td>
            <td>{shown(Figure::Amount(eaten.quantity, eaten.unit))}</td>
            {nutrient_cells(eaten.nutrients)}
            <td>
                <button class="remove" type="button" on:click=remove aria-label="Remove this intake">
                    "×"
                </button>
            </td>
        </tr>
    }
}

/// What a food new to the catalog is typed with, its name aside.
#[derive(Clone, Copy)]
struct NewFoodFigures {
    description: RwSignal<String>,
    unit: RwSignal<String>,
    portion: RwSignal<String>,
    energy: RwSignal<String>,
    protein: RwSignal<String>,
    fat: RwSignal<String>,
    fibre: RwSignal<String>,
    carbs: RwSignal<String>,
}

impl NewFoodFigures {
    fn new() -> Self {
        Self {
            description: RwSignal::new(String::new()),
            unit: RwSignal::new("Gram".into()),
            portion: RwSignal::new("100".into()),
            energy: RwSignal::new(String::new()),
            protein: RwSignal::new(String::new()),
            fat: RwSignal::new(String::new()),
            fibre: RwSignal::new(String::new()),
            carbs: RwSignal::new(String::new()),
        }
    }

    fn typed(self, name: &str) -> Option<FoodForm> {
        let [portion, energy_kcal, protein_g, carbs_g, fat_g, fibre_g] = figures_typed([
            &self.portion.get(),
            &self.energy.get(),
            &self.protein.get(),
            &self.carbs.get(),
            &self.fat.get(),
            &self.fibre.get(),
        ])?;
        Some(FoodForm {
            name: name.trim().into(),
            unit: unit_named(&self.unit.get()),
            portion: portion.unwrap_or(0.0),
            nutrients: Nutrients {
                energy_kcal: energy_kcal.unwrap_or(0.0),
                protein_g: protein_g.unwrap_or(0.0),
                carbs_g: carbs_g.unwrap_or(0.0),
                fat_g: fat_g.unwrap_or(0.0),
                fibre_g: fibre_g.unwrap_or(0.0),
            },
            description: self.description.get(),
        })
    }

    fn cleared(self) {
        for typed in [
            self.description,
            self.energy,
            self.protein,
            self.fat,
            self.fibre,
            self.carbs,
        ] {
            typed.set(String::new());
        }
        self.unit.set("Gram".into());
        self.portion.set("100".into());
    }
}

#[component]
fn NewFoodFields(name: RwSignal<String>, figures: NewFoodFigures) -> impl IntoView {
    view! {
        <p class="new">{move || format!("New {}", name.get().trim())}</p>
        <label class="field">
            "Description"
            <input
                name="new_food_description"
                autocomplete="off"
                placeholder="Optional"
                prop:value=move || figures.description.get()
                on:input=move |event| figures.description.set(event_target_value(&event))
            />
        </label>
        <div class="row thirds">
            <FigureField label="Per" name="new_food_portion" value=figures.portion />
            <label class="field">
                "Unit"
                <select
                    name="new_food_unit"
                    prop:value=move || figures.unit.get()
                    on:change=move |event| figures.unit.set(event_target_value(&event))
                >
                    <option value="Gram">"g"</option>
                    <option value="Millilitre">"mL"</option>
                    <option value="Piece">"piece"</option>
                </select>
            </label>
            <FigureField label="kcal" name="new_food_energy" value=figures.energy />
        </div>
        <div class="row quarters">
            <FigureField label="Protein (g)" name="new_food_protein" value=figures.protein />
            <FigureField label="Fat (g)" name="new_food_fat" value=figures.fat />
            <FigureField label="Fibre (g)" name="new_food_fibre" value=figures.fibre />
            <FigureField label="Carbs (g)" name="new_food_carbs" value=figures.carbs />
        </div>
    }
}

#[component]
pub fn FuelDay(day: RwSignal<String>) -> impl IntoView {
    let fuel = RwSignal::new(None::<Fuel>);
    let catalog = RwSignal::new(Vec::<Food>::new());
    let food = RwSignal::new(String::new());
    let quantity = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let shown_fuel = Callback::new(move |answer: Fuel| fuel.set(Some(answer)));
    let names = Signal::derive(move || {
        catalog
            .get()
            .into_iter()
            .map(|known| known.name)
            .collect::<Vec<_>>()
    });
    let figures = NewFoodFigures::new();
    let creating = Memo::new(move |_| nothing_matches(&names.get(), &food.get()));
    let chosen_food = move || {
        chosen(&names.get(), &food.get())
            .and_then(|name| catalog.get().into_iter().find(|known| known.name == name))
    };
    let new_food = move || {
        let form = figures.typed(&food.get())?;
        food_of(
            &form.name,
            form.unit,
            form.portion,
            form.nutrients,
            &form.description,
        )
        .ok()
    };
    let eaten_food = move || chosen_food().or_else(|| creating.get().then(new_food).flatten());
    let preview = move || {
        let known = eaten_food()?;
        let [typed] = figures_typed([&quantity.get()])?;
        let eaten = typed.unwrap_or(known.portion);
        Some(lines::figures_of(&nutrients_of(&known, eaten)))
    };
    spawn_local(async move {
        match tauri::foods().await {
            Ok(known) => catalog.set(known),
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
    let logged = move |answer: Result<Fuel, Failure>| match answer {
        Ok(answer) => {
            shown_fuel.run(answer);
            food.set(String::new());
            quantity.set(String::new());
            figures.cleared();
            said.set(None);
            recorded.told();
        }
        Err(failure) => said.set(Some(lines::failure(&failure))),
    };
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([quantity_typed]) = figures_typed([&quantity.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let date = day.get_untracked();
        if creating.get_untracked() {
            let Some(added) = untrack(|| figures.typed(&food.get())) else {
                said.set(Some(lines::UNREADABLE.into()));
                return;
            };
            spawn_local(async move {
                match tauri::add_food(&added).await {
                    Ok(known) => {
                        catalog.set(known);
                        let form = IntakeForm {
                            date,
                            food: added.name,
                            quantity: quantity_typed,
                        };
                        logged(tauri::log_intake(&form).await);
                    }
                    Err(failure) => said.set(Some(lines::failure(&failure))),
                }
            });
            return;
        }
        let Some(eaten) = untrack(chosen_food) else {
            said.set(Some(lines::CHOOSE_FIRST.into()));
            return;
        };
        let form = IntakeForm {
            date,
            food: eaten.name,
            quantity: quantity_typed,
        };
        spawn_local(async move { logged(tauri::log_intake(&form).await) });
    };
    view! {
        <section class="card">
            <h2>"Fuel"</h2>
            <form class="intake" on:submit=log>
                <div class="row">
                    <Chooser label="Food" name="food" options=names typed=food />
                    {move || {
                        let portion = eaten_food();
                        view! {
                            <FigureField
                                label=match portion.as_ref().map(|known| known.unit) {
                                    Some(Unit::Gram) => "Quantity (g)",
                                    Some(Unit::Millilitre) => "Quantity (mL)",
                                    Some(Unit::Piece) => "Quantity (pieces)",
                                    None => "Quantity",
                                }
                                name="quantity"
                                value=quantity
                                placeholder=portion
                                    .map(|known| bare(Figure::Amount(known.portion, known.unit)))
                            />
                        }
                    }}
                </div>
                {move || {
                    creating.get().then(|| view! { <NewFoodFields name=food figures=figures /> })
                }}
                {preview_line(preview)}
                <button class="cta" type="submit">
                    {move || if creating.get() { "Add and log" } else { "Fuel" }}
                </button>
            </form>
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
        </section>
        {move || {
            fuel.get()
                .map(|answer| {
                    let energy_class = answer.energy.map(lines::verdict_class).unwrap_or("hint");
                    view! {
                        <section class="card bands">
                            <h2>"Where the day stands"</h2>
                            <p class=format!("figures {energy_class}")>{lines::energy_line(&answer)}</p>
                            {match answer.band {
                                Some(band) => {
                                    view! { <FuelGauge eaten=answer.eaten.energy_kcal band=band /> }
                                        .into_any()
                                }
                                None => view! { <p class="hint">{lines::NO_BAND}</p> }.into_any(),
                            }}
                        </section>
                    }
                })
        }}
        {move || {
            fuel.get()
                .and_then(|answer| {
                    answer.macros.map(|macros| view! { <MacrosCard macros=macros eaten=answer.eaten /> })
                })
        }}
        <section class="card">
            <h2>"The day's intakes"</h2>
            {move || {
                fuel.get()
                    .is_some_and(|answer| answer.intakes.is_empty())
                    .then(|| view! { <p class="empty">{lines::NO_INTAKE}</p> })
            }}
            <div class="scroll">
                <table class="intakes">
                    <thead>
                        <tr>
                            <th>"Food"</th>
                            <th>"Qty"</th>
                            <th>"kcal"</th>
                            <th>"Prot (g)"</th>
                            <th>"Fat (g)"</th>
                            <th>"Fib (g)"</th>
                            <th>"Carbs (g)"</th>
                            <th></th>
                        </tr>
                    </thead>
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
            </div>
        </section>
    }
}

#[component]
pub fn FuelTab() -> impl IntoView {
    let day = RwSignal::new(today());
    view! {
        <DayCard day=day />
        <FuelDay day=day />
        <ExploreInStats />
        <FuelShelf />
    }
}
