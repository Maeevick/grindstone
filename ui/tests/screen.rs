#![cfg(target_arch = "wasm32")]

use grind_ui::app::App;
use grind_ui::screens::Recorded;
use grind_ui::screens::burn::{BurnDay, BurnWeekCard};
use grind_ui::screens::catalog::{BurnShelf, FuelShelf, HydrateShelf};
use grind_ui::screens::charts::Charts;
use grind_ui::screens::fuel::{FuelDay, FuelWeekCard};
use grind_ui::screens::gather::GatherTab;
use grind_ui::screens::gauge::{AimGauge, BurnGauge, FuelGauge, Gauge};
use grind_ui::screens::hydrate::{HydrateDay, HydrateWeekCard};
use grind_ui::screens::last_days::FuelLastDays;
use grind_ui::screens::plate::PlateInProgress;
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
             "fuel_targets_on": { "values": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }, "recommended": false } }"#,
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
async fn the_profile_ends_past_a_divider_on_the_settings_the_drafts_records_then_the_danger_zone() {
    answering(
        r#"{ "profile_on": null, "goals_on": null, "usual_day_on": null,
             "fuel_targets_on": { "values": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }, "recommended": false },
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

    let headings = texts_in_order(&screen, "h2");
    assert_eq!(
        headings[headings.len().saturating_sub(3)..],
        ["Settings", "The draft's records", "Danger zone"],
        "{headings:?}"
    );
    found::<Element>(&screen, "section#draft button.import");
    assert!(screen.query_selector(".gear").ok().flatten().is_none());
    assert_eq!(
        text_of(&screen, "hr.divider + header.screen-top"),
        "Settings"
    );
}

#[wasm_bindgen_test]
async fn the_profile_asks_the_aches_the_day_owes_before_the_forgotten_bucket_of_sweat() {
    answering(
        r#"{ "profile_on": null, "goals_on": null, "usual_day_on": null,
             "fuel_targets_on": { "values": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }, "recommended": false },
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

    assert_eq!(
        texts_in_order(&screen, "h2")[..7],
        [
            "Profile",
            "The rusty you behind the numbers",
            "Where the grindstone is heading",
            "How to feed the forge",
            "Hydrate the material",
            "The aches the day owes",
            "The forgotten bucket of sweat"
        ]
    );
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
             "fuel_targets_on": { "values": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }, "recommended": false } }"#,
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

    for card in [
        ".goals",
        ".fuel-targets",
        ".water-target",
        ".usual-day",
        ".burn-target",
    ] {
        found::<Element>(
            &screen,
            &format!("{card} button.recommend ~ button[type=submit]"),
        );
    }
    for card in [
        ".goals",
        ".fuel-targets",
        ".water-target",
        ".usual-day",
        ".burn-target",
    ] {
        found::<Element>(
            &screen,
            &format!("{card} button[type=submit] ~ a.references"),
        );
    }
    assert!(screen.query_selector(".actions").ok().flatten().is_none());
    found::<Element>(&screen, ".disclaimer + hr.divider");
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
        r#"{ "profile_on": null, "goals_on": null, "set_goals": { "values": { "body_mass_kg": 70.0, "fat_ratio": 0.1 }, "recommended": false } }"#,
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
        r#"{ "profile_on": ["2026-03-01", { "birthday": "1990-06-01", "gender": "Female", "height_cm": 165.0, "neck_cm": 33.0, "hip_cm": 98.0 }],
             "goals_on": { "values": { "body_mass_kg": 60.0, "fat_ratio": 0.22 }, "recommended": false } }"#,
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

const A_MONTH_GATHERED: &str = r#"{ "month": "2026-10", "earlier": null, "rows": [
    { "day": "2026-10-10", "body_mass_kg": 80.0, "waist_cm": 80.0, "lean_mass_kg": 70.7, "fat_mass_kg": 9.3, "fat_ratio": 0.1162 },
    { "day": "2026-10-07", "body_mass_kg": 80.6, "waist_cm": null, "lean_mass_kg": 71.0, "fat_mass_kg": 9.6, "fat_ratio": 0.1158 } ] }"#;

const A_MONTH_NOT_GATHERED_YET: &str = r#"{ "month": "2026-10", "earlier": null, "rows": [] }"#;

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
async fn the_gathered_day_stands_on_a_line_for_its_body_mass_green_within_1_percent_and_one_for_its_lean_mass_green_from_its_goal_up_each_labelled_by_its_ends()
 {
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
    assert_eq!(
        texts_of(&screen, ".aim-gauge .tick-label"),
        ["Goal 66.0 kg", "Goal 75.0 kg"]
    );
    let body: Element = found(&screen, ".aim-gauge");
    assert_eq!(
        texts_in_order(&body, ".gauge-labels span"),
        ["67.5 kg", "82.5 kg"]
    );
    let lean: Element = screen
        .query_selector_all(".aim-gauge")
        .expect("the gauges")
        .item(1)
        .expect("the lean mass's gauge")
        .unchecked_into();
    let drawn = |selector: &str, attribute: &str| -> f64 {
        found::<Element>(&lean, selector)
            .get_attribute(attribute)
            .and_then(|value| value.parse().ok())
            .expect(attribute)
    };
    let band_from = drawn("rect.band", "x");
    let band_to = band_from + drawn("rect.band", "width");
    assert!(
        (band_from - drawn("line.optimal", "x1")).abs() < 1e-9 && (band_to - 360.0).abs() < 1e-9,
        "{band_from} to {band_to}"
    );
    assert_eq!(
        texts_in_order(&lean, ".gauge-labels span"),
        ["59.4 kg", "72.6 kg"]
    );
}

#[wasm_bindgen_test]
async fn where_you_stand_tells_the_body_masss_band_and_how_far_the_lean_mass_stands_from_its_goal()
{
    let titles_aiming_at = |aims: &'static str| async move {
        answering(&format!(
            r#"{{ "gather_week": {{ "day": {A_GATHERED_DAY}, "days": [{A_GATHERED_DAY}],
                  "tendency": {{ "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null }},
                  "aims": {aims} }} }}"#
        ));
        let screen = screen();
        let _mounted = the_gather_tab(&screen);
        settled().await;
        texts_in_order(&screen, ".aim-gauge .gauge-title")
    };

    let own_goals =
        titles_aiming_at(r#"{ "body_mass_kg": 70.0, "lean_mass_kg": 61.6, "recommended": false }"#)
            .await;
    let recommended =
        titles_aiming_at(r#"{ "body_mass_kg": 80.0, "lean_mass_kg": 72.0, "recommended": true }"#)
            .await;

    assert_eq!(
        [own_goals, recommended],
        [
            [
                "Body mass: 80.0 kg, the band runs from 69.3 to 70.7 kg.",
                "Lean mass: 70.7 kg, 9.1 kg above the 61.6 kg goal."
            ],
            [
                "Body mass: 80.0 kg, the band runs from 79.2 to 80.8 kg.",
                "Lean mass: 70.7 kg, 1.3 kg to reach the healthy 72.0 kg."
            ]
        ]
    );
}

#[wasm_bindgen_test]
async fn the_gap_a_line_tells_adds_up_with_the_figures_it_shows() {
    answering(&format!(
        r#"{{ "exercises": [], "circuits": [], "fuel_day": {A_DAY_NOT_FUELLED_YET},
             "burn_day": {} }}"#,
        A_BURN_SHORT_OF_ITS_TARGET
            .replace(r#""burnt_kcal": 60.0"#, r#""burnt_kcal": 372.0"#)
            .replace(r#""target_kcal": 80.0"#, r#""target_kcal": 76.5"#)
            .replace(r#""verdict": "Low""#, r#""verdict": "InBand""#)
    ));
    let burnt_on = screen();
    let burnt = the_burn_day(&burnt_on);
    settled().await;
    let burn_said = text_of(&burnt_on, ".bands p.figures");
    drop(burnt);
    answering(
        r#"{ "gather_week": { "day": { "day": "2026-10-10", "body_mass_kg": 70.5, "waist_cm": 80.0, "age": 41,
                 "composition": { "fat_ratio": 0.137, "fat_mass_kg": 9.66, "lean_mass_kg": 60.84 } },
              "days": [{ "day": "2026-10-10", "body_mass_kg": 70.5, "waist_cm": 80.0, "age": 41,
                 "composition": { "fat_ratio": 0.137, "fat_mass_kg": 9.66, "lean_mass_kg": 60.84 } }],
              "tendency": { "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null },
              "aims": { "body_mass_kg": 70.0, "lean_mass_kg": 61.56, "recommended": false } } }"#,
    );
    let gathered_on = screen();
    let _gathered = the_gather_tab(&gathered_on);
    settled().await;
    let lean_said = texts_in_order(&gathered_on, ".aim-gauge .gauge-title")
        .pop()
        .unwrap_or_default();

    assert_eq!(
        [burn_said, lean_said],
        [
            "372 kcal burnt in workouts, 295 kcal past the 77 kcal target.",
            "Lean mass: 60.8 kg, 0.8 kg to reach the 61.6 kg goal."
        ]
    );
}

async fn said_of_a_pace_past_the_safe_loss(lean_mass: &str) -> String {
    answering(&format!(
        r#"{{ "gather_week": {{ "day": {A_GATHERED_DAY}, "days": [{A_GATHERED_DAY}],
              "tendency": {{ "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null }},
              "aims": {{ "body_mass_kg": 70.0, "lean_mass_kg": 61.6, "recommended": false }},
              "pace": {{ "body_mass": {{ "by": -0.623, "heading": "Toward" }},
                         "lean_mass": {lean_mass},
                         "fat_mass": {{ "by": -0.4, "heading": "Toward" }},
                         "fat_ratio": {{ "by": -0.005, "heading": "Toward" }},
                         "judgement": "LosingTooFast", "safe_loss_kg": 0.498, "safe_loss_by": "FatStore" }} }},
              "gather_month": {{ "month": "2026-10", "earlier": null, "rows": [] }} }}"#
    ));
    let screen = screen();
    let _mounted = the_gather_tab(&screen);
    settled().await;
    text_of(&screen, ".pace p")
}

#[wasm_bindgen_test]
async fn past_the_safe_loss_the_warning_speaks_of_lean_mass_only_when_the_tape_sees_it_leave() {
    let leaving = said_of_a_pace_past_the_safe_loss(r#"{ "by": -0.2, "heading": "Away" }"#).await;
    let holding = said_of_a_pace_past_the_safe_loss(r#"{ "by": 0.24, "heading": "Toward" }"#).await;

    for said in [&leaving, &holding] {
        assert!(
            said.starts_with("“-0.62 kg a week, past the 0.50 kg your fat can spare"),
            "{said}"
        );
        assert!(!said.contains("floor"), "{said}");
    }
    assert!(
        leaving.contains("your lean mass is going too (-0.20 kg a week)"),
        "{leaving}"
    );
    assert!(
        holding.contains("on paper, though your lean mass holds so far")
            && !holding.contains("going too"),
        "{holding}"
    );
}

#[wasm_bindgen_test]
async fn the_weeks_pace_breaks_down_each_measure_colouring_its_figure_alone_and_judges_the_body_mass_in_a_plain_quoted_tip()
 {
    answering(&format!(
        r#"{{ "gather_week": {{ "day": {A_GATHERED_DAY}, "days": [{A_GATHERED_DAY}],
              "tendency": {{ "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null }},
              "aims": {{ "body_mass_kg": 75.0, "lean_mass_kg": 66.0, "recommended": false }},
              "pace": {{ "body_mass": {{ "by": -0.4, "heading": "Toward" }},
                         "lean_mass": {{ "by": -0.1, "heading": "Away" }},
                         "fat_mass": {{ "by": -0.3, "heading": "Toward" }},
                         "fat_ratio": {{ "by": 0.0, "heading": "Steady" }},
                         "judgement": "VeryGood", "safe_loss_kg": 0.48, "safe_loss_by": "FatStore" }} }} }}"#
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
    assert!(attributes_of(&screen, ".pace li", "class").is_empty());
    assert!(attributes_of(&screen, ".pace li > span", "class").is_empty());
    assert_eq!(
        texts_in_order(&screen, ".pace li span span"),
        ["-0.4 kg", "-0.1 kg", "-0.3 kg", "steady"]
    );
    assert_eq!(
        attributes_of(&screen, ".pace li span span", "class"),
        ["toward", "away", "toward", "steady"]
    );
    let tip = text_of(&screen, ".pace p");
    assert_eq!(attributes_of(&screen, ".pace p", "class"), ["tip"]);
    assert!(tip.starts_with('“') && tip.ends_with('”'), "{tip}");
}

const A_WEEK_NOT_GATHERED_YET: &str = r#"{ "day": null, "days": [],
    "tendency": { "body_mass": null, "waist": null, "fat_ratio": null, "lean_mass": null } }"#;

fn the_gather_tab(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        provide_context(Recorded::default());
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
async fn the_last_days_of_gather_list_each_day_under_its_units_the_chosen_one_highlighted() {
    answering(&format!(
        r#"{{ "gather_week": {}, "gather_month": {A_MONTH_GATHERED} }}"#,
        a_gathered_week()
    ));
    let screen = screen();
    let _mounted = the_gather_tab(&screen);
    settled().await;

    typed_into(&screen, "day", "2026-10-10");
    settled().await;

    let heads = text_of(&screen, ".last-days thead");
    for unit in ["Mass (kg)", "Waist (cm)", "Fat (%)", "Lean (kg)"] {
        assert!(heads.contains(unit), "{unit} in {heads}");
    }
    assert!(text_of(&screen, ".last-days tr.chosen").contains("2026-10-10"));
    let listed = text_of(&screen, ".last-days tbody");
    assert!(
        listed.contains("80.6") && !listed.contains(" kg"),
        "{listed}"
    );
}

#[wasm_bindgen_test]
async fn after_logging_a_measurement_the_day_stays_its_form_shows_it_and_it_joins_the_last_days() {
    answering(&format!(
        r#"{{ "gather_week": {A_WEEK_NOT_GATHERED_YET}, "gather_month": {A_MONTH_NOT_GATHERED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_gather_tab(&screen);
    settled().await;
    typed_into(&screen, "day", "2026-10-10");
    settled().await;

    answering(&format!(
        r#"{{ "gather_week": {}, "log_measurement": {}, "gather_month": {A_MONTH_GATHERED} }}"#,
        a_gathered_week(),
        a_gathered_week()
    ));
    typed_into(&screen, "mass", "80");
    typed_into(&screen, "waist", "80");
    found::<HtmlFormElement>(&screen, "form.measurement")
        .request_submit()
        .expect("the measurement is logged");
    settled().await;
    settled().await;

    let listed = text_of(&screen, ".last-days tbody");
    assert!(
        listed.contains("80.0") && listed.contains("11.6"),
        "{listed}"
    );
    assert_eq!(
        [value_of(&screen, "mass"), value_of(&screen, "waist")],
        ["80", "80"]
    );
    assert_eq!(value_of(&screen, "day"), "2026-10-10");
}

#[wasm_bindgen_test]
async fn the_days_weigh_in_removed_from_its_card_leaves_the_day_ungathered() {
    answering(&format!(
        r#"{{ "gather_week": {}, "gather_month": {A_MONTH_GATHERED},
              "remove_measurement": {A_WEEK_NOT_GATHERED_YET} }}"#,
        a_gathered_week()
    ));
    let screen = screen();
    let _mounted = the_gather_tab(&screen);
    settled().await;
    typed_into(&screen, "day", "2026-10-10");
    settled().await;

    clicked(&screen, "button.remove-weigh-in");
    settled().await;

    assert!(asked("remove_measurement").contains("2026-10-10"));
    assert_eq!(
        [value_of(&screen, "mass"), value_of(&screen, "waist")],
        ["", ""]
    );
    assert!(
        screen
            .query_selector("button.remove-weigh-in")
            .ok()
            .flatten()
            .is_none()
    );
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
    "body_mass_kg": 80.0, "target_kcal": 80.0, "verdict": "Low" }"#;

#[wasm_bindgen_test]
async fn the_days_burn_says_what_its_workouts_burnt_and_how_far_it_stands_from_its_target() {
    let said_of = |burn_day: String| async move {
        answering(&format!(
            r#"{{ "exercises": [], "circuits": [], "burn_day": {burn_day},
                 "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
        ));
        let screen = screen();
        let _mounted = the_burn_day(&screen);
        settled().await;
        text_of(&screen, ".bands p.figures")
    };

    let short = said_of(A_BURN_SHORT_OF_ITS_TARGET.to_string()).await;
    let past = said_of(
        A_BURN_SHORT_OF_ITS_TARGET
            .replace(r#""burnt_kcal": 60.0"#, r#""burnt_kcal": 100.0"#)
            .replace(r#""verdict": "Low""#, r#""verdict": "InBand""#),
    )
    .await;
    let reached_as_shown = said_of(
        A_BURN_SHORT_OF_ITS_TARGET
            .replace(r#""burnt_kcal": 60.0"#, r#""burnt_kcal": 79.6"#)
            .replace(r#""verdict": "Low""#, r#""verdict": "InBand""#),
    )
    .await;

    assert_eq!(
        [short, past, reached_as_shown],
        [
            "60 kcal burnt in workouts, 20 kcal to reach the 80 kcal target.",
            "100 kcal burnt in workouts, 20 kcal past the 80 kcal target.",
            "80 kcal burnt in workouts, 0 kcal past the 80 kcal target."
        ]
    );
}

#[wasm_bindgen_test]
async fn the_days_burn_line_is_red_short_of_its_target_and_green_once_it_reaches_it() {
    let class_of = |burn_day: String| async move {
        answering(&format!(
            r#"{{ "exercises": [], "circuits": [], "burn_day": {burn_day},
                 "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
        ));
        let screen = screen();
        let _mounted = the_burn_day(&screen);
        settled().await;
        found::<Element>(&screen, ".bands p.figures").class_name()
    };

    let short = class_of(A_BURN_SHORT_OF_ITS_TARGET.to_string()).await;
    let reached = class_of(
        A_BURN_SHORT_OF_ITS_TARGET
            .replace(r#""burnt_kcal": 60.0"#, r#""burnt_kcal": 100.0"#)
            .replace(r#""verdict": "Low""#, r#""verdict": "InBand""#),
    )
    .await;

    assert_eq!([short, reached], ["figures low", "figures good"]);
}

#[wasm_bindgen_test]
async fn the_days_burn_shows_its_band_from_nothing_the_target_tick_and_green_from_the_target_on() {
    answering(&format!(
        r#"{{ "exercises": [], "circuits": [], "burn_day": {A_BURN_SHORT_OF_ITS_TARGET},
             "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    let band: Element = found(&screen, ".burn-gauge svg .band");

    assert_eq!(band.get_attribute("x").as_deref(), Some("144"));
    assert_eq!(
        text_of(&screen, ".burn-gauge .tick-label"),
        "Target 80 kcal"
    );
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
async fn a_circuit_chosen_in_the_workout_form_asks_its_sets_alone_and_previews_their_burn() {
    answering(&format!(
        r#"{{ "exercises": [], "circuits": [{MORNING_RUST}],
             "burn_day": {A_BURN_PRICED_AT_80_KG}, "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    typed_into(&screen, "exercise", "morning rust");
    settled().await;
    let one_set = text_of(&screen, "form.workout p.preview");
    typed_into(&screen, "sets", "3");
    settled().await;

    assert_eq!(
        attributes_of(&screen, "form.workout input", "name"),
        ["exercise", "sets"]
    );
    assert_eq!(
        [one_set, text_of(&screen, "form.workout p.preview")],
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
async fn an_exercise_nothing_matches_turns_the_workout_form_into_a_new_one_added_then_logged_in_one_tap()
 {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Walking", "met": 4.0 }}], "circuits": [],
             "burn_day": {A_BURN_PRICED_AT_80_KG}, "fuel_day": {A_DAY_NOT_FUELLED_YET},
             "add_exercise": [{{ "name": "Rowing", "met": 7.0, "description": "Concept2, damper on 5" }},
                              {{ "name": "Walking", "met": 4.0 }}],
             "log_workout": {AN_EMPTY_BURN} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    typed_into(&screen, "exercise", "Rowing");
    settled().await;
    assert_eq!(texts_in_order(&screen, "form.workout .new"), ["New Rowing"]);
    typed_into(&screen, "new_exercise_description", "Concept2, damper on 5");
    typed_into(&screen, "new_exercise_met", "7");
    typed_into(&screen, "minutes", "20");
    settled().await;
    let previewed = text_of(&screen, "form.workout p.preview");
    let offered = text_of(&screen, "form.workout button[type=submit]");
    found::<HtmlFormElement>(&screen, "form.workout")
        .request_submit()
        .expect("the new exercise is added, then logged");
    settled().await;
    settled().await;

    assert_eq!(offered, "Add and log");
    assert!(previewed.contains("168 kcal"), "{previewed}");
    let added = asked("add_exercise");
    for sent in [
        r#""name":"Rowing""#,
        r#""met":7"#,
        r#""description":"Concept2, damper on 5""#,
    ] {
        assert!(added.contains(sent), "{sent} in {added}");
    }
    let logged = asked("log_workout");
    assert!(
        logged.contains(r#""exercise":"Rowing""#) && logged.contains(r#""minutes":20"#),
        "{logged}"
    );
    assert!(texts_in_order(&screen, "form.workout .new").is_empty());
}

#[wasm_bindgen_test]
async fn the_burn_card_offers_a_workout_or_the_day_off_the_clock_on_its_pills() {
    answering(&format!(
        r#"{{ "exercises": [{{ "name": "Walking", "met": 4.0 }}], "circuits": [{MORNING_RUST}],
             "burn_day": {AN_EMPTY_BURN}, "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;
    let shown = |selector: &str| screen.query_selector(selector).ok().flatten().is_some();
    let pills = texts_in_order(&screen, "button.pill");
    let at_first = shown("form.workout") && !shown("form.off-the-clock");

    clicked(&screen, "button.burn-OffTheClock");
    settled().await;

    assert_eq!(pills, ["Workout", "Off the clock"]);
    assert!(at_first);
    assert!(shown("form.off-the-clock") && !shown("form.workout"));
    assert_eq!(value_of(&screen, "sitting"), "480");
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
        provide_context(PlateInProgress::default());
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
        r#"{ "exercises": [], "circuits": [],
             "add_exercise": [{ "name": "Kettlebell swing", "met": 9.8, "description": "Both hands, a hip hinge" }] }"#,
    );
    let screen = screen();
    let _mounted = the_burn_shelf(&screen);
    settled().await;
    assert_eq!(tables_in_order(&screen), ["exercises"]);
    found::<Element>(&screen, "button.shelf-Exercises.chosen");
    found::<Element>(&screen, "button.shelf-Circuits");
    let swing =
        r#"[{ "name": "Kettlebell swing", "met": 9.8, "description": "Both hands, a hip hinge" }]"#;
    answering(&format!(
        r#"{{ "exercises": {swing}, "circuits": [], "add_exercise": {swing} }}"#
    ));

    typed_into(&screen, "exercise_name", "Kettlebell swing");
    typed_into(&screen, "exercise_description", "Both hands, a hip hinge");
    typed_into(&screen, "exercise_met", "9.8");
    found::<HtmlFormElement>(&screen, "form.exercise")
        .request_submit()
        .expect("the exercise is added");
    settled().await;

    assert!(asked("add_exercise").contains(r#""description":"Both hands, a hip hinge""#));
    let rows = text_of(&screen, "table.exercises tbody");
    assert!(rows.contains("9.8"), "{rows}");
    assert_eq!(
        [
            text_of(&screen, "table.exercises tbody .name"),
            text_of(&screen, "table.exercises tbody .description")
        ],
        ["Kettlebell swing", "Both hands, a hip hinge"]
    );
    assert_eq!(value_of(&screen, "exercise_name"), "");
}

#[wasm_bindgen_test]
async fn every_gauge_labels_its_tick_under_it_in_gold_capitalised_and_anchored_to_stay_on_the_gauge()
 {
    let a_cut = grind_core::fuel::Band {
        minimum_kcal_per_kg_of_lean_mass: 25.0,
        minimum_kcal: 1767.5,
        floor_kcal: 2199.4,
        cap_kcal: 2650.2,
        maintenance_kcal: 2650.2,
    };
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), move || {
        view! {
            <AimGauge
                label="Body mass"
                amount=80.0
                aim=grind_core::heading::Aim::Reach(75.0)
                recommended=false
            />
            <BurnGauge burnt=60.0 target=80.0 ghost=None />
            <Gauge
                amount=1500.0
                floor=1800.0
                optimal=2000.0
                cap=2200.0
                shown_as=grind_core::figures::Figure::Volume
            />
            <FuelGauge eaten=108.0 band=a_cut ghost=None />
        }
    });
    settled().await;

    let gauges = screen.query_selector_all(".gauge").expect("the gauges");
    let labelled: Vec<(String, bool, String, String)> = (0..gauges.length())
        .filter_map(|at| gauges.item(at))
        .map(|gauge| {
            let gauge: Element = gauge.unchecked_into();
            let share = 100.0
                * found::<Element>(&gauge, "line.optimal, line.maintenance")
                    .get_attribute("x1")
                    .and_then(|x| x.parse::<f64>().ok())
                    .expect("a tick")
                / 360.0;
            let label = gauge
                .query_selector(".gauge-under-ticks .tick-label")
                .ok()
                .flatten();
            (
                label
                    .as_ref()
                    .and_then(|label| label.text_content())
                    .unwrap_or_default(),
                label
                    .as_ref()
                    .is_some_and(|label| label.class_list().contains("acc")),
                label
                    .and_then(|label| label.get_attribute("style"))
                    .unwrap_or_default(),
                format!("left: {share}%; transform: translateX(-{share}%)"),
            )
        })
        .collect();

    assert_eq!(
        labelled
            .iter()
            .map(|(text, ..)| text.as_str())
            .collect::<Vec<_>>(),
        [
            "Goal 75.0 kg",
            "Target 80 kcal",
            "Optimal 2000 mL",
            "Maintenance 2650"
        ]
    );
    assert!(
        labelled
            .iter()
            .all(|(_, gold, style, under_its_tick)| *gold && style == under_its_tick),
        "{labelled:?}"
    );
}

const A_FUELLED_DAY: &str = r#"{ "day": "2026-10-10",
    "intakes": [[1, { "day": "2026-10-10", "food": "Skyr", "unit": "Gram", "quantity": 200.0,
        "nutrients": { "energy_kcal": 108.0, "protein_g": 19.0, "carbs_g": 8.0, "fat_g": 0.0, "fibre_g": 0.0 } }]],
    "eaten": { "energy_kcal": 108.0, "protein_g": 19.0, "carbs_g": 8.0, "fat_g": 0.0, "fibre_g": 0.0 },
    "band": { "minimum_kcal_per_kg_of_lean_mass": 25.0, "minimum_kcal": 1767.5, "floor_kcal": 2199.4, "cap_kcal": 2650.2, "maintenance_kcal": 2424.8 },
    "energy": "Low",
    "macros": {
        "protein": { "eaten_g": 19.0, "floor_g": 160.0, "ceiling_g": 9.0, "verdict": "Low" },
        "fat": { "eaten_g": 0.0, "floor_g": 72.0, "ceiling_g": 4.0, "verdict": "Low" },
        "fibre": { "eaten_g": 0.0, "floor_g": 32.0, "ceiling_g": 2.2, "verdict": "Low" },
        "carbs": { "eaten_g": 8.0, "floor_g": 212.1, "ceiling_g": 295.0, "verdict": "Low" } } }"#;

const A_FUELLED_DAY_WITH_ITS_BUDGET: &str = r#"{ "day": "2026-10-10", "intakes": [],
    "eaten": { "energy_kcal": 0.0, "protein_g": 0.0, "carbs_g": 0.0, "fat_g": 0.0, "fibre_g": 0.0 },
    "band": { "minimum_kcal_per_kg_of_lean_mass": 25.0, "minimum_kcal": 1767.5, "floor_kcal": 2199.4, "cap_kcal": 2650.2, "maintenance_kcal": 2650.2 },
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

fn the_fuel_week(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        let day = RwSignal::new("2026-10-10".to_string());
        view! {
            <Router>
                <FuelWeekCard day=day />
            </Router>
        }
    })
}

const A_FUEL_WEEK: &str = r#"{ "logged_days": 6,
    "energy": { "per_day": 2100.0, "floor": 1800.0, "ceiling": 2400.0, "in_band": 4 },
    "protein": { "per_day": 96.0, "floor": 115.0, "ceiling": 190.0, "in_band": 1 },
    "fat": { "per_day": 71.0, "floor": 48.0, "ceiling": 57.0, "in_band": 2 },
    "fibre": { "per_day": 19.0, "floor": 25.0, "ceiling": 60.0, "in_band": 0 },
    "carbs": { "per_day": 330.0, "floor": 200.0, "ceiling": 310.0, "in_band": 2 } }"#;

#[wasm_bindgen_test]
async fn the_weeks_average_tells_each_measure_a_day_against_its_band_and_how_many_days_stood_in_it()
{
    answering(&format!(r#"{{ "fuel_week": {A_FUEL_WEEK} }}"#));
    let screen = screen();
    let _mounted = the_fuel_week(&screen);
    settled().await;

    assert_eq!(text_of(&screen, ".week h2"), "The week's average");
    assert_eq!(
        cells_by_row(&screen, ".week tbody tr"),
        [
            ["Energy (kcal)", "2100", "1800–2400", "4/6"],
            ["Protein (g)", "96", "115–190", "1/6"],
            ["Fat (g)", "71", "48–57", "2/6"],
            ["Fibre (g)", "19", "25–60", "0/6"],
            ["Carbs (g)", "330", "200–310", "2/6"]
        ]
    );
    assert_eq!(
        attributes_of(&screen, ".week tbody td:nth-child(2)", "class"),
        ["good", "low", "high", "low", "high"]
    );
    assert_eq!(text_of(&screen, ".week p.hint"), "Over 6 logged days.");
}

#[wasm_bindgen_test]
async fn the_weeks_tip_names_what_runs_short_then_what_runs_over_furthest_first_between_quotation_marks()
 {
    answering(&format!(r#"{{ "fuel_week": {A_FUEL_WEEK} }}"#));
    let screen = screen();
    let _mounted = the_fuel_week(&screen);
    settled().await;

    let tip = text_of(&screen, ".week p.tip");

    assert!(
        tip.starts_with(
            "“Short on fibre (-6 g a day) and protein (-19 g), over on fat (+14 g) and carbs (+20 g). "
        ) && tip.ends_with('”'),
        "{tip}"
    );
}

#[wasm_bindgen_test]
async fn a_week_without_a_finished_logged_day_says_there_is_nothing_to_average_yet() {
    answering(r#"{ "fuel_week": null }"#);
    let screen = screen();
    let _mounted = the_fuel_week(&screen);
    settled().await;

    let said = text_of(&screen, ".week p.empty");

    assert!(said.contains("No finished day"), "{said}");
}

fn the_hydrate_week(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        let day = RwSignal::new("2026-10-10".to_string());
        view! {
            <Router>
                <HydrateWeekCard day=day />
            </Router>
        }
    })
}

#[wasm_bindgen_test]
async fn hydrates_week_average_tells_a_days_drinks_against_the_band_its_days_in_it_and_how_far_it_runs_short()
 {
    answering(
        r#"{ "hydrate_week": { "logged_days": 6,
            "drunk": { "per_day": 1900.0, "floor": 2000.0, "ceiling": 2400.0, "in_band": 3 } } }"#,
    );
    let screen = screen();
    let _mounted = the_hydrate_week(&screen);
    settled().await;

    assert_eq!(text_of(&screen, ".week h2"), "The week's average");
    assert_eq!(
        cells_by_row(&screen, ".week tbody tr"),
        [["Drinks (mL)", "1900", "2000–2400", "3/6"]]
    );
    assert_eq!(
        attributes_of(&screen, ".week tbody td:nth-child(2)", "class"),
        ["low"]
    );
    assert_eq!(text_of(&screen, ".week p.hint"), "Over 6 logged days.");
    let tip = text_of(&screen, ".week p.tip");
    assert!(
        tip.starts_with("“Short of the band by 100 mL a day. ") && tip.ends_with('”'),
        "{tip}"
    );
}

#[wasm_bindgen_test]
async fn a_hydrate_week_without_a_finished_logged_day_says_there_is_nothing_to_average_yet() {
    answering(r#"{ "hydrate_week": null }"#);
    let screen = screen();
    let _mounted = the_hydrate_week(&screen);
    settled().await;

    let said = text_of(&screen, ".week p.empty");

    assert!(said.contains("No finished day"), "{said}");
}

fn the_burn_week(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        let day = RwSignal::new("2026-10-10".to_string());
        view! {
            <Router>
                <BurnWeekCard day=day />
            </Router>
        }
    })
}

const A_BURN_WEEK: &str = r#"{ "active_days": 3, "burnt_kcal": 458.0, "target_kcal": 308.0, "exercises": [
    { "exercise": "Walking", "times": 2, "minutes": 90.0, "reps": null, "burnt_kcal": 378.0 },
    { "exercise": "Push up", "times": 2, "minutes": 3.0, "reps": 72.0, "burnt_kcal": 80.0 } ] }"#;

#[wasm_bindgen_test]
async fn burns_week_sum_up_totals_each_exercise_with_its_average_a_time_beneath() {
    answering(&format!(r#"{{ "burn_week": {A_BURN_WEEK} }}"#));
    let screen = screen();
    let _mounted = the_burn_week(&screen);
    settled().await;

    assert_eq!(text_of(&screen, ".week h2"), "The week's sum-up");
    assert_eq!(
        texts_in_order(&screen, ".week tbody td:first-child"),
        ["Walking", "Push up"]
    );
    assert_eq!(
        texts_in_order(&screen, ".week tbody td:nth-child(2)"),
        ["2", "2"]
    );
    assert_eq!(
        texts_in_order(&screen, ".week tbody .total"),
        ["90", "–", "378", "3", "72", "80"]
    );
    assert_eq!(
        texts_in_order(&screen, ".week tbody .a-time"),
        ["45", "", "189", "1.5", "36", "40"]
    );
}

#[wasm_bindgen_test]
async fn burns_week_sum_up_holds_the_weeks_burn_against_its_target_over_its_active_days() {
    answering(&format!(r#"{{ "burn_week": {A_BURN_WEEK} }}"#));
    let screen = screen();
    let _mounted = the_burn_week(&screen);
    settled().await;

    assert_eq!(
        text_of(&screen, ".week p.week-burn"),
        "458 kcal burnt against the week's 308 kcal target, over 3 active days."
    );
    assert_eq!(
        attributes_of(&screen, ".week p.week-burn span", "class"),
        ["good"]
    );
}

#[wasm_bindgen_test]
async fn a_week_without_a_workout_says_there_is_nothing_to_sum_up_yet() {
    answering(r#"{ "burn_week": null }"#);
    let screen = screen();
    let _mounted = the_burn_week(&screen);
    settled().await;

    let said = text_of(&screen, ".week p.empty");

    assert!(said.contains("No workout"), "{said}");
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
async fn the_food_chooser_offers_meals_then_foods_each_in_alphabetical_order_behind_its_icon() {
    answering(&format!(
        r#"{{ "foods": {PASTA_AND_CREAM}, "meals": [{PASTA_IN_CREAM}], "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    focused(&screen, "food");
    settled().await;
    let offered = choices(&screen);
    picked(&screen, "🥦 Pasta");
    settled().await;

    assert_eq!(
        offered,
        ["💪 Pasta in cream", "🥦 Crème fraîche", "🥦 Pasta"]
    );
    assert_eq!(value_of(&screen, "food"), "Pasta");
}

#[wasm_bindgen_test]
async fn the_exercise_chooser_offers_circuits_then_exercises_each_in_alphabetical_order_behind_its_icon()
 {
    answering(&format!(
        r#"{{ "exercises": {BURPEES_AND_PUSH_UPS}, "circuits": [{MORNING_RUST}],
             "burn_day": {AN_EMPTY_BURN}, "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    focused(&screen, "exercise");
    settled().await;
    let offered = choices(&screen);
    picked(&screen, "🔄 Morning rust");
    settled().await;

    assert_eq!(offered, ["🔄 Morning rust", "💪 Burpee", "💪 Push up"]);
    assert_eq!(value_of(&screen, "exercise"), "Morning rust");
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
    assert_eq!(choices(&screen), vec!["🥦 Skyr".to_string()]);

    picked(&screen, "🥦 Skyr");
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
async fn a_new_food_typed_previews_the_day_with_it_as_its_figures_change() {
    answering(&format!(
        r#"{{ "foods": {THREE_FOODS}, "fuel_day": {}, "fuel_day_with": {} }}"#,
        a_fuelled_day_of(108.0, 21.6, (2199.4, 2424.8, 2650.2)),
        a_fuelled_day_of(268.0, 43.0, (2220.8, 2446.2, 2671.6))
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    typed_into(&screen, "food", "Kefir");
    settled().await;
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
    settled().await;
    let a_quarter_litre = asked("fuel_day_with");
    let said = texts_in_order(&screen, ".bands .with-the-preview").concat();
    typed_into(&screen, "quantity", "500");
    settled().await;
    settled().await;
    let half_a_litre = asked("fuel_day_with");

    assert!(
        a_quarter_litre.contains(r#""more":{"energy_kcal":160"#)
            && half_a_litre.contains(r#""more":{"energy_kcal":320"#),
        "{a_quarter_litre} then {half_a_litre}"
    );
    assert_eq!(
        said,
        "With the food: 268 kcal, 21 of them spent digesting it."
    );
}

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
async fn the_energy_gauge_ticks_maintenance_where_it_falls_in_the_band() {
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
            && gauge.contains("Maintenance 2425")
            && !gauge.contains("Alpert"),
        "{gauge}"
    );
    found::<Element>(&screen, ".fuel-gauge line.minimum");
    found::<Element>(&screen, ".fuel-gauge rect.band");
    found::<Element>(&screen, ".fuel-gauge circle.eaten");
    assert_eq!(
        text_of(&screen, ".fuel-gauge .tick-label"),
        "Maintenance 2425"
    );
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

fn fuelled_on(date: &str, kcal: f64) -> String {
    format!(
        r#"{{ "day": "{date}", "eaten": {{ "energy_kcal": {kcal}, "protein_g": 100.0, "carbs_g": 200.0,
            "fat_g": 60.0, "fibre_g": 30.0 }} }}"#
    )
}

#[wasm_bindgen_test]
async fn the_last_days_show_the_week_up_to_the_day_then_the_rest_of_its_month_then_the_month_before()
 {
    answering(&format!(
        r#"{{ "fuel_month": {{ "__by": "month",
            "2026-10": {{ "month": "2026-10", "earlier": "2026-08", "rows": [{}, {}, {}, {}] }},
            "2026-08": {{ "month": "2026-08", "earlier": null, "rows": [{}, {}] }} }} }}"#,
        fuelled_on("2026-10-10", 2400.0),
        fuelled_on("2026-10-09", 2300.0),
        fuelled_on("2026-10-05", 2200.0),
        fuelled_on("2026-10-02", 2100.0),
        fuelled_on("2026-08-31", 2000.0),
        fuelled_on("2026-08-15", 1900.0),
    ));
    let screen = screen();
    let _mounted = leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        let day = RwSignal::new("2026-10-10".to_string());
        view! {
            <Router>
                <FuelLastDays day=day />
            </Router>
        }
    });
    settled().await;
    let days = |screen: &Element| texts_in_order(screen, ".last-days tbody tr td:first-child");

    let a_week = days(&screen);
    let first_more = text_of(&screen, ".last-days button.more");
    clicked(&screen, ".last-days button.more");
    settled().await;
    let the_month = days(&screen);
    let then_more = text_of(&screen, ".last-days button.more");
    clicked(&screen, ".last-days button.more");
    settled().await;

    assert_eq!(a_week, ["2026-10-10", "2026-10-09", "2026-10-05"]);
    assert_eq!(first_more, "Load the rest of October 2026");
    assert_eq!(
        the_month,
        ["2026-10-10", "2026-10-09", "2026-10-05", "2026-10-02"]
    );
    assert_eq!(then_more, "Load August 2026");
    assert_eq!(
        days(&screen),
        [
            "2026-10-10",
            "2026-10-09",
            "2026-10-05",
            "2026-10-02",
            "2026-08-31",
            "2026-08-15"
        ]
    );
    assert!(
        screen
            .query_selector(".last-days button.more")
            .ok()
            .flatten()
            .is_none()
    );
    assert!(text_of(&screen, ".last-days tr.chosen").contains("2026-10-10"));
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
            "nutrients": { "energy_kcal": 54.0, "protein_g": 9.5, "carbs_g": 4.0, "fat_g": 0.0, "fibre_g": 0.0 },
            "description": "Siggi's, the plain 0% pot" }] }"#,
    );
    let screen = screen();
    let _mounted = the_fuel_shelf(&screen);
    settled().await;
    assert_eq!(tables_in_order(&screen), ["foods"]);
    found::<Element>(&screen, "button.shelf-Foods.chosen");
    found::<Element>(&screen, "button.shelf-Meals");
    assert_eq!(
        fields_in_rows(&screen, "form.food"),
        [
            vec!["food_name"],
            vec!["food_description"],
            vec!["food_portion", "food_unit", "food_energy"],
            vec!["food_protein", "food_fat", "food_fibre", "food_carbs"],
        ]
    );
    let skyr = r#"[{ "name": "Skyr", "unit": "Gram", "portion": 100.0,
        "nutrients": { "energy_kcal": 54.0, "protein_g": 9.5, "carbs_g": 4.0, "fat_g": 0.0, "fibre_g": 0.0 },
        "description": "Siggi's, the plain 0% pot" }]"#;
    answering(&format!(
        r#"{{ "foods": {skyr}, "meals": [], "add_food": {skyr} }}"#
    ));

    typed_into(&screen, "food_name", "Skyr");
    typed_into(&screen, "food_description", "Siggi's, the plain 0% pot");
    typed_into(&screen, "food_portion", "100");
    typed_into(&screen, "food_energy", "54");
    found::<HtmlFormElement>(&screen, "form.food")
        .request_submit()
        .expect("the food is added");
    settled().await;

    assert!(asked("add_food").contains(r#""description":"Siggi's, the plain 0% pot""#));
    let row = texts_of(&screen, "table.foods tbody tr td");
    for cell in ["100 g", "54", "10", "0", "4"] {
        assert!(row.contains(&cell.to_string()), "{cell} in {row:?}");
    }
    assert_eq!(
        [
            text_of(&screen, "table.foods tbody .name"),
            text_of(&screen, "table.foods tbody .description")
        ],
        ["Skyr", "Siggi's, the plain 0% pot"]
    );
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

#[wasm_bindgen_test]
async fn a_kept_meal_shows_its_description_and_ingredients_on_a_click_on_the_shelf() {
    answering(&format!(
        r#"{{ "foods": {PASTA_AND_CREAM}, "meals": [{PASTA_IN_CREAM}] }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_shelf(&screen);
    settled().await;
    assert!(screen.query_selector("form.meal").ok().flatten().is_none());

    clicked(&screen, "button.shelf-Meals");
    settled().await;

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

async fn added_to_the_plate(screen: &Element, food: &str, quantity: &str) {
    typed_into(screen, "food", food);
    typed_into(screen, "quantity", quantity);
    clicked(screen, "button.to-the-plate");
    settled().await;
}

/// A day of Fuel as the core would answer it: so much eaten, its band and what its digestion costs.
fn a_fuelled_day_of(
    kcal: f64,
    digestion_kcal: f64,
    (floor, maintenance, cap): (f64, f64, f64),
) -> String {
    format!(
        r#"{{ "day": "2026-10-10", "intakes": [],
            "eaten": {{ "energy_kcal": {kcal}, "protein_g": 0.0, "carbs_g": 0.0, "fat_g": 0.0, "fibre_g": 0.0 }},
            "band": {{ "minimum_kcal_per_kg_of_lean_mass": 25.0, "minimum_kcal": 1767.5,
                       "floor_kcal": {floor}, "cap_kcal": {cap}, "maintenance_kcal": {maintenance} }},
            "energy": "Low", "macros": null,
            "budget": {{ "resting_kcal": 1718.1, "organs_kcal": 102.5, "digestion_kcal": {digestion_kcal},
                         "spontaneous_kcal": 234.0, "volitional_kcal": 197.52 }} }}"#
    )
}

const A_FUELLED_DAY_WITH_THE_PASTA: &str = r#"{ "day": "2026-10-10", "intakes": [],
    "eaten": { "energy_kcal": 468.0, "protein_g": 31.0, "carbs_g": 78.0, "fat_g": 1.5, "fibre_g": 3.0 },
    "band": { "minimum_kcal_per_kg_of_lean_mass": 25.0, "minimum_kcal": 1767.5, "floor_kcal": 2235.3, "cap_kcal": 2686.1, "maintenance_kcal": 2460.7 },
    "energy": "Low",
    "macros": {
        "protein": { "eaten_g": 31.0, "floor_g": 160.0, "ceiling_g": 39.0, "verdict": "Low" },
        "fat": { "eaten_g": 1.5, "floor_g": 72.0, "ceiling_g": 17.3, "verdict": "Low" },
        "fibre": { "eaten_g": 3.0, "floor_g": 32.0, "ceiling_g": 9.4, "verdict": "Low" },
        "carbs": { "eaten_g": 78.0, "floor_g": 220.0, "ceiling_g": 302.0, "verdict": "Low" } } }"#;

#[wasm_bindgen_test]
async fn the_macros_preview_their_gaps_against_the_goals_of_the_day_previewed() {
    answering(&format!(
        r#"{{ "foods": {PASTA_AND_CREAM}, "fuel_day": {A_FUELLED_DAY}, "fuel_day_with": {A_FUELLED_DAY_WITH_THE_PASTA} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    typed_into(&screen, "food", "pasta");
    typed_into(&screen, "quantity", "100");
    settled().await;
    settled().await;

    assert_eq!(
        texts_in_order(&screen, "table.macros tr.carbs .ghost"),
        ["+70", "-142"]
    );
    assert_eq!(
        attributes_of(
            &screen,
            "table.macros tr.carbs td:nth-child(4) .ghost",
            "class"
        ),
        ["ghost low"]
    );
}

#[wasm_bindgen_test]
async fn what_is_about_to_be_eaten_previews_the_day_with_it_its_band_lifted_by_its_digestion() {
    let today = a_fuelled_day_of(108.0, 21.6, (2199.4, 2424.8, 2650.2));
    let answering_with = |previewed: String| {
        answering(&format!(
            r#"{{ "foods": {PASTA_AND_CREAM}, "fuel_day": {today}, "fuel_day_with": {previewed} }}"#
        ))
    };
    answering_with(a_fuelled_day_of(468.0, 57.5, (2235.3, 2460.7, 2686.1)));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;
    let previewed = || texts_in_order(&screen, ".bands .with-the-preview").concat();

    typed_into(&screen, "food", "pasta");
    typed_into(&screen, "quantity", "100");
    settled().await;
    settled().await;
    let the_food = previewed();
    let sent = asked("fuel_day_with");
    let ghost_band = screen
        .query_selector(".fuel-gauge rect.ghost-band")
        .ok()
        .flatten()
        .is_some();
    clicked(&screen, "button.to-the-plate");
    settled().await;
    settled().await;
    let the_plate = previewed();
    answering_with(a_fuelled_day_of(1068.0, 76.9, (2254.6, 2480.1, 2705.5)));
    typed_into(&screen, "food", "creme fraiche");
    typed_into(&screen, "quantity", "200");
    settled().await;
    settled().await;
    let the_plate_and_the_food = previewed();

    assert!(sent.contains(r#""more":{"energy_kcal":360"#), "{sent}");
    assert!(ghost_band);
    assert_eq!(
        [the_food, the_plate, the_plate_and_the_food],
        [
            "With the food: 468 kcal, 36 of them spent digesting it.",
            "With the plate: 468 kcal, 36 of them spent digesting it.",
            "With the plate and the food: 1068 kcal, 55 of them spent digesting them."
        ]
    );
}

#[wasm_bindgen_test]
async fn foods_added_to_the_plate_wait_there_adding_up_and_ghosting_the_band_and_the_macros() {
    answering(&format!(
        r#"{{ "foods": {PASTA_AND_CREAM}, "fuel_day": {A_FUELLED_DAY}, "fuel_day_with": {A_FUELLED_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    added_to_the_plate(&screen, "pasta", "100").await;
    added_to_the_plate(&screen, "creme fraiche", "200").await;

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
    assert_eq!(
        texts_in_order(&screen, "form.intake button[type=submit]"),
        ["Add to the plate"]
    );
    found::<Element>(&screen, ".fuel-gauge circle.ghost");
    assert_eq!(text_of(&screen, "table.macros tr.protein .ghost"), "+16");
}

#[wasm_bindgen_test]
async fn logging_the_plate_logs_its_ingredients_in_one_go_then_clears_it() {
    answering(&format!(
        r#"{{ "foods": {PASTA_AND_CREAM}, "fuel_day": {A_DAY_NOT_FUELLED_YET}, "log_plate": {A_FUELLED_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;
    added_to_the_plate(&screen, "pasta", "100").await;
    added_to_the_plate(&screen, "creme fraiche", "200").await;

    clicked(&screen, "button.log-the-plate");
    settled().await;
    settled().await;

    let sent = asked("log_plate");
    for ingredient in [
        r#""date":"2026-10-10""#,
        r#""food":"Pasta","quantity":100"#,
        r#""food":"Crème fraîche","quantity":200"#,
    ] {
        assert!(sent.contains(ingredient), "{ingredient} in {sent}");
    }
    assert!(screen.query_selector("table.pot").ok().flatten().is_none());
    assert!(text_of(&screen, "table.intakes tbody").contains("Skyr"));
}

async fn a_plate_of_pasta_and_cream_kept_as(screen: &Element, meal: &str, eaten: &str) {
    added_to_the_plate(screen, "pasta", "100").await;
    added_to_the_plate(screen, "creme fraiche", "200").await;
    typed_into(screen, "ratio_0", "2,5");
    typed_into(screen, "ratio_1", "0.66");
    clicked(screen, "button.keep-as-a-meal");
    settled().await;
    typed_into(screen, "meal_name", meal);
    written_into(screen, "meal_description", "Boil, reduce, stir.");
    typed_into(screen, "eaten", eaten);
    found::<HtmlFormElement>(screen, "form.meal")
        .request_submit()
        .expect("the meal is kept");
    settled().await;
    settled().await;
}

fn a_plate_kept() -> String {
    format!(
        r#""foods": {PASTA_AND_CREAM}, "fuel_day": {A_DAY_NOT_FUELLED_YET},
            "cook_meal": [{PASTA_IN_CREAM}], "log_intake": {A_FUELLED_DAY}"#
    )
}

#[wasm_bindgen_test]
async fn keeping_the_plate_as_a_meal_cooks_it_with_its_ratios_then_logs_the_portion_eaten() {
    answering(&format!("{{ {} }}", a_plate_kept()));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    a_plate_of_pasta_and_cream_kept_as(&screen, "Pasta in cream", "250").await;

    let cooked = asked("cook_meal");
    for sent in [
        r#""name":"Pasta in cream""#,
        r#""description":"Boil, reduce, stir.""#,
        r#""food":"Crème fraîche""#,
        r#""cooked_per_raw":2.5"#,
        r#""cooked_per_raw":0.66"#,
    ] {
        assert!(cooked.contains(sent), "{sent} in {cooked}");
    }
    let logged = asked("log_intake");
    assert!(
        logged.contains(r#""food":"Pasta in cream""#) && logged.contains(r#""quantity":250"#),
        "{logged}"
    );
    assert!(screen.query_selector("table.pot").ok().flatten().is_none());
}

#[wasm_bindgen_test]
async fn a_meal_kept_without_its_portion_is_eaten_whole() {
    answering(&format!("{{ {} }}", a_plate_kept()));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    a_plate_of_pasta_and_cream_kept_as(&screen, "Pasta in cream", "").await;

    let logged = asked("log_intake");
    assert!(logged.contains(r#""quantity":382"#), "{logged}");
}

#[wasm_bindgen_test]
async fn a_meal_kept_with_a_portion_of_nothing_waits_for_later() {
    answering(&format!("{{ {} }}", a_plate_kept()));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;

    a_plate_of_pasta_and_cream_kept_as(&screen, "Pasta in cream", "0").await;

    assert!(asked("cook_meal").contains("Pasta in cream"));
    assert_eq!(asked("log_intake"), "");
    assert!(text_of(&screen, ".said").contains("Pasta in cream waits"));
}

#[wasm_bindgen_test]
async fn an_ingredient_on_the_plate_left_without_its_ratio_keeps_its_own_mass() {
    answering(&format!("{{ {} }}", a_plate_kept()));
    let screen = screen();
    let _mounted = the_fuel_day(&screen);
    settled().await;
    added_to_the_plate(&screen, "Pasta", "100").await;
    assert_eq!(
        found::<HtmlInputElement>(&screen, "input[name=ratio_0]").placeholder(),
        "1"
    );

    clicked(&screen, "button.keep-as-a-meal");
    settled().await;
    typed_into(&screen, "meal_name", "Plain pasta");
    found::<HtmlFormElement>(&screen, "form.meal")
        .request_submit()
        .expect("the meal is kept");
    settled().await;

    let sent = asked("cook_meal");
    assert!(
        sent.contains(r#""food":"Pasta""#) && !sent.contains("cooked_per_raw"),
        "{sent}"
    );
}

#[wasm_bindgen_test]
async fn the_plate_waits_in_fuel_while_the_cook_checks_another_tab() {
    answering_all(&[
        A_HOME,
        EVERY_TAB,
        &format!(
            r#"{{ "profile_on": null, "foods": {PASTA_AND_CREAM}, "meals": [],
                  "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
        ),
    ]);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;
    went_to("/fuel");
    settled().await;
    added_to_the_plate(&screen, "pasta", "100").await;
    typed_into(&screen, "ratio_0", "2.5");
    clicked(&screen, "button.keep-as-a-meal");
    settled().await;
    typed_into(&screen, "meal_name", "Pasta in cream");
    written_into(&screen, "meal_description", "Boil, reduce, stir.");

    went_to("/hydrate");
    settled().await;
    went_to("/fuel");
    settled().await;

    assert_eq!(
        [
            value_of(&screen, "meal_name"),
            found::<HtmlInputElement>(&screen, "textarea[name=meal_description]").value(),
            value_of(&screen, "ratio_0"),
        ],
        ["Pasta in cream", "Boil, reduce, stir.", "2.5"]
    );
    assert_eq!(
        cells_by_row(&screen, "table.pot tbody tr"),
        [["100 g Pasta", "", "360", "12", "2", "3", "70", "×"]]
    );
}

const A_HOME: &str = r#"{ "profile_on": ["2026-03-01", { "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null }],
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

const EVERY_TAB: &str = r#"{ "foods": [], "meals": [], "drinks": [], "exercises": [], "circuits": [], "fuel_week": null, "hydrate_week": null, "burn_week": null,
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
/// by the words of the links it sets side by side, else by its class.
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
                .or_else(|| {
                    part.class_list()
                        .contains("side-by-side")
                        .then(|| texts_in_order(&part, "a").join(" | "))
                })
                .unwrap_or_else(|| part.class_name())
        })
        .collect()
}

#[wasm_bindgen_test]
async fn each_tab_runs_from_its_day_through_its_log_band_breakdown_and_entries_to_its_grimoire_beside_the_numbers()
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
                "🎱 Explore the numbers"
            ],
            vec![
                "Today",
                "card day",
                "Fuel",
                "Where the day stands",
                "The four macros",
                "The day's intakes",
                "The week's average",
                "The last days",
                "📚 Open the grimoire | 🎱 Explore the numbers"
            ],
            vec![
                "Today",
                "card day",
                "Hydrate",
                "Where the day stands",
                "The day's drinks",
                "The week's average",
                "The last days",
                "📚 Open the grimoire | 🎱 Explore the numbers"
            ],
            vec![
                "Today",
                "card day",
                "Burn",
                "Where the day stands",
                "The day's expenditures",
                "The week's sum-up",
                "The last days",
                "📚 Open the grimoire | 🎱 Explore the numbers"
            ],
        ]
    );
}

#[wasm_bindgen_test]
async fn each_grimoire_opens_from_its_tab_on_a_page_of_its_own_holding_the_shelf_its_tab_still_lit()
{
    answering_all(&[A_HOME, EVERY_TAB]);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;

    let (mut opened, mut shown, mut lit) = (Vec::new(), Vec::new(), Vec::new());
    for tab in ["/fuel", "/hydrate", "/burn"] {
        went_to(tab);
        settled().await;
        let grimoire = attributes_of(&screen, "main.wrap a.grimoire", "href").concat();
        went_to(&grimoire);
        settled().await;
        opened.push(grimoire);
        shown.push(shown_in_order(&screen));
        lit.push(attributes_of(&screen, "nav.tabs a[aria-current='page']", "aria-label").concat());
    }

    assert_eq!(
        opened,
        ["/fuel/grimoire", "/hydrate/grimoire", "/burn/grimoire"]
    );
    assert_eq!(
        shown,
        [
            vec![
                "Today",
                "Fuel's grimoire",
                "Add to the catalog",
                "On the shelf"
            ],
            vec![
                "Today",
                "Hydrate's grimoire",
                "Add to the catalog",
                "On the shelf"
            ],
            vec![
                "Today",
                "Burn's grimoire",
                "Add to the catalog",
                "On the shelf"
            ],
        ]
    );
    assert_eq!(lit, ["Fuel", "Hydrate", "Burn"]);
}

#[wasm_bindgen_test]
async fn a_drink_added_and_logged_in_its_tab_joins_its_grimoire() {
    answering_all(&[A_HOME, EVERY_TAB]);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;
    went_to("/hydrate");
    settled().await;

    let kombucha = r#"[{ "name": "Kombucha", "volume_ml": 330.0, "kind": "Other" }]"#;
    let added = format!(
        r#"{{ "drinks": {kombucha}, "add_drink": {kombucha}, "log_drink": {A_DAY_WITH_TWO_DRINKS} }}"#
    );
    answering_all(&[A_HOME, EVERY_TAB, &added]);
    typed_into(&screen, "drink", "Kombucha");
    settled().await;
    typed_into(&screen, "new_drink_serving", "330");
    found::<HtmlFormElement>(&screen, "form.drinking")
        .request_submit()
        .expect("the new drink is added, then logged");
    settled().await;
    settled().await;
    went_to("/hydrate/grimoire");
    settled().await;

    assert!(text_of(&screen, "table.drinks tbody").contains("Kombucha"));
}

#[wasm_bindgen_test]
async fn a_food_added_in_the_grimoire_is_offered_by_the_fuel_form() {
    answering_all(&[A_HOME, EVERY_TAB]);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;
    went_to("/fuel/grimoire");
    settled().await;

    let added = format!(r#"{{ "foods": [{KEFIR}], "add_food": [{KEFIR}] }}"#);
    answering_all(&[A_HOME, EVERY_TAB, &added]);
    typed_into(&screen, "food_name", "Kefir");
    typed_into(&screen, "food_energy", "64");
    found::<HtmlFormElement>(&screen, "form.food")
        .request_submit()
        .expect("the food is added");
    settled().await;
    settled().await;
    went_to("/fuel");
    settled().await;
    typed_into(&screen, "food", "kefir");
    settled().await;

    assert!(texts_in_order(&screen, "form.intake .new").is_empty());
    assert!(text_of(&screen, "form.intake p.preview").contains("64 kcal"));
}

#[wasm_bindgen_test]
async fn the_streak_card_holds_its_ticks_to_the_gauges_bands_and_tells_of_no_grace() {
    answering(A_HOME);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;
    settled().await;

    let card = text_of(&screen, "section.card.streak");
    assert!(!card.contains("5%"), "{card}");
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
async fn home_holds_the_streak_alone_in_its_card_and_ends_on_the_numbers() {
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
    assert_eq!(
        text_of(&screen, "main.wrap > a.explore:last-child"),
        "🎱 Explore the numbers"
    );
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

#[wasm_bindgen_test]
async fn stats_offer_the_balance_as_what_was_eaten_against_maintenance() {
    answering(&format!(r#"{{ "explore": {TWO_MEASURES_EXPLORED} }}"#));
    let screen = screen();
    let _mounted = the_stats(&screen);
    settled().await;

    let offered = texts_of(&screen, "select option[value=Balance]");

    assert_eq!(offered, ["Eaten minus maintenance"; 2]);
}

const MORNING_RUST: &str = r#"{ "name": "Morning rust", "description": "Found on YouTube, the rusty goblin channel.", "steps": [
    { "exercise": "Push up", "met": 3.8, "reps": 12, "minutes": 0.5, "load_kg": 0.0 },
    { "exercise": "Burpee", "met": 8.0, "reps": null, "minutes": 1.0, "load_kg": 5.0 } ] }"#;

#[wasm_bindgen_test]
async fn a_kept_circuit_shows_its_price_a_set_and_its_steps_on_a_click_on_the_shelf() {
    answering(&format!(
        r#"{{ "exercises": {BURPEES_AND_PUSH_UPS}, "circuits": [{MORNING_RUST}],
             "priced_circuits": [{{ "circuit": {MORNING_RUST}, "burnt_per_set_kcal": 12.3 }}] }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_shelf(&screen);
    settled().await;
    assert!(
        screen
            .query_selector("form.circuit")
            .ok()
            .flatten()
            .is_none()
    );

    clicked(&screen, "button.shelf-Circuits");
    settled().await;

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

async fn added_to_the_session(
    screen: &Element,
    exercise: &str,
    minutes: &str,
    reps: &str,
    load: &str,
) {
    typed_into(screen, "exercise", exercise);
    typed_into(screen, "minutes", minutes);
    typed_into(screen, "reps", reps);
    typed_into(screen, "load", load);
    clicked(screen, "button.to-the-session");
    settled().await;
}

const BURPEES_AND_PUSH_UPS: &str =
    r#"[{ "name": "Burpee", "met": 8.0 }, { "name": "Push up", "met": 3.8 }]"#;

fn a_session_at_80_kg() -> String {
    format!(
        r#""exercises": {BURPEES_AND_PUSH_UPS}, "circuits": [], "burn_day": {A_BURN_SHORT_OF_ITS_TARGET},
            "fuel_day": {A_DAY_NOT_FUELLED_YET}"#
    )
}

#[wasm_bindgen_test]
async fn what_is_about_to_be_burnt_previews_the_day_with_it() {
    answering(&format!("{{ {} }}", a_session_at_80_kg()));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;
    let previewed = || texts_in_order(&screen, ".bands .with-the-preview").concat();

    typed_into(&screen, "exercise", "burpee");
    typed_into(&screen, "minutes", "1");
    typed_into(&screen, "load", "5");
    settled().await;
    let the_workout = previewed();
    clicked(&screen, "button.to-the-session");
    settled().await;
    let the_session = previewed();
    typed_into(&screen, "exercise", "push up");
    typed_into(&screen, "minutes", "0.5");
    typed_into(&screen, "reps", "12");
    settled().await;
    let the_session_and_the_workout = previewed();

    assert_eq!(
        [the_workout, the_session, the_session_and_the_workout],
        [
            "With the workout: 70 kcal.",
            "With the session: 70 kcal.",
            "With the session and the workout: 72 kcal."
        ]
    );
}

#[wasm_bindgen_test]
async fn steps_added_to_the_session_wait_there_priced_a_set_and_ghosting_the_band() {
    answering(&format!("{{ {} }}", a_session_at_80_kg()));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    added_to_the_session(&screen, "burpee", "1", "", "5").await;
    added_to_the_session(&screen, "push up", "0.5", "12", "").await;
    typed_into(&screen, "session_sets", "3");
    settled().await;

    assert_eq!(
        texts_in_order(&screen, "ol.steps li .kcal"),
        ["10 kcal", "2 kcal"]
    );
    assert_eq!(text_of(&screen, "p.per-set"), "12 kcal a set");
    assert_eq!(
        texts_in_order(&screen, "form.workout button[type=submit]"),
        ["Add to the session"]
    );
    assert!(
        screen
            .query_selector("input[name=sets]")
            .ok()
            .flatten()
            .is_none()
    );
    assert_eq!(
        text_of(&screen, ".bands .with-the-preview"),
        "With the session: 97 kcal."
    );
    found::<Element>(&screen, ".burn-gauge circle.ghost");
}

async fn a_session_of_burpees_and_push_ups(screen: &Element, sets: &str) {
    added_to_the_session(screen, "burpee", "1", "", "5").await;
    added_to_the_session(screen, "push up", "0.5", "12", "").await;
    typed_into(screen, "session_sets", sets);
    settled().await;
}

#[wasm_bindgen_test]
async fn logging_the_session_logs_its_steps_in_one_go_then_clears_it() {
    answering(&format!(
        r#"{{ {}, "log_session": {A_DAY_OF_MORNING_RUST} }}"#,
        a_session_at_80_kg()
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;
    a_session_of_burpees_and_push_ups(&screen, "3").await;

    clicked(&screen, "button.log-the-session");
    settled().await;
    settled().await;

    let sent = asked("log_session");
    for part in [
        r#""date":"2026-10-10""#,
        r#""exercise":"Burpee""#,
        r#""exercise":"Push up""#,
        r#""sets":3"#,
    ] {
        assert!(sent.contains(part), "{part} in {sent}");
    }
    assert!(screen.query_selector("ol.steps").ok().flatten().is_none());
    assert!(text_of(&screen, "table.expenditures tbody").contains("Push up"));
}

async fn kept_as_a_circuit(screen: &Element) {
    clicked(screen, "button.keep-as-a-circuit");
    settled().await;
    typed_into(screen, "circuit_name", "Morning rust");
    written_into(
        screen,
        "circuit_description",
        "Found on YouTube, the rusty goblin channel.",
    );
    found::<HtmlFormElement>(screen, "form.circuit")
        .request_submit()
        .expect("the circuit is kept");
    settled().await;
    settled().await;
}

#[wasm_bindgen_test]
async fn keeping_the_session_as_a_circuit_crafts_it_then_logs_its_sets() {
    answering(&format!(
        r#"{{ {}, "craft_circuit": [{MORNING_RUST}], "log_circuit": {A_DAY_OF_MORNING_RUST} }}"#,
        a_session_at_80_kg()
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;
    a_session_of_burpees_and_push_ups(&screen, "3").await;

    kept_as_a_circuit(&screen).await;

    let crafted = asked("craft_circuit");
    for part in [
        r#""name":"Morning rust""#,
        r#""description":"Found on YouTube, the rusty goblin channel.""#,
        r#""exercise":"Burpee""#,
    ] {
        assert!(crafted.contains(part), "{part} in {crafted}");
    }
    let logged = asked("log_circuit");
    assert!(
        logged.contains(r#""circuit":"Morning rust""#) && logged.contains(r#""sets":3"#),
        "{logged}"
    );
    assert!(screen.query_selector("ol.steps").ok().flatten().is_none());
}

#[wasm_bindgen_test]
async fn a_circuit_kept_with_sets_of_nothing_waits_for_later() {
    answering(&format!(
        r#"{{ {}, "craft_circuit": [{MORNING_RUST}], "log_circuit": {A_DAY_OF_MORNING_RUST} }}"#,
        a_session_at_80_kg()
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;
    a_session_of_burpees_and_push_ups(&screen, "0").await;

    kept_as_a_circuit(&screen).await;

    assert!(asked("craft_circuit").contains("Morning rust"));
    assert_eq!(asked("log_circuit"), "");
    assert!(text_of(&screen, ".said").contains("Morning rust waits"));
}

#[wasm_bindgen_test]
async fn a_step_added_by_mistake_leaves_the_session() {
    answering(&format!("{{ {} }}", a_session_at_80_kg()));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;
    added_to_the_session(&screen, "burpee", "1", "", "").await;

    clicked(&screen, "ol.steps button.remove");
    settled().await;

    assert!(screen.query_selector("ol.steps").ok().flatten().is_none());
}

#[wasm_bindgen_test]
async fn the_session_waits_in_burn_while_the_athlete_checks_another_tab() {
    answering_all(&[
        A_HOME,
        EVERY_TAB,
        &format!(
            r#"{{ "exercises": {BURPEES_AND_PUSH_UPS}, "burn_day": {A_BURN_SHORT_OF_ITS_TARGET} }}"#
        ),
    ]);
    let screen = screen();
    let _mounted = the_app(&screen);
    settled().await;
    went_to("/burn");
    settled().await;
    added_to_the_session(&screen, "burpee", "1", "", "5").await;
    typed_into(&screen, "session_sets", "3");
    clicked(&screen, "button.keep-as-a-circuit");
    settled().await;
    typed_into(&screen, "circuit_name", "Morning rust");

    went_to("/hydrate");
    settled().await;
    went_to("/burn");
    settled().await;

    assert_eq!(
        [
            value_of(&screen, "session_sets"),
            value_of(&screen, "circuit_name")
        ],
        ["3", "Morning rust"]
    );
    assert!(text_of(&screen, "ol.steps").contains("Burpee"));
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
    clicked(&screen, "button.shelf-Circuits");
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
async fn a_circuit_logged_from_the_workout_form_runs_by_its_sets() {
    answering(&format!(
        r#"{{ "exercises": [], "circuits": [{MORNING_RUST}], "burn_day": {AN_EMPTY_BURN},
             "fuel_day": {A_DAY_NOT_FUELLED_YET}, "log_circuit": {A_DAY_OF_MORNING_RUST} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    typed_into(&screen, "exercise", "Morning rust");
    typed_into(&screen, "sets", "3");
    found::<HtmlFormElement>(&screen, "form.workout")
        .request_submit()
        .expect("the circuit is logged");
    settled().await;

    let sent = asked("log_circuit");
    assert!(
        sent.contains(r#""circuit":"Morning rust""#) && sent.contains(r#""sets":3"#),
        "{sent}"
    );
    let rows = text_of(&screen, "table.expenditures tbody");
    assert!(
        rows.contains("Push up") && rows.contains("Burpee"),
        "{rows}"
    );
}

#[wasm_bindgen_test]
async fn a_circuit_put_in_the_session_brings_its_steps() {
    answering(&format!(
        r#"{{ "exercises": {BURPEES_AND_PUSH_UPS}, "circuits": [{MORNING_RUST}],
             "burn_day": {A_BURN_PRICED_AT_80_KG}, "fuel_day": {A_DAY_NOT_FUELLED_YET} }}"#
    ));
    let screen = screen();
    let _mounted = the_burn_day(&screen);
    settled().await;

    typed_into(&screen, "exercise", "Morning rust");
    settled().await;
    clicked(&screen, "button.to-the-session");
    settled().await;

    let steps = texts_in_order(&screen, "ol.steps li");
    assert_eq!(steps.len(), 2, "{steps:?}");
    assert!(
        steps[0].contains("Push up") && steps[1].contains("Burpee"),
        "{steps:?}"
    );
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

#[wasm_bindgen_test]
async fn a_drink_chosen_previews_the_day_with_it() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "hydrate_day": {A_DAY_WITH_TWO_DRINKS} }}"#
    ));
    let screen = screen();
    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-10".to_string()));
    settled().await;
    let previewed = || texts_in_order(&screen, ".bands .with-the-preview").concat();

    typed_into(&screen, "drink", "tea");
    settled().await;
    let a_serving = previewed();
    let ghosted = screen
        .query_selector(".bands .gauge circle.ghost")
        .ok()
        .flatten()
        .is_some();
    typed_into(&screen, "volume", "500");
    settled().await;
    let half_a_litre = previewed();

    assert!(ghosted);
    assert_eq!(
        [a_serving, half_a_litre],
        ["With the drink: 480 mL.", "With the drink: 780 mL."]
    );
}

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
async fn the_days_drinks_add_up_against_its_band_as_fuels_energy_does() {
    answering(&format!(
        r#"{{ "drinks": {THE_CATALOG}, "hydrate_day": {A_DAY_WITH_TWO_DRINKS} }}"#
    ));
    let screen = screen();

    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-10".to_string()));
    settled().await;

    let line: Element = found(&screen, ".bands p.figures");
    assert_eq!(
        (
            line.text_content().unwrap_or_default(),
            line.class_list().contains("low")
        ),
        (
            "280 mL drunk, the band runs from 2027 to 2477 mL.".to_string(),
            true
        )
    );
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
    assert!(
        shown.contains("0 mL drunk, the band runs from 1800 to 2200 mL."),
        "{shown}"
    );
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
async fn the_drink_chooser_offers_water_then_the_other_drinks_each_in_alphabetical_order_behind_its_icon()
 {
    answering(&format!(
        r#"{{ "drinks": [{{ "name": "Tea", "volume_ml": 200.0, "kind": "Other" }},
                        {{ "name": "Water", "volume_ml": 250.0, "kind": "Water" }},
                        {{ "name": "Expresso", "volume_ml": 80.0, "kind": "Other" }},
                        {{ "name": "Eau pétillante", "volume_ml": 330.0, "kind": "Water" }}],
             "hydrate_day": {A_DRY_DAY} }}"#
    ));
    let screen = screen();
    let _mounted = the_day_of(&screen, RwSignal::new("2026-10-11".to_string()));
    settled().await;

    focused(&screen, "drink");
    settled().await;
    let offered = choices(&screen);
    picked(&screen, "💧 Water");
    settled().await;

    assert_eq!(
        offered,
        [
            "💧 Eau pétillante",
            "💧 Water",
            "☕\u{fe0f} Expresso",
            "☕\u{fe0f} Tea"
        ]
    );
    assert_eq!(value_of(&screen, "drink"), "Water");
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
    picked(&screen, "☕\u{fe0f} Tea");
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
        r#"{ "drinks": [], "add_drink": [{ "name": "Tea", "volume_ml": 200.0, "kind": "Other", "description": "Green, no sugar" }] }"#,
    );
    let screen = screen();
    let _mounted = the_hydrate_shelf(&screen);
    settled().await;
    assert_eq!(
        found::<HtmlSelectElement>(&screen, "select[name=drink_kind]").length(),
        2
    );
    let tea = r#"[{ "name": "Tea", "volume_ml": 200.0, "kind": "Other", "description": "Green, no sugar" }]"#;
    answering(&format!(r#"{{ "drinks": {tea}, "add_drink": {tea} }}"#));

    typed_into(&screen, "drink_name", "Tea");
    typed_into(&screen, "drink_description", "Green, no sugar");
    typed_into(&screen, "drink_serving", "200");
    found::<HtmlFormElement>(&screen, "form.drink")
        .request_submit()
        .expect("the drink is added");
    settled().await;

    assert_eq!(value_of(&screen, "drink_name"), "");
    assert!(asked("add_drink").contains(r#""description":"Green, no sugar""#));
    let rows = text_of(&screen, "table.drinks tbody");
    assert!(rows.contains("200 mL"), "{rows}");
    assert_eq!(
        [
            text_of(&screen, "table.drinks tbody .name"),
            text_of(&screen, "table.drinks tbody .description")
        ],
        ["Tea", "Green, no sugar"]
    );
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
        labels.contains("Min 1800 mL")
            && labels.contains("Optimal 2000 mL")
            && labels.contains("Max 2200 mL"),
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
async fn the_home_chart_draws_gather_on_one_axis_with_its_goals_then_each_chip_its_own_lines_burn_with_its_target()
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
    found::<Element>(&screen, ".plot .label.left.tick");
    assert!(
        screen
            .query_selector(".plot .label.right.tick")
            .ok()
            .flatten()
            .is_none()
    );

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

    assert_eq!(count(&screen, ".charts svg line.axis"), 1);
    assert_eq!(count(&screen, ".charts svg line.grid"), 4);
    assert_eq!(
        texts_of(&screen, ".charts .label.tick"),
        ["60.0", "80.0", "90.0"]
    );

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
async fn the_unwanted_tips_note_what_is_worth_sharpening_on_the_chosen_chip_between_the_chart_and_the_history_each_between_quotation_marks()
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
        [gather, fuel, hydrate, burn].iter().all(|tip| {
            naming_the_cast(tip).is_empty()
                && !tip.contains('[')
                && tip.starts_with('“')
                && tip.ends_with('”')
        }),
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
async fn the_goals_card_tells_the_healthy_ranges_for_the_height_and_age_under_its_targets_from_the_start()
 {
    answering(&format!(
        r#"{{ "profile_on": ["2026-03-01", {{ "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null }}],
             "goals_on": {{ "values": {{ "body_mass_kg": 68.0, "fat_ratio": 0.1 }}, "recommended": false }}, "fuel_targets_on": null,
             "usual_day_on": null, "recommended_on": {RECOMMENDED} }}"#
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

    let healthy = text_of(&screen, "form.goals p.healthy");

    assert!(
        healthy.starts_with("Healthy at your height and age: 59.9 kg to 80.7 kg")
            && healthy.contains("11.0% to 22.0% body fat")
            && healthy.contains("ask a doctor"),
        "{healthy}"
    );
}

const IN_FORCE_ON_EVERY_CARD: &str = r#""profile_on": ["2026-03-01", { "birthday": "1985-04-12", "gender": "Male", "height_cm": 180.0, "neck_cm": 38.5, "hip_cm": null }],
    "goals_on": { "values": { "body_mass_kg": 68.0, "fat_ratio": 0.12 }, "recommended": false, "from": "2026-10-10" },
    "fuel_targets_on": { "values": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.8, "fibre_g_per_kg": 0.31 }, "recommended": true, "from": null },
    "usual_day_on": { "values": { "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 6.0 }, "recommended": true, "from": "2026-09-01" },
    "burn_target_on": { "values": 77.14, "recommended": true, "from": null },
    "hydrate_target_on": { "values": 2800.0, "recommended": false, "from": "2026-10-12" }"#;

const KEPT_AS_RECOMMENDED: &str = r#""set_goals": { "values": { "body_mass_kg": 70.308, "fat_ratio": 0.165 }, "recommended": true },
    "set_fuel_targets": { "values": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.8, "fibre_g_per_kg": 0.31 }, "recommended": true },
    "set_usual_day": { "values": { "sitting_min": 480.0, "standing_min": 120.0, "moving_km": 6.0 }, "recommended": true },
    "set_burn_target": { "values": 77.14, "recommended": true },
    "set_hydrate_target": { "values": 2600.0, "recommended": true }"#;

fn the_profile_screen(screen: &Element) -> impl Drop {
    leptos::mount::mount_to(screen.clone().unchecked_into(), || {
        view! {
            <Router>
                <ProfileScreen />
            </Router>
        }
    })
}

const OWN: &str = "Your own values, kept as typed.";
const RECOMMENDED_AND_FOLLOWED: &str = "The recommendation, followed as your body changes.";

#[wasm_bindgen_test]
async fn each_goal_card_shows_what_is_in_force_and_whose_it_is() {
    answering(&format!(
        r#"{{ {IN_FORCE_ON_EVERY_CARD}, "recommended_on": {RECOMMENDED} }}"#
    ));
    let screen = screen();
    let _mounted = the_profile_screen(&screen);
    settled().await;

    assert_eq!(
        [
            "target_mass",
            "target_fat",
            "target_protein",
            "target_fat_g",
            "target_fibre",
            "water_target",
            "usual_sitting",
            "usual_moving",
            "burn_target"
        ]
        .map(|name| value_of(&screen, name)),
        [
            "68", "12", "1.6", "0.8", "0.31", "2800", "480", "6", "77.14"
        ]
    );
    assert_eq!(
        [
            "form.goals",
            "form.fuel-targets",
            "form.water-target",
            "form.usual-day",
            "form.burn-target"
        ]
        .map(|form| text_of(&screen, &format!("{form} .whose"))),
        [
            OWN,
            RECOMMENDED_AND_FOLLOWED,
            OWN,
            RECOMMENDED_AND_FOLLOWED,
            RECOMMENDED_AND_FOLLOWED
        ]
    );
}

#[wasm_bindgen_test]
async fn each_card_opens_on_the_day_its_values_were_kept_from_today_while_nothing_is_kept() {
    answering(&format!(
        r#"{{ {IN_FORCE_ON_EVERY_CARD}, "recommended_on": {RECOMMENDED} }}"#
    ));
    let screen = screen();
    let _mounted = the_profile_screen(&screen);
    settled().await;

    let today = grind_ui::screens::today();
    assert_eq!(
        [
            "from",
            "goals_from",
            "targets_from",
            "water_target_from",
            "burn_target_from",
            "usual_from"
        ]
        .map(|name| value_of(&screen, name)),
        [
            "2026-03-01",
            "2026-10-10",
            &today,
            "2026-10-12",
            &today,
            "2026-09-01"
        ]
    );
}

#[wasm_bindgen_test]
async fn each_goal_card_shows_whether_its_tick_counts_in_the_streak_and_the_usual_day_has_none() {
    answering_all(&[
        &format!(r#"{{ {IN_FORCE_ON_EVERY_CARD}, "recommended_on": {RECOMMENDED} }}"#),
        r#"{ "scope_on": { "weighed": true, "fuelled": false, "worked_out": true, "hydrated": false } }"#,
    ]);
    let screen = screen();
    let _mounted = the_profile_screen(&screen);
    settled().await;

    assert_eq!(
        [
            "form.goals",
            "form.fuel-targets",
            "form.water-target",
            "form.burn-target"
        ]
        .map(|form| texts_in_order(&screen, &format!("{form} .pill.chosen")).concat()),
        ["Counted", "Left out", "Left out", "Counted"]
    );
    assert!(texts_in_order(&screen, "form.usual-day .pill").is_empty());
}

#[wasm_bindgen_test]
async fn either_button_keeps_the_cards_switch_from_its_day_and_says_when_its_tick_stays_out() {
    answering(&format!(
        r#"{{ {IN_FORCE_ON_EVERY_CARD}, {KEPT_AS_RECOMMENDED}, "recommended_on": {RECOMMENDED} }}"#
    ));
    let screen = screen();
    let _mounted = the_profile_screen(&screen);
    settled().await;

    clicked(&screen, "form.goals .pill.left-out");
    clicked(&screen, "form.goals button.recommend");
    clicked(&screen, "form.water-target .pill.left-out");
    typed_into(&screen, "water_target_from", "2026-03-01");
    found::<HtmlFormElement>(&screen, "form.water-target")
        .request_submit()
        .expect("the water target is kept");
    clicked(&screen, "form.burn-target button.recommend");
    settled().await;

    assert!(
        asked("set_goals").contains(r#""kept":"Recommended","counted":false"#)
            && asked("set_hydrate_target")
                .contains(r#""from":"2026-03-01","kept":{"Own":2800},"counted":false"#)
            && asked("set_burn_target").contains(r#""counted":true"#),
        "{} {} {}",
        asked("set_goals"),
        asked("set_hydrate_target"),
        asked("set_burn_target")
    );
    assert_eq!(
        ["form.goals", "form.water-target", "form.burn-target"].map(|form| text_of(
            &screen,
            &format!("{form} .said")
        )
        .contains("out of the streak")),
        [true, true, false]
    );
}

#[wasm_bindgen_test]
async fn use_the_recommendation_keeps_it_from_the_cards_day_and_shows_it_followed() {
    answering(&format!(
        r#"{{ {IN_FORCE_ON_EVERY_CARD}, {KEPT_AS_RECOMMENDED}, "recommended_on": {RECOMMENDED} }}"#
    ));
    let screen = screen();
    let _mounted = the_profile_screen(&screen);
    settled().await;
    typed_into(&screen, "goals_from", "2026-10-15");

    let mut kept = Vec::new();
    for (form, command) in [
        ("form.goals", "set_goals"),
        ("form.fuel-targets", "set_fuel_targets"),
        ("form.water-target", "set_hydrate_target"),
        ("form.usual-day", "set_usual_day"),
        ("form.burn-target", "set_burn_target"),
    ] {
        clicked(&screen, &format!("{form} button.recommend"));
        settled().await;
        kept.push((
            asked(command).contains(r#""kept":"Recommended""#),
            text_of(&screen, &format!("{form} .whose")),
        ));
    }

    assert!(
        kept.iter()
            .all(|(recommended, whose)| *recommended && whose == RECOMMENDED_AND_FOLLOWED),
        "{kept:?}"
    );
    assert!(asked("set_goals").contains(r#""from":"2026-10-15""#));
    assert_eq!(
        [
            value_of(&screen, "target_mass"),
            value_of(&screen, "target_fat")
        ],
        ["70.31", "16.5"]
    );
}

#[wasm_bindgen_test]
async fn hydrates_card_keeps_a_water_target_of_your_own() {
    answering(&format!(
        r#"{{ {IN_FORCE_ON_EVERY_CARD}, "recommended_on": {RECOMMENDED},
             "set_hydrate_target": {{ "values": 3000.0, "recommended": false }} }}"#
    ));
    let screen = screen();
    let _mounted = the_profile_screen(&screen);
    settled().await;

    typed_into(&screen, "water_target", "3000");
    found::<HtmlFormElement>(&screen, "form.water-target")
        .request_submit()
        .expect("the water target is kept");
    settled().await;

    assert!(
        asked("set_hydrate_target").contains(r#""kept":{"Own":3000}"#),
        "{}",
        asked("set_hydrate_target")
    );
    assert_eq!(text_of(&screen, "form.water-target .whose"), OWN);
}

#[wasm_bindgen_test]
async fn the_burn_target_card_keeps_its_own_kcal_from_a_date() {
    answering(&format!(
        r#"{{ {IN_FORCE_ON_EVERY_CARD}, "recommended_on": {RECOMMENDED},
             "set_burn_target": {{ "values": 80.0, "recommended": false }} }}"#
    ));
    let screen = screen();
    let _mounted = the_profile_screen(&screen);
    settled().await;

    typed_into(&screen, "burn_target_from", "2026-10-15");
    typed_into(&screen, "burn_target", "80");
    found::<HtmlFormElement>(&screen, "form.burn-target")
        .request_submit()
        .expect("the target is kept");
    settled().await;

    assert!(asked("set_burn_target").contains(r#""kept":{"Own":80}"#));
    let said = text_of(&screen, "form.burn-target .said");
    assert!(said.contains("kept from 2026-10-15"), "{said}");
}

#[wasm_bindgen_test]
async fn before_any_weigh_in_the_burn_target_card_says_a_single_workout_holds_the_tick() {
    answering(
        r#"{ "profile_on": null, "goals_on": null, "usual_day_on": null,
             "fuel_targets_on": { "values": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }, "recommended": false },
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
    assert_eq!((topics, pages), (6, 31));
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
             "fuel_targets_on": { "values": { "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }, "recommended": false } }"#,
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
             "fuel_targets_on": {{ "values": {{ "protein_g_per_kg": 1.6, "fat_g_per_kg": 0.9, "fibre_g_per_kg": 0.4 }}, "recommended": false }},
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
    clicked(&fuel_shelf, "button.shelf-Meals");
    clicked(&burn_shelf, "button.shelf-Circuits");
    settled().await;
    said.extend([spoken_on(&fuel_shelf), spoken_on(&burn_shelf)]);

    let burn = screen();
    let _burn = the_burn_day(&burn);
    settled().await;
    clicked(&burn, "button.burn-OffTheClock");
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
