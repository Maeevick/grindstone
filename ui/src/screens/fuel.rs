use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, shown, signed};
use grind_core::food::{Food, Nutrients, Unit, food as food_of, nutrients_of, summed};
use grind_core::for_fuelling::IntakeForm;
use grind_core::fuel::{EnergyShares, Fuel, MacroRow, Macros, energy_shares};
use grind_core::intake::Intake;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::{chosen, nothing_matches};
use crate::screens::catalog::FuelShelf;
use crate::screens::cells::nutrient_cells;
use crate::screens::entries::{FoodFields, FoodFigures, NewEntry};
use crate::screens::fields::{Chooser, FigureField, preview_line};
use crate::screens::gauge::FuelGauge;
use crate::screens::plate::{InThePot, Plate, PlateInProgress};
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
    plated_g: Option<f64>,
) -> impl IntoView {
    let verdict = lines::verdict_class(row.verdict);
    view! {
        <tr class=key>
            <td>{name}</td>
            <td>{bare(Figure::Grams(row.floor_g))}</td>
            <td class=verdict>
                {bare(Figure::Grams(row.eaten_g))}
                {plated_g
                    .map(|grams| view! { <span class="ghost">{format!("+{}", bare(Figure::Grams(grams)))}</span> })}
            </td>
            <td class=verdict>
                {match plated_g {
                    Some(grams) => {
                        let with_the_plate = MacroRow {
                            eaten_g: row.eaten_g + grams,
                            ..row
                        };
                        view! { <span class="ghost">{signed(Figure::Grams(with_the_plate.gap_g()))}</span> }
                            .into_any()
                    }
                    None => signed(Figure::Grams(row.gap_g())).into_any(),
                }}
            </td>
            <td class="hint">{share.map_or_else(|| "-".to_string(), |share| bare(Figure::Ratio(share)))}</td>
        </tr>
    }
}

#[component]
fn MacrosCard(macros: Macros, eaten: Nutrients, plated: Option<Nutrients>) -> impl IntoView {
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
                    <MacroLine
                        name="Protein"
                        key="protein"
                        row=macros.protein
                        share=share(|of| of.protein)
                        plated_g=plated.map(|plated| plated.protein_g)
                    />
                    <MacroLine
                        name="Fat"
                        key="fat"
                        row=macros.fat
                        share=share(|of| of.fat)
                        plated_g=plated.map(|plated| plated.fat_g)
                    />
                    <MacroLine
                        name="Fibre"
                        key="fibre"
                        row=macros.fibre
                        share=share(|of| of.fibre)
                        plated_g=plated.map(|plated| plated.fibre_g)
                    />
                    <MacroLine
                        name="Carbs"
                        key="carbs"
                        row=macros.carbs
                        share=share(|of| of.carbs)
                        plated_g=plated.map(|plated| plated.carbs_g)
                    />
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
    let figures = FoodFigures::default();
    let plate = use_context::<PlateInProgress>().unwrap_or_default();
    let on_the_plate = Memo::new(move |_| !plate.pot.get().is_empty());
    let plated = Signal::derive(move || {
        let pot = plate.pot.get();
        (!pot.is_empty()).then(|| summed(pot.iter().map(|added| added.nutrients)))
    });
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
    let plated_now = move |served: &Food, quantity_typed: Option<f64>| {
        let on_it = InThePot::of(served, quantity_typed.unwrap_or(served.portion));
        plate.pot.update(|pot| pot.push(on_it));
        food.set(String::new());
        quantity.set(String::new());
        figures.cleared();
        said.set(None);
    };
    // The food typed, added to the catalog first when new, then logged or put on the plate.
    let served = move |onto_the_plate: bool| {
        let Some([quantity_typed]) = figures_typed([&quantity.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let date = day.get_untracked();
        let added = if creating.get_untracked() {
            let Some(form) = untrack(|| figures.typed(&food.get())) else {
                said.set(Some(lines::UNREADABLE.into()));
                return;
            };
            Some(form)
        } else {
            None
        };
        let known = untrack(chosen_food);
        if added.is_none() && known.is_none() {
            said.set(Some(lines::CHOOSE_FIRST.into()));
            return;
        }
        spawn_local(async move {
            let served = match added {
                Some(form) => match tauri::add_food(&form).await {
                    Ok(listed) => {
                        let found = listed.iter().find(|known| known.name == form.name).cloned();
                        catalog.set(listed);
                        found
                    }
                    Err(failure) => {
                        said.set(Some(lines::failure(&failure)));
                        return;
                    }
                },
                None => known,
            };
            let Some(served) = served else {
                return;
            };
            if onto_the_plate {
                plated_now(&served, quantity_typed);
                return;
            }
            let form = IntakeForm {
                date,
                food: served.name,
                quantity: quantity_typed,
            };
            logged(tauri::log_intake(&form).await);
        });
    };
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        served(on_the_plate.get_untracked());
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
                    creating
                        .get()
                        .then(|| {
                            view! {
                                <NewEntry name=food />
                                <FoodFields prefix="new_food" figures=figures />
                            }
                        })
                }}
                {preview_line(preview)}
                <button
                    class=move || if on_the_plate.get() { "cta to-the-plate" } else { "cta" }
                    type="submit"
                >
                    {move || {
                        if on_the_plate.get() {
                            "Add to the plate"
                        } else if creating.get() {
                            "Add and log"
                        } else {
                            "Fuel"
                        }
                    }}
                </button>
                {move || {
                    (!on_the_plate.get())
                        .then(|| {
                            view! {
                                <button
                                    class="cta ghost to-the-plate"
                                    type="button"
                                    on:click=move |_| served(true)
                                >
                                    "Add to the plate"
                                </button>
                            }
                        })
                }}
            </form>
            {move || {
                on_the_plate
                    .get()
                    .then(|| view! { <Plate plate=plate day=day shown_fuel=shown_fuel said=said /> })
            }}
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
        </section>
        {move || {
            fuel.get()
                .map(|answer| {
                    let energy_class = answer.energy.map(lines::verdict_class).unwrap_or("hint");
                    let eaten = answer.eaten.energy_kcal;
                    let with_the_plate = plated.get().map(|plated| eaten + plated.energy_kcal);
                    view! {
                        <section class="card bands">
                            <h2>"Where the day stands"</h2>
                            <p class=format!("figures {energy_class}")>{lines::energy_line(&answer)}</p>
                            {with_the_plate
                                .map(|kcal| {
                                    view! { <p class="hint with-the-plate">{lines::with_the_plate(kcal)}</p> }
                                })}
                            {match answer.band {
                                Some(band) => {
                                    view! { <FuelGauge eaten=eaten band=band ghost=with_the_plate /> }
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
                    answer
                        .macros
                        .map(|macros| {
                            view! { <MacrosCard macros=macros eaten=answer.eaten plated=plated.get() /> }
                        })
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
