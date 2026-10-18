use grind_core::figures::{Figure, bare};
use leptos::prelude::*;
use leptos_router::components::A;

use crate::screens::charts::Charts;

use crate::screens::{lines, today};
use crate::tauri;

#[component]
fn Welcome() -> impl IntoView {
    view! {
        <section class="card welcome">
            <h2>"Welcome"</h2>
            <p>{lines::WELCOME}</p>
            <A href="/profile" attr:class="cta">
                "Set up your profile"
            </A>
            <A href="/settings" attr:class="cta ghost">
                "Import the draft"
            </A>
        </section>
    }
}

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
fn StreakCard() -> impl IntoView {
    let streak = LocalResource::new(move || async move { tauri::streak(&today()).await });
    let fuel = LocalResource::new(move || async move { tauri::fuel_day(&today()).await });
    let burn = LocalResource::new(move || async move { tauri::burn_day(&today()).await });
    let hydrate = LocalResource::new(move || async move { tauri::hydrate_day(&today()).await });
    let week = LocalResource::new(move || async move { tauri::gather_week(&today()).await });
    let not_yet = || "-".to_string();
    let ticks = Signal::derive(move || {
        streak
            .get()
            .and_then(Result::ok)
            .and_then(|streak| streak.history.last().map(|today| today.ticks))
    });
    view! {
        <section class="card streak">
            <h2>"Streak"</h2>
            <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
                {move || {
                    streak
                        .get()
                        .map(|streak| match streak {
                            Ok(streak) => {
                                view! {
                                    <p class="count">
                                        <span class="current">{lines::streak_count(streak.days)}</span>
                                        " "
                                        <span class="longest">{lines::longest_streak(streak.longest)}</span>
                                    </p>
                                    <p class="hint">{lines::streak_remark(&streak)}</p>
                                }
                                    .into_any()
                            }
                            Err(failure) => {
                                view! { <p class="bad">{lines::failure(&failure)}</p> }.into_any()
                            }
                        })
                }}
            </Transition>
            <div class="today">
                <h2>"Today"</h2>
                <div class="tiles">
                    <Tile
                        href="/tracker?segment=Gather"
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
                        href="/tracker?segment=Fuel"
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
                        href="/tracker?segment=Hydrate"
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
                        href="/tracker?segment=Burn"
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
                <p class="hint">{lines::STREAK_GRACE}</p>
            </div>
        </section>
    }
}

#[component]
pub fn Home() -> impl IntoView {
    let today = today();
    let profile = LocalResource::new(move || {
        let today = today.clone();
        async move { tauri::profile_on(&today).await }
    });
    view! {
        <Suspense fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
            {move || {
                profile
                    .get()
                    .map(|profile| match profile {
                        Ok(None) => view! { <Welcome /> }.into_any(),
                        Ok(Some(_)) => {
                            view! {
                                <StreakCard />
                                <Charts today=crate::screens::today() />
                                <A href="/stats" attr:class="cta ghost explore">
                                    "Explore in Stats"
                                </A>
                            }
                                .into_any()
                        }
                        Err(failure) => {
                            view! { <p class="bad">{lines::failure(&failure)}</p> }.into_any()
                        }
                    })
            }}
        </Suspense>
    }
}
