use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::NavigateOptions;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::hooks::use_navigate;
use leptos_router::path;

use crate::lines;
use crate::screens::burn::BurnTab;
use crate::screens::catalog::{BurnShelf, FuelShelf, HydrateShelf};
use crate::screens::fuel::FuelTab;
use crate::screens::gather::GatherTab;
use crate::screens::header::Header;
use crate::screens::hydrate::HydrateTab;
use crate::screens::plate::PlateInProgress;
use crate::screens::profile::ProfileScreen;
use crate::screens::session::SessionInProgress;
use crate::screens::{Recorded, today};
use crate::tauri;

/// Where the app opens: Gather once a profile is kept, the Profile and its welcome before.
#[component]
fn Landing() -> impl IntoView {
    let navigate = use_navigate();
    spawn_local(async move {
        let to = match tauri::profile_on(&today()).await {
            Ok(Some(_)) => "/gather",
            _ => "/profile",
        };
        navigate(
            to,
            NavigateOptions {
                replace: true,
                ..NavigateOptions::default()
            },
        );
    });
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(PlateInProgress::default());
    provide_context(SessionInProgress::default());
    provide_context(Recorded::default());
    view! {
        <Router>
            <main class="wrap">
                <Header />
                <Routes fallback=|| view! { <p class="empty">{lines::LOST}</p> }>
                    <Route path=path!("/") view=Landing />
                    <Route path=path!("/gather") view=GatherTab />
                    <Route path=path!("/fuel") view=FuelTab />
                    <Route path=path!("/fuel/grimoire") view=FuelShelf />
                    <Route path=path!("/hydrate") view=HydrateTab />
                    <Route path=path!("/hydrate/grimoire") view=HydrateShelf />
                    <Route path=path!("/burn") view=BurnTab />
                    <Route path=path!("/burn/grimoire") view=BurnShelf />
                    <Route path=path!("/profile") view=ProfileScreen />
                </Routes>
            </main>
            <nav class="tabs">
                <A href="/gather" attr:aria-label="Gather">
                    "⚖️"
                </A>
                <A href="/fuel" attr:aria-label="Fuel">
                    "⚡"
                </A>
                <A href="/hydrate" attr:aria-label="Hydrate">
                    "💧"
                </A>
                <A href="/burn" attr:aria-label="Burn">
                    "🔥"
                </A>
            </nav>
        </Router>
    }
}
