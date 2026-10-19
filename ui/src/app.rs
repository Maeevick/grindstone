use leptos::prelude::*;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::path;

use crate::screens::burn::BurnTab;
use crate::screens::catalog::Catalog;
use crate::screens::fuel::FuelTab;
use crate::screens::home::Home;
use crate::screens::hydrate::HydrateTab;
use crate::screens::lines;
use crate::screens::meals::MealInProgress;
use crate::screens::profile::ProfileScreen;
use crate::screens::references::ReferencesScreen;
use crate::screens::settings::Settings;
use crate::screens::stats::Stats;
use crate::screens::tracker::{GatherTab, Tracker};

#[component]
pub fn App() -> impl IntoView {
    provide_context(MealInProgress::default());
    view! {
        <Router>
            <main class="wrap">
                <header class="top">
                    <img class="logo" src="/icon.svg" alt="" />
                    <h1>"GRINDSTONE"</h1>
                    <span class="tag">"sharpen this rusty you!"</span>
                </header>
                <Routes fallback=|| view! { <p class="empty">{lines::LOST}</p> }>
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/gather") view=GatherTab />
                    <Route path=path!("/fuel") view=FuelTab />
                    <Route path=path!("/hydrate") view=HydrateTab />
                    <Route path=path!("/burn") view=BurnTab />
                    <Route path=path!("/tracker") view=Tracker />
                    <Route path=path!("/catalog") view=Catalog />
                    <Route path=path!("/profile") view=ProfileScreen />
                    <Route path=path!("/settings") view=Settings />
                    <Route path=path!("/stats") view=Stats />
                    <Route path=path!("/references") view=ReferencesScreen />
                </Routes>
            </main>
            <nav class="tabs">
                <A href="/" exact=true attr:aria-label="Home">
                    "🏠"
                </A>
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
