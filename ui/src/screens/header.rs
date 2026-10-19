use grind_core::figures::{Figure, bare};
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use crate::screens::{Recorded, today};
use crate::tauri;

pub(crate) fn tick_class(held: Option<bool>) -> &'static str {
    match held {
        Some(true) => "good",
        Some(false) => "bad",
        None => "",
    }
}

#[component]
fn Tile(
    href: &'static str,
    kind: &'static str,
    icon: &'static str,
    #[prop(into)] figure: Signal<String>,
    unit: &'static str,
    #[prop(into)] held: Signal<Option<bool>>,
) -> impl IntoView {
    view! {
        <A href=href attr:class=move || format!("tile {kind} {}", tick_class(held.get()))>
            <span class="icon">{icon}</span>
            <span class="big">{move || figure.get()}</span>
            <span class="unit">{unit}</span>
        </A>
    }
}

#[component]
fn TodaysTiles() -> impl IntoView {
    let recorded = Recorded::here();
    let pathname = use_location().pathname;
    let again = move || {
        recorded.heard();
        pathname.track();
    };
    let streak = LocalResource::new(move || {
        again();
        async move { tauri::streak(&today()).await }
    });
    let fuel = LocalResource::new(move || {
        again();
        async move { tauri::fuel_day(&today()).await }
    });
    let burn = LocalResource::new(move || {
        again();
        async move { tauri::burn_day(&today()).await }
    });
    let hydrate = LocalResource::new(move || {
        again();
        async move { tauri::hydrate_day(&today()).await }
    });
    let week = LocalResource::new(move || {
        again();
        async move { tauri::gather_week(&today()).await }
    });
    let not_yet = || "-".to_string();
    let ticks = Signal::derive(move || {
        streak
            .get()
            .and_then(Result::ok)
            .and_then(|streak| streak.history.last().map(|today| today.ticks))
    });
    view! {
        <div class="today">
            <h2>"Today"</h2>
            <div class="tiles">
                <Tile
                    href="/gather"
                    kind="weighed"
                    icon="⚖️"
                    figure=Signal::derive(move || {
                        week.get()
                            .and_then(Result::ok)
                            .and_then(|week| week.day)
                            .map_or_else(not_yet, |own| bare(Figure::Mass(own.body_mass_kg)))
                    })
                    unit="kg weighed"
                    held=Signal::derive(move || ticks.get().and_then(|ticks| ticks.weighed))
                />
                <Tile
                    href="/fuel"
                    kind="fuelled"
                    icon="⚡"
                    figure=Signal::derive(move || {
                        fuel.get()
                            .and_then(Result::ok)
                            .map_or_else(not_yet, |fuel| bare(Figure::Energy(fuel.eaten.energy_kcal)))
                    })
                    unit="kcal eaten"
                    held=Signal::derive(move || ticks.get().and_then(|ticks| ticks.fuelled))
                />
                <Tile
                    href="/hydrate"
                    kind="hydrated"
                    icon="💧"
                    figure=Signal::derive(move || {
                        hydrate
                            .get()
                            .and_then(Result::ok)
                            .map_or_else(not_yet, |hydrate| bare(Figure::Volume(hydrate.drunk_ml)))
                    })
                    unit="mL drunk"
                    held=Signal::derive(move || ticks.get().and_then(|ticks| ticks.hydrated))
                />
                <Tile
                    href="/burn"
                    kind="worked-out"
                    icon="🔥"
                    figure=Signal::derive(move || {
                        burn.get()
                            .and_then(Result::ok)
                            .and_then(|burn| burn.burnt_kcal)
                            .map_or_else(not_yet, |kcal| bare(Figure::Energy(kcal)))
                    })
                    unit="kcal burnt"
                    held=Signal::derive(move || ticks.get().and_then(|ticks| ticks.worked_out))
                />
            </div>
        </div>
    }
}

/// Worn by every screen: who the app is, who you are, and how today stands.
#[component]
pub fn Header() -> impl IntoView {
    view! {
        <header class="top">
            <div class="brand">
                <img class="logo" src="/icon.svg" alt="" />
                <div class="title">
                    <h1>"GRINDSTONE"</h1>
                    <span class="tag">"sharpen this rusty you!"</span>
                </div>
                <A href="/profile" attr:class="person" attr:aria-label="Profile">
                    "👤"
                </A>
            </div>
            <TodaysTiles />
        </header>
    }
}
