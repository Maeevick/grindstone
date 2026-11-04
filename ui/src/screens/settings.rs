use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::{Recorded, lines, today};
use crate::tauri;

#[component]
pub fn Settings() -> impl IntoView {
    view! {
        <hr class="divider" />
        <header class="screen-top">
            <h2>"Settings"</h2>
        </header>
        <VaultsCopy />
        <DangerZone />
    }
}

const RESTORE: &str = "RESTORE";

#[component]
fn VaultsCopy() -> impl IntoView {
    let said = RwSignal::new(None::<String>);
    let confirmed = RwSignal::new(String::new());
    let recorded = Recorded::here();
    let restore = move |_| {
        spawn_local(async move {
            said.set(Some(match tauri::restore_a_copy().await {
                Ok(counted) => {
                    confirmed.set(String::new());
                    recorded.told();
                    lines::restored(counted)
                }
                Err(failure) => lines::failure(&failure),
            }));
        });
    };
    let copy = move |_| {
        spawn_local(async move {
            said.set(Some(match tauri::keep_a_copy(&today()).await {
                Ok(counted) => lines::copied(counted),
                Err(failure) => lines::failure(&failure),
            }));
        });
    };
    view! {
        <section class="card" id="copy">
            <h2>"The vault's copy"</h2>
            <p class="hint">{lines::COPY_HOW}</p>
            <button class="cta copy" type="button" on:click=copy>
                "Copy the records"
            </button>
            <p class="hint">{lines::RESTORE_WARNING}</p>
            <label class="field">
                {format!("Type {RESTORE} to confirm")}
                <input
                    name="confirm_restore"
                    autocomplete="off"
                    prop:value=move || confirmed.get()
                    on:input=move |event| confirmed.set(event_target_value(&event))
                />
            </label>
            <button
                class="cta restore"
                type="button"
                disabled=move || confirmed.get().trim() != RESTORE
                on:click=restore
            >
                "Restore the copy"
            </button>
            {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
        </section>
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
