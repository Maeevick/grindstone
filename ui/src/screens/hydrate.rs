use grind_core::drink::{Drink, DrinkKind};
use grind_core::drinking::Drinking;
use grind_core::figures::{Figure, shown};
use grind_core::for_hydrating::DrinkingForm;
use grind_core::for_setting_up_the_profile::HydrateBaseForm;
use grind_core::hydrate::Hydrate;
use grind_core::hydrate_base::DEFAULT_BASE_SOURCE;
use grind_core::verdict::Verdict;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::fields::{DateField, FigureField};
use crate::screens::gauge::Gauge;
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
    let chosen = RwSignal::new(String::new());
    let servings = RwSignal::new(String::from("1"));
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        match tauri::drinks().await {
            Ok(drinks) => {
                chosen.set(
                    drinks
                        .first()
                        .map(|drink| drink.name.clone())
                        .unwrap_or_default(),
                );
                catalog.set(drinks);
            }
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
        let Some([servings_typed]) = figures_typed([&servings.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = DrinkingForm {
            date: day.get_untracked(),
            drink: chosen.get_untracked(),
            servings: servings_typed.unwrap_or(0.0),
        };
        spawn_local(async move {
            match tauri::log_drink(&form).await {
                Ok(logged) => {
                    hydrate.set(Some(logged));
                    servings.set(String::from("1"));
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
                    <label class="field grow2">
                        "Drink" <select
                            name="drink"
                            prop:value=move || chosen.get()
                            on:change=move |event| chosen.set(event_target_value(&event))
                        >
                            <For
                                each=move || catalog.get()
                                key=|drink| drink.name.clone()
                                children=|drink| {
                                    let label = drink.name.clone();
                                    view! { <option value=drink.name>{label}</option> }
                                }
                            />
                        </select>
                    </label>
                    <FigureField label="Servings" name="servings" value=servings />
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
    catalog: RwSignal<Vec<Drink>>,
    said: RwSignal<Option<String>>,
) -> impl IntoView {
    let name = drink.name.clone();
    let remove = move |_| {
        let name = name.clone();
        spawn_local(async move {
            match tauri::remove_drink(&name).await {
                Ok(left) => catalog.set(left),
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

const KINDS: [(DrinkKind, &str); 4] = [
    (DrinkKind::Water, "water"),
    (DrinkKind::Coffee, "coffee"),
    (DrinkKind::Tea, "tea"),
    (DrinkKind::Other, "other"),
];

fn kind_named(word: &str) -> DrinkKind {
    KINDS
        .into_iter()
        .find(|(named, _)| format!("{named:?}") == word)
        .map_or(DrinkKind::Other, |(named, _)| named)
}

#[component]
pub fn DrinksCatalog() -> impl IntoView {
    let catalog = RwSignal::new(Vec::<Drink>::new());
    let name = RwSignal::new(String::new());
    let serving = RwSignal::new(String::new());
    let kind = RwSignal::new(DrinkKind::Water);
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        match tauri::drinks().await {
            Ok(drinks) => catalog.set(drinks),
            Err(failure) => said.set(Some(lines::failure(&failure))),
        }
    });
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
                Ok(drinks) => {
                    catalog.set(drinks);
                    name.set(String::new());
                    serving.set(String::new());
                    said.set(None);
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <section class="card">
            <h2>"The drinks catalog"</h2>
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
                            on:change=move |event| {
                                kind.set(kind_named(&event_target_value(&event)))
                            }
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
            <table>
                <tbody>
                    <For
                        each=move || catalog.get()
                        key=|drink| format!("{drink:?}")
                        children=move |drink| {
                            view! { <DrinkRow drink=drink catalog=catalog said=said /> }
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
            <h2>"What a day drinks"</h2>
            <form class="hydrate-base" on:submit=save>
                <div class="row">
                    <DateField label="From" name="hydrate_base_from" value=from />
                    <FigureField label="Base (mL)" name="hydrate_base" value=base />
                </div>
                {move || {
                    default_ml
                        .get()
                        .map(|ml| {
                            let source = DEFAULT_BASE_SOURCE;
                            view! {
                                <p class="hint">{lines::default_hydrate_base(ml)}</p>
                                <ul class="sources">
                                    <li>
                                        <a
                                            class="source"
                                            href=source.link
                                            target="_blank"
                                            rel="noopener"
                                        >
                                            {format!(
                                                "{} ({}), {}",
                                                source.authors,
                                                source.year,
                                                source.title,
                                            )}
                                        </a>
                                    </li>
                                </ul>
                            }
                        })
                }}
                <button class="cta" type="submit">
                    "Keep this base"
                </button>
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
        </section>
    }
}
