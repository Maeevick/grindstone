#![cfg(target_arch = "wasm32")]

use grind_ui::app::App;
use grind_ui::screens::profile::ProfileScreen;
use leptos::prelude::*;
use leptos_router::components::Router;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Element, Event, HtmlFormElement, HtmlInputElement, HtmlSelectElement};

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

#[wasm_bindgen_test]
async fn the_hip_shows_in_the_profile_for_a_woman_only() {
    answering(r#"{ "profile_on": null, "goals_on": null }"#);
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <ProfileScreen />
            </Router>
        }
    });
    settled().await;
    let hip_shown = |screen: &Element| {
        screen
            .query_selector("input[name=hip]")
            .ok()
            .flatten()
            .is_some()
    };
    assert!(!hip_shown(&screen), "a man's profile asks for no hip");

    let gender: HtmlSelectElement = found(&screen, "select[name=gender]");
    gender.set_value("Female");
    gender
        .dispatch_event(&Event::new("change").expect("a change event"))
        .expect("the gender changes");
    settled().await;

    assert!(hip_shown(&screen), "a woman's profile asks for her hip");
}

fn typed_into(screen: &Element, name: &str, text: &str) {
    let field: HtmlInputElement = found(screen, &format!("input[name={name}]"));
    field.set_value(text);
    field
        .dispatch_event(&Event::new("input").expect("an input event"))
        .expect("the field is typed into");
}

#[wasm_bindgen_test]
async fn a_saved_profile_says_from_which_day_it_is_kept() {
    answering(
        r#"{ "profile_on": null, "goals_on": null,
             "set_profile": { "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null } }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <ProfileScreen />
            </Router>
        }
    });
    settled().await;

    typed_into(&screen, "birthday", "1985-04-12");
    typed_into(&screen, "height", "180");
    typed_into(&screen, "neck", "38,5");
    found::<HtmlFormElement>(&screen, "form.profile")
        .request_submit()
        .expect("the profile is saved");
    settled().await;

    let said = screen.text_content().unwrap_or_default();
    assert!(said.contains("kept from"), "{said}");
}

#[wasm_bindgen_test]
async fn saved_goals_say_from_which_day_they_are_kept() {
    answering(
        r#"{ "profile_on": null, "goals_on": null, "set_goals": { "body_mass_kg": 70.0, "fat_ratio": 0.1 } }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <ProfileScreen />
            </Router>
        }
    });
    settled().await;

    typed_into(&screen, "target_mass", "70");
    typed_into(&screen, "target_fat", "10");
    found::<HtmlFormElement>(&screen, "form.goals")
        .request_submit()
        .expect("the goals are saved");
    settled().await;

    let said: Element = found(&screen, "form.goals .said");
    assert!(
        said.text_content()
            .is_some_and(|line| line.contains("kept from"))
    );
}
