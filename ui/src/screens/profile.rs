use grind_core::for_setting_up_the_profile::{GoalsForm, ProfileForm};
use grind_core::gender::Gender;
use grind_core::goals::fat_percentage;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;

use crate::screens::fields::{DateField, FigureField};
use crate::screens::{lines, today};
use crate::tauri;
use crate::typed::figure_typed;

#[component]
fn GoalsCard() -> impl IntoView {
    let from = RwSignal::new(today());
    let target_mass = RwSignal::new(String::new());
    let target_fat = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        if let Ok(Some(goals)) = tauri::goals_on(&today()).await {
            target_mass.set(goals.body_mass_kg.to_string());
            target_fat.set(fat_percentage(&goals).to_string());
        }
    });
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let form = GoalsForm {
            from: from.get_untracked(),
            body_mass_kg: figure_typed(&target_mass.get_untracked()).unwrap_or(f64::NAN),
            fat_percentage: figure_typed(&target_fat.get_untracked()).unwrap_or(f64::NAN),
        };
        spawn_local(async move {
            said.set(Some(match tauri::set_goals(&form).await {
                Ok(_) => lines::goals_kept(&form.from),
                Err(failure) => lines::failure(&failure),
            }));
        });
    };
    view! {
        <section class="card">
            <h2>"Where the grindstone is heading"</h2>
            <form class="goals" on:submit=save>
                <div class="row">
                    <DateField label="From" name="goals_from" value=from />
                    <FigureField label="Target mass (kg)" name="target_mass" value=target_mass />
                    <FigureField label="Target fat (%)" name="target_fat" value=target_fat />
                </div>
                <button class="cta" type="submit">
                    "Keep these goals"
                </button>
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
        </section>
    }
}

#[component]
pub fn ProfileScreen() -> impl IntoView {
    let from = RwSignal::new(today());
    let birthday = RwSignal::new(String::new());
    let gender = RwSignal::new(Gender::Male);
    let height = RwSignal::new(String::new());
    let neck = RwSignal::new(String::new());
    let hip = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        if let Ok(Some(profile)) = tauri::profile_on(&today()).await {
            birthday.set(profile.birthday.to_string());
            gender.set(profile.gender);
            height.set(profile.height_cm.to_string());
            neck.set(profile.neck_cm.to_string());
            hip.set(profile.hip_cm.map(|cm| cm.to_string()).unwrap_or_default());
        }
    });

    let chosen = move |event: leptos::ev::Event| {
        gender.set(if event_target_value(&event) == "Female" {
            Gender::Female
        } else {
            Gender::Male
        });
    };
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let form = ProfileForm {
            from: from.get_untracked(),
            birthday: birthday.get_untracked(),
            gender: gender.get_untracked(),
            height_cm: figure_typed(&height.get_untracked()).unwrap_or(f64::NAN),
            neck_cm: figure_typed(&neck.get_untracked()).unwrap_or(f64::NAN),
            hip_cm: figure_typed(&hip.get_untracked()),
        };
        spawn_local(async move {
            said.set(Some(match tauri::set_profile(&form).await {
                Ok(_) => lines::profile_kept(&form.from),
                Err(failure) => lines::failure(&failure),
            }));
        });
    };

    view! {
        <header class="screen-top">
            <h2>"Profile"</h2>
            <A href="/settings" attr:class="gear">
                "⚙ Settings"
            </A>
        </header>
        <section class="card">
            <h2>"The body behind the numbers"</h2>
            <form class="profile" on:submit=save>
                <div class="row">
                    <DateField label="From" name="from" value=from />
                    <DateField label="Birthday" name="birthday" value=birthday />
                </div>
                <div class="row">
                    <label class="field">
                        "Gender" <select
                            name="gender"
                            prop:value=move || {
                                if gender.get() == Gender::Female { "Female" } else { "Male" }
                            }
                            on:change=chosen
                        >
                            <option value="Male">"Male"</option>
                            <option value="Female">"Female"</option>
                        </select>
                    </label>
                    <FigureField label="Height (cm)" name="height" value=height />
                </div>
                <div class="row">
                    <FigureField label="Neck (cm)" name="neck" value=neck />
                    <Show when=move || gender.get() == Gender::Female>
                        <FigureField label="Hip (cm)" name="hip" value=hip />
                    </Show>
                </div>
                <button class="cta" type="submit">
                    "Keep this profile"
                </button>
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
        </section>
        <GoalsCard />
    }
}
