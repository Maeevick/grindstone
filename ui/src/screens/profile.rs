use grind_core::for_setting_up_the_profile::{
    FuelTargetsForm, GoalsForm, ProfileForm, UsualDayForm,
};
use grind_core::gender::Gender;
use grind_core::goals::fat_percentage;
use grind_core::sources::FUEL_RECOMMENDATIONS;
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
fn UsualDayCard() -> impl IntoView {
    let from = RwSignal::new(today());
    let sitting = RwSignal::new(String::new());
    let standing = RwSignal::new(String::new());
    let moving = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        if let Ok(Some(usual)) = tauri::usual_day_on(&today()).await {
            sitting.set(usual.sitting_min.to_string());
            standing.set(usual.standing_min.to_string());
            moving.set(usual.moving_km.to_string());
        }
    });
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let form = UsualDayForm {
            from: from.get_untracked(),
            sitting_min: figure_typed(&sitting.get_untracked()).unwrap_or(0.0),
            standing_min: figure_typed(&standing.get_untracked()).unwrap_or(0.0),
            moving_km: figure_typed(&moving.get_untracked()).unwrap_or(0.0),
        };
        spawn_local(async move {
            said.set(Some(match tauri::set_usual_day(&form).await {
                Ok(_) => lines::usual_day_kept(&form.from),
                Err(failure) => lines::failure(&failure),
            }));
        });
    };
    view! {
        <section class="card">
            <h2>"The usual day off the clock"</h2>
            <p class="hint">
                "Outside workouts: the minutes you usually sit and stand, the kilometres you usually move. It prices the day's NEAT."
            </p>
            <form class="usual-day" on:submit=save>
                <div class="row">
                    <DateField label="From" name="usual_from" value=from />
                    <FigureField label="Sitting (min)" name="usual_sitting" value=sitting />
                    <FigureField label="Standing (min)" name="usual_standing" value=standing />
                    <FigureField label="Moving (km)" name="usual_moving" value=moving />
                </div>
                <button class="cta" type="submit">
                    "Keep this usual day"
                </button>
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
        </section>
    }
}

#[component]
fn FuelTargetsCard() -> impl IntoView {
    let from = RwSignal::new(today());
    let protein = RwSignal::new(String::new());
    let fat = RwSignal::new(String::new());
    let fibre = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    spawn_local(async move {
        if let Ok(targets) = tauri::fuel_targets_on(&today()).await {
            protein.set(targets.protein_g_per_kg.to_string());
            fat.set(targets.fat_g_per_kg.to_string());
            fibre.set(targets.fibre_g_per_kg.to_string());
        }
    });
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let form = FuelTargetsForm {
            from: from.get_untracked(),
            protein_g_per_kg: figure_typed(&protein.get_untracked()).unwrap_or(f64::NAN),
            fat_g_per_kg: figure_typed(&fat.get_untracked()).unwrap_or(f64::NAN),
            fibre_g_per_kg: figure_typed(&fibre.get_untracked()).unwrap_or(f64::NAN),
        };
        spawn_local(async move {
            said.set(Some(match tauri::set_fuel_targets(&form).await {
                Ok(_) => lines::fuel_targets_kept(&form.from),
                Err(failure) => lines::failure(&failure),
            }));
        });
    };
    view! {
        <section class="card">
            <h2>"Fuel targets"</h2>
            <p class="hint">
                "Grams per kilo of body mass a day: each is the floor of its band, which also asks for 20 to 33.3% of the day's energy (4% for fibre)."
            </p>
            <form class="fuel-targets" on:submit=save>
                <div class="row">
                    <DateField label="From" name="targets_from" value=from />
                    <FigureField label="Protein (g/kg)" name="target_protein" value=protein />
                    <FigureField label="Fat (g/kg)" name="target_fat_g" value=fat />
                    <FigureField label="Fibre (g/kg)" name="target_fibre" value=fibre />
                </div>
                <button class="cta" type="submit">
                    "Keep these targets"
                </button>
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
            <ul class="sources">
                {FUEL_RECOMMENDATIONS
                    .iter()
                    .map(|recommendation| {
                        let source = recommendation.source;
                        view! {
                            <li>
                                <strong>{recommendation.about}</strong>
                                {format!(": {}. ", recommendation.says)}
                                <a href=source.link target="_blank" rel="noopener">
                                    {format!("{} ({}), {}", source.authors, source.year, source.title)}
                                </a>
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
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
        <FuelTargetsCard />
        <UsualDayCard />
    }
}
