use grind_core::bibliography::Topic;
use grind_core::drink::{Drink, DrinkKind};
use grind_core::drinking::Drinking;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, shown, typed_back};
use grind_core::for_hydrating::DrinkingForm;
use grind_core::for_setting_up_the_profile::HydrateTargetForm;
use grind_core::hydrate::{Hydrate, WATER_SHARE_FLOOR_PERCENT};
use grind_core::kept::Kept;
use grind_core::verdict::{Verdict, verdict_of};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::choice::{chosen, grouped, nothing_matches};
use crate::screens::entries::{DrinkFields, DrinkFigures, KINDS, NameField, NewEntry, name_cell};
use crate::screens::fields::{Chooser, FigureField};
use crate::screens::gauge::Gauge;
use crate::screens::last_days::{HydrateLastDays, entries_failed, entries_said};
use crate::screens::profile::{Advised, ProfileCard, keep_and_show, shown_in_force};
use crate::screens::tab::{DayCard, FormTitle};
use crate::screens::{Recorded, lines, today};
use crate::tauri;
use crate::typed::figures_typed;

/// A drinking among its day's entries: the drink by name, its volume, no share of water of its own, and its
/// removal.
#[component]
fn DrinkingEntry(id: i64, drinking: Drinking, said: RwSignal<Option<String>>) -> impl IntoView {
    let date = drinking.day.to_string();
    let recorded = Recorded::here();
    let remove = move |_| {
        let date = date.clone();
        spawn_local(async move {
            match tauri::remove_drinking(&date, id).await {
                Ok(_) => recorded.told(),
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <tr class="entry">
            <td>{drinking.drink}</td>
            <td>{bare(Figure::Volume(drinking.volume_ml))}</td>
            <td></td>
            <td>
                <button class="remove" type="button" on:click=remove aria-label="Remove this drink">
                    "×"
                </button>
            </td>
        </tr>
    }
}

/// A day's drinkings, opened under its row in the last days.
#[component]
pub fn HydrateEntries(date: String) -> impl IntoView {
    let recorded = Recorded::here();
    let said = RwSignal::new(None::<String>);
    let hydrate = LocalResource::new(move || {
        recorded.heard();
        let date = date.clone();
        async move { tauri::hydrate_day(&date).await }
    });
    view! {
        {move || {
            hydrate
                .get()
                .map(|answer| match answer {
                    Ok(hydrate) => {
                        hydrate
                            .drinkings
                            .into_iter()
                            .map(|(id, drinking)| {
                                view! { <DrinkingEntry id=id drinking=drinking said=said /> }
                            })
                            .collect_view()
                            .into_any()
                    }
                    Err(failure) => entries_failed(&failure),
                })
        }}
        {move || said.get().map(entries_said)}
    }
}

/// Where the day stands: what was drunk against its band, and what the drink about to be logged would make it.
#[component]
fn WhereTheDayStands(of_the_day: Hydrate, about_to_drink: Signal<Option<f64>>) -> impl IntoView {
    let (drunk_ml, floor_ml, target_ml, cap_ml) = (
        of_the_day.drunk_ml,
        of_the_day.floor_ml,
        of_the_day.target_ml,
        of_the_day.cap_ml,
    );
    let drunk_with_it = move || about_to_drink.get().map(|ml| drunk_ml + ml);
    let total = lines::drunk_line(&of_the_day);
    let verdict_class = lines::verdict_class(of_the_day.verdict);
    let past_the_cap = of_the_day.verdict == Verdict::High;
    let (water_ml, water_share) = (of_the_day.water_ml(), of_the_day.water_share());
    let others_share = if drunk_ml > 0.0 {
        100.0 - water_share
    } else {
        0.0
    };
    let water_class =
        lines::verdict_class(verdict_of(water_share, WATER_SHARE_FLOOR_PERCENT, 100.0));
    view! {
        <section class="card bands">
            <h2>"Where the day stands"</h2>
            <p class=format!("figures {verdict_class}")>{total}</p>
            {move || {
                drunk_with_it()
                    .map(|ml| view! { <p class="hint with-the-preview">{lines::with_the_drink(ml)}</p> })
            }}
            {move || {
                view! {
                    <Gauge
                        amount=drunk_ml
                        floor=floor_ml
                        optimal=target_ml
                        cap=cap_ml
                        shown_as=Figure::Volume
                        ghost=drunk_with_it()
                    />
                }
            }}
            {past_the_cap.then(|| view! { <p class="bad">{lines::PAST_THE_CAP}</p> })}
            <table class="kinds">
                <thead>
                    <tr>
                        <th>"Kind"</th>
                        <th>"Drunk (mL)"</th>
                        <th>"%"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr class="water">
                        <td>"Water"</td>
                        <td>{bare(Figure::Volume(water_ml))}</td>
                        <td class=water_class>{bare(Figure::Ratio(water_share / 100.0))}</td>
                    </tr>
                    <tr class="others">
                        <td>"Others"</td>
                        <td>{bare(Figure::Volume(drunk_ml - water_ml))}</td>
                        <td>{bare(Figure::Ratio(others_share / 100.0))}</td>
                    </tr>
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
    let about_to_drink = Signal::derive(move || {
        let [typed] = figures_typed([&volume.get()])?;
        let serving = chosen_drink().map(|known| known.volume_ml).or_else(|| {
            creating
                .get()
                .then(|| figures_typed([&figures.serving.get()]).and_then(|[serving]| serving))
                .flatten()
        });
        typed.or(serving)
    });
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
        recorded.heard();
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
            <FormTitle title="Hydrate" grimoire="/hydrate/grimoire" />
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
                    view! {
                        <WhereTheDayStands of_the_day=of_the_day about_to_drink=about_to_drink />
                    }
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
    let advised = Advised::here();
    let said = RwSignal::new(None::<String>);
    let show = move |ml: Option<f64>| water.set(ml.map(typed_back).unwrap_or_default());
    shown_in_force(
        async { tauri::hydrate_target_on(&today()).await },
        show,
        from,
    );
    let keep = move |kept: Kept<f64>, filed: fn(&str) -> String| {
        let form = HydrateTargetForm {
            from: from.get_untracked(),
            kept,
        };
        let filed = filed(&form.from);
        keep_and_show(
            async move { tauri::set_hydrate_target(&form).await },
            show,
            said,
            (filed, recorded),
        );
    };
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([typed]) = figures_typed([&water.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        match typed {
            Some(ml) => keep(Kept::Own(ml), lines::water_target_kept),
            None => keep(Kept::Recommended, lines::recommendation_kept),
        }
    };
    let forget = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
    view! {
        <ProfileCard
            title="Hydrate the material"
            form="water-target"
            how=lines::WATER_HOW
            why=lines::WATER_WHY.to_string()
            topic=Topic::Water
            save="Store this aim"
            forget=forget
            said=said
            on_submit=Callback::new(save)
        >
            <div class="row">
                <FigureField
                    label="Water a day (mL)"
                    name="water_target"
                    value=water
                    placeholder=advised.placeholder(|advised| advised.water_ml)
                />
            </div>
        </ProfileCard>
    }
}

#[component]
pub fn HydrateTab() -> impl IntoView {
    let day = RwSignal::new(today());
    view! {
        <DayCard day=day />
        <HydrateDay day=day />
        <HydrateLastDays day=day />
    }
}
