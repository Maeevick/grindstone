use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::lines;
use crate::tauri;

#[component]
pub fn Settings() -> impl IntoView {
    let said = RwSignal::new(None::<String>);
    let import = move |_| {
        spawn_local(async move {
            said.set(Some(match tauri::import_draft().await {
                Ok(imported) => lines::imported(&imported),
                Err(failure) => lines::import_failure(&failure),
            }));
        });
    };
    view! {
        <header class="screen-top">
            <h2>"Settings"</h2>
        </header>
        <section class="card">
            <h2>"The draft's records"</h2>
            <p class="hint">{lines::IMPORT_HOW}</p>
            <button class="cta import" type="button" on:click=import>
                "Import the draft"
            </button>
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
        </section>
    }
}
