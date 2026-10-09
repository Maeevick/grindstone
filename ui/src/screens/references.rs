use grind_core::bibliography::{Reference, TOPICS, Topic, references_about};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;

use crate::screens::lines;
use crate::tauri;

pub fn anchor_of(topic: Topic) -> &'static str {
    match topic {
        Topic::BodyComposition => "body-composition",
        Topic::HealthyRanges => "healthy-ranges",
        Topic::Energy => "energy",
        Topic::Activity => "activity",
        Topic::Macronutrients => "macronutrients",
        Topic::Water => "water",
    }
}

#[component]
pub fn ReferencesLink(topic: Topic) -> impl IntoView {
    view! {
        <A href=format!("/references#{}", anchor_of(topic)) attr:class="references">
            "Scientific references"
        </A>
    }
}

#[component]
fn PublicPage(page: &'static str) -> impl IntoView {
    let open = move |event: leptos::ev::MouseEvent| {
        event.prevent_default();
        spawn_local(async move {
            let _ = tauri::open_page(page).await;
        });
    };
    view! {
        <a class="page" href=page on:click=open>
            "public page"
        </a>
    }
}

#[component]
fn Cited(reference: Reference) -> impl IntoView {
    view! {
        <li>
            {format!("{} ({}). ", reference.authors, reference.year)}
            <em>{reference.title}</em>
            {format!(". {}. ", reference.published_in)}
            {reference.page.map(|page| view! { <PublicPage page=page /> })}
            <p class="hint">{reference.used_for}</p>
        </li>
    }
}

#[component]
pub fn ReferencesScreen() -> impl IntoView {
    view! {
        <header class="screen-top">
            <h2>"Scientific references"</h2>
        </header>
        <p class="hint">{lines::REFERENCES_INTRO}</p>
        <p class="quote">{lines::READING}</p>
        {TOPICS
            .into_iter()
            .map(|topic| {
                view! {
                    <section class="card topic" id=anchor_of(topic)>
                        <h2>{lines::topic_name(topic)}</h2>
                        <ol class="bibliography">
                            {references_about(topic)
                                .into_iter()
                                .map(|reference| view! { <Cited reference=reference /> })
                                .collect_view()}
                        </ol>
                    </section>
                }
            })
            .collect_view()}
    }
}
