use grind_core::bibliography::Topic;
use grind_core::drink::{Drink, DrinkKind};
use grind_core::drinking::Drinking;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, shown, typed_back};
use grind_core::for_hydrating::DrinkingForm;
use grind_core::for_setting_up_the_profile::HydrateTargetForm;
use grind_core::hydrate::Hydrate;
use grind_core::kept::Kept;
use grind_core::verdict::Verdict;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::{chosen, grouped, nothing_matches};
use crate::screens::cells::week_table;
use crate::screens::entries::{DrinkFields, DrinkFigures, KINDS, NameField, NewEntry, name_cell};
use crate::screens::fields::{Chooser, DateField, FigureField};
use crate::screens::gauge::Gauge;
use crate::screens::last_days::HydrateLastDays;
use crate::screens::profile::{
    Counted, Recommend, Whose, keep_and_show, shown_counted, shown_in_force,
};
use crate::screens::references::ReferencesLink;
use crate::screens::tab::{DayCard, GrimoireBesideTheNumbers};
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
    let options = Signal::derive(move || {
        let (water, other): (Vec<Drink>, Vec<Drink>) = catalog
            .get()
            .into_iter()
            .partition(|known| known.kind == DrinkKind::Water);
        let named = |drinks: Vec<Drink>| drinks.into_iter().map(|known| known.name).collect();
        grouped(
            (lines::WATER_ICON, named(water)),
            (lines::OTHER_DRINK_ICON, named(other)),
        )
    });
    let names = Signal::derive(move || {
        options
            .get()
            .into_iter()
            .map(|choice| choice.name)
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

#[component]
pub fn HydrateTargetCard() -> impl IntoView {
    let recorded = Recorded::here();
    let from = RwSignal::new(today());
    let water = RwSignal::new(String::new());
    let whose = RwSignal::new(None::<bool>);
    let counted = RwSignal::new(true);
    let said = RwSignal::new(None::<String>);
    let show = move |ml: f64| water.set(typed_back(ml));
    shown_in_force(
        async { tauri::hydrate_target_on(&today()).await },
        show,
        whose,
    );
    shown_counted(counted, |scope| scope.hydrated);
    let keep = move |kept: Kept<f64>, filed: fn(&str) -> String| {
        let form = HydrateTargetForm {
            from: from.get_untracked(),
            kept,
            counted: counted.get_untracked(),
        };
        let filed = lines::filed_with_its_tick(filed(&form.from), form.counted);
        keep_and_show(
            async move { tauri::set_hydrate_target(&form).await },
            show,
            whose,
            said,
            (filed, recorded),
        );
    };
    let recommend = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([typed]) = figures_typed([&water.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let Some(ml) = typed else {
            said.set(Some(lines::NO_WATER_TYPED.into()));
            return;
        };
        keep(Kept::Own(ml), lines::water_target_kept);
    };
    view! {
        <section class="card">
            <h2>"Hydrate the material"</h2>
            <p class="hint">{lines::HYDRATE_TARGET_HINT}</p>
            <form class="water-target" on:submit=save>
                <div class="row">
                    <DateField label="From" name="water_target_from" value=from />
                    <FigureField label="Water a day (mL)" name="water_target" value=water />
                </div>
                <Whose whose=whose />
                <Counted counted=counted />
                <Recommend keep=recommend />
                <button class="cta" type="submit">
                    "Keep this target"
                </button>
                <ReferencesLink topic=Topic::Water />
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
        </section>
    }
}

/// The week up to the tab's day, a logged day's drinks on average, against the band.
#[component]
pub fn HydrateWeekCard(day: RwSignal<String>) -> impl IntoView {
    let recorded = Recorded::here();
    let week = LocalResource::new(move || {
        recorded.heard();
        let date = day.get();
        async move { tauri::hydrate_week(&date, &today()).await }
    });
    view! {
        <section class="card week">
            <h2>"The week's average"</h2>
            <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
                {move || {
                    week.get()
                        .map(|answer| match answer {
                            Ok(Some(week)) => {
                                view! {
                                    {week_table(
                                        week.logged_days,
                                        vec![("Drinks (mL)", week.drunk, Figure::Volume)],
                                    )}
                                    <p class="tip">{lines::quoted(&lines::hydrate_week_said(&week))}</p>
                                }
                                    .into_any()
                            }
                            Ok(None) => view! { <p class="empty">{lines::NO_WEEK_YET}</p> }.into_any(),
                            Err(failure) => {
                                view! { <p class="bad">{lines::failure(&failure)}</p> }.into_any()
                            }
                        })
                }}
            </Transition>
        </section>
    }
}

#[component]
pub fn HydrateTab() -> impl IntoView {
    let day = RwSignal::new(today());
    view! {
        <DayCard day=day />
        <HydrateDay day=day />
        <HydrateWeekCard day=day />
        <HydrateLastDays day=day />
        <GrimoireBesideTheNumbers grimoire="/hydrate/grimoire" />
    }
}
