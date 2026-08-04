#![cfg(target_arch = "wasm32")]

use grind_ui::app::App;
use grind_ui::screens::burn::BurnDay;
use grind_ui::screens::catalog::Catalog;
use grind_ui::screens::fuel::FuelDay;
use grind_ui::screens::profile::ProfileScreen;
use grind_ui::screens::settings::Settings;
use grind_ui::screens::tracker::Tracker;
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

fn value_of(screen: &Element, name: &str) -> String {
    found::<HtmlInputElement>(screen, &format!("input[name={name}]")).value()
}

#[wasm_bindgen_test]
async fn the_profile_and_goals_in_force_today_fill_their_forms() {
    answering(
        r#"{ "profile_on": { "birthday": "1990-06-01", "gender": "Female", "height_cm": 165.0, "neck_cm": 33.0, "hip_cm": 98.0 },
             "goals_on": { "body_mass_kg": 60.0, "fat_ratio": 0.22 } }"#,
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

    assert_eq!(value_of(&screen, "birthday"), "1990-06-01");
    assert_eq!(
        found::<HtmlSelectElement>(&screen, "select[name=gender]").value(),
        "Female"
    );
    assert_eq!(
        [
            value_of(&screen, "height"),
            value_of(&screen, "neck"),
            value_of(&screen, "hip")
        ],
        ["165", "33", "98"]
    );
    assert_eq!(
        [
            value_of(&screen, "target_mass"),
            value_of(&screen, "target_fat")
        ],
        ["60", "22"]
    );
}

const A_MEASURED_DAY: &str = r#"[{ "day": "2026-10-10", "body_mass_kg": 80.0, "waist_cm": 80.0, "age": 41,
    "composition": { "fat_ratio": 0.1162, "fat_mass_kg": 9.3, "lean_mass_kg": 70.7 } }]"#;

#[wasm_bindgen_test]
async fn after_logging_a_measurement_the_day_stays_and_the_form_is_ready_for_the_next() {
    answering(&format!(
        r#"{{ "recent_days": [], "log_measurement": {A_MEASURED_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Tracker />
            </Router>
        }
    });
    settled().await;
    let chosen_day = value_of(&screen, "day");

    typed_into(&screen, "mass", "80");
    typed_into(&screen, "waist", "80");
    found::<HtmlFormElement>(&screen, "form.measurement")
        .request_submit()
        .expect("the measurement is logged");
    settled().await;

    let listed: Element = found(&screen, "tbody");
    assert!(
        listed
            .text_content()
            .is_some_and(|days| days.contains("80.0 kg") && days.contains("11.6%"))
    );
    assert_eq!(
        [value_of(&screen, "mass"), value_of(&screen, "waist")],
        ["", ""]
    );
    assert_eq!(value_of(&screen, "day"), chosen_day);
}

#[wasm_bindgen_test]
async fn removing_a_day_takes_it_off_the_list() {
    answering(&format!(
        r#"{{ "recent_days": {A_MEASURED_DAY}, "remove_measurement": [] }}"#
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Tracker />
            </Router>
        }
    });
    settled().await;

    found::<web_sys::HtmlElement>(&screen, "button.remove").click();
    settled().await;

    let listed: Element = found(&screen, "tbody");
    assert_eq!(listed.text_content().unwrap_or_default(), "");
}

#[wasm_bindgen_test]
async fn the_window_pills_redraw_the_card_over_the_days_they_name() {
    answering(
        r#"{ "profile_on": { "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null },
             "gather_card": { "__by": "window",
                 "Days30": { "masses": [["2026-09-20", 80.4], ["2026-10-10", 80.0]], "trend": [], "target_kg": 70.0,
                             "progress": null, "fat_change_kg": null, "lean_change_kg": null },
                 "Days90": { "masses": [["2026-08-01", 82.0], ["2026-09-20", 80.4], ["2026-10-10", 80.0]], "trend": [],
                             "target_kg": 70.0, "progress": null, "fat_change_kg": null, "lean_change_kg": null } } }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), App);
    settled().await;
    let masses_drawn = |screen: &Element| {
        screen
            .query_selector_all("svg circle")
            .map(|drawn| drawn.length())
            .unwrap_or_default()
    };
    assert_eq!(masses_drawn(&screen), 2);

    found::<web_sys::HtmlElement>(&screen, "button.window-Days90").click();
    settled().await;

    assert_eq!(masses_drawn(&screen), 3);
}

#[wasm_bindgen_test]
async fn importing_the_draft_tells_what_it_brought() {
    answering(
        r#"{ "import_draft": { "measurements": 215, "profiles": 5, "goals": 1, "exercises": 34, "workouts": 659, "foods": 154, "intakes": 1030 } }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Settings />
            </Router>
        }
    });
    settled().await;

    found::<web_sys::HtmlElement>(&screen, "button.import").click();
    settled().await;

    let said = screen.text_content().unwrap_or_default();
    assert!(said.contains("215 days, 5 profiles, 1 goal"), "{said}");
    assert!(
        said.contains("34 exercises, 659 workouts, 154 foods and 1030 intakes"),
        "{said}"
    );
}

const AN_EMPTY_BURN: &str = r#"{ "day": "2026-10-10", "workouts": [], "burnt_kcal": 0.0,
    "off_the_clock": { "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 3.0 }, "off_the_clock_is_usual": true }"#;

#[wasm_bindgen_test]
async fn logging_a_workout_lists_it_with_what_it_burnt() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Walking", "met": 4.0 }}], "burn_day": {AN_EMPTY_BURN},
             "log_workout": {{ "day": "2026-10-10", "burnt_kcal": 252.0, "off_the_clock": null, "off_the_clock_is_usual": true,
                 "workouts": [{{ "id": 1, "burnt_kcal": 252.0, "workout": {{ "day": "2026-10-10", "exercise": "Walking",
                     "met": 4.0, "minutes": 60.0, "reps": null, "sets": 1, "load_kg": 0.0 }} }}] }} }}"#
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        let day = RwSignal::new("2026-10-10".to_string());
        view! {
            <Router>
                <BurnDay day=day />
            </Router>
        }
    });
    settled().await;
    assert_eq!(value_of(&screen, "sitting"), "480");

    typed_into(&screen, "minutes", "60");
    found::<HtmlFormElement>(&screen, "form.workout")
        .request_submit()
        .expect("the workout is logged");
    settled().await;

    let listed: Element = found(&screen, "table.workouts tbody");
    let rows = listed.text_content().unwrap_or_default();
    assert!(
        rows.contains("Walking") && rows.contains("252 kcal"),
        "{rows}"
    );
}

#[wasm_bindgen_test]
async fn the_catalog_adds_an_exercise_to_its_list() {
    answering(
        r#"{ "exercises": [], "add_exercise": [{ "name": "Kettlebell swing", "met": 9.8 }] }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Catalog />
            </Router>
        }
    });
    settled().await;

    typed_into(&screen, "exercise_name", "Kettlebell swing");
    typed_into(&screen, "exercise_met", "9.8");
    found::<HtmlFormElement>(&screen, "form.exercise")
        .request_submit()
        .expect("the exercise is added");
    settled().await;

    let listed: Element = found(&screen, "table.exercises tbody");
    assert!(
        listed
            .text_content()
            .is_some_and(|rows| rows.contains("Kettlebell swing") && rows.contains("9.8"))
    );
}

const A_FUELLED_DAY: &str = r#"{ "day": "2026-10-10",
    "intakes": [[1, { "day": "2026-10-10", "food": "Skyr", "unit": "Gram", "quantity": 200.0,
        "nutrients": { "energy_kcal": 108.0, "protein_g": 19.0, "carbs_g": 8.0, "fat_g": 0.0, "fibre_g": 0.0 } }]],
    "eaten": { "energy_kcal": 108.0, "protein_g": 19.0, "carbs_g": 8.0, "fat_g": 0.0, "fibre_g": 0.0 },
    "band": { "minimum_kcal": 1767.5, "floor_kcal": 2199.4, "cap_kcal": 2650.2 },
    "energy": "Low",
    "macros": {
        "protein": { "eaten_g": 19.0, "floor_g": 160.0, "ceiling_g": 9.0, "verdict": "Low" },
        "fat": { "eaten_g": 0.0, "floor_g": 72.0, "ceiling_g": 4.0, "verdict": "Low" },
        "fibre": { "eaten_g": 0.0, "floor_g": 32.0, "ceiling_g": 2.2, "verdict": "Low" },
        "carbs": { "eaten_g": 8.0, "floor_g": 212.1, "ceiling_g": 295.0, "verdict": "Low" } } }"#;

#[wasm_bindgen_test]
async fn logging_an_intake_shows_it_with_the_days_energy_against_its_band() {
    answering(&format!(
        r#"{{ "foods": [{{ "name": "Skyr", "unit": "Gram", "portion": 100.0,
                "nutrients": {{ "energy_kcal": 54.0, "protein_g": 9.5, "carbs_g": 4.0, "fat_g": 0.0, "fibre_g": 0.0 }} }}],
             "fuel_day": {{ "day": "2026-10-10", "intakes": [], "band": null, "energy": null, "macros": null,
                 "eaten": {{ "energy_kcal": 0.0, "protein_g": 0.0, "carbs_g": 0.0, "fat_g": 0.0, "fibre_g": 0.0 }} }},
             "log_intake": {A_FUELLED_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        let day = RwSignal::new("2026-10-10".to_string());
        view! {
            <Router>
                <FuelDay day=day />
            </Router>
        }
    });
    settled().await;

    typed_into(&screen, "quantity", "200");
    found::<HtmlFormElement>(&screen, "form.intake")
        .request_submit()
        .expect("the intake is logged");
    settled().await;

    let said = screen.text_content().unwrap_or_default();
    assert!(said.contains("108 kcal") && said.contains("2199 to 2650 kcal"), "{said}");
    let listed: Element = found(&screen, "table.intakes tbody");
    assert!(listed.text_content().is_some_and(|rows| rows.contains("Skyr") && rows.contains("200 g")));
    assert!(screen.query_selector(".macros .low").ok().flatten().is_some());
}

#[wasm_bindgen_test]
async fn the_catalog_adds_a_food_to_its_list() {
    answering(
        r#"{ "exercises": [], "foods": [], "add_food": [{ "name": "Skyr", "unit": "Gram", "portion": 100.0,
            "nutrients": { "energy_kcal": 54.0, "protein_g": 9.5, "carbs_g": 4.0, "fat_g": 0.0, "fibre_g": 0.0 } }] }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Catalog />
            </Router>
        }
    });
    settled().await;

    typed_into(&screen, "food_name", "Skyr");
    typed_into(&screen, "food_portion", "100");
    typed_into(&screen, "food_energy", "54");
    found::<HtmlFormElement>(&screen, "form.food")
        .request_submit()
        .expect("the food is added");
    settled().await;

    let listed: Element = found(&screen, "table.foods tbody");
    assert!(listed.text_content().is_some_and(|rows| rows.contains("Skyr") && rows.contains("54 kcal")));
}
