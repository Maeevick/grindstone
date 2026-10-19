#![cfg(target_arch = "wasm32")]

use grind_core::burn::Burn;
use grind_ui::app::App;
use grind_ui::screens::burn::BurnDay;
use grind_ui::screens::catalog::{BurnShelf, FuelShelf, HydrateShelf};
use grind_ui::screens::charts::Charts;
use grind_ui::screens::circuits::CircuitLog;
use grind_ui::screens::fuel::FuelDay;
use grind_ui::screens::gather::GatherTab;
use grind_ui::screens::hydrate::{HydrateCountedCard, HydrateDay};
use grind_ui::screens::meals::MealInProgress;
use grind_ui::screens::profile::ProfileScreen;
use grind_ui::screens::references::ReferencesScreen;
use grind_ui::screens::settings::Settings;
use grind_ui::screens::stats::Stats;
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
export function back_home() {
    window.history.replaceState(null, '', '/');
}
export function went_to(path) {
    window.history.pushState(null, '', path);
    window.dispatchEvent(new PopStateEvent('popstate', { state: null }));
}
")]
extern "C" {
    fn answer_commands_with(answers: &JsValue);
    fn asked(command: &str) -> String;
    fn a_moment() -> js_sys::Promise;
    fn back_home();
    /// Moves to another tab through the history: a click on a tab is cancelled by the link
    /// handlers the routers of earlier tests leave behind.
    fn went_to(path: &str);
}

fn answering(answers: &str) {
    answer_commands_with(&js_sys::JSON::parse(answers).expect("answers in JSON"));
}

/// Several sets of answers as one, the later answering a command the earlier also answer.
fn answering_all(parts: &[&str]) {
    let all = js_sys::Object::new();
    for part in parts {
        let answers = js_sys::JSON::parse(part).expect("answers in JSON");
        js_sys::Object::assign(&all, &answers.unchecked_into());
    }
    answer_commands_with(&all);
}

fn attributes_of(screen: &Element, selector: &str, attribute: &str) -> Vec<String> {
    let found = screen.query_selector_all(selector).expect(selector);
    (0..found.length())
        .filter_map(|at| found.item(at))
        .filter_map(|node| node.unchecked_into::<Element>().get_attribute(attribute))
        .collect()
}

fn texts_in_order(screen: &Element, selector: &str) -> Vec<String> {
    let found = screen.query_selector_all(selector).expect(selector);
    (0..found.length())
        .filter_map(|at| found.item(at))
        .filter_map(|node| node.text_content())
        .collect()
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

/// The whole app, from Home whatever page an earlier test left the browser on.
fn the_app(screen: &Element) -> impl Drop {
    back_home();
    leptos::mount::mount_to(screen.clone().unchecked_into(), App)
}

async fn settled() {
    JsFuture::from(a_moment()).await.expect("a moment passes");
}

#[wasm_bindgen_test]
async fn a_first_launch_welcomes_you_and_points_to_the_profile_and_the_draft_import() {
    answering(r#"{ "profile_on": null }"#);
    let screen = screen();

    let _mounted = the_app(&screen);
    settled().await;

    let welcome = screen.text_content().unwrap_or_default();
    assert!(welcome.contains("Greetings, rusty one"), "{welcome}");
    let to_the_profile: Element = found(&screen, ".welcome a[href='/profile']");
    let to_the_import: Element = found(&screen, ".welcome a[href='/profile#draft']");
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
async fn each_profile_card_gives_its_day_a_row_and_shares_the_next_between_its_figures() {
    answering(
        r#"{ "profile_on": null, "goals_on": null, "usual_day_on": null,
             "fuel_targets_on": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 },
             "hydrate_counted_on": true }"#,
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

    assert_eq!(
        fields_in_rows(&screen, "form.goals"),
        [vec!["goals_from"], vec!["target_mass", "target_fat"]]
    );
    assert_eq!(
        fields_in_rows(&screen, "form.fuel-targets"),
        [
            vec!["targets_from"],
            vec!["target_protein", "target_fat_g", "target_fibre"]
        ]
    );
    assert_eq!(
        fields_in_rows(&screen, "form.usual-day"),
        [
            vec!["usual_from"],
            vec!["usual_sitting", "usual_standing", "usual_moving"]
        ]
    );
}

#[wasm_bindgen_test]
async fn the_profile_ends_on_the_settings_the_drafts_records_then_the_danger_zone() {
    answering(
        r#"{ "profile_on": null, "goals_on": null, "usual_day_on": null,
             "fuel_targets_on": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 },
             "hydrate_counted_on": true, "burn_target_on": null }"#,
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

    let headings = texts_in_order(&screen, "h2");
    assert_eq!(
        headings[headings.len().saturating_sub(3)..],
        ["Settings", "The draft's records", "Danger zone"],
        "{headings:?}"
    );
    found::<Element>(&screen, "section#draft button.import");
    assert!(screen.query_selector(".gear").ok().flatten().is_none());
}

#[wasm_bindgen_test]
async fn the_hip_stays_in_its_place_open_to_a_woman_only() {
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
    let hip_closed =
        |screen: &Element| found::<HtmlInputElement>(screen, "input[name=hip]").disabled();
    assert!(hip_closed(&screen), "a man's profile asks for no hip");

    let gender: HtmlSelectElement = found(&screen, "select[name=gender]");
    gender.set_value("Female");
    gender
        .dispatch_event(&Event::new("change").expect("a change event"))
        .expect("the gender changes");
    settled().await;

    assert!(!hip_closed(&screen), "a woman's profile asks for her hip");
}

#[wasm_bindgen_test]
async fn each_card_offers_its_recommendation_above_its_keeping_and_every_card_its_references_below()
{
    answering(
        r#"{ "profile_on": null, "goals_on": null, "usual_day_on": null,
             "fuel_targets_on": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 },
             "hydrate_counted_on": true }"#,
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

    for card in [".goals", ".fuel-targets", ".usual-day", ".burn-target"] {
        found::<Element>(
            &screen,
            &format!("{card} button.recommend ~ button[type=submit]"),
        );
    }
    for card in [
        ".goals",
        ".fuel-targets",
        ".hydrate-counted",
        ".usual-day",
        ".burn-target",
    ] {
        found::<Element>(
            &screen,
            &format!("{card} button[type=submit] ~ a.references"),
        );
    }
    assert!(screen.query_selector(".actions").ok().flatten().is_none());
    let disclaimer: Element = found(&screen, ".disclaimer");
    assert_eq!(
        disclaimer
            .next_element_sibling()
            .and_then(|next| next.text_content())
            .as_deref(),
        Some("Settings"),
        "the disclaimer closes the profile's own cards"
    );
}

fn written_into(screen: &Element, name: &str, text: &str) {
    let area: HtmlInputElement = found(screen, &format!("textarea[name={name}]"));
    area.set_value(text);
    area.dispatch_event(&Event::new("input").expect("an input event"))
        .expect("the area is written into");
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

#[wasm_bindgen_test]
async fn the_gathered_day_stands_on_a_band_for_its_body_mass_and_one_for_its_lean_mass() {
    answering(&format!(
        r#"{{ "gather_week": {{ "day": {A_GATHERED_DAY}, "days": [{A_GATHERED_DAY}],
              "tendency": {{ "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null }},
              "aims": {{ "body_mass_kg": 75.0, "lean_mass_kg": 66.0, "recommended": false }} }} }}"#
    ));
    let screen = screen();
    let _mounted = the_gather_tab(&screen);
    settled().await;

    let first = text_of(&screen, ".aim-gauge");
    let bands = texts_of(&screen, ".aim-gauge .gauge-labels");

    assert!(first.starts_with("Body mass: 80.0 kg"), "{first}");
    assert_eq!(bands.len(), 2, "{bands:?}");
    for aim in ["goal 75.0 kg", "goal 66.0 kg"] {
        assert!(
            bands.iter().any(|band| band.contains(aim)),
            "{aim} in {bands:?}"
        );
    }
}

#[wasm_bindgen_test]
async fn the_weeks_pace_breaks_down_each_measure_and_judges_the_body_mass() {
    answering(&format!(
        r#"{{ "gather_week": {{ "day": {A_GATHERED_DAY}, "days": [{A_GATHERED_DAY}],
              "tendency": {{ "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null }},
              "aims": {{ "body_mass_kg": 75.0, "lean_mass_kg": 66.0, "recommended": false }},
              "pace": {{ "body_mass": {{ "by": -0.4, "heading": "Toward" }},
                         "lean_mass": {{ "by": -0.1, "heading": "Away" }},
                         "fat_mass": {{ "by": -0.3, "heading": "Toward" }},
                         "fat_ratio": {{ "by": 0.0, "heading": "Steady" }},
                         "judgement": "VeryGood", "safe_loss_kg": 0.48 }} }} }}"#
    ));
    let screen = screen();
    let _mounted = the_gather_tab(&screen);
    settled().await;

    let pace = text_of(&screen, ".pace");

    for measure in ["Body mass", "Lean mass", "Fat mass", "Fat"] {
        assert!(pace.contains(measure), "{measure} in {pace}");
    }
    assert!(pace.contains("-0.4 kg a week"), "{pace}");
    assert!(pace.contains("inside the safe lane"), "{pace}");
    found::<Element>(&screen, ".pace li.toward");
    found::<Element>(&screen, ".pace li.away");
}

const A_WEEK_NOT_GATHERED_YET: &str = r#"{ "day": null, "days": [],
    "tendency": { "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null } }"#;

fn the_gather_tab(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <GatherTab />
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
    let _mounted = the_gather_tab(&screen);
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
    let _mounted = the_gather_tab(&screen);
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
    let _mounted = the_gather_tab(&screen);
    settled().await;

    typed_into(&screen, "day", "2026-10-10");
    settled().await;

    let text = |selector: &str| {
        found::<Element>(&screen, selector)
            .text_content()
            .unwrap_or_default()
    };
    assert!(
        text(".last-days thead").contains("Mass (kg)")
            && text(".last-days thead").contains("Fat (%)"),
        "{}",
        text(".last-days thead")
    );
    assert!(text(".last-days tr.chosen").contains("2026-10-10"));
    assert!(text(".last-days tbody").contains("80.6") && !text(".last-days tbody").contains(" kg"));
    assert_eq!(text(".last-days tfoot .toward"), "-0.6");
    assert_eq!(text(".last-days tfoot .steady"), "-");
    assert_eq!(text(".last-days tfoot .away"), "-0.3");
}

#[wasm_bindgen_test]
async fn after_logging_a_measurement_the_day_stays_and_the_form_shows_what_was_gathered() {
    answering(&format!(
        r#"{{ "gather_week": {A_WEEK_NOT_GATHERED_YET}, "log_measurement": {} }}"#,
        a_gathered_week()
    ));
    let screen = screen();
    let _mounted = the_gather_tab(&screen);
    settled().await;
    typed_into(&screen, "day", "2026-10-10");
    settled().await;

    typed_into(&screen, "mass", "80");
    typed_into(&screen, "waist", "80");
    found::<HtmlFormElement>(&screen, "form.measurement")
        .request_submit()
        .expect("the measurement is logged");
    settled().await;

    let listed: Element = found(&screen, ".last-days tbody");
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
    let _mounted = the_gather_tab(&screen);
    settled().await;
    typed_into(&screen, "day", "2026-10-10");
    settled().await;

    found::<web_sys::HtmlElement>(&screen, ".last-days button.remove").click();
    settled().await;

    let listed: Element = found(&screen, ".last-days tbody");
    assert_eq!(listed.text_content().unwrap_or_default(), "");
    assert!(asked("remove_measurement").contains("2026-10-10"));
}

#[wasm_bindgen_test]
async fn the_window_pills_start_on_seven_days_and_redraw_the_chart_over_the_days_they_name() {
    answering(&format!(
        r#"{{ {THE_CHARTS}, "gather_month": {{ "month": "2026-10", "earlier": null, "rows": [] }},
             "gather_chart": {{ "__by": "window",
                 "Days7": {{ "masses": [["2026-10-04", 80.4], ["2026-10-10", 80.0]], "lean_masses": [],
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
    found::<Element>(&screen, "button.window-Days7.chosen");
    assert_eq!(text_of(&screen, "button.window-Everything"), "∞");

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

fn the_burn_day(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        let day = RwSignal::new("2026-10-10".to_string());
        view! {
            <Router>
                <BurnDay day=day />
            </Router>
        }
    })
}

const A_BURN_PRICED_AT_80_KG: &str = r#"{ "day": "2026-10-10", "workouts": [], "burnt_kcal": 0.0,
    "off_the_clock": { "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 3.0 }, "off_the_clock_is_usual": true,
    "body_mass_kg": 80.0 }"#;

const A_BURN_SHORT_OF_ITS_TARGET: &str = r#"{ "day": "2026-10-10", "workouts": [], "burnt_kcal": 60.0,
    "off_the_clock": { "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 3.0 }, "off_the_clock_is_usual": true,
    "body_mass_kg": 80.0, "target_kcal": 80.0 }"#;

#[wasm_bindgen_test]
async fn the_days_burn_shows_its_band_from_nothing_the_target_tick_and_the_grace_in_green() {
    answering(&format!(
        r#"{{ "exercises": [], "circuits": [], "burn_day": {A_BURN_SHORT_OF_ITS_TARGET},
             "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    let band: Element = found(&screen, ".burn-gauge svg .band");
    let labels = text_of(&screen, ".burn-gauge .gauge-labels");

    assert_eq!(band.get_attribute("x").as_deref(), Some("136.8"));
    assert!(labels.contains("target 80 kcal"), "{labels}");
}

#[wasm_bindgen_test]
async fn the_workout_form_previews_what_the_workout_will_burn_on_the_days_body_mass() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Walking", "met": 4.0 }}], "circuits": [],
             "burn_day": {A_BURN_PRICED_AT_80_KG}, "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    typed_into(&screen, "exercise", "walking");
    settled().await;
    let left_blank = text_of(&screen, "form.workout p.preview");
    typed_into(&screen, "minutes", "60");
    typed_into(&screen, "sets", "2");
    typed_into(&screen, "load", "10");
    settled().await;

    assert_eq!(
        [left_blank, text_of(&screen, "form.workout p.preview")],
        ["4 kcal", "567 kcal"]
    );
}

#[wasm_bindgen_test]
async fn without_a_weigh_in_the_workout_form_previews_nothing() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Walking", "met": 4.0 }}], "circuits": [],
             "burn_day": {AN_EMPTY_BURN}, "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    typed_into(&screen, "exercise", "walking");
    typed_into(&screen, "minutes", "60");
    settled().await;

    assert!(screen.query_selector("p.preview").ok().flatten().is_none());
}

#[wasm_bindgen_test]
async fn the_circuit_log_previews_what_its_sets_will_burn() {
    answering(&format!(
        r#"{{ "exercises": [], "circuits": [{MORNING_RUST}],
             "burn_day": {A_BURN_PRICED_AT_80_KG}, "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;
    clicked(&screen, "button.burn-Circuit");
    settled().await;

    typed_into(&screen, "circuit", "morning rust");
    settled().await;
    let one_set = text_of(&screen, "form.circuit-log p.preview");
    typed_into(&screen, "circuit_sets", "3");
    settled().await;

    assert_eq!(
        [one_set, text_of(&screen, "form.circuit-log p.preview")],
        ["12 kcal", "37 kcal"]
    );
}

#[wasm_bindgen_test]
async fn logging_a_workout_lists_it_with_what_it_burnt() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Walking", "met": 4.0 }}], "circuits": [], "burn_day": {AN_EMPTY_BURN},
             "fuel_day": {A_DAY_NOT_FUELLED_YET},
             "log_workout": {{ "day": "2026-10-10", "burnt_kcal": 252.0, "off_the_clock": null, "off_the_clock_is_usual": true,
                 "workouts": [{{ "id": 1, "burnt_kcal": 252.0, "workout": {{ "day": "2026-10-10", "exercise": "Walking",
                     "met": 4.0, "minutes": 60.0, "reps": null, "sets": 1, "load_kg": 0.0, "circuit": null }} }}] }} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;
    assert_eq!(value_of(&screen, "sitting"), "480");

    typed_into(&screen, "exercise", "Walking");
    typed_into(&screen, "minutes", "60");
    found::<HtmlFormElement>(&screen, "form.workout")
        .request_submit()
        .expect("the workout is logged");
    settled().await;

    let rows = text_of(&screen, "table.expenditures tbody");
    assert!(rows.contains("Walking") && rows.contains("252"), "{rows}");
}

#[wasm_bindgen_test]
async fn the_burn_card_switches_between_a_workout_and_a_circuit() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Walking", "met": 4.0 }}], "circuits": [{MORNING_RUST}],
             "burn_day": {AN_EMPTY_BURN}, "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;
    let shown = |selector: &str| screen.query_selector(selector).ok().flatten().is_some();
    assert!(shown("form.workout") && !shown("form.circuit-log"));

    found::<web_sys::HtmlElement>(&screen, "button.burn-Circuit").click();
    settled().await;

    assert!(shown("form.circuit-log") && !shown("form.workout"));
}

#[wasm_bindgen_test]
async fn a_workout_left_blank_logs_a_minute_one_rep_one_set_and_no_load() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Push up", "met": 3.8 }}, {{ "name": "Walking", "met": 4.0 }}], "circuits": [],
             "burn_day": {AN_EMPTY_BURN}, "fuel_day": {A_DAY_NOT_FUELLED_YET}, "log_workout": {AN_EMPTY_BURN} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;
    let placeholder = |name: &str| {
        found::<HtmlInputElement>(&screen, &format!("input[name={name}]")).placeholder()
    };
    assert_eq!(
        ["exercise", "minutes", "reps", "sets", "load"].map(placeholder),
        ["Select me", "1", "1", "1", "0"]
    );

    typed_into(&screen, "exercise", "push up");
    found::<HtmlFormElement>(&screen, "form.workout")
        .request_submit()
        .expect("the workout is logged");
    settled().await;

    let sent = asked("log_workout");
    assert!(sent.contains(r#""exercise":"Push up""#), "{sent}");
    for left_blank in ["minutes", "reps", "sets", "load_kg"] {
        assert!(!sent.contains(left_blank), "{sent}");
    }
}

#[wasm_bindgen_test]
async fn the_days_expenditures_list_its_workouts_newest_first_then_off_the_clock_and_fidgeting() {
    answering(&format!(
        r#"{{ "exercises": [], "circuits": [], "burn_day": {A_DAY_OF_MORNING_RUST},
             "fuel_day": {A_FUELLED_DAY_WITH_ITS_BUDGET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    let heads = text_of(&screen, "table.expenditures thead");
    assert!(
        heads.contains("Activity")
            && heads.contains("kcal")
            && heads.contains("min")
            && heads.contains("reps"),
        "{heads}"
    );
    let rows = screen
        .query_selector_all("table.expenditures tbody tr")
        .expect("the rows");
    let row = |at: u32| {
        rows.item(at)
            .and_then(|row| row.text_content())
            .unwrap_or_default()
    };
    assert_eq!(rows.length(), 4);
    assert!(
        row(0).contains("Burpee") && row(0).contains("80") && row(0).contains('-'),
        "{}",
        row(0)
    );
    assert!(
        row(1).contains("Push up") && row(1).contains("1.5") && row(1).contains("36"),
        "{}",
        row(1)
    );
    assert!(
        row(2).contains("Off the clock") && row(2).contains("198"),
        "{}",
        row(2)
    );
    assert!(
        row(3).contains("Fidgeting") && row(3).contains("234"),
        "{}",
        row(3)
    );
}

fn the_fuel_shelf(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        provide_context(MealInProgress::default());
        view! {
            <Router>
                <FuelShelf />
            </Router>
        }
    })
}

fn the_hydrate_shelf(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <HydrateShelf />
            </Router>
        }
    })
}

fn the_burn_shelf(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <BurnShelf />
            </Router>
        }
    })
}

fn clicked(screen: &Element, selector: &str) {
    found::<web_sys::HtmlElement>(screen, selector).click();
}

fn fields_in_rows(screen: &Element, form: &str) -> Vec<Vec<String>> {
    let rows = screen
        .query_selector_all(&format!("{form} .row"))
        .expect("the rows");
    (0..rows.length())
        .filter_map(|at| rows.item(at))
        .map(|row| {
            let fields = row
                .unchecked_into::<Element>()
                .query_selector_all("input, select")
                .expect("the fields");
            (0..fields.length())
                .filter_map(|at| fields.item(at))
                .filter_map(|field| field.unchecked_into::<Element>().get_attribute("name"))
                .collect()
        })
        .collect()
}

fn tables_in_order(screen: &Element) -> Vec<String> {
    let tables = screen.query_selector_all("table").expect("the tables");
    (0..tables.length())
        .filter_map(|at| tables.item(at))
        .filter_map(|table| table.unchecked_into::<Element>().get_attribute("class"))
        .collect()
}

#[wasm_bindgen_test]
async fn the_burn_shelf_adds_an_exercise_from_its_form_card_its_chip_showing_the_exercises_alone() {
    answering(
        r#"{ "exercises": [], "circuits": [], "add_exercise": [{ "name": "Kettlebell swing", "met": 9.8 }] }"#,
    );
    let screen = screen();
    let _mounted = the_burn_shelf(&screen);
    settled().await;
    assert_eq!(tables_in_order(&screen), ["exercises"]);
    found::<Element>(&screen, "button.form-Exercise.chosen");
    found::<Element>(&screen, "button.form-Circuit");

    typed_into(&screen, "exercise_name", "Kettlebell swing");
    typed_into(&screen, "exercise_met", "9.8");
    found::<HtmlFormElement>(&screen, "form.exercise")
        .request_submit()
        .expect("the exercise is added");
    settled().await;

    let rows = text_of(&screen, "table.exercises tbody");
    assert!(
        rows.contains("Kettlebell swing") && rows.contains("9.8"),
        "{rows}"
    );
    assert_eq!(value_of(&screen, "exercise_name"), "");
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

const A_FUELLED_DAY_WITH_ITS_BUDGET: &str = r#"{ "day": "2026-10-10", "intakes": [],
    "eaten": { "energy_kcal": 0.0, "protein_g": 0.0, "carbs_g": 0.0, "fat_g": 0.0, "fibre_g": 0.0 },
    "band": { "minimum_kcal_per_kg_of_lean_mass": 25.0, "minimum_kcal": 1767.5, "floor_kcal": 2199.4, "cap_kcal": 2650.2 },
    "energy": "Low", "macros": null,
    "budget": { "resting_kcal": 1718.1, "organs_kcal": 102.5, "digestion_kcal": 275.8,
                "spontaneous_kcal": 234.0, "volitional_kcal": 197.52 } }"#;

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
        sent.contains(r#""food":"Skyr""#) && !sent.contains(r#""quantity""#),
        "{sent}"
    );
    assert!(text_of(&screen, "table.intakes tbody").contains("Skyr"));
}

const KEFIR: &str = r#"{ "name": "Kefir", "unit": "Gram", "portion": 100.0, "description": "Plain, from the farm",
    "nutrients": { "energy_kcal": 64.0, "protein_g": 3.4, "carbs_g": 4.1, "fat_g": 3.6, "fibre_g": 0.0 } }"#;

#[wasm_bindgen_test]
async fn a_food_nothing_matches_turns_the_fuel_form_into_a_new_one_added_then_logged_in_one_tap() {
    answering(&format!(
        r#"{{ "foods": {THREE_FOODS}, "fuel_day": {A_DAY_NOT_FUELLED_YET},
              "add_food": [{KEFIR}], "log_intake": {A_FUELLED_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    typed_into(&screen, "food", "Kefir");
    settled().await;
    assert_eq!(texts_in_order(&screen, "form.intake .new"), ["New Kefir"]);
    typed_into(&screen, "new_food_description", "Plain, from the farm");
    for (field, figure) in [
        ("new_food_portion", "100"),
        ("new_food_energy", "64"),
        ("new_food_protein", "3.4"),
        ("new_food_fat", "3.6"),
        ("new_food_fibre", "0"),
        ("new_food_carbs", "4.1"),
        ("quantity", "250"),
    ] {
        typed_into(&screen, field, figure);
    }
    settled().await;
    let offered = text_of(&screen, "form.intake button[type=submit]");
    found::<HtmlFormElement>(&screen, "form.intake")
        .request_submit()
        .expect("the new food is added, then logged");
    settled().await;
    settled().await;

    assert_eq!(offered, "Add and log");
    let added = asked("add_food");
    for sent in [
        r#""name":"Kefir""#,
        r#""description":"Plain, from the farm""#,
        r#""energy_kcal":64"#,
        r#""carbs_g":4.1"#,
    ] {
        assert!(added.contains(sent), "{sent} in {added}");
    }
    let logged = asked("log_intake");
    assert!(
        logged.contains(r#""food":"Kefir""#) && logged.contains(r#""quantity":250"#),
        "{logged}"
    );
    assert!(texts_in_order(&screen, "form.intake .new").is_empty());
}

#[wasm_bindgen_test]
async fn the_fuel_form_previews_what_an_intake_will_record_one_portion_when_no_quantity_is_typed() {
    answering(&format!(
        r#"{{ "foods": {THREE_FOODS}, "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    typed_into(&screen, "food", "skyr");
    settled().await;
    let a_portion = text_of(&screen, "form.intake p.preview");
    typed_into(&screen, "quantity", "200");
    settled().await;

    assert_eq!(
        [a_portion, text_of(&screen, "form.intake p.preview")],
        [
            "54 kcal · P 10 g · F 0 g · Fib 0 g · C 4 g",
            "108 kcal · P 19 g · F 0 g · Fib 0 g · C 8 g"
        ]
    );
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
async fn the_fuel_shelf_adds_a_food_from_its_form_card_in_rows_to_the_foods_its_macros_in_columns()
{
    answering(
        r#"{ "foods": [], "meals": [], "add_food": [{ "name": "Skyr", "unit": "Gram", "portion": 100.0,
            "nutrients": { "energy_kcal": 54.0, "protein_g": 9.5, "carbs_g": 4.0, "fat_g": 0.0, "fibre_g": 0.0 } }] }"#,
    );
    let screen = screen();
    let _mounted = the_fuel_shelf(&screen);
    settled().await;
    assert_eq!(tables_in_order(&screen), ["foods"]);
    found::<Element>(&screen, "button.form-Food.chosen");
    found::<Element>(&screen, "button.form-Meal");
    assert_eq!(
        fields_in_rows(&screen, "form.food"),
        [
            vec!["food_name"],
            vec!["food_portion", "food_unit", "food_energy"],
            vec!["food_protein", "food_fat", "food_fibre", "food_carbs"],
        ]
    );

    typed_into(&screen, "food_name", "Skyr");
    typed_into(&screen, "food_portion", "100");
    typed_into(&screen, "food_energy", "54");
    found::<HtmlFormElement>(&screen, "form.food")
        .request_submit()
        .expect("the food is added");
    settled().await;

    let row = texts_of(&screen, "table.foods tbody tr td");
    for cell in ["Skyr", "100 g", "54", "10", "0", "4"] {
        assert!(row.contains(&cell.to_string()), "{cell} in {row:?}");
    }
    assert_eq!(value_of(&screen, "food_name"), "");
}

const PASTA_AND_CREAM: &str = r#"[
    { "name": "Pasta in cream", "unit": "Gram", "portion": 100.0,
      "nutrients": { "energy_kcal": 251.3, "protein_g": 4.2, "carbs_g": 19.9, "fat_g": 16.1, "fibre_g": 0.8 } },
    { "name": "Pasta", "unit": "Gram", "portion": 100.0,
      "nutrients": { "energy_kcal": 360.0, "protein_g": 12.0, "carbs_g": 70.0, "fat_g": 1.5, "fibre_g": 3.0 } },
    { "name": "Crème fraîche", "unit": "Gram", "portion": 100.0,
      "nutrients": { "energy_kcal": 300.0, "protein_g": 2.0, "carbs_g": 3.0, "fat_g": 30.0, "fibre_g": 0.0 } }]"#;

const PASTA_IN_CREAM: &str = r#"{ "name": "Pasta in cream", "description": "Boil, reduce, stir.",
    "ingredients": [
        { "food": "Pasta", "unit": "Gram", "quantity": 100.0, "cooked_per_raw": 2.5,
          "nutrients": { "energy_kcal": 360.0, "protein_g": 12.0, "carbs_g": 70.0, "fat_g": 1.5, "fibre_g": 3.0 } },
        { "food": "Crème fraîche", "unit": "Gram", "quantity": 200.0, "cooked_per_raw": 0.66,
          "nutrients": { "energy_kcal": 600.0, "protein_g": 4.0, "carbs_g": 6.0, "fat_g": 60.0, "fibre_g": 0.0 } } ],
    "raw_g": 300.0, "cooked_g": 382.0,
    "totals": { "energy_kcal": 960.0, "protein_g": 16.0, "carbs_g": 76.0, "fat_g": 61.5, "fibre_g": 3.0 } }"#;

async fn added_to_the_pot(screen: &Element, food: &str, quantity: &str) {
    typed_into(screen, "ingredient", food);
    typed_into(screen, "ingredient_quantity", quantity);
    clicked(screen, "button.add-ingredient");
    settled().await;
}

#[wasm_bindgen_test]
async fn a_meal_cooked_from_its_ingredients_and_their_ratios_joins_the_meals_its_ingredients_on_a_click()
 {
    answering(&format!(
        r#"{{ "foods": {PASTA_AND_CREAM}, "meals": [], "cook_meal": [{PASTA_IN_CREAM}] }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_shelf(&screen);
    settled().await;
    clicked(&screen, "button.form-Meal");
    settled().await;

    typed_into(&screen, "meal_name", "Pasta in cream");
    added_to_the_pot(&screen, "pasta", "100").await;
    added_to_the_pot(&screen, "creme fraiche", "200").await;
    assert_eq!(
        found::<HtmlInputElement>(&screen, "input[name=ratio_1]").placeholder(),
        "1"
    );
    typed_into(&screen, "ratio_0", "2,5");
    typed_into(&screen, "ratio_1", "0.66");
    found::<HtmlFormElement>(&screen, "form.meal")
        .request_submit()
        .expect("the meal is cooked");
    settled().await;

    let sent = asked("cook_meal");
    assert!(
        sent.contains(r#""food":"Crème fraîche""#)
            && sent.contains(r#""cooked_per_raw":2.5"#)
            && sent.contains(r#""cooked_per_raw":0.66"#),
        "{sent}"
    );
    assert_eq!(value_of(&screen, "meal_name"), "");
    assert_eq!(tables_in_order(&screen), ["meals"]);
    let meal = found::<Element>(&screen, "table.meals tr.meal");
    assert!(
        meal.text_content()
            .unwrap_or_default()
            .contains("Pasta in cream")
    );
    assert!(
        screen
            .query_selector("table.meals tr.details")
            .ok()
            .flatten()
            .is_none()
    );
    meal.unchecked_into::<web_sys::HtmlElement>().click();
    settled().await;
    let details = text_of(&screen, "table.meals tr.details");
    for figure in ["Boil, reduce, stir.", "× 0.66", "132 g"] {
        assert!(details.contains(figure), "{figure} in {details}");
    }
    assert!(
        !details.contains("960 kcal") && !details.contains("Per 100 g"),
        "{details}"
    );
}

fn cells_by_row(screen: &Element, rows: &str) -> Vec<Vec<String>> {
    let rows = screen.query_selector_all(rows).expect("the rows");
    (0..rows.length())
        .filter_map(|at| rows.item(at))
        .map(|row| {
            let cells = row
                .unchecked_into::<Element>()
                .query_selector_all("td")
                .expect("the cells");
            (0..cells.length())
                .filter_map(|at| cells.item(at))
                .filter_map(|cell| cell.text_content())
                .collect()
        })
        .collect()
}

#[wasm_bindgen_test]
async fn the_cooker_previews_an_ingredient_at_its_quantity_and_the_pot_adds_up_what_each_brings() {
    answering(&format!(r#"{{ "foods": {PASTA_AND_CREAM}, "meals": [] }}"#));
    let screen = screen();
    let _mounted = the_fuel_shelf(&screen);
    settled().await;
    clicked(&screen, "button.form-Meal");
    settled().await;

    typed_into(&screen, "ingredient", "pasta");
    typed_into(&screen, "ingredient_quantity", "100");
    settled().await;
    assert_eq!(
        text_of(&screen, "p.preview"),
        "360 kcal · P 12 g · F 2 g · Fib 3 g · C 70 g"
    );

    clicked(&screen, "button.add-ingredient");
    settled().await;
    added_to_the_pot(&screen, "creme fraiche", "200").await;

    assert!(screen.query_selector("p.preview").ok().flatten().is_none());
    assert_eq!(
        cells_by_row(&screen, "table.pot tbody tr"),
        [
            ["100 g Pasta", "", "360", "12", "2", "3", "70", "×"],
            ["200 g Crème fraîche", "", "600", "4", "60", "0", "6", "×"]
        ]
    );
    assert_eq!(
        cells_by_row(&screen, "table.pot tfoot tr"),
        [["Total", "", "960", "16", "62", "3", "76", ""]]
    );
}

#[wasm_bindgen_test]
async fn a_meal_in_progress_waits_on_the_fuel_shelf_while_the_cook_checks_another_tab() {
    answering_all(&[
        A_HOME,
        EVERY_TAB,
        &format!(r#"{{ "profile_on": null, "foods": {PASTA_AND_CREAM}, "meals": [] }}"#),
    ]);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;
    went_to("/fuel");
    settled().await;
    clicked(&screen, "button.form-Meal");
    settled().await;
    typed_into(&screen, "meal_name", "Pasta in cream");
    written_into(&screen, "meal_description", "Boil, reduce, stir.");
    added_to_the_pot(&screen, "pasta", "100").await;
    typed_into(&screen, "ratio_0", "2.5");
    typed_into(&screen, "ingredient", "creme fraiche");

    went_to("/hydrate");
    settled().await;
    went_to("/fuel");
    settled().await;

    assert_eq!(
        [
            value_of(&screen, "meal_name"),
            found::<HtmlInputElement>(&screen, "textarea[name=meal_description]").value(),
            value_of(&screen, "ratio_0"),
            value_of(&screen, "ingredient"),
        ],
        [
            "Pasta in cream",
            "Boil, reduce, stir.",
            "2.5",
            "creme fraiche"
        ]
    );
    assert_eq!(
        cells_by_row(&screen, "table.pot tbody tr"),
        [["100 g Pasta", "", "360", "12", "2", "3", "70", "×"]]
    );
}

#[wasm_bindgen_test]
async fn an_ingredient_left_without_its_ratio_cooks_at_its_own_mass() {
    answering(&format!(
        r#"{{ "foods": {PASTA_AND_CREAM}, "meals": [], "cook_meal": [{PASTA_IN_CREAM}] }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_shelf(&screen);
    settled().await;
    clicked(&screen, "button.form-Meal");
    settled().await;

    typed_into(&screen, "meal_name", "Plain pasta");
    added_to_the_pot(&screen, "Pasta", "100").await;
    found::<HtmlFormElement>(&screen, "form.meal")
        .request_submit()
        .expect("the meal is cooked");
    settled().await;

    let sent = asked("cook_meal");
    assert!(
        sent.contains(r#""food":"Pasta""#) && !sent.contains("cooked_per_raw"),
        "{sent}"
    );
}

const A_HOME: &str = r#"{ "profile_on": { "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null },
             "gather_card": { "masses": [], "trend": [], "target_kg": null, "progress": null, "fat_change_kg": null, "lean_change_kg": null },
             "streak": { "days": 2, "longest": 5, "at_risk": true, "history": [
                 { "day": "2026-10-08", "status": "Success", "ticks": { "weighed": true, "fuelled": true, "worked_out": true, "hydrated": null } },
                 { "day": "2026-10-09", "status": "Joker", "ticks": { "weighed": true, "fuelled": false, "worked_out": true, "hydrated": null } },
                 { "day": "2026-10-10", "status": "Pending", "ticks": { "weighed": true, "fuelled": false, "worked_out": false, "hydrated": true } } ] },
             "fuel_day": { "day": "2026-10-10", "intakes": [], "band": null, "energy": null, "macros": null,
                 "eaten": { "energy_kcal": 0.0, "protein_g": 0.0, "carbs_g": 0.0, "fat_g": 0.0, "fibre_g": 0.0 } },
             "burn_day": { "day": "2026-10-10", "workouts": [], "burnt_kcal": 0.0, "off_the_clock": null, "off_the_clock_is_usual": true },
             "gather_week": { "day": { "day": "2026-10-10", "body_mass_kg": 71.7, "waist_cm": null, "age": 41, "composition": null },
                 "days": [], "tendency": { "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null } } }"#;

const EVERY_TAB: &str = r#"{ "foods": [], "meals": [], "drinks": [], "exercises": [], "circuits": [],
    "priced_circuits": [], "hydrate_day": { "day": "2026-10-10", "drinkings": [], "drunk_ml": 0.0,
        "target_ml": 2000.0, "floor_ml": 1800.0, "cap_ml": 2200.0, "verdict": "Low" } }"#;

#[wasm_bindgen_test]
async fn the_icon_bar_leads_home_and_to_each_tab() {
    answering_all(&[A_HOME, EVERY_TAB]);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;

    assert_eq!(
        attributes_of(&screen, "nav.tabs a", "href"),
        ["/", "/gather", "/fuel", "/hydrate", "/burn"]
    );
    assert_eq!(
        texts_in_order(&screen, "nav.tabs a"),
        ["🏠", "⚖️", "⚡", "💧", "🔥"]
    );
    assert_eq!(
        attributes_of(&screen, "nav.tabs a", "aria-label"),
        ["Home", "Gather", "Fuel", "Hydrate", "Burn"]
    );
    let mut opened = Vec::new();
    for (tab, form) in [
        ("/gather", "form.measurement"),
        ("/fuel", "form.intake"),
        ("/hydrate", "form.drinking"),
        ("/burn", "form.workout"),
    ] {
        went_to(tab);
        settled().await;
        opened.push(screen.query_selector(form).ok().flatten().is_some());
    }
    assert_eq!(opened, [true; 4]);
}

/// What a screen shows, top to bottom: each part by its title, else by its words for a link, else
/// by its class.
fn shown_in_order(screen: &Element) -> Vec<String> {
    let parts = screen
        .query_selector_all("main.wrap > *")
        .expect("the screen's parts");
    (0..parts.length())
        .filter_map(|at| parts.item(at))
        .map(|part| {
            let part: Element = part.unchecked_into();
            part.query_selector("h2")
                .ok()
                .flatten()
                .and_then(|title| title.text_content())
                .or_else(|| {
                    (part.tag_name() == "A")
                        .then(|| part.text_content())
                        .flatten()
                })
                .unwrap_or_else(|| part.class_name())
        })
        .collect()
}

#[wasm_bindgen_test]
async fn each_tab_runs_from_its_day_through_its_log_band_breakdown_and_entries_to_stats_then_its_shelf()
 {
    let a_day_in_each_tab = format!(
        r#"{{ "gather_week": {{ "day": {A_GATHERED_DAY}, "days": [{A_GATHERED_DAY}],
                  "tendency": {{ "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null }},
                  "aims": {{ "body_mass_kg": 75.0, "lean_mass_kg": 66.0, "recommended": false }} }},
              "fuel_day": {A_FUELLED_DAY}, "hydrate_day": {A_DAY_WITH_TWO_DRINKS},
              "burn_day": {A_BURN_SHORT_OF_ITS_TARGET} }}"#
    );
    answering_all(&[A_HOME, EVERY_TAB, &a_day_in_each_tab]);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;

    let mut tabs = Vec::new();
    for tab in ["/gather", "/fuel", "/hydrate", "/burn"] {
        went_to(tab);
        settled().await;
        settled().await;
        tabs.push(shown_in_order(&screen));
    }

    assert_eq!(
        tabs,
        [
            vec![
                "Today",
                "card day",
                "Gather",
                "Where you stand",
                "The week's pace",
                "The last days",
                "Explore in Stats"
            ],
            vec![
                "Today",
                "card day",
                "Fuel",
                "Where the day stands",
                "The four macros",
                "The day's intakes",
                "Explore in Stats",
                "Add to the catalog",
                "Foods"
            ],
            vec![
                "Today",
                "card day",
                "Hydrate",
                "Where the day stands",
                "The day's drinks",
                "Explore in Stats",
                "Add to the catalog",
                "Drinks"
            ],
            vec![
                "Today",
                "card day",
                "Burn",
                "Off the clock",
                "Where the day stands",
                "The day's expenditures",
                "Explore in Stats",
                "Add to the catalog",
                "Exercises"
            ],
        ]
    );
}

#[wasm_bindgen_test]
async fn the_streak_card_says_each_tick_holds_within_5_percent_of_the_gauges_strict_bands() {
    answering(A_HOME);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;
    settled().await;

    let card = text_of(&screen, "section.card.streak");
    assert!(card.contains("within 5% of its band"), "{card}");
}

#[wasm_bindgen_test]
async fn every_screen_wears_the_header_its_person_opening_the_profile_and_todays_four_tiles() {
    answering_all(&[A_HOME, EVERY_TAB]);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;

    let mut worn = Vec::new();
    for page in ["/", "/burn", "/profile", "/stats"] {
        went_to(page);
        settled().await;
        worn.push((
            page,
            screen
                .query_selector("header.top img.logo")
                .ok()
                .flatten()
                .is_some(),
            attributes_of(&screen, "header.top a.person", "href"),
            attributes_of(&screen, "header.top .today a.tile", "href").len(),
        ));
    }

    assert!(
        worn.iter()
            .all(|(_, logo, person, tiles)| *logo && *person == ["/profile"] && *tiles == 4),
        "{worn:?}"
    );
}

#[wasm_bindgen_test]
async fn todays_tiles_show_each_figure_green_or_red_by_its_tick_and_open_its_tab() {
    answering(A_HOME);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;
    settled().await;

    assert_eq!(
        texts_in_order(&screen, "header.top .today .tile .icon"),
        ["⚖️", "⚡", "💧", "🔥"]
    );
    assert!(text_of(&screen, ".today .tile.weighed").contains("71.7"));
    let coloured: Vec<bool> = [
        ".weighed.good",
        ".fuelled.bad",
        ".hydrated.good",
        ".worked-out.bad",
    ]
    .into_iter()
    .map(|tile| {
        screen
            .query_selector(&format!(".today .tile{tile}"))
            .ok()
            .flatten()
            .is_some()
    })
    .collect();
    assert_eq!(coloured, [true, true, true, true]);
    assert_eq!(
        attributes_of(&screen, ".today a.tile", "href"),
        ["/gather", "/fuel", "/hydrate", "/burn"]
    );
}

#[wasm_bindgen_test]
async fn home_holds_the_streak_alone_in_its_card_and_ends_on_stats() {
    answering(A_HOME);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;
    settled().await;

    assert_eq!(texts_of(&screen, "section.card.streak h2"), ["Streak"]);
    assert_eq!(text_of(&screen, ".streak .current"), "2 days");
    assert_eq!(text_of(&screen, ".streak .longest"), "(longest: 5 days)");
    for elsewhere in [".streak .tile", ".streak .dot"] {
        assert!(
            screen.query_selector(elsewhere).ok().flatten().is_none(),
            "{elsewhere}"
        );
    }
    found::<Element>(&screen, "main.wrap > a.explore:last-child");
}

const TWO_MEASURES_EXPLORED: &str = r#"{ "one": [["2026-10-08", 80.4], ["2026-10-09", 80.2], ["2026-10-10", 80.0]],
    "other": [["2026-10-08", 2600.0], ["2026-10-09", 2500.0], ["2026-10-10", 2400.0]],
    "one_summary": { "first": 80.4, "last": 80.0, "mean": 80.2, "lowest": 80.0, "highest": 80.4 },
    "other_summary": { "first": 2600.0, "last": 2400.0, "mean": 2500.0, "lowest": 2400.0, "highest": 2600.0 },
    "pairs": [[80.4, 2600.0], [80.2, 2500.0], [80.0, 2400.0]], "correlation": 1.0 }"#;

fn the_stats(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <Stats />
            </Router>
        }
    })
}

#[wasm_bindgen_test]
async fn stats_draw_two_measures_on_their_own_y_axes_and_tell_how_they_move_together() {
    answering(&format!(r#"{{ "explore": {TWO_MEASURES_EXPLORED} }}"#));
    let screen = screen();
    let _mounted = the_stats(&screen);
    settled().await;

    let count = |selector: &str| {
        screen
            .query_selector_all(selector)
            .map(|found| found.length())
            .unwrap_or_default()
    };
    let said = screen.text_content().unwrap_or_default();
    assert_eq!(
        (
            count(".timeline svg polyline"),
            count(".scatter svg circle")
        ),
        (2, 3)
    );
    assert_eq!(
        (
            count(".timeline svg line.axis"),
            count(".scatter svg line.axis")
        ),
        (2, 1)
    );
    assert_eq!(
        texts_of(&screen, ".timeline .label.tick.one"),
        ["80.0", "80.2", "80.4"]
    );
    assert_eq!(
        texts_of(&screen, ".timeline .label.tick.other"),
        ["2400", "2500", "2600"]
    );
    assert_eq!(
        texts_of(&screen, ".scatter .label.tick.other"),
        ["2400", "2500", "2600"]
    );
    assert!(
        said.contains("80.4 kg") && said.contains("2400 kcal") && said.contains("1.00"),
        "{said}"
    );
}

const MORNING_RUST: &str = r#"{ "name": "Morning rust", "description": "Found on YouTube, the rusty goblin channel.", "steps": [
    { "exercise": "Push up", "met": 3.8, "reps": 12, "minutes": 0.5, "load_kg": 0.0 },
    { "exercise": "Burpee", "met": 8.0, "reps": null, "minutes": 1.0, "load_kg": 5.0 } ] }"#;

#[wasm_bindgen_test]
async fn the_crafter_crafts_a_circuit_the_lab_lists_priced_per_set_its_steps_on_a_click() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Burpee", "met": 8.0 }}, {{ "name": "Push up", "met": 3.8 }}],
             "circuits": [], "craft_circuit": [{MORNING_RUST}],
             "priced_circuits": [{{ "circuit": {MORNING_RUST}, "burnt_per_set_kcal": 12.3 }}] }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_shelf(&screen);
    settled().await;
    clicked(&screen, "button.form-Circuit");
    settled().await;

    typed_into(&screen, "circuit_name", "Morning rust");
    written_into(
        &screen,
        "circuit_description",
        "Found on YouTube, the rusty goblin channel.",
    );
    typed_into(&screen, "step_exercise", "Push up");
    typed_into(&screen, "step_reps", "12");
    typed_into(&screen, "step_minutes", "0.5");
    clicked(&screen, "button.add-step");
    typed_into(&screen, "step_exercise", "Burpee");
    typed_into(&screen, "step_minutes", "1");
    typed_into(&screen, "step_load", "5");
    clicked(&screen, "button.add-step");
    settled().await;
    let steps = text_of(&screen, "ol.steps");
    assert!(
        steps.contains("Push up") && steps.contains("Burpee"),
        "{steps}"
    );
    found::<HtmlFormElement>(&screen, "form.circuit")
        .request_submit()
        .expect("the circuit is crafted");
    settled().await;

    assert!(
        asked("craft_circuit")
            .contains(r#""description":"Found on YouTube, the rusty goblin channel.""#)
    );
    assert_eq!(value_of(&screen, "circuit_name"), "");
    assert_eq!(
        texts_of(&screen, "table.circuits thead th"),
        ["", "Name", "kcal / set"]
    );
    let row = text_of(&screen, "table.circuits tbody tr");
    assert!(row.contains("Morning rust") && row.contains("12"), "{row}");
    clicked(&screen, "table.circuits tbody tr");
    settled().await;
    let details = text_of(&screen, "table.circuits tr.details");
    assert!(
        details.contains("Found on YouTube, the rusty goblin channel.")
            && details.contains("Push up")
            && details.contains("Burpee"),
        "{details}"
    );
}

#[wasm_bindgen_test]
async fn the_crafter_prices_each_step_and_the_set_they_add_up_to() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Burpee", "met": 8.0 }}, {{ "name": "Push up", "met": 3.8 }}],
             "circuits": [], "priced_circuits": [], "burn_day": {A_BURN_PRICED_AT_80_KG} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_shelf(&screen);
    settled().await;
    clicked(&screen, "button.form-Circuit");
    settled().await;

    typed_into(&screen, "step_exercise", "burpee");
    typed_into(&screen, "step_minutes", "1");
    typed_into(&screen, "step_load", "5");
    settled().await;
    let burpee = text_of(&screen, "form.circuit p.preview");
    clicked(&screen, "button.add-step");
    settled().await;
    typed_into(&screen, "step_exercise", "push up");
    typed_into(&screen, "step_reps", "12");
    typed_into(&screen, "step_minutes", "0.5");
    clicked(&screen, "button.add-step");
    settled().await;

    let priced: Vec<String> = screen
        .query_selector_all("ol.steps li .kcal")
        .map(|found| {
            (0..found.length())
                .filter_map(|at| found.item(at))
                .filter_map(|step| step.text_content())
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(burpee, "10 kcal");
    assert_eq!(priced, ["10 kcal", "2 kcal"]);
    assert_eq!(text_of(&screen, "form.circuit p.per-set"), "12 kcal a set");
}

#[wasm_bindgen_test]
async fn a_step_added_by_mistake_leaves_the_circuit_before_it_is_crafted() {
    answering(
        r#"{ "exercises": [{ "name": "Burpee", "met": 8.0 }, { "name": "Push up", "met": 3.8 }], "circuits": [] }"#,
    );
    let screen = screen();
    let _mounted = the_burn_shelf(&screen);
    settled().await;
    clicked(&screen, "button.form-Circuit");
    settled().await;
    typed_into(&screen, "step_exercise", "Burpee");
    typed_into(&screen, "step_minutes", "1");
    clicked(&screen, "button.add-step");
    settled().await;

    clicked(&screen, "ol.steps button.remove");
    settled().await;

    assert_eq!(text_of(&screen, "ol.steps"), "");
}

#[wasm_bindgen_test]
async fn removing_a_circuit_takes_it_out_of_the_lab() {
    answering(&format!(
        r#"{{ "exercises": [], "circuits": [{MORNING_RUST}], "remove_circuit": [],
              "priced_circuits": [{{ "circuit": {MORNING_RUST}, "burnt_per_set_kcal": null }}] }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_shelf(&screen);
    settled().await;
    clicked(&screen, "button.form-Circuit");
    settled().await;
    assert_eq!(tables_in_order(&screen), ["circuits"]);

    clicked(&screen, "table.circuits button.remove");
    settled().await;

    assert_eq!(text_of(&screen, "table.circuits tbody"), "");
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
                    body_mass=Signal::stored(None)
                    on_logged=Callback::new(move |burn: Burn| handed_over.set(Some(burn)))
                />
                <p class="handed-over">
                    {move || handed_over.get().map(|burn| burn.workouts.len().to_string())}
                </p>
            </Router>
        }
    });
    settled().await;
    assert_eq!(
        found::<HtmlInputElement>(&screen, "input[name=circuit_sets]").placeholder(),
        "1"
    );

    typed_into(&screen, "circuit", "Morning rust");
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

const A_DAY_PAST_THE_CAP: &str = r#"{ "day": "2026-10-12",
    "drinkings": [[1, { "day": "2026-10-12", "drink": "Tea", "kind": "Other", "servings": 20.0, "volume_ml": 4000.0 }]],
    "drunk_ml": 4000.0, "target_ml": 3000.0, "floor_ml": 2400.0, "cap_ml": 3600.0, "verdict": "High" }"#;

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
async fn a_drink_nothing_matches_turns_the_hydrate_form_into_a_new_one_added_then_logged_in_one_tap()
 {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "hydrate_day": {A_DRY_DAY},
              "add_drink": [{{ "name": "Kombucha", "volume_ml": 330.0, "kind": "Other", "description": "The ginger one" }}],
              "log_drink": {A_DAY_WITH_TWO_DRINKS} }}"#
    ));
    let screen = screen();
    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-11".to_string()));
    settled().await;

    typed_into(&screen, "drink", "Kombucha");
    settled().await;
    assert_eq!(
        texts_in_order(&screen, "form.drinking .new"),
        ["New Kombucha"]
    );
    typed_into(&screen, "new_drink_description", "The ginger one");
    typed_into(&screen, "new_drink_serving", "330");
    let kind: HtmlSelectElement = found(&screen, "select[name=new_drink_kind]");
    kind.set_value("Other");
    kind.dispatch_event(&Event::new("change").expect("a change event"))
        .expect("the kind changes");
    settled().await;
    let offered = text_of(&screen, "form.drinking button[type=submit]");
    found::<HtmlFormElement>(&screen, "form.drinking")
        .request_submit()
        .expect("the new drink is added, then logged");
    settled().await;
    settled().await;

    assert_eq!(offered, "Add and log");
    let added = asked("add_drink");
    for sent in [
        r#""name":"Kombucha""#,
        r#""volume_ml":330"#,
        r#""kind":"Other""#,
        r#""description":"The ginger one""#,
    ] {
        assert!(added.contains(sent), "{sent} in {added}");
    }
    assert!(asked("log_drink").contains(r#""drink":"Kombucha""#));
    assert!(texts_in_order(&screen, "form.drinking .new").is_empty());
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
async fn a_logged_drink_joins_the_day_in_millilitres_and_the_form_empties() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "hydrate_day": {A_DRY_DAY},
              "log_drink": {{ "day": "2026-10-11", "drinkings": [[1, {{ "day": "2026-10-11", "drink": "Tea", "kind": "Other", "servings": 1.5, "volume_ml": 300.0 }}]],
                              "drunk_ml": 300.0, "target_ml": 2000.0, "floor_ml": 1800.0, "cap_ml": 2200.0, "verdict": "Low" }} }}"#
    ));
    let screen = screen();
    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-11".to_string()));
    settled().await;

    typed_into(&screen, "drink", "Tea");
    typed_into(&screen, "volume", "300");
    found::<HtmlFormElement>(&screen, "form.drinking")
        .request_submit()
        .expect("the drink is logged");
    settled().await;

    assert!(asked("log_drink").contains(r#""volume_ml":300"#));
    let listed: Element = found(&screen, "tbody");
    assert!(
        listed
            .text_content()
            .is_some_and(|drunk| drunk.contains("Tea") && drunk.contains("300 mL"))
    );
    assert_eq!(
        [value_of(&screen, "drink"), value_of(&screen, "volume")],
        ["", ""]
    );
}

#[wasm_bindgen_test]
async fn a_drink_logged_in_its_tab_reaches_todays_tile_at_once() {
    let drinks = format!(r#"{{ "drinks": {THE_CATALOG} }}"#);
    answering_all(&[A_HOME, EVERY_TAB, &drinks]);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;
    went_to("/hydrate");
    settled().await;
    let before = text_of(&screen, ".today .tile.hydrated .big");

    let drunk = format!(
        r#"{{ "hydrate_day": {A_DAY_WITH_TWO_DRINKS}, "log_drink": {A_DAY_WITH_TWO_DRINKS} }}"#
    );
    answering_all(&[A_HOME, EVERY_TAB, &drinks, &drunk]);
    typed_into(&screen, "drink", "Tea");
    found::<HtmlFormElement>(&screen, "form.drinking")
        .request_submit()
        .expect("the drink is logged");
    settled().await;
    settled().await;

    assert_eq!(
        [before, text_of(&screen, ".today .tile.hydrated .big")],
        ["0", "280"]
    );
}

#[wasm_bindgen_test]
async fn the_drink_chooser_starts_on_select_me_and_a_volume_left_empty_logs_one_serving() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "hydrate_day": {A_DRY_DAY}, "log_drink": {A_DRY_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-11".to_string()));
    settled().await;
    let drink: HtmlInputElement = found(&screen, "input[name=drink]");
    assert_eq!(drink.placeholder(), "Select me");

    focused(&screen, "drink");
    settled().await;
    picked(&screen, "Tea");
    settled().await;
    let volume: HtmlInputElement = found(&screen, "input[name=volume]");
    assert_eq!(volume.placeholder(), "200");
    assert!(label_of(&screen, "volume").contains("(mL)"));
    found::<HtmlFormElement>(&screen, "form.drinking")
        .request_submit()
        .expect("the drink is logged");
    settled().await;

    let sent = asked("log_drink");
    assert!(
        sent.contains(r#""drink":"Tea""#) && !sent.contains("volume_ml"),
        "{sent}"
    );
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

#[wasm_bindgen_test]
async fn a_drink_added_from_the_hydrate_shelfs_form_card_joins_its_list_its_kind_water_or_other() {
    answering(
        r#"{ "drinks": [], "add_drink": [{ "name": "Tea", "volume_ml": 200.0, "kind": "Other" }] }"#,
    );
    let screen = screen();
    let _mounted = the_hydrate_shelf(&screen);
    settled().await;
    assert_eq!(
        found::<HtmlSelectElement>(&screen, "select[name=drink_kind]").length(),
        2
    );

    typed_into(&screen, "drink_name", "Tea");
    typed_into(&screen, "serving", "200");
    found::<HtmlFormElement>(&screen, "form.drink")
        .request_submit()
        .expect("the drink is added");
    settled().await;

    assert_eq!(value_of(&screen, "drink_name"), "");
    let rows = text_of(&screen, "table.drinks tbody");
    assert!(rows.contains("Tea") && rows.contains("200 mL"), "{rows}");
}

#[wasm_bindgen_test]
async fn removing_a_drink_takes_it_off_the_hydrate_shelf() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "remove_drink": [{{ "name": "Tea", "volume_ml": 200.0, "kind": "Other" }}] }}"#
    ));
    let screen = screen();
    let _mounted = the_hydrate_shelf(&screen);
    settled().await;

    clicked(&screen, "table.drinks button.remove");
    settled().await;

    let rows = text_of(&screen, "table.drinks tbody");
    assert!(!rows.contains("Expresso") && rows.contains("Tea"), "{rows}");
}

fn the_hydrate_card_in(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <HydrateCountedCard />
            </Router>
        }
    })
}

#[wasm_bindgen_test]
async fn the_profile_card_switches_hydrate_on_or_off_from_a_date() {
    answering(r#"{ "hydrate_counted_on": true, "set_hydrate_counted": false }"#);
    let screen = screen();
    let _mounted = the_hydrate_card_in(&screen);
    settled().await;
    let in_force = text_of(&screen, "form.hydrate-counted .pill.chosen");

    clicked(&screen, "form.hydrate-counted .pill.left-out");
    typed_into(&screen, "hydrate_counted_from", "2026-03-01");
    found::<HtmlFormElement>(&screen, "form.hydrate-counted")
        .request_submit()
        .expect("the switch is kept");
    settled().await;

    assert_eq!(in_force, "Counted");
    assert!(asked("set_hydrate_counted").contains(r#""counted":false"#));
    let said = text_of(&screen, "form.hydrate-counted .said");
    assert!(said.contains("out of the streak from 2026-03-01"), "{said}");
}

#[wasm_bindgen_test]
async fn drinking_past_the_cap_says_to_spread_the_cups_out() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "hydrate_day": {A_DAY_PAST_THE_CAP} }}"#
    ));
    let screen = screen();

    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-12".to_string()));
    settled().await;

    let day = screen.text_content().unwrap_or_default();
    assert!(day.contains("about a litre an hour"), "{day}");
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

#[wasm_bindgen_test]
async fn the_history_marks_each_day_with_its_streak_dot_under_headers_holding_the_units() {
    answering(&format!(
        r#"{{ {THE_CHARTS}, "gather_month": {{ "month": "2026-10", "earlier": null, "rows": [
                {{ "day": "2026-10-10", "body_mass_kg": 80.0, "lean_mass_kg": 70.7, "fat_mass_kg": 9.3, "fat_ratio": 0.116 }},
                {{ "day": "2026-10-09", "body_mass_kg": 80.2, "lean_mass_kg": 70.9, "fat_mass_kg": 9.3, "fat_ratio": 0.116 }} ] }} }}"#
    ));
    let screen = screen();
    let _mounted = the_charts(&screen);
    settled().await;

    let heads = text_of(&screen, ".history thead");
    for head in ["Day", "BM (kg)", "FFM (kg)", "FM (kg)", "Fat %", "Streak"] {
        assert!(heads.contains(head), "{head} in {heads}");
    }
    let rows = text_of(&screen, ".history tbody");
    assert!(rows.contains("80.0") && !rows.contains("kg"), "{rows}");
    found::<Element>(
        &screen,
        ".history tbody tr:first-child td:last-child .dot.Pending",
    );
    found::<Element>(
        &screen,
        ".history tbody tr:nth-child(2) td:last-child .dot.Joker",
    );
    found::<Element>(&screen, ".history .legend .dot.Success");
}

#[wasm_bindgen_test]
async fn a_history_row_tapped_shows_its_days_four_ticks() {
    answering(&format!(
        r#"{{ {THE_CHARTS}, "gather_month": {{ "month": "2026-10", "earlier": null, "rows": [
                {{ "day": "2026-10-10", "body_mass_kg": 80.0, "lean_mass_kg": 70.7, "fat_mass_kg": 9.3, "fat_ratio": 0.116 }},
                {{ "day": "2026-10-09", "body_mass_kg": 80.2, "lean_mass_kg": 70.9, "fat_mass_kg": 9.3, "fat_ratio": 0.116 }} ] }} }}"#
    ));
    let screen = screen();
    let _mounted = the_charts(&screen);
    settled().await;

    clicked(&screen, ".history tbody tr:nth-child(2)");
    settled().await;

    let detail = text_of(&screen, ".history tbody tr.ticks");
    assert!(detail.contains("a joker"), "{detail}");
    for tick in ["⚖️ ✓", "⚡ ✗", "💧 ✓", "🔥 ✓"] {
        assert!(detail.contains(tick), "{tick} in {detail}");
    }
}

const THE_CHARTS: &str = r#""gather_card": { "masses": [], "trend": [], "target_kg": 70.0, "progress": null, "fat_change_kg": null, "lean_change_kg": null },
    "gather_chart": { "masses": [["2026-10-09", 80.2], ["2026-10-10", 80.0]], "lean_masses": [["2026-10-09", 70.9], ["2026-10-10", 70.7]],
                      "goal_kg": 70.0, "goal_lean_kg": 63.0 },
    "fuel_chart": { "amounts": [["2026-10-09", 2390.0], ["2026-10-10", 1900.0]], "floors": [["2026-10-09", 2199.0], ["2026-10-10", 2180.0]],
                    "caps": [["2026-10-09", 2650.0], ["2026-10-10", 2630.0]] },
    "hydrate_chart": { "amounts": [["2026-10-09", 1500.0], ["2026-10-10", 2100.0]], "floors": [["2026-10-09", 1800.0], ["2026-10-10", 1800.0]],
                       "caps": [["2026-10-09", 2200.0], ["2026-10-10", 2200.0]] },
    "burn_chart": { "burnt": [["2026-10-09", 120.0], ["2026-10-10", 252.0]],
                    "targets": [["2026-10-09", 80.0], ["2026-10-10", 80.0]] },
    "fuel_month": { "month": "2026-10", "earlier": null, "rows": [] },
    "hydrate_month": { "month": "2026-10", "earlier": null, "rows": [] },
    "burn_month": { "month": "2026-10", "earlier": null, "rows": [] },
    "streak": { "days": 2, "longest": 5, "at_risk": true, "history": [
        { "day": "2026-10-09", "status": "Joker", "ticks": { "weighed": true, "fuelled": false, "worked_out": true, "hydrated": true } },
        { "day": "2026-10-10", "status": "Pending", "ticks": { "weighed": true, "fuelled": true, "worked_out": false, "hydrated": true } } ] }"#;

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
async fn the_home_chart_draws_gather_on_two_axes_with_its_goals_then_each_chip_its_own_lines_burn_with_its_target()
 {
    answering(&format!(
        r#"{{ {THE_CHARTS}, "gather_month": {{ "month": "2026-10", "earlier": null, "rows": [] }} }}"#
    ));
    let screen = screen();
    let _mounted = the_charts(&screen);
    settled().await;

    assert_eq!(lines_drawn(&screen), 4);
    assert!(
        screen
            .query_selector(".charts ul.figures")
            .ok()
            .flatten()
            .is_none()
    );
    let labels = screen.text_content().unwrap_or_default();
    assert!(
        labels.contains("80.0 kg") && labels.contains("70.0 kg") && labels.contains("Lean mass"),
        "{labels}"
    );
    found::<Element>(&screen, ".plot .label.left.tick.mass");
    found::<Element>(&screen, ".plot .label.right.tick.lean");

    for (chip, lines, said) in [
        ("Fuel", 3, "1900 kcal"),
        ("Hydrate", 3, "2100 mL"),
        ("Burn", 2, "Target"),
    ] {
        found::<web_sys::HtmlElement>(&screen, &format!("button.chip-{chip}")).click();
        settled().await;

        assert_eq!(lines_drawn(&screen), lines, "{chip}");
        let labels = screen.text_content().unwrap_or_default();
        assert!(labels.contains(said), "{chip}: {labels}");
    }
    let legend = text_of(&screen, ".charts ul.legend");
    assert!(
        legend.contains("Burnt (kcal)") && legend.contains("Target") && !legend.contains("Minutes"),
        "{legend}"
    );
    found::<Element>(&screen, ".plot polyline.goal.dashed");
}

fn texts_of(screen: &Element, selector: &str) -> Vec<String> {
    let found = screen.query_selector_all(selector).expect(selector);
    let mut texts: Vec<String> = (0..found.length())
        .filter_map(|at| found.item(at))
        .filter_map(|node| node.text_content())
        .collect();
    texts.sort();
    texts
}

#[wasm_bindgen_test]
async fn every_home_chart_draws_its_y_axis_on_round_ticks_over_thin_grey_lines() {
    answering(&format!(
        r#"{{ {THE_CHARTS}, {THE_INSIGHTS}, "gather_month": {{ "month": "2026-10", "earlier": null, "rows": [] }} }}"#
    ));
    let screen = screen();
    let _mounted = the_charts(&screen);
    settled().await;
    let count = |screen: &Element, selector: &str| {
        screen
            .query_selector_all(selector)
            .map(|found| found.length())
            .unwrap_or_default()
    };

    assert_eq!(count(&screen, ".charts svg line.axis"), 2);
    assert!(count(&screen, ".charts .label.left.tick.mass") >= 2);
    assert!(count(&screen, ".charts .label.right.tick.lean") >= 2);
    assert!(count(&screen, ".charts svg line.grid") >= 2);

    clicked(&screen, "button.chip-Fuel");
    settled().await;
    assert_eq!(
        texts_of(&screen, ".charts .label.tick"),
        ["1750", "2000", "2250", "2500", "2750"]
    );

    clicked(&screen, "button.chip-Burn");
    settled().await;
    assert_eq!(count(&screen, ".charts svg line.axis"), 1);
    assert_eq!(
        texts_of(&screen, ".charts .label.tick.burnt"),
        ["0", "100", "200", "300"]
    );
    assert_eq!(count(&screen, ".charts .label.tick.minutes"), 0);
}

#[wasm_bindgen_test]
async fn a_chart_dates_its_x_axis_from_its_oldest_day_to_its_newest_with_three_between() {
    answering(&format!(
        r#"{{ {THE_CHARTS}, "gather_month": {{ "month": "2026-10", "earlier": null, "rows": [] }},
             "gather_chart": {{ "masses": [["2026-09-10", 81.0], ["2026-10-10", 80.0]], "lean_masses": [],
                                "goal_kg": null, "goal_lean_kg": null }} }}"#
    ));
    let screen = screen();
    let _mounted = the_charts(&screen);
    settled().await;

    assert_eq!(
        texts_of(&screen, ".charts .label.x"),
        ["09-10", "09-18", "09-25", "10-03", "10-10"]
    );
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

const THE_INSIGHTS: &str = r#""insights": { "days": 30, "weighed_days": 25, "fuelled_days": 20,
    "energy": { "low": 4, "in_band": 14, "high": 2 }, "protein": { "low": 2, "in_band": 18, "high": 0 },
    "fat": { "low": 1, "in_band": 17, "high": 2 }, "fibre": { "low": 15, "in_band": 5, "high": 0 },
    "carbs": { "low": 3, "in_band": 16, "high": 1 }, "hydrated_days": 10,
    "hydrate": { "low": 6, "in_band": 4, "high": 0 }, "active_days": 12, "burnt_per_active_day": 210.0 }"#;

#[wasm_bindgen_test]
async fn the_unwanted_tips_note_what_is_worth_sharpening_on_the_chosen_chip_between_the_chart_and_the_history()
 {
    answering(&format!(
        r#"{{ {THE_CHARTS}, {THE_INSIGHTS}, "gather_month": {{ "month": "2026-10", "earlier": null, "rows": [] }} }}"#
    ));
    let screen = screen();
    let _mounted = the_charts(&screen);
    settled().await;

    let cards = screen
        .query_selector_all("section.card")
        .expect("the cards");
    let order: Vec<String> = (0..cards.length())
        .filter_map(|at| cards.item(at))
        .filter_map(|card| card.unchecked_into::<Element>().get_attribute("class"))
        .collect();
    assert_eq!(order, ["card charts", "card tips", "card history"]);
    assert_eq!(text_of(&screen, ".tips h2"), "The unwanted tips");
    let gather = text_of(&screen, ".tips p");
    assert!(gather.contains("Too few weigh-ins"), "{gather}");

    clicked(&screen, "button.chip-Fuel");
    settled().await;
    let fuel = text_of(&screen, ".tips p");
    assert!(fuel.contains("Fibre") && fuel.contains("5 of 20"), "{fuel}");

    clicked(&screen, "button.chip-Hydrate");
    settled().await;
    let hydrate = text_of(&screen, ".tips p");
    assert!(hydrate.contains("4 of 10"), "{hydrate}");

    clicked(&screen, "button.chip-Burn");
    settled().await;
    let burn = text_of(&screen, ".tips p");
    assert!(burn.contains("2.8") && burn.contains("210 kcal"), "{burn}");
    assert!(
        [gather, fuel, hydrate, burn]
            .iter()
            .all(|tip| naming_the_cast(tip).is_empty() && !tip.contains('[')),
        "{:?}",
        texts_of(&screen, ".tips p")
    );
}

#[wasm_bindgen_test]
async fn a_figure_that_is_no_number_is_caught_on_the_screen_before_reaching_the_app() {
    answering(r#"{ "recent_days": [] }"#);
    let screen = screen();
    let _mounted = the_gather_tab(&screen);
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
    "usual_day": { "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 6.0 }, "burn_target_kcal": 77.14 }"#;

#[wasm_bindgen_test]
async fn the_recommendation_fills_each_card_on_demand() {
    answering(&format!(
        r#"{{ "profile_on": {{ "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null }},
             "goals_on": null, "fuel_targets_on": {{ "protein_g_per_kg": 2.025, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }},
             "usual_day_on": null, "hydrate_counted_on": true,
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

    for card in [".goals", ".fuel-targets", ".usual-day"] {
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
async fn the_burn_target_card_fills_from_the_recommendation_and_keeps_from_a_date() {
    answering(&format!(
        r#"{{ "profile_on": null, "goals_on": null, "usual_day_on": null, "hydrate_counted_on": true,
             "fuel_targets_on": {{ "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }},
             "burn_target_on": null, "recommended_on": {RECOMMENDED}, "set_burn_target": 77.14 }}"#
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

    clicked(&screen, ".burn-target button.recommend");
    settled().await;
    let filled = value_of(&screen, "burn_target");
    typed_into(&screen, "burn_target_from", "2026-10-15");
    found::<HtmlFormElement>(&screen, "form.burn-target")
        .request_submit()
        .expect("the target is kept");
    settled().await;

    assert_eq!(filled, "77.14");
    assert!(asked("set_burn_target").contains(r#""kcal":77.14"#));
    let said = text_of(&screen, "form.burn-target .said");
    assert!(said.contains("kept from 2026-10-15"), "{said}");
}

#[wasm_bindgen_test]
async fn before_any_weigh_in_the_burn_target_card_says_a_single_workout_holds_the_tick() {
    answering(
        r#"{ "profile_on": null, "goals_on": null, "usual_day_on": null, "hydrate_counted_on": true,
             "fuel_targets_on": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 },
             "burn_target_on": null }"#,
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

    let card = text_of(&screen, "form.burn-target");
    assert!(card.contains("a single workout holds the tick"), "{card}");
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
    assert_eq!((topics, pages), (6, 30));
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
             "hydrate_counted_on": true }"#,
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

fn naming_the_cast(said: &str) -> Vec<&'static str> {
    ["Veil", "Trix", "Grimble"]
        .into_iter()
        .filter(|name| said.contains(name))
        .collect()
}

fn spoken_on(screen: &Element) -> String {
    screen.text_content().unwrap_or_default()
}

#[wasm_bindgen_test]
async fn the_screens_keep_their_voice_without_naming_the_websites_cast() {
    answering(&format!(
        r#"{{ "profile_on": null, "goals_on": null, "usual_day_on": null,
             "fuel_targets_on": {{ "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }},
             "hydrate_counted_on": true,
             "foods": [], "meals": [], "exercises": [], "circuits": [], "priced_circuits": [],
             "burn_day": {AN_EMPTY_BURN}, "fuel_day": {A_DAY_NOT_FUELLED_YET},
             "plugin:opener|open_url": null, "explore": {TWO_MEASURES_EXPLORED} }}"#
    ));
    let mut said = Vec::new();

    let welcome = screen();
    let _welcome = the_app(&welcome);
    let charts = screen();
    let _charts = the_charts(&charts);
    said.push(spoken_on(&charts));
    settled().await;
    said.extend([spoken_on(&welcome), spoken_on(&charts)]);

    let fuel_shelf = screen();
    let _fuel_shelf = the_fuel_shelf(&fuel_shelf);
    let burn_shelf = screen();
    let _burn_shelf = the_burn_shelf(&burn_shelf);
    settled().await;
    clicked(&fuel_shelf, "button.form-Meal");
    clicked(&burn_shelf, "button.form-Circuit");
    settled().await;
    said.extend([spoken_on(&fuel_shelf), spoken_on(&burn_shelf)]);

    let burn = screen();
    let _burn = the_burn_day(&burn);
    settled().await;
    clicked(&burn, "button.burn-Circuit");
    settled().await;
    said.push(spoken_on(&burn));

    let profile = screen();
    let _profile = leptos::mount::mount_to(profile.clone().unchecked_into(), || {
        view! {
            <Router>
                <ProfileScreen />
            </Router>
        }
    });
    let references = screen();
    let _references = leptos::mount::mount_to(references.clone().unchecked_into(), || {
        view! {
            <Router>
                <ReferencesScreen />
            </Router>
        }
    });
    let stats = screen();
    let _stats = the_stats(&stats);
    settled().await;
    said.extend([&profile, &references, &stats].map(spoken_on));

    let named: Vec<Vec<&str>> = said.iter().map(|text| naming_the_cast(text)).collect();
    assert!(named.iter().all(Vec::is_empty), "{named:?}");
    assert!(
        texts_of(&profile, "h2").contains(&"How to feed the forge".to_string()),
        "{:?}",
        texts_of(&profile, "h2")
    );
}

#[wasm_bindgen_test]
async fn a_quote_stands_on_its_own_line_in_curly_marks() {
    answering(&format!(
        r#"{{ "plugin:opener|open_url": null, "explore": {TWO_MEASURES_EXPLORED} }}"#
    ));
    let references = screen();
    let _references = leptos::mount::mount_to(references.clone().unchecked_into(), || {
        view! {
            <Router>
                <ReferencesScreen />
            </Router>
        }
    });
    let stats = screen();
    let _stats = the_stats(&stats);
    settled().await;

    assert_eq!(
        [text_of(&references, "p.quote"), text_of(&stats, "p.quote")],
        ["“shhhhh! I’m reading...”", "“shhhhh! I’m hunting...”"]
    );
    assert!(
        [&references, &stats]
            .map(spoken_on)
            .iter()
            .all(|said| !said.contains('"')),
        "a straight quotation mark is left"
    );
}
