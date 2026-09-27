use grind_core::bibliography::Topic;
use grind_core::drink::{Drink, DrinkKind};
use grind_core::drinking::Drinking;
use grind_core::figures::{Figure, bare, shown, typed_back};
use grind_core::for_hydrating::DrinkingForm;
use grind_core::for_setting_up_the_profile::HydrateBaseForm;
use grind_core::hydrate::Hydrate;
use grind_core::recommendations::Recommendations;
use grind_core::verdict::Verdict;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::chosen;
use crate::screens::fields::{Chooser, DateField, FigureField};
use crate::screens::gauge::Gauge;
use crate::screens::profile::Recommend;
use crate::screens::references::ReferencesLink;
use crate::screens::{lines, today};
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
    let remove = move |_| {
        let date = date.clone();
        spawn_local(async move {
            match tauri::remove_drinking(&date, id).await {
                Ok(left) => hydrate.set(Some(left)),
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
    let dry = of_the_day.drinkings.is_empty();
    view! {
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
    }
}

#[component]
pub fn HydrateDay(day: RwSignal<String>) -> impl IntoView {
    let hydrate = RwSignal::new(None::<Hydrate>);
    let catalog = RwSignal::new(Vec::<Drink>::new());
    let drink = RwSignal::new(String::new());
    let volume = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    let names = Signal::derive(move || {
        catalog
            .get()
            .into_iter()
            .map(|known| known.name)
            .collect::<Vec<_>>()
    });
    let chosen_drink = move || {
        chosen(&names.get(), &drink.get())
            .and_then(|name| catalog.get().into_iter().find(|known| known.name == name))
    };
    spawn_local(async move {
        match tauri::drinks().await {
            Ok(drinks) => catalog.set(drinks),
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
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
    let log = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some(drunk) = chosen_drink() else {
            said.set(Some(lines::CHOOSE_FIRST.into()));
            return;
        };
        let Some([volume_typed]) = figures_typed([&volume.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = DrinkingForm {
            date: day.get_untracked(),
            drink: drunk.name,
            volume_ml: volume_typed,
        };
        spawn_local(async move {
            match tauri::log_drink(&form).await {
                Ok(logged) => {
                    hydrate.set(Some(logged));
                    drink.set(String::new());
                    volume.set(String::new());
                    said.set(None);
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"Hydrate"</h2>
            <form class="drinking" on:submit=log>
                <div class="row">
                    <Chooser label="Drink" name="drink" options=names typed=drink />
                    <FigureField
                        label="Volume (mL)"
                        name="volume"
                        value=volume
                        placeholder=Signal::derive(move || {
                            chosen_drink().map(|known| bare(Figure::Volume(known.volume_ml)))
                        })
                    />
                </div>
                <button class="cta" type="submit">
                    "Hydrate"
                </button>
                {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
            </form>
            {move || {
                hydrate
                    .get()
                    .map(|of_the_day| {
                        view! { <DrinkingsOfTheDay of_the_day=of_the_day hydrate=hydrate said=said /> }
                    })
            }}
        </section>
    }
}

#[component]
fn DrinkRow(
    drink: Drink,
    drinks: RwSignal<Vec<Drink>>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let name = drink.name.clone();
    let remove = move |_| {
        let name = name.clone();
        spawn_local(async move {
            match tauri::remove_drink(&name).await {
                Ok(left) => drinks.set(left),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <tr>
            <td>{drink.name}</td>
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

const KINDS: [(DrinkKind, &str); 2] = [(DrinkKind::Water, "water"), (DrinkKind::Other, "other")];

fn kind_named(word: &str) -> DrinkKind {
    KINDS
        .into_iter()
        .find(|(named, _)| format!("{named:?}") == word)
        .map_or(DrinkKind::Other, |(named, _)| named)
}

#[component]
pub fn NewDrink(drinks: RwSignal<Vec<Drink>>, done: Callback<String>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let serving = RwSignal::new(String::new());
    let kind = RwSignal::new(DrinkKind::Water);
    let said = RwSignal::new(None::<String>);
    let add = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let named = name.get_untracked();
        let Some([volume_ml]) = figures_typed([&serving.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let volume_ml = volume_ml.unwrap_or(0.0);
        spawn_local(async move {
            match tauri::add_drink(&named, volume_ml, kind.get_untracked()).await {
                Ok(known) => {
                    drinks.set(known);
                    name.set(String::new());
                    serving.set(String::new());
                    done.run(lines::added(named.trim()));
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <form class="drink" on:submit=add>
            <div class="row">
                <label class="field grow2">
                    "Name"
                    <input
                        name="drink_name"
                        autocomplete="off"
                        prop:value=move || name.get()
                        on:input=move |event| name.set(event_target_value(&event))
                    />
                </label>
                <FigureField label="Serving (mL)" name="serving" value=serving />
                <label class="field">
                    "Kind"
                    <select
                        name="drink_kind"
                        on:change=move |event| kind.set(kind_named(&event_target_value(&event)))
                    >
                        {KINDS
                            .into_iter()
                            .map(|(named, label)| {
                                view! { <option value=format!("{named:?}")>{label}</option> }
                            })
                            .collect_view()}
                    </select>
                </label>
            </div>
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
        <section class="card">
            <h2>"Drinks"</h2>
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
        </section>
    }
}

#[component]
pub fn HydrateBaseCard() -> impl IntoView {
    let from = RwSignal::new(today());
    let base = RwSignal::new(String::new());
    let default_ml = RwSignal::new(None::<f64>);
    let said = RwSignal::new(None::<String>);
    let fill = Callback::new(move |advised: Recommendations| {
        base.set(typed_back(advised.hydrate_base_ml));
    });
    spawn_local(async move {
        if let Ok(in_force) = tauri::hydrate_base_on(&today()).await {
            base.set(in_force.base_ml.to_string());
            default_ml.set(in_force.is_default.then_some(in_force.base_ml));
        }
    });
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([base_ml]) = figures_typed([&base.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = HydrateBaseForm {
            from: from.get_untracked(),
            base_ml: base_ml.unwrap_or(0.0),
        };
        spawn_local(async move {
            said.set(Some(match tauri::set_hydrate_base(&form).await {
                Ok(_) => {
                    default_ml.set(None);
                    lines::hydrate_base_kept(&form.from)
                }
                Err(failure) => lines::failure(&failure),
            }));
        });
    };
    view! {
        <section class="card">
            <h2>"Hydrate the material"</h2>
            <form class="hydrate-base" on:submit=save>
                <div class="row">
                    <DateField label="From" name="hydrate_base_from" value=from />
                    <FigureField label="Base (mL)" name="hydrate_base" value=base />
                </div>
                {move || {
                    default_ml
                        .get()
                        .map(|ml| view! { <p class="hint">{lines::default_hydrate_base(ml)}</p> })
                }}
                <Recommend fill=fill said=said />
                <button class="cta" type="submit">
                    "Keep this base"
                </button>
                <ReferencesLink topic=Topic::Water />
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
        </section>
    }
}
