#![cfg(target_arch = "wasm32")]

use grind_core::burn::Burn;
use grind_ui::app::App;
use grind_ui::screens::burn::BurnDay;
use grind_ui::screens::catalog::Catalog;
use grind_ui::screens::charts::Charts;
use grind_ui::screens::circuits::{CircuitCrafter, CircuitLog};
use grind_ui::screens::fuel::FuelDay;
use grind_ui::screens::hydrate::{DrinksCatalog, HydrateBaseCard, HydrateDay};
use grind_ui::screens::meals::MealCooker;
use grind_ui::screens::profile::ProfileScreen;
use grind_ui::screens::references::ReferencesScreen;
use grind_ui::screens::settings::Settings;
use grind_ui::screens::stats::Stats;
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
    window.__asked = {};
    window.__TAURI__ = { core: { invoke: async (command, args) => {
        window.__asked[command] = JSON.stringify(args);
        if (!(command in answers)) { throw { Broken: `no answer for ${command}` }; }
        const answer = answers[command];
        if (answer !== null && typeof answer === 'object' && '__by' in answer) {
            return answer[args[answer.__by]];
        }
        return answer;
    } } };
}
export function asked(command) {
    return window.__asked[command] || '';
}
export function a_moment() {
    return new Promise((resolve) => setTimeout(resolve, 0));
}
")]
extern "C" {
    fn answer_commands_with(answers: &JsValue);
    fn asked(command: &str) -> String;
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

const A_GATHERED_DAY: &str = r#"{ "day": "2026-10-10", "body_mass_kg": 80.0, "waist_cm": 80.0, "age": 41,
    "composition": { "fat_ratio": 0.1162, "fat_mass_kg": 9.3, "lean_mass_kg": 70.7 } }"#;

fn a_gathered_week() -> String {
    format!(
        r#"{{ "day": {A_GATHERED_DAY},
              "days": [{A_GATHERED_DAY},
                       {{ "day": "2026-10-07", "body_mass_kg": 80.6, "waist_cm": null, "age": 41,
                          "composition": {{ "fat_ratio": 0.1158, "fat_mass_kg": 9.6, "lean_mass_kg": 71.0 }} }}],
              "tendency": {{ "body_mass": {{ "by": -0.6, "heading": "Toward" }}, "waist": null,
                             "fat_ratio": {{ "by": 0.0004, "heading": "Steady" }},
                             "lean_mass": {{ "by": -0.3, "heading": "Away" }} }} }}"#
    )
}

const A_WEEK_NOT_GATHERED_YET: &str = r#"{ "day": null, "days": [],
    "tendency": { "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null } }"#;

fn the_tracker(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Tracker />
            </Router>
        }
    })
}

fn label_of(screen: &Element, name: &str) -> String {
    found::<HtmlInputElement>(screen, &format!("input[name={name}]"))
        .parent_element()
        .and_then(|label| label.text_content())
        .unwrap_or_default()
}

#[wasm_bindgen_test]
async fn the_chosen_day_shows_as_year_month_day() {
    answering(&format!(
        r#"{{ "gather_week": {A_WEEK_NOT_GATHERED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_tracker(&screen);
    settled().await;

    typed_into(&screen, "day", "2026-10-10");
    settled().await;

    let shown: Element = found(&screen, ".date .shown");
    assert_eq!(shown.text_content().unwrap_or_default(), "2026-10-10");
}

#[wasm_bindgen_test]
async fn a_gathered_day_fills_its_fields_and_a_day_not_gathered_yet_marks_the_body_mass_required() {
    answering(&format!(
        r#"{{ "gather_week": {{ "__by": "date", "2026-10-10": {}, "2026-10-11": {A_WEEK_NOT_GATHERED_YET} }} }}"#,
        a_gathered_week()
    ));
    let screen = screen();
    let _mounted = the_tracker(&screen);
    settled().await;

    typed_into(&screen, "day", "2026-10-10");
    settled().await;
    assert_eq!(
        [value_of(&screen, "mass"), value_of(&screen, "waist")],
        ["80", "80"]
    );
    assert!(!label_of(&screen, "mass").contains('*'));

    typed_into(&screen, "day", "2026-10-11");
    settled().await;
    assert_eq!(
        [value_of(&screen, "mass"), value_of(&screen, "waist")],
        ["", ""]
    );
    assert!(label_of(&screen, "mass").contains('*'));
}

#[wasm_bindgen_test]
async fn the_week_lists_its_days_under_their_units_the_chosen_one_highlighted_with_a_tendency_row()
{
    answering(&format!(r#"{{ "gather_week": {} }}"#, a_gathered_week()));
    let screen = screen();
    let _mounted = the_tracker(&screen);
    settled().await;

    typed_into(&screen, "day", "2026-10-10");
    settled().await;

    let text = |selector: &str| {
        found::<Element>(&screen, selector)
            .text_content()
            .unwrap_or_default()
    };
    assert!(
        text(".week thead").contains("Mass (kg)") && text(".week thead").contains("Fat (%)"),
        "{}",
        text(".week thead")
    );
    assert!(text(".week tr.chosen").contains("2026-10-10"));
    assert!(text(".week tbody").contains("80.6") && !text(".week tbody").contains(" kg"));
    assert_eq!(text(".week tfoot .toward"), "-0.6");
    assert_eq!(text(".week tfoot .steady"), "-");
    assert_eq!(text(".week tfoot .away"), "-0.3");
    found::<Element>(&screen, ".week a[href='/stats']");
}

#[wasm_bindgen_test]
async fn after_logging_a_measurement_the_day_stays_and_the_form_shows_what_was_gathered() {
    answering(&format!(
        r#"{{ "gather_week": {A_WEEK_NOT_GATHERED_YET}, "log_measurement": {} }}"#,
        a_gathered_week()
    ));
    let screen = screen();
    let _mounted = the_tracker(&screen);
    settled().await;
    typed_into(&screen, "day", "2026-10-10");
    settled().await;

    typed_into(&screen, "mass", "80");
    typed_into(&screen, "waist", "80");
    found::<HtmlFormElement>(&screen, "form.measurement")
        .request_submit()
        .expect("the measurement is logged");
    settled().await;

    let listed: Element = found(&screen, ".week tbody");
    assert!(
        listed
            .text_content()
            .is_some_and(|days| days.contains("80.0") && days.contains("11.6"))
    );
    assert_eq!(
        [value_of(&screen, "mass"), value_of(&screen, "waist")],
        ["80", "80"]
    );
    assert_eq!(value_of(&screen, "day"), "2026-10-10");
}

#[wasm_bindgen_test]
async fn removing_a_day_takes_it_off_the_week() {
    answering(&format!(
        r#"{{ "gather_week": {}, "remove_measurement": {A_WEEK_NOT_GATHERED_YET} }}"#,
        a_gathered_week()
    ));
    let screen = screen();
    let _mounted = the_tracker(&screen);
    settled().await;
    typed_into(&screen, "day", "2026-10-10");
    settled().await;

    found::<web_sys::HtmlElement>(&screen, ".week button.remove").click();
    settled().await;

    let listed: Element = found(&screen, ".week tbody");
    assert_eq!(listed.text_content().unwrap_or_default(), "");
    assert!(asked("remove_measurement").contains("2026-10-10"));
}

#[wasm_bindgen_test]
async fn the_window_pills_redraw_the_chart_over_the_days_they_name() {
    answering(&format!(
        r#"{{ {THE_CHARTS}, "gather_month": {{ "month": "2026-10", "earlier": null, "rows": [] }},
             "gather_chart": {{ "__by": "window",
                 "Days30": {{ "masses": [["2026-09-20", 80.4], ["2026-10-10", 80.0]], "lean_masses": [],
                              "goal_kg": 70.0, "goal_lean_kg": null }},
                 "Days90": {{ "masses": [["2026-08-01", 82.0], ["2026-09-20", 80.4], ["2026-10-10", 80.0]],
                              "lean_masses": [], "goal_kg": 70.0, "goal_lean_kg": null }} }} }}"#
    ));
    let screen = screen();
    let _mounted = the_charts(&screen);
    settled().await;
    let masses_drawn = |screen: &Element| {
        screen
            .query_selector("svg polyline.mass")
            .ok()
            .flatten()
            .and_then(|line| line.get_attribute("points"))
            .map(|points| points.split_whitespace().count())
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
        r#"{ "import_draft": { "measurements": 215, "profiles": 5, "goals": 1, "exercises": 34, "workouts": 659, "foods": 154, "intakes": 1030, "drinks": 2, "drinkings": 221 } }"#,
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
        said.contains(
            "34 exercises, 659 workouts, 154 foods, 1030 intakes, 2 drinks and 221 drinkings"
        ),
        "{said}"
    );
}

const AN_EMPTY_BURN: &str = r#"{ "day": "2026-10-10", "workouts": [], "burnt_kcal": 0.0,
    "off_the_clock": { "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 3.0 }, "off_the_clock_is_usual": true }"#;

#[wasm_bindgen_test]
async fn logging_a_workout_lists_it_with_what_it_burnt() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Walking", "met": 4.0 }}], "circuits": [], "burn_day": {AN_EMPTY_BURN},
             "log_workout": {{ "day": "2026-10-10", "burnt_kcal": 252.0, "off_the_clock": null, "off_the_clock_is_usual": true,
                 "workouts": [{{ "id": 1, "burnt_kcal": 252.0, "workout": {{ "day": "2026-10-10", "exercise": "Walking",
                     "met": 4.0, "minutes": 60.0, "reps": null, "sets": 1, "load_kg": 0.0, "circuit": null }} }}] }} }}"#
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
        r#"{ "foods": [], "exercises": [], "add_exercise": [{ "name": "Kettlebell swing", "met": 9.8 }] }"#,
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
    found::<web_sys::HtmlElement>(&screen, "button.shelf-Exercises").click();
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
    "band": { "minimum_kcal_per_kg_of_lean_mass": 25.0, "minimum_kcal": 1767.5, "floor_kcal": 2199.4, "cap_kcal": 2650.2 },
    "energy": "Low",
    "macros": {
        "protein": { "eaten_g": 19.0, "floor_g": 160.0, "ceiling_g": 9.0, "verdict": "Low" },
        "fat": { "eaten_g": 0.0, "floor_g": 72.0, "ceiling_g": 4.0, "verdict": "Low" },
        "fibre": { "eaten_g": 0.0, "floor_g": 32.0, "ceiling_g": 2.2, "verdict": "Low" },
        "carbs": { "eaten_g": 8.0, "floor_g": 212.1, "ceiling_g": 295.0, "verdict": "Low" } } }"#;

const THREE_FOODS: &str = r#"[
    { "name": "Ail", "unit": "Gram", "portion": 10.0,
      "nutrients": { "energy_kcal": 15.0, "protein_g": 0.6, "carbs_g": 3.3, "fat_g": 0.1, "fibre_g": 0.2 } },
    { "name": "Skyr", "unit": "Gram", "portion": 100.0,
      "nutrients": { "energy_kcal": 54.0, "protein_g": 9.5, "carbs_g": 4.0, "fat_g": 0.0, "fibre_g": 0.0 } },
    { "name": "Tapenade", "unit": "Gram", "portion": 30.0,
      "nutrients": { "energy_kcal": 150.0, "protein_g": 0.5, "carbs_g": 1.0, "fat_g": 16.0, "fibre_g": 1.0 } }]"#;

const A_DAY_NOT_FUELLED_YET: &str = r#"{ "day": "2026-10-10", "intakes": [], "band": null, "energy": null, "macros": null,
    "eaten": { "energy_kcal": 0.0, "protein_g": 0.0, "carbs_g": 0.0, "fat_g": 0.0, "fibre_g": 0.0 } }"#;

fn the_fuel_day(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        let day = RwSignal::new("2026-10-10".to_string());
        view! {
            <Router>
                <FuelDay day=day />
            </Router>
        }
    })
}

fn focused(screen: &Element, name: &str) {
    found::<HtmlInputElement>(screen, &format!("input[name={name}]"))
        .dispatch_event(&Event::new("focus").expect("a focus event"))
        .expect("the field is focused");
}

fn choices(screen: &Element) -> Vec<String> {
    let offered = screen
        .query_selector_all("ul.choices li")
        .expect("the choices");
    (0..offered.length())
        .filter_map(|at| offered.item(at))
        .filter_map(|choice| choice.text_content())
        .collect()
}

fn picked(screen: &Element, option: &str) {
    let offered = screen
        .query_selector_all("ul.choices li")
        .expect("the choices");
    (0..offered.length())
        .filter_map(|at| offered.item(at))
        .find(|choice| choice.text_content().as_deref() == Some(option))
        .expect(option)
        .dispatch_event(&Event::new("mousedown").expect("a mousedown event"))
        .expect("the option is picked");
}

fn text_of(screen: &Element, selector: &str) -> String {
    found::<Element>(screen, selector)
        .text_content()
        .unwrap_or_default()
}

#[wasm_bindgen_test]
async fn the_food_chooser_starts_on_select_me_and_narrows_from_three_letters() {
    answering(&format!(
        r#"{{ "foods": {THREE_FOODS}, "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;
    let food: HtmlInputElement = found(&screen, "input[name=food]");
    assert_eq!(
        (food.value(), food.placeholder()),
        (String::new(), "Select me".to_string())
    );

    focused(&screen, "food");
    settled().await;
    assert_eq!(choices(&screen).len(), 3);

    typed_into(&screen, "food", "sky");
    settled().await;
    assert_eq!(choices(&screen), vec!["Skyr".to_string()]);

    picked(&screen, "Skyr");
    settled().await;
    assert_eq!(value_of(&screen, "food"), "Skyr");
    assert!(choices(&screen).is_empty());
}

#[wasm_bindgen_test]
async fn a_quantity_left_empty_logs_one_portion_of_the_food_its_placeholder_shows() {
    answering(&format!(
        r#"{{ "foods": {THREE_FOODS}, "fuel_day": {A_DAY_NOT_FUELLED_YET}, "log_intake": {A_FUELLED_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    typed_into(&screen, "food", "skyr");
    settled().await;
    let quantity: HtmlInputElement = found(&screen, "input[name=quantity]");
    assert_eq!(quantity.placeholder(), "100");
    assert!(label_of(&screen, "quantity").contains("(g)"));
    found::<HtmlFormElement>(&screen, "form.intake")
        .request_submit()
        .expect("the intake is logged");
    settled().await;

    let sent = asked("log_intake");
    assert!(
        sent.contains(r#""food":"Skyr""#) && sent.contains(r#""quantity":null"#),
        "{sent}"
    );
    assert!(text_of(&screen, "table.intakes tbody").contains("Skyr"));
}

#[wasm_bindgen_test]
async fn the_days_energy_sits_on_a_gauge_from_its_minimum_through_alpert_to_maintenance() {
    answering(&format!(
        r#"{{ "foods": [], "fuel_day": {A_FUELLED_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    let said = screen.text_content().unwrap_or_default();
    assert!(
        said.contains("108 kcal") && said.contains("2199 to 2650 kcal"),
        "{said}"
    );
    let gauge = text_of(&screen, ".fuel-gauge");
    assert!(
        gauge.contains("EA25 1768")
            && gauge.contains("Alpert 2199")
            && gauge.contains("Maintenance 2650"),
        "{gauge}"
    );
    found::<Element>(&screen, ".fuel-gauge line.minimum");
    found::<Element>(&screen, ".fuel-gauge rect.band");
    found::<Element>(&screen, ".fuel-gauge circle.eaten");
}

#[wasm_bindgen_test]
async fn the_macros_card_tells_each_goal_what_was_eaten_the_gap_and_its_share_of_the_energy() {
    answering(&format!(
        r#"{{ "foods": [], "fuel_day": {A_FUELLED_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    let heads = text_of(&screen, "table.macros thead");
    assert!(
        heads.contains("Goal (g)")
            && heads.contains("Current (g)")
            && heads.contains("Δ (g)")
            && heads.contains("% cal"),
        "{heads}"
    );
    let protein = text_of(&screen, "table.macros tr.protein");
    assert!(
        protein.contains("160")
            && protein.contains("19")
            && protein.contains("-141")
            && protein.contains("70.4"),
        "{protein}"
    );
    found::<Element>(&screen, "table.macros tr.protein .low");
    assert!(text_of(&screen, "table.macros tfoot").contains("100.0"));
}

#[wasm_bindgen_test]
async fn the_days_intakes_list_each_food_with_its_quantity_energy_and_macros() {
    answering(&format!(
        r#"{{ "foods": [], "fuel_day": {A_FUELLED_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    let heads = text_of(&screen, "table.intakes thead");
    assert!(
        heads.contains("kcal") && heads.contains("Prot (g)") && heads.contains("Carbs (g)"),
        "{heads}"
    );
    let row = text_of(&screen, "table.intakes tbody tr");
    assert!(
        row.contains("Skyr") && row.contains("200 g") && row.contains("108") && row.contains("19"),
        "{row}"
    );
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
    assert!(
        listed
            .text_content()
            .is_some_and(|rows| rows.contains("Skyr") && rows.contains("54 kcal"))
    );
}

const A_COOKED_MEAL: &str = r#"{ "name": "Rice and chicken", "recipe": "Boil, sear, stir.",
    "ingredients": [{ "food": "Rice", "unit": "Gram", "quantity": 200.0,
        "nutrients": { "energy_kcal": 700.0, "protein_g": 14.0, "carbs_g": 150.0, "fat_g": 1.0, "fibre_g": 2.0 } }],
    "raw_g": 200.0, "cooked_g": 500.0,
    "totals": { "energy_kcal": 700.0, "protein_g": 14.0, "carbs_g": 150.0, "fat_g": 1.0, "fibre_g": 2.0 } }"#;

#[wasm_bindgen_test]
async fn a_meal_weighed_up_shows_its_totals_and_once_cooked_joins_the_list() {
    answering(&format!(
        r#"{{ "foods": [{{ "name": "Rice", "unit": "Gram", "portion": 100.0,
                "nutrients": {{ "energy_kcal": 350.0, "protein_g": 7.0, "carbs_g": 75.0, "fat_g": 0.5, "fibre_g": 1.0 }} }}],
             "meals": [], "preview_meal": {A_COOKED_MEAL}, "cook_meal": [{A_COOKED_MEAL}] }}"#
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <MealCooker />
            </Router>
        }
    });
    settled().await;

    typed_into(&screen, "meal_name", "Rice and chicken");
    typed_into(&screen, "ingredient_quantity", "200");
    found::<web_sys::HtmlElement>(&screen, "button.add-ingredient").click();
    settled().await;
    typed_into(&screen, "cooked", "500");
    found::<web_sys::HtmlElement>(&screen, "button.weigh-up").click();
    settled().await;

    let said = screen.text_content().unwrap_or_default();
    assert!(
        said.contains("700 kcal") && said.contains("140 kcal"),
        "{said}"
    );

    found::<HtmlFormElement>(&screen, "form.meal")
        .request_submit()
        .expect("the meal is cooked");
    settled().await;

    let listed: Element = found(&screen, "table.meals tbody");
    assert!(
        listed
            .text_content()
            .is_some_and(|rows| rows.contains("Rice and chicken"))
    );
}

#[wasm_bindgen_test]
async fn home_shows_the_streak_its_recent_days_and_todays_ticks() {
    answering(
        r#"{ "profile_on": { "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null },
             "gather_card": { "masses": [], "trend": [], "target_kg": null, "progress": null, "fat_change_kg": null, "lean_change_kg": null },
             "streak": { "days": 2, "at_risk": true, "history": [
                 { "day": "2026-10-08", "status": "Success", "ticks": { "weighed": true, "fuelled": true, "worked_out": true, "hydrated": null } },
                 { "day": "2026-10-09", "status": "Joker", "ticks": { "weighed": true, "fuelled": false, "worked_out": true, "hydrated": null } },
                 { "day": "2026-10-10", "status": "Pending", "ticks": { "weighed": true, "fuelled": false, "worked_out": false, "hydrated": true } } ] },
             "fuel_day": { "day": "2026-10-10", "intakes": [], "band": null, "energy": null, "macros": null,
                 "eaten": { "energy_kcal": 0.0, "protein_g": 0.0, "carbs_g": 0.0, "fat_g": 0.0, "fibre_g": 0.0 } },
             "burn_day": { "day": "2026-10-10", "workouts": [], "burnt_kcal": 0.0, "off_the_clock": null, "off_the_clock_is_usual": true } }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), App);
    settled().await;
    settled().await;

    let said = screen.text_content().unwrap_or_default();
    assert!(said.contains("2 days"), "{said}");
    let dots = screen
        .query_selector_all(".streak .week .dot")
        .map(|found| found.length())
        .unwrap_or_default();
    assert_eq!(dots, 3);
    assert!(
        screen
            .query_selector(".streak .week .dot.Joker")
            .ok()
            .flatten()
            .is_some()
    );
    assert!(
        screen
            .query_selector(".ticks .held")
            .ok()
            .flatten()
            .is_some()
    );
}

#[wasm_bindgen_test]
async fn stats_draw_two_measures_and_tell_how_they_move_together() {
    answering(
        r#"{ "explore": { "one": [["2026-10-08", 80.4], ["2026-10-09", 80.2], ["2026-10-10", 80.0]],
               "other": [["2026-10-08", 2600.0], ["2026-10-09", 2500.0], ["2026-10-10", 2400.0]],
               "one_summary": { "first": 80.4, "last": 80.0, "mean": 80.2, "lowest": 80.0, "highest": 80.4 },
               "other_summary": { "first": 2600.0, "last": 2400.0, "mean": 2500.0, "lowest": 2400.0, "highest": 2600.0 },
               "pairs": [[80.4, 2600.0], [80.2, 2500.0], [80.0, 2400.0]], "correlation": 1.0 } }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Stats />
            </Router>
        }
    });
    settled().await;

    let lines_drawn = screen
        .query_selector_all("svg.timeline polyline")
        .map(|found| found.length())
        .unwrap_or_default();
    let pairs_drawn = screen
        .query_selector_all("svg.scatter circle")
        .map(|found| found.length())
        .unwrap_or_default();
    let said = screen.text_content().unwrap_or_default();
    assert_eq!((lines_drawn, pairs_drawn), (2, 3));
    assert!(
        said.contains("80.4 kg") && said.contains("2400 kcal") && said.contains("1.00"),
        "{said}"
    );
}

const MORNING_RUST: &str = r#"{ "name": "Morning rust", "steps": [
    { "exercise": "Push up", "met": 3.8, "reps": 12, "minutes": 0.5, "load_kg": 0.0 },
    { "exercise": "Burpee", "met": 8.0, "reps": null, "minutes": 1.0, "load_kg": 5.0 } ] }"#;

fn chosen_in(screen: &Element, name: &str, value: &str) {
    let field: HtmlSelectElement = found(screen, &format!("select[name={name}]"));
    field.set_value(value);
    field
        .dispatch_event(&Event::new("change").expect("a change event"))
        .expect("the choice is made");
}

#[wasm_bindgen_test]
async fn the_crafter_crafts_a_circuit_from_its_steps_added_one_by_one() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Burpee", "met": 8.0 }}, {{ "name": "Push up", "met": 3.8 }}],
             "circuits": [], "craft_circuit": [{MORNING_RUST}] }}"#
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <CircuitCrafter />
            </Router>
        }
    });
    settled().await;

    typed_into(&screen, "circuit_name", "Morning rust");
    chosen_in(&screen, "step_exercise", "Push up");
    typed_into(&screen, "step_reps", "12");
    typed_into(&screen, "step_minutes", "0.5");
    found::<web_sys::HtmlElement>(&screen, "button.add-step").click();
    chosen_in(&screen, "step_exercise", "Burpee");
    typed_into(&screen, "step_minutes", "1");
    typed_into(&screen, "step_load", "5");
    found::<web_sys::HtmlElement>(&screen, "button.add-step").click();
    settled().await;
    let added: Element = found(&screen, "ol.steps");
    let steps = added.text_content().unwrap_or_default();
    assert!(
        steps.contains("Push up") && steps.contains("Burpee"),
        "{steps}"
    );
    found::<HtmlFormElement>(&screen, "form.circuit")
        .request_submit()
        .expect("the circuit is crafted");
    settled().await;

    let listed: Element = found(&screen, "table.circuits tbody");
    let rows = listed.text_content().unwrap_or_default();
    assert!(
        rows.contains("Morning rust") && rows.contains("Push up") && rows.contains("Burpee"),
        "{rows}"
    );
    assert_eq!(value_of(&screen, "circuit_name"), "");
}

#[wasm_bindgen_test]
async fn a_step_added_by_mistake_leaves_the_circuit_before_it_is_crafted() {
    answering(
        r#"{ "exercises": [{ "name": "Burpee", "met": 8.0 }, { "name": "Push up", "met": 3.8 }], "circuits": [] }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <CircuitCrafter />
            </Router>
        }
    });
    settled().await;
    typed_into(&screen, "step_minutes", "1");
    found::<web_sys::HtmlElement>(&screen, "button.add-step").click();
    settled().await;

    found::<web_sys::HtmlElement>(&screen, "ol.steps button.remove").click();
    settled().await;

    let added: Element = found(&screen, "ol.steps");
    assert_eq!(added.text_content().unwrap_or_default(), "");
}

#[wasm_bindgen_test]
async fn removing_a_circuit_takes_it_out_of_the_lab() {
    answering(&format!(
        r#"{{ "exercises": [], "circuits": [{MORNING_RUST}], "remove_circuit": [] }}"#
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <CircuitCrafter />
            </Router>
        }
    });
    settled().await;

    found::<web_sys::HtmlElement>(&screen, "table.circuits button.remove").click();
    settled().await;

    let listed: Element = found(&screen, "table.circuits tbody");
    assert_eq!(listed.text_content().unwrap_or_default(), "");
}

const A_DAY_OF_MORNING_RUST: &str = r#"{ "day": "2026-10-10", "burnt_kcal": 120.0, "off_the_clock": null,
    "off_the_clock_is_usual": true, "workouts": [
    { "id": 1, "burnt_kcal": 40.0, "workout": { "day": "2026-10-10", "exercise": "Push up", "met": 3.8,
        "minutes": 0.5, "reps": 12, "sets": 3, "load_kg": 0.0, "circuit": "Morning rust" } },
    { "id": 2, "burnt_kcal": 80.0, "workout": { "day": "2026-10-10", "exercise": "Burpee", "met": 8.0,
        "minutes": 1.0, "reps": null, "sets": 3, "load_kg": 5.0, "circuit": "Morning rust" } } ] }"#;

#[wasm_bindgen_test]
async fn logging_a_circuit_hands_over_the_days_burn_with_a_workout_per_step() {
    answering(&format!(
        r#"{{ "circuits": [{MORNING_RUST}], "log_circuit": {A_DAY_OF_MORNING_RUST} }}"#
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        let day = RwSignal::new("2026-10-10".to_string());
        let handed_over = RwSignal::new(None::<Burn>);
        view! {
            <Router>
                <CircuitLog
                    day=day
                    on_logged=Callback::new(move |burn: Burn| handed_over.set(Some(burn)))
                />
                <p class="handed-over">
                    {move || handed_over.get().map(|burn| burn.workouts.len().to_string())}
                </p>
            </Router>
        }
    });
    settled().await;
    assert_eq!(value_of(&screen, "circuit_sets"), "1");

    typed_into(&screen, "circuit_sets", "3");
    found::<HtmlFormElement>(&screen, "form.circuit-log")
        .request_submit()
        .expect("the circuit is logged");
    settled().await;

    let handed_over: Element = found(&screen, "p.handed-over");
    assert_eq!(handed_over.text_content().unwrap_or_default(), "2");
}

const THE_CATALOG: &str = r#"[{ "name": "Expresso", "volume_ml": 80.0, "kind": "Other" }, { "name": "Tea", "volume_ml": 200.0, "kind": "Other" }]"#;

const A_DAY_WITH_TWO_DRINKS: &str = r#"{ "day": "2026-10-10",
    "drinkings": [[1, { "day": "2026-10-10", "drink": "Expresso", "kind": "Other", "servings": 1.0, "volume_ml": 80.0 }],
                  [2, { "day": "2026-10-10", "drink": "Tea", "kind": "Other", "servings": 1.0, "volume_ml": 200.0 }]],
    "drunk_ml": 280.0, "target_ml": 2252.0, "floor_ml": 2026.8, "cap_ml": 2477.2, "verdict": "Low" }"#;

const A_DRY_DAY: &str = r#"{ "day": "2026-10-11", "drinkings": [], "drunk_ml": 0.0, "target_ml": 2000.0, "floor_ml": 1800.0, "cap_ml": 2200.0, "verdict": "Low" }"#;

fn the_day_of(screen: &Element, day: RwSignal<String>) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), move || {
        view! {
            <Router>
                <HydrateDay day=day />
            </Router>
        }
    })
}

#[wasm_bindgen_test]
async fn the_days_drinks_add_up_against_its_target() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "hydrate_day": {A_DAY_WITH_TWO_DRINKS} }}"#
    ));
    let screen = screen();

    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-10".to_string()));
    settled().await;

    let day = screen.text_content().unwrap_or_default();
    assert!(day.contains("280 mL of 2252 mL"), "{day}");
    let listed: Element = found(&screen, "tbody");
    assert!(
        listed
            .text_content()
            .is_some_and(|drunk| drunk.contains("Expresso") && drunk.contains("200 mL"))
    );
}

#[wasm_bindgen_test]
async fn the_days_hydrate_reloads_when_the_day_changes() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG},
              "hydrate_day": {{ "__by": "date", "2026-10-10": {A_DAY_WITH_TWO_DRINKS}, "2026-10-11": {A_DRY_DAY} }} }}"#
    ));
    let screen = screen();
    let day = RwSignal::new("2026-10-10".to_string());
    let _mounted = the_day_of(&screen, day);
    settled().await;

    day.set("2026-10-11".into());
    settled().await;

    let shown = screen.text_content().unwrap_or_default();
    assert!(shown.contains("0 mL of 2000 mL"), "{shown}");
}

#[wasm_bindgen_test]
async fn a_logged_drink_joins_the_day_and_the_servings_go_back_to_one() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "hydrate_day": {A_DRY_DAY},
              "log_drink": {{ "day": "2026-10-11", "drinkings": [[1, {{ "day": "2026-10-11", "drink": "Tea", "kind": "Other", "servings": 1.5, "volume_ml": 300.0 }}]],
                              "drunk_ml": 300.0, "target_ml": 2000.0, "floor_ml": 1800.0, "cap_ml": 2200.0, "verdict": "Low" }} }}"#
    ));
    let screen = screen();
    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-11".to_string()));
    settled().await;
    assert_eq!(value_of(&screen, "servings"), "1");

    typed_into(&screen, "servings", "1,5");
    found::<HtmlFormElement>(&screen, "form.drinking")
        .request_submit()
        .expect("the drink is logged");
    settled().await;

    let listed: Element = found(&screen, "tbody");
    assert!(
        listed
            .text_content()
            .is_some_and(|drunk| drunk.contains("Tea") && drunk.contains("300 mL"))
    );
    assert_eq!(value_of(&screen, "servings"), "1");
}

#[wasm_bindgen_test]
async fn removing_a_drinking_takes_it_off_the_day() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "hydrate_day": {A_DAY_WITH_TWO_DRINKS}, "remove_drinking": {A_DRY_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-10".to_string()));
    settled().await;

    found::<web_sys::HtmlElement>(&screen, "button.remove").click();
    settled().await;

    let listed: Element = found(&screen, "tbody");
    assert_eq!(listed.text_content().unwrap_or_default(), "");
}

fn the_catalog_in(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <DrinksCatalog />
            </Router>
        }
    })
}

#[wasm_bindgen_test]
async fn an_added_drink_joins_the_catalog_and_the_form_empties() {
    answering(
        r#"{ "drinks": [], "add_drink": [{ "name": "Tea", "volume_ml": 200.0, "kind": "Other" }] }"#,
    );
    let screen = screen();
    let _mounted = the_catalog_in(&screen);
    settled().await;

    typed_into(&screen, "drink_name", "Tea");
    typed_into(&screen, "serving", "200");
    found::<HtmlFormElement>(&screen, "form.drink")
        .request_submit()
        .expect("the drink is added");
    settled().await;

    let listed: Element = found(&screen, "tbody");
    assert!(
        listed
            .text_content()
            .is_some_and(|drinks| drinks.contains("Tea") && drinks.contains("200 mL"))
    );
    assert_eq!(
        [
            value_of(&screen, "drink_name"),
            value_of(&screen, "serving")
        ],
        ["", ""]
    );
}

#[wasm_bindgen_test]
async fn removing_a_drink_takes_it_off_the_catalog() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "remove_drink": [{{ "name": "Tea", "volume_ml": 200.0, "kind": "Other" }}] }}"#
    ));
    let screen = screen();
    let _mounted = the_catalog_in(&screen);
    settled().await;

    found::<web_sys::HtmlElement>(&screen, "button.remove").click();
    settled().await;

    let listed: Element = found(&screen, "tbody");
    assert!(
        listed
            .text_content()
            .is_some_and(|drinks| !drinks.contains("Expresso") && drinks.contains("Tea"))
    );
}

fn the_base_card_in(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <HydrateBaseCard />
            </Router>
        }
    })
}

#[wasm_bindgen_test]
async fn with_no_base_set_the_card_fills_in_the_efsa_default_and_points_to_its_references() {
    answering(r#"{ "hydrate_base_on": { "base_ml": 2000.0, "is_default": true } }"#);
    let screen = screen();
    let _mounted = the_base_card_in(&screen);
    settled().await;

    assert_eq!(value_of(&screen, "hydrate_base"), "2000");
    let references: Element = found(&screen, "a.references");
    assert_eq!(
        references.get_attribute("href").as_deref(),
        Some("/references#water")
    );
    let card = screen.text_content().unwrap_or_default();
    assert!(card.contains("EFSA's default holds: 2000 mL"), "{card}");
}

#[wasm_bindgen_test]
async fn a_saved_base_says_from_which_day_it_is_kept_and_the_default_steps_aside() {
    answering(
        r#"{ "hydrate_base_on": { "base_ml": 2000.0, "is_default": true }, "set_hydrate_base": 2400.0 }"#,
    );
    let screen = screen();
    let _mounted = the_base_card_in(&screen);
    settled().await;

    typed_into(&screen, "hydrate_base", "2400");
    found::<HtmlFormElement>(&screen, "form.hydrate-base")
        .request_submit()
        .expect("the base is saved");
    settled().await;

    let said: Element = found(&screen, "form.hydrate-base .said");
    assert!(
        said.text_content()
            .is_some_and(|line| line.contains("kept from"))
    );
    assert!(
        screen.query_selector("a.source").ok().flatten().is_none(),
        "a base set is no default"
    );
}

#[wasm_bindgen_test]
async fn the_days_hydrate_shows_its_scale_from_min_through_optimal_to_max() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "hydrate_day": {A_DRY_DAY} }}"#
    ));
    let screen = screen();

    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-11".to_string()));
    settled().await;

    let scale: Element = found(&screen, ".gauge");
    let labels = scale.text_content().unwrap_or_default();
    assert!(
        labels.contains("min 1800 mL")
            && labels.contains("optimal 2000 mL")
            && labels.contains("max 2200 mL"),
        "{labels}"
    );
    assert!(scale.query_selector("svg .band").ok().flatten().is_some());
    assert!(scale.query_selector("svg .amount").ok().flatten().is_some());
}

fn a_streak_of(days: u32) -> String {
    let first = chrono::NaiveDate::from_ymd_opt(2026, 10, 10).expect("a real day")
        - chrono::Days::new(u64::from(days) - 1);
    let history: Vec<String> = (0..days)
        .map(|offset| {
            format!(
                r#"{{ "day": "{}", "status": "Success", "ticks": {{ "weighed": true, "fuelled": true, "worked_out": true, "hydrated": true }} }}"#,
                first + chrono::Days::new(u64::from(offset))
            )
        })
        .collect();
    format!(
        r#"{{ "days": {days}, "at_risk": false, "history": [{}] }}"#,
        history.join(", ")
    )
}

#[wasm_bindgen_test]
async fn a_long_streak_shows_its_latest_weeks_and_loads_the_earlier_ones() {
    answering(&format!(
        r#"{{ "profile_on": {{ "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null }},
             "gather_card": {{ "masses": [], "trend": [], "target_kg": null, "progress": null, "fat_change_kg": null, "lean_change_kg": null }},
             "streak": {} }}"#,
        a_streak_of(60)
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), App);
    settled().await;
    settled().await;
    let weeks_shown = |screen: &Element| {
        screen
            .query_selector_all(".streak .week")
            .map(|found| found.length())
            .unwrap_or_default()
    };
    assert_eq!(weeks_shown(&screen), 5);

    found::<web_sys::HtmlElement>(&screen, ".streak button.more").click();
    settled().await;

    assert_eq!(weeks_shown(&screen), 9);
    assert!(
        screen
            .query_selector(".streak .legend")
            .ok()
            .flatten()
            .is_some()
    );
}

const THE_CHARTS: &str = r#""gather_card": { "masses": [], "trend": [], "target_kg": 70.0, "progress": null, "fat_change_kg": null, "lean_change_kg": null },
    "gather_chart": { "masses": [["2026-10-09", 80.2], ["2026-10-10", 80.0]], "lean_masses": [["2026-10-09", 70.9], ["2026-10-10", 70.7]],
                      "goal_kg": 70.0, "goal_lean_kg": 63.0 },
    "fuel_chart": { "amounts": [["2026-10-09", 2390.0], ["2026-10-10", 1900.0]], "floors": [["2026-10-09", 2199.0], ["2026-10-10", 2180.0]],
                    "caps": [["2026-10-09", 2650.0], ["2026-10-10", 2630.0]] },
    "hydrate_chart": { "amounts": [["2026-10-09", 1500.0], ["2026-10-10", 2100.0]], "floors": [["2026-10-09", 1800.0], ["2026-10-10", 1800.0]],
                       "caps": [["2026-10-09", 2200.0], ["2026-10-10", 2200.0]] },
    "burn_chart": { "burnt": [["2026-10-09", 120.0], ["2026-10-10", 252.0]], "reps": [["2026-10-09", 36.0], ["2026-10-10", 0.0]],
                    "minutes": [["2026-10-09", 6.0], ["2026-10-10", 60.0]] },
    "fuel_month": { "month": "2026-10", "earlier": null, "rows": [] },
    "hydrate_month": { "month": "2026-10", "earlier": null, "rows": [] },
    "burn_month": { "month": "2026-10", "earlier": null, "rows": [] }"#;

fn the_charts(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Charts today="2026-10-10".to_string() />
            </Router>
        }
    })
}

fn lines_drawn(screen: &Element) -> u32 {
    screen
        .query_selector_all(".charts svg polyline")
        .map(|found| found.length())
        .unwrap_or_default()
}

#[wasm_bindgen_test]
async fn the_home_chart_draws_gather_with_its_goals_then_each_chip_its_own_lines_and_legend() {
    answering(&format!(
        r#"{{ {THE_CHARTS}, "gather_month": {{ "month": "2026-10", "earlier": null, "rows": [] }} }}"#
    ));
    let screen = screen();
    let _mounted = the_charts(&screen);
    settled().await;

    assert_eq!(lines_drawn(&screen), 4);
    let labels = screen.text_content().unwrap_or_default();
    assert!(
        labels.contains("80.0 kg") && labels.contains("70.0 kg") && labels.contains("Lean mass"),
        "{labels}"
    );

    for (chip, lines, said) in [
        ("Fuel", 3, "1900 kcal"),
        ("Hydrate", 3, "2100 mL"),
        ("Burn", 3, "60 min"),
    ] {
        found::<web_sys::HtmlElement>(&screen, &format!("button.chip-{chip}")).click();
        settled().await;

        assert_eq!(lines_drawn(&screen), lines, "{chip}");
        let labels = screen.text_content().unwrap_or_default();
        assert!(labels.contains(said), "{chip}: {labels}");
    }
}

#[wasm_bindgen_test]
async fn the_history_beneath_the_chart_loads_the_month_before_on_demand() {
    answering(&format!(
        r#"{{ {THE_CHARTS}, "gather_month": {{ "__by": "month",
            "2026-10": {{ "month": "2026-10", "earlier": "2026-09", "rows": [
                {{ "day": "2026-10-10", "body_mass_kg": 80.0, "lean_mass_kg": 70.7, "fat_mass_kg": 9.3, "fat_ratio": 0.116 }},
                {{ "day": "2026-10-09", "body_mass_kg": 80.2, "lean_mass_kg": 70.9, "fat_mass_kg": 9.3, "fat_ratio": 0.116 }} ] }},
            "2026-09": {{ "month": "2026-09", "earlier": null, "rows": [
                {{ "day": "2026-09-30", "body_mass_kg": 80.4, "lean_mass_kg": null, "fat_mass_kg": null, "fat_ratio": null }} ] }} }} }}"#
    ));
    let screen = screen();
    let _mounted = the_charts(&screen);
    settled().await;
    let rows_listed = |screen: &Element| {
        screen
            .query_selector_all(".history tbody tr")
            .map(|found| found.length())
            .unwrap_or_default()
    };
    assert_eq!(rows_listed(&screen), 2);

    found::<web_sys::HtmlElement>(&screen, ".history button.more").click();
    settled().await;

    assert_eq!(rows_listed(&screen), 3);
    assert!(
        screen
            .query_selector(".history button.more")
            .ok()
            .flatten()
            .is_none()
    );
}

#[wasm_bindgen_test]
async fn stats_list_the_habits_to_sharpen_the_weakest_first() {
    answering(
        r#"{ "explore": { "one": [], "other": [], "one_summary": null, "other_summary": null, "pairs": [], "correlation": null },
             "insights": { "days": 30, "weighed_days": 25, "fuelled_days": 20,
                 "energy": { "low": 4, "in_band": 14, "high": 2 }, "protein": { "low": 2, "in_band": 18, "high": 0 },
                 "fat": { "low": 1, "in_band": 17, "high": 2 }, "fibre": { "low": 15, "in_band": 5, "high": 0 },
                 "carbs": { "low": 3, "in_band": 16, "high": 1 }, "hydrated_days": 10,
                 "hydrate": { "low": 6, "in_band": 4, "high": 0 }, "active_days": 12, "burnt_per_active_day": 210.0 } }"#,
    );
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Stats />
            </Router>
        }
    });
    settled().await;

    let first: Element = found(&screen, "ol.sharpen li");
    let weakest = first.text_content().unwrap_or_default();
    assert!(
        weakest.contains("Fibre") && weakest.contains("5 of 20"),
        "{weakest}"
    );
    let said = screen.text_content().unwrap_or_default();
    assert!(
        said.contains("12 of 30 days") && said.contains("25 of 30 days"),
        "{said}"
    );
}

#[wasm_bindgen_test]
async fn a_figure_that_is_no_number_is_caught_on_the_screen_before_reaching_the_app() {
    answering(r#"{ "recent_days": [] }"#);
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Tracker />
            </Router>
        }
    });
    settled().await;

    typed_into(&screen, "mass", "eighty");
    found::<HtmlFormElement>(&screen, "form.measurement")
        .request_submit()
        .expect("the form submits");
    settled().await;

    let said = screen.text_content().unwrap_or_default();
    assert!(said.contains("That is no number I can read"), "{said}");
}

#[wasm_bindgen_test]
async fn the_danger_zone_erases_everything_once_erase_is_typed() {
    answering(r#"{ "erase_everything": null }"#);
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Settings />
            </Router>
        }
    });
    settled().await;
    let erase: web_sys::HtmlButtonElement = found(&screen, "button.erase");
    assert!(erase.disabled());

    typed_into(&screen, "confirm_erase", "ERASE");
    settled().await;
    assert!(!erase.disabled());
    erase.click();
    settled().await;

    let said = screen.text_content().unwrap_or_default();
    assert!(said.contains("Everything is erased"), "{said}");
}

const RECOMMENDED: &str = r#"{ "healthy_mass_kg": [59.94, 80.676], "goal_mass_kg": 70.308,
    "healthy_fat_ratio": [0.11, 0.22], "goal_fat_ratio": 0.165,
    "fuel": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.8, "fibre_g_per_kg": 0.35 },
    "hydrate_base_ml": 2000.0, "usual_day": { "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 6.0 } }"#;

#[wasm_bindgen_test]
async fn the_recommendation_fills_each_card_on_demand() {
    answering(&format!(
        r#"{{ "profile_on": {{ "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null }},
             "goals_on": null, "fuel_targets_on": {{ "protein_g_per_kg": 2.025, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }},
             "usual_day_on": null, "hydrate_base_on": {{ "base_ml": 2000.0, "is_default": true }},
             "recommended_on": {RECOMMENDED} }}"#
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <ProfileScreen />
            </Router>
        }
    });
    settled().await;

    for card in [".goals", ".fuel-targets", ".hydrate-base", ".usual-day"] {
        found::<web_sys::HtmlElement>(&screen, &format!("{card} button.recommend")).click();
    }
    settled().await;

    assert_eq!(
        [
            value_of(&screen, "target_mass"),
            value_of(&screen, "target_fat"),
            value_of(&screen, "target_protein"),
            value_of(&screen, "usual_sitting"),
            value_of(&screen, "usual_moving"),
        ],
        ["70.31", "16.5", "1.6", "480", "6"]
    );
    let said = screen.text_content().unwrap_or_default();
    assert!(
        said.contains("59.9 kg to 80.7 kg") && said.contains("11.0% to 22.0%"),
        "{said}"
    );
    assert!(said.contains("not medical advice"), "{said}");
}

#[wasm_bindgen_test]
async fn the_scientific_references_cite_each_source_under_its_topic_with_its_public_page() {
    answering(r#"{ "plugin:opener|open_url": null }"#);
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <ReferencesScreen />
            </Router>
        }
    });
    settled().await;

    let topics = screen
        .query_selector_all("section.topic")
        .map(|found| found.length())
        .unwrap_or_default();
    let pages = screen
        .query_selector_all("ol.bibliography a.page")
        .map(|found| found.length())
        .unwrap_or_default();
    let ranges: Element = found(&screen, "section#healthy-ranges");
    let cited = ranges.text_content().unwrap_or_default();
    assert_eq!((topics, pages), (6, 21));
    assert!(
        cited.contains("Gallagher D.")
            && cited.contains("American Journal of Clinical Nutrition 72(3):694-701"),
        "{cited}"
    );
}

#[wasm_bindgen_test]
async fn each_profile_card_points_to_its_scientific_references() {
    answering(
        r#"{ "profile_on": null, "goals_on": null, "usual_day_on": null,
             "fuel_targets_on": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 },
             "hydrate_base_on": { "base_ml": 2000.0, "is_default": true } }"#,
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

    let pointed: Vec<String> = ["healthy-ranges", "macronutrients", "water", "activity"]
        .into_iter()
        .filter(|topic| {
            screen
                .query_selector(&format!("a.references[href='/references#{topic}']"))
                .ok()
                .flatten()
                .is_some()
        })
        .map(String::from)
        .collect();
    assert_eq!(pointed.len(), 4, "{pointed:?}");
    assert!(screen.query_selector("ul.sources").ok().flatten().is_none());
}
