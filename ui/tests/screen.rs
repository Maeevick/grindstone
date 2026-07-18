#![cfg(target_arch = "wasm32")]

use grind_ui::app::App;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Element, Event, HtmlFormElement, HtmlInputElement};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "
export function answer_commands_with(answers) {
    window.__TAURI__ = { core: { invoke: async (command, args) => {
        if (!(command in answers)) { throw { Broken: `no answer for ${command}` }; }
        const answer = answers[command];
        if (answer !== null && typeof answer === 'object' && '__by' in answer) {
            return answer[args[answer.__by]];
        }
        return answer;
    } } };
}
export function a_moment() {
    return new Promise((resolve) => setTimeout(resolve, 0));
}
")]
extern "C" {
    fn answer_commands_with(answers: &JsValue);
    fn a_moment() -> js_sys::Promise;
}

fn answering(answers: &str) {
    answer_commands_with(&js_sys::JSON::parse(answers).expect("answers in JSON"));
}

fn screen() -> Element {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .expect("a document");
    let screen = document
        .create_element("section")
        .expect("a section for the screen");
    document
        .body()
        .expect("a body")
        .append_child(&screen)
        .expect("the section joins the page");
    screen
}

fn found<T: JsCast>(screen: &Element, selector: &str) -> T {
    screen
        .query_selector(selector)
        .ok()
        .flatten()
        .expect(selector)
        .unchecked_into()
}

async fn settled() {
    JsFuture::from(a_moment()).await.expect("a moment passes");
}

#[wasm_bindgen_test]
async fn a_first_launch_shows_that_no_weigh_in_exists_yet() {
    answering(r#"{ "recent_weigh_ins": [] }"#);
    let screen = screen();

    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), App);
    settled().await;

    assert!(
        screen
            .text_content()
            .unwrap_or_default()
            .contains("No weigh-in yet")
    );
}

#[wasm_bindgen_test]
async fn saving_a_weigh_in_lists_it_and_empties_the_mass_for_the_next_one() {
    answering(
        r#"{ "recent_weigh_ins": [], "log_weigh_in": [{ "date": "2026-10-10", "kg": 72.4 }] }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), App);
    settled().await;
    let mass: HtmlInputElement = found(&screen, "input[inputmode=decimal]");

    mass.set_value("72.4");
    mass.dispatch_event(&Event::new("input").expect("an input event"))
        .expect("the mass is typed");
    found::<HtmlFormElement>(&screen, "form")
        .request_submit()
        .expect("the form submits");
    settled().await;

    let rows = found::<Element>(&screen, "tbody")
        .text_content()
        .unwrap_or_default();
    assert!(rows.contains("2026-10-10") && rows.contains("72.4"));
    assert_eq!(mass.value(), "");
}

#[wasm_bindgen_test]
async fn a_first_launch_welcomes_you_and_points_to_the_profile_and_the_draft_import() {
    answering(r#"{ "profile_on": null }"#);
    let screen = screen();

    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), App);
    settled().await;

    let welcome = screen.text_content().unwrap_or_default();
    assert!(welcome.contains("Greetings, rusty one"), "{welcome}");
    assert!(
        screen
            .query_selector("a[href='/profile']")
            .ok()
            .flatten()
            .is_some()
    );
    assert!(
        screen
            .query_selector("a[href='/settings']")
            .ok()
            .flatten()
            .is_some()
    );
}
