use grind_core::bibliography::Topic;
use grind_core::drink::Drink;
use grind_core::drinking::Drinking;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, shown};
use grind_core::for_hydrating::DrinkingForm;
use grind_core::for_setting_up_the_profile::HydrateCountedForm;
use grind_core::hydrate::Hydrate;
use grind_core::verdict::Verdict;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::{Choice, chosen, nothing_matches};
use crate::screens::catalog::HydrateShelf;
use crate::screens::entries::{DrinkFields, DrinkFigures, KINDS, NameField, NewEntry, name_cell};
use crate::screens::fields::{Chooser, DateField, FigureField};
use crate::screens::gauge::Gauge;
use crate::screens::last_days::HydrateLastDays;
use crate::screens::references::ReferencesLink;
use crate::screens::tab::{DayCard, ExploreInStats};
use crate::screens::{Recorded, lines, today};
use crate::tauri;
use crate::typed::figures_typed;

#[component]
fn DrinkingRow(
    id: i64,
    drinking: Drinking,
    hydrate: RwSignal<Option<Hydrate>>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let date = drinking.day.to_string();
    let recorded = Recorded::here();
    let remove = move |_| {
        let date = date.clone();
        spawn_local(async move {
            match tauri::remove_drinking(&date, id).await {
                Ok(left) => {
                    hydrate.set(Some(left));
                    recorded.told();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <tr>
            <td>{drinking.drink}</td>
            <td>{shown(Figure::Volume(drinking.volume_ml))}</td>
            <td>
                <button class="remove" type="button" on:click=remove aria-label="Remove this drink">
                    "×"
                </button>
            </td>
        </tr>
    }
}

#[component]
fn DrinkingsOfTheDay(
    of_the_day: Hydrate,
    hydrate: RwSignal<Option<Hydrate>>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let total = lines::drunk_against_target(&of_the_day);
    let reached = of_the_day.verdict == Verdict::InBand;
    let past_the_cap = of_the_day.verdict == Verdict::High;
    let dry = of_the_day.drinkings.is_empty();
    view! {
        <section class="card bands">
            <h2>"Where the day stands"</h2>
            <p class="figures" class:good=reached>
                {total}
            </p>
            <Gauge
                amount=of_the_day.drunk_ml
                floor=of_the_day.floor_ml
                optimal=of_the_day.target_ml
                cap=of_the_day.cap_ml
                shown_as=Figure::Volume
            />
            {reached.then(|| view! { <p class="said">{lines::TARGET_DRUNK}</p> })}
            {past_the_cap.then(|| view! { <p class="bad">{lines::PAST_THE_CAP}</p> })}
        </section>
        <section class="card">
            <h2>"The day's drinks"</h2>
            {dry.then(|| view! { <p class="empty">{lines::NO_DRINKING}</p> })}
            <table>
                <tbody>
                    {of_the_day
                        .drinkings
                        .into_iter()
                        .map(|(id, drinking)| {
                            view! { <DrinkingRow id=id drinking=drinking hydrate=hydrate said=said /> }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </section>
    }
}

#[component]
pub fn HydrateDay(day: RwSignal<String>) -> impl IntoView {
    let hydrate = RwSignal::new(None::<Hydrate>);
    let catalog = RwSignal::new(Vec::<Drink>::new());
    let drink = RwSignal::new(String::new());
    let volume = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let names = Signal::derive(move || {
        catalog
            .get()
            .into_iter()
            .map(|known| known.name)
            .collect::<Vec<_>>()
    });
    let options = Signal::derive(move || {
        names
            .get()
            .into_iter()
            .map(|name| Choice { name, icon: "" })
            .collect::<Vec<_>>()
    });
    let figures = DrinkFigures::default();
    let creating = Memo::new(move |_| nothing_matches(&names.get(), &drink.get()));
    let chosen_drink = move || {
        chosen(&names.get(), &drink.get())
            .and_then(|name| catalog.get().into_iter().find(|known| known.name == name))
    };
    Effect::new(move |_| {
        recorded.heard();
        spawn_local(async move {
            match tauri::drinks().await {
                Ok(drinks) => catalog.set(drinks),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    });
    Effect::new(move |_| {
        let date = day.get();
        spawn_local(async move {
            match tauri::hydrate_day(&date).await {
                Ok(of_the_day) => hydrate.set(Some(of_the_day)),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    });
    let logged = move |answer: Result<Hydrate, Failure>| match answer {
        Ok(logged) => {
            hydrate.set(Some(logged));
            recorded.told();
            drink.set(String::new());
            volume.set(String::new());
            figures.cleared();
            said.set(None);
        }
        Err(failure) => said.set(Some(lines::failure(&failure))),
    };
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([volume_typed]) = figures_typed([&volume.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let date = day.get_untracked();
        if creating.get_untracked() {
            let Some([serving]) = figures_typed([&figures.serving.get_untracked()]) else {
                said.set(Some(lines::UNREADABLE.into()));
                return;
            };
            let name = drink.get_untracked().trim().to_string();
            let description = figures.description.get_untracked();
            let kind = figures.kind.get_untracked();
            spawn_local(async move {
                match tauri::add_drink(&name, serving.unwrap_or(0.0), kind, &description).await {
                    Ok(known) => {
                        catalog.set(known);
                        let form = DrinkingForm {
                            date,
                            drink: name,
                            volume_ml: volume_typed,
                        };
                        logged(tauri::log_drink(&form).await);
                    }
                    Err(failure) => said.set(Some(lines::failure(&failure))),
                }
            });
            return;
        }
        let Some(drunk) = untrack(chosen_drink) else {
            said.set(Some(lines::CHOOSE_FIRST.into()));
            return;
        };
        let form = DrinkingForm {
            date,
            drink: drunk.name,
            volume_ml: volume_typed,
        };
        spawn_local(async move { logged(tauri::log_drink(&form).await) });
    };
    view! {
        <section class="card">
            <h2>"Hydrate"</h2>
            <form class="drinking" on:submit=log>
                <div class="row">
                    <Chooser label="Drink" name="drink" options=options typed=drink />
                    <FigureField
                        label="Volume (mL)"
                        name="volume"
                        value=volume
                        placeholder=Signal::derive(move || {
                            chosen_drink()
                                .map(|known| bare(Figure::Volume(known.volume_ml)))
                                .or_else(|| {
                                    creating.get().then(|| figures.serving.get()).filter(|serving| !serving.is_empty())
                                })
                        })
                    />
                </div>
                {move || {
                    creating
                        .get()
                        .then(|| {
                            view! {
                                <NewEntry name=drink />
                                <DrinkFields prefix="new_drink" figures=figures />
                            }
                        })
                }}
                <button class="cta" type="submit">
                    {move || if creating.get() { "Add and log" } else { "Hydrate" }}
                </button>
                {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
            </form>
        </section>
        {move || {
            hydrate
                .get()
                .map(|of_the_day| {
                    view! { <DrinkingsOfTheDay of_the_day=of_the_day hydrate=hydrate said=said /> }
                })
        }}
    }
}

#[component]
fn DrinkRow(
    drink: Drink,
    drinks: RwSignal<Vec<Drink>>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let name = drink.name.clone();
    let recorded = Recorded::here();
    let remove = move |_| {
        let name = name.clone();
        spawn_local(async move {
            match tauri::remove_drink(&name).await {
                Ok(left) => {
                    drinks.set(left);
                    recorded.told();
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <tr>
            {name_cell(drink.name, drink.description)}
            <td>{shown(Figure::Volume(drink.volume_ml))}</td>
            <td class="hint">
                {KINDS
                    .into_iter()
                    .find(|(named, _)| *named == drink.kind)
                    .map_or("other", |(_, label)| label)}
            </td>
            <td>
                <button class="remove" type="button" on:click=remove aria-label="Remove this drink">
                    "×"
                </button>
            </td>
        </tr>
    }
}

#[component]
pub fn NewDrink(drinks: RwSignal<Vec<Drink>>, done: Callback<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let figures = DrinkFigures::default();
    let said = RwSignal::new(None::<String>);
    let add = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let named = name.get_untracked();
        let Some([volume_ml]) = figures_typed([&figures.serving.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let volume_ml = volume_ml.unwrap_or(0.0);
        let kind = figures.kind.get_untracked();
        let description = figures.description.get_untracked();
        spawn_local(async move {
            match tauri::add_drink(&named, volume_ml, kind, &description).await {
                Ok(known) => {
                    drinks.set(known);
                    name.set(String::new());
                    figures.cleared();
                    done.run(lines::added(named.trim()));
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <form class="drink" on:submit=add>
            <div class="row">
                <NameField name="drink_name" value=name />
            </div>
            <DrinkFields prefix="drink" figures=figures />
            <button class="cta" type="submit">
                "Add to the catalog"
            </button>
            {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
        </form>
    }
}

#[component]
pub fn DrinksList(drinks: RwSignal<Vec<Drink>>, said: RwSignal<Option<String>>) -> impl IntoView {
    view! {
            <table class="drinks">
                <tbody>
                    <For
                        each=move || drinks.get()
                        key=|drink| format!("{drink:?}")
                        children=move |drink| {
                            view! { <DrinkRow drink=drink drinks=drinks said=said /> }
                        }
                    />
                </tbody>
            </table>
    }
}

const SWITCHES: [(bool, &str, &str); 2] = [
    (true, "Counted", "counted"),
    (false, "Left out", "left-out"),
];

#[component]
pub fn HydrateCountedCard() -> impl IntoView {
    let from = RwSignal::new(today());
    let counted = RwSignal::new(true);
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    spawn_local(async move {
        if let Ok(in_force) = tauri::hydrate_counted_on(&today()).await {
            counted.set(in_force);
        }
    });
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let form = HydrateCountedForm {
            from: from.get_untracked(),
            counted: counted.get_untracked(),
        };
        spawn_local(async move {
            let kept = tauri::set_hydrate_counted(&form).await;
            if kept.is_ok() {
                recorded.told();
            }
            said.set(Some(match kept {
                Ok(true) => lines::hydrate_counted_from(&form.from),
                Ok(false) => lines::hydrate_left_out_from(&form.from),
                Err(failure) => lines::failure(&failure),
            }));
        });
    };
    view! {
        <section class="card">
            <h2>"Hydrate the material"</h2>
            <form class="hydrate-counted" on:submit=save>
                <p class="hint">{lines::HYDRATE_COUNTED_HINT}</p>
                <div class="row">
                    <DateField label="From" name="hydrate_counted_from" value=from />
                </div>
                <div class="pills">
                    {SWITCHES
                        .into_iter()
                        .map(|(on, label, named)| {
                            view! {
                                <button
                                    type="button"
                                    class=format!("pill {named}")
                                    class:chosen=move || counted.get() == on
                                    on:click=move |_| counted.set(on)
                                >
                                    {label}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
                <button class="cta" type="submit">
                    "Keep from this day"
                </button>
                <ReferencesLink topic=Topic::Water />
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
        </section>
    }
}

#[component]
pub fn HydrateTab() -> impl IntoView {
    let day = RwSignal::new(today());
    view! {
        <DayCard day=day />
        <HydrateDay day=day />
        <HydrateLastDays day=day />
        <ExploreInStats />
        <HydrateShelf />
    }
}
