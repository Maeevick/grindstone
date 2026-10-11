use leptos::prelude::*;
use leptos_router::components::{A, Route, Router, Routes};
use leptos_router::path;

use crate::screens::catalog::Catalog;
use crate::screens::home::Home;
use crate::screens::lines;
use crate::screens::meals::MealInProgress;
use crate::screens::profile::ProfileScreen;
use crate::screens::references::ReferencesScreen;
use crate::screens::settings::Settings;
use crate::screens::stats::Stats;
use crate::screens::tracker::Tracker;

#[component]
pub fn App() -> impl IntoView {
    provide_context(MealInProgress::default());
    view! {
        <Router>
            <main class="wrap">
                <header class="top">
                    <h1>"GRINDSTONE"</h1>
                    <span class="tag">"sharpen this rusty you!"</span>
                </header>
                <Routes fallback=|| view! { <p class="empty">{lines::LOST}</p> }>
                    <Route path=path!("/") view=Home />
                    <Route path=path!("/tracker") view=Tracker />
                    <Route path=path!("/catalog") view=Catalog />
                    <Route path=path!("/profile") view=ProfileScreen />
                    <Route path=path!("/settings") view=Settings />
                    <Route path=path!("/stats") view=Stats />
                    <Route path=path!("/references") view=ReferencesScreen />
                </Routes>
            </main>
            <nav class="tabs">
                <A href="/">"Home"</A>
                <A href="/tracker">"Tracker"</A>
                <A href="/catalog">"Catalog"</A>
                <A href="/profile">"Profile"</A>
            </nav>
        </Router>
    }
}
