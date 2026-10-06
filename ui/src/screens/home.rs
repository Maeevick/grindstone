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

#[component]
fn StreakCard() -> impl IntoView {
    let streak = LocalResource::new(move || async move { tauri::streak(&today()).await });
    view! {
        <section class="card streak">
            <h2>"Streak"</h2>
            <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
                {move || {
                    streak
                        .get()
                        .map(|streak| match streak {
                            Ok(streak) => {
                                let today_ticks = streak
                                    .history
                                    .last()
                                    .map(|today| lines::ticks_of(&today.ticks))
                                    .unwrap_or_default();
                                view! {
                                    <p class="current">{lines::streak_count(streak.days)}</p>
                                    <p class="longest">{lines::longest_streak(streak.longest)}</p>
                                    <p class="hint">{lines::streak_remark(&streak)}</p>
                                    <ul class="ticks">
                                        {today_ticks
                                            .into_iter()
                                            .map(|(name, held)| {
                                                view! {
                                                    <li class=if held { "held" } else { "open" }>
                                                        {if held { "✓ " } else { "○ " }}
                                                        {name}
                                                    </li>
                                                }
                                            })
                                            .collect_view()}
                                    </ul>
                                }
                                    .into_any()
                            }
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
fn Tile(
    icon: &'static str,
    #[prop(into)] figure: Signal<String>,
    unit: &'static str,
    #[prop(into)] class: Signal<&'static str>,
) -> impl IntoView {
    view! {
        <div class=move || format!("tile {}", class.get())>
            <span class="icon">{icon}</span>
            <span class="big">{move || figure.get()}</span>
            <span class="unit">{unit}</span>
        </div>
    }
}

#[component]
fn TodayCard() -> impl IntoView {
    let fuel = LocalResource::new(move || async move { tauri::fuel_day(&today()).await });
    let burn = LocalResource::new(move || async move { tauri::burn_day(&today()).await });
    let hydrate = LocalResource::new(move || async move { tauri::hydrate_day(&today()).await });
    let not_yet = || "-".to_string();
    view! {
        <section class="card today">
            <h2>"Today"</h2>
            <div class="tiles">
                <Tile
                    icon="⚡"
                    figure=Signal::derive(move || {
                        fuel.get()
                            .and_then(Result::ok)
                            .map_or_else(not_yet, |fuel| bare(Figure::Energy(fuel.eaten.energy_kcal)))
                    })
                    unit="kcal eaten"
                    class=Signal::derive(move || {
                        fuel.get()
                            .and_then(Result::ok)
                            .and_then(|fuel| fuel.energy)
                            .map_or("", lines::verdict_class)
                    })
                />
                <Tile
                    icon="💧"
                    figure=Signal::derive(move || {
                        hydrate
                            .get()
                            .and_then(Result::ok)
                            .map_or_else(not_yet, |hydrate| bare(Figure::Volume(hydrate.drunk_ml)))
                    })
                    unit="mL drunk"
                    class=Signal::derive(move || {
                        hydrate
                            .get()
                            .and_then(Result::ok)
                            .map_or("", |hydrate| lines::verdict_class(hydrate.verdict))
                    })
                />
                <Tile
                    icon="🔥"
                    figure=Signal::derive(move || {
                        burn.get()
                            .and_then(Result::ok)
                            .and_then(|burn| burn.burnt_kcal)
                            .map_or_else(not_yet, |kcal| bare(Figure::Energy(kcal)))
                    })
                    unit="kcal burnt"
                    class=Signal::derive(|| "")
                />
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
                                <TodayCard />
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
