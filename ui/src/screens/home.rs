use grind_core::for_reviewing_progress::{GatherCard, Window};
use leptos::prelude::*;
use leptos_router::components::A;

use crate::chart::chart_of;

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

const WINDOWS: [(Window, &str); 3] = [
    (Window::Days30, "30 days"),
    (Window::Days90, "90 days"),
    (Window::Everything, "Everything"),
];

#[component]
fn Chart(card: GatherCard) -> impl IntoView {
    chart_of(&card, 360.0, 180.0).map(|chart| {
        view! {
            <svg class="chart" viewBox="0 0 360 180" preserveAspectRatio="none">
                {chart
                    .target
                    .map(|y| view! { <line class="target" x1="0" x2="360" y1=y y2=y /> })}
                <polyline class="trend" points=chart.trend />
                {chart
                    .masses
                    .into_iter()
                    .map(|(x, y)| view! { <circle class="mass" cx=x cy=y r="2.5" /> })
                    .collect_view()}
            </svg>
        }
    })
}

#[component]
fn GatherCardView() -> impl IntoView {
    let window = RwSignal::new(Window::Days30);
    let card = LocalResource::new(move || {
        let chosen = window.get();
        async move { tauri::gather_card(&today(), chosen).await }
    });
    view! {
        <section class="card">
            <h2>"Gather"</h2>
            <div class="pills">
                {WINDOWS
                    .into_iter()
                    .map(|(named, label)| {
                        view! {
                            <button
                                type="button"
                                class=format!("pill window-{named:?}")
                                class:chosen=move || window.get() == named
                                on:click=move |_| window.set(named)
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
                {move || {
                    card.get()
                        .map(|card| match card {
                            Ok(card) => {
                                view! {
                                    <Chart card=card.clone() />
                                    <ul class="figures">
                                        {lines::gather_figures(&card)
                                            .into_iter()
                                            .map(|line| view! { <li>{line}</li> })
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
                                    .recent
                                    .last()
                                    .map(|today| lines::ticks_of(&today.ticks))
                                    .unwrap_or_default();
                                view! {
                                    <p class="figures">{lines::streak_line(&streak)}</p>
                                    <div class="dots">
                                        {streak
                                            .recent
                                            .iter()
                                            .map(|streaked| {
                                                view! {
                                                    <span
                                                        class=format!("dot {:?}", streaked.status)
                                                        title=format!(
                                                            "{}: {}",
                                                            streaked.day,
                                                            lines::status_word(streaked.status),
                                                        )
                                                    ></span>
                                                }
                                            })
                                            .collect_view()}
                                    </div>
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
fn TodayCard() -> impl IntoView {
    let fuel = LocalResource::new(move || async move { tauri::fuel_day(&today()).await });
    let burn = LocalResource::new(move || async move { tauri::burn_day(&today()).await });
    view! {
        <section class="card">
            <h2>"Today"</h2>
            <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
                {move || {
                    fuel.get()
                        .and_then(Result::ok)
                        .map(|fuel| {
                            let class = fuel.energy.map(lines::verdict_class).unwrap_or("hint");
                            view! { <p class=format!("figures {class}")>{lines::energy_line(&fuel)}</p> }
                        })
                }}
                {move || {
                    burn.get()
                        .and_then(Result::ok)
                        .map(|burn| view! { <p class="figures">{lines::burnt(&burn)}</p> })
                }}
            </Transition>
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
                                <GatherCardView />
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
