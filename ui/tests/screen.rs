#![cfg(target_arch = "wasm32")]

use grind_ui::app::App;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::Element;

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
async fn a_first_launch_welcomes_you_and_points_to_the_profile_and_the_draft_import() {
    answering(r#"{ "profile_on": null }"#);
    let screen = screen();

    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), App);
    settled().await;

    let welcome = screen.text_content().unwrap_or_default();
    assert!(welcome.contains("Greetings, rusty one"), "{welcome}");
    let to_the_profile: Element = found(&screen, "a[href='/profile']");
    let to_the_import: Element = found(&screen, "a[href='/settings']");
    assert!(
        to_the_profile
            .text_content()
            .is_some_and(|label| label.contains("profile"))
    );
    assert!(
        to_the_import
            .text_content()
            .is_some_and(|label| label.contains("draft"))
    );
}
