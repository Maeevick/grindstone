use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::{Recorded, lines, today};
use crate::tauri;

#[component]
pub fn Settings() -> impl IntoView {
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let import = move |_| {
        spawn_local(async move {
            said.set(Some(match tauri::import_draft(&today()).await {
                Ok(imported) => {
                    recorded.told();
                    lines::imported(&imported)
                }
                Err(failure) => lines::import_failure(&failure),
            }));
        });
    };
    view! {
        <header class="screen-top">
            <h2>"Settings"</h2>
        </header>
        <section class="card" id="draft">
            <h2>"The draft's records"</h2>
            <p class="hint">{lines::IMPORT_HOW}</p>
            <button class="cta import" type="button" on:click=import>
                "Import the draft"
            </button>
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
        </section>
        <DangerZone />
    }
}

const ERASE: &str = "ERASE";

#[component]
fn DangerZone() -> impl IntoView {
    let confirmed = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let erase = move |_| {
        spawn_local(async move {
            said.set(Some(match tauri::erase_everything().await {
                Ok(()) => {
                    confirmed.set(String::new());
                    recorded.told();
                    lines::ERASED.into()
                }
                Err(failure) => lines::failure(&failure),
            }));
        });
    };
    view! {
        <section class="card danger">
            <h2>"Danger zone"</h2>
            <p class="hint">{lines::ERASE_WARNING}</p>
            <label class="field">
                {format!("Type {ERASE} to confirm")}
                <input
                    name="confirm_erase"
                    autocomplete="off"
                    prop:value=move || confirmed.get()
                    on:input=move |event| confirmed.set(event_target_value(&event))
                />
            </label>
            <button
                class="cta erase"
                type="button"
                disabled=move || confirmed.get().trim() != ERASE
                on:click=erase
            >
                "Erase everything"
            </button>
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
        </section>
    }
}
