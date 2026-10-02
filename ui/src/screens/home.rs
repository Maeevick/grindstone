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
                                    <p class="figures">{lines::streak_line(&streak)}</p>
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
    let hydrate = LocalResource::new(move || async move { tauri::hydrate_day(&today()).await });
    view! {
        <section class="card">
            <h2>"Today"</h2>
            <A href="/stats" attr:class="cta ghost">
                "Explore in Stats"
            </A>
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
                    hydrate
                        .get()
                        .and_then(Result::ok)
                        .map(|hydrate| {
                            let class = lines::verdict_class(hydrate.verdict);
                            view! {
                                <p class=format!("figures {class}")>
                                    {lines::drunk_against_target(&hydrate)}
                                </p>
                            }
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
                                <Charts today=crate::screens::today() />
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
