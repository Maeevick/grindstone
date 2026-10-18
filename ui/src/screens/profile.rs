use grind_core::bibliography::Topic;
use grind_core::figures::typed_back;
use grind_core::for_setting_up_the_profile::{
    BurnTargetForm, FuelTargetsForm, GoalsForm, ProfileForm, UsualDayForm,
};
use grind_core::gender::Gender;
use grind_core::goals::fat_percentage;
use grind_core::recommendations::Recommendations;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;

use crate::screens::fields::{DateField, FigureField};
use crate::screens::hydrate::HydrateCountedCard;
use crate::screens::references::ReferencesLink;
use crate::screens::{lines, today};
use crate::tauri;
use crate::typed::figures_typed;

#[component]
pub fn Recommend(fill: Callback<Recommendations>, said: RwSignal<Option<String>>) -> impl IntoView {
    let ask = move |_| {
        spawn_local(async move {
            match tauri::recommended_on(&today()).await {
                Ok(advised) => {
                    fill.run(advised);
                    said.set(Some(lines::RECOMMENDATION_FILLED.into()));
                }
                Err(failure) => said.set(Some(lines::failure(&failure))),
            }
        });
    };
    view! {
        <button class="cta ghost recommend" type="button" on:click=ask>
            "Use the recommendation"
        </button>
    }
}

#[component]
fn GoalsCard() -> impl IntoView {
    let from = RwSignal::new(today());
    let target_mass = RwSignal::new(String::new());
    let target_fat = RwSignal::new(String::new());
    let healthy = RwSignal::new(None::<String>);
    let said = RwSignal::new(None::<String>);
    let fill = Callback::new(move |advised: Recommendations| {
        target_mass.set(typed_back(advised.goal_mass_kg));
        target_fat.set(typed_back(advised.goal_fat_ratio * 100.0));
        healthy.set(Some(lines::healthy_ranges(&advised)));
    });
    spawn_local(async move {
        if let Ok(Some(goals)) = tauri::goals_on(&today()).await {
            target_mass.set(goals.body_mass_kg.to_string());
            target_fat.set(fat_percentage(&goals).to_string());
        }
    });
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([body_mass_kg, fat_percentage]) =
            figures_typed([&target_mass.get_untracked(), &target_fat.get_untracked()])
        else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = GoalsForm {
            from: from.get_untracked(),
            body_mass_kg: body_mass_kg.unwrap_or(0.0),
            fat_percentage: fat_percentage.unwrap_or(0.0),
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
                </div>
                <div class="row">
                    <FigureField label="Target mass (kg)" name="target_mass" value=target_mass />
                    <FigureField label="Target fat (%)" name="target_fat" value=target_fat />
                </div>
                {move || healthy.get().map(|line| view! { <p class="hint">{line}</p> })}
                <Recommend fill=fill said=said />
                <button class="cta" type="submit">
                    "Keep these goals"
                </button>
                <ReferencesLink topic=Topic::HealthyRanges />
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
    let fill = Callback::new(move |advised: Recommendations| {
        sitting.set(typed_back(advised.usual_day.sitting_min));
        standing.set(typed_back(advised.usual_day.standing_min));
        moving.set(typed_back(advised.usual_day.moving_km));
    });
    spawn_local(async move {
        if let Ok(Some(usual)) = tauri::usual_day_on(&today()).await {
            sitting.set(usual.sitting_min.to_string());
            standing.set(usual.standing_min.to_string());
            moving.set(usual.moving_km.to_string());
        }
    });
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([sitting_min, standing_min, moving_km]) = figures_typed([
            &sitting.get_untracked(),
            &standing.get_untracked(),
            &moving.get_untracked(),
        ]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = UsualDayForm {
            from: from.get_untracked(),
            sitting_min: sitting_min.unwrap_or(0.0),
            standing_min: standing_min.unwrap_or(0.0),
            moving_km: moving_km.unwrap_or(0.0),
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
            <h2>"The forgotten bucket of sweat"</h2>
            <p class="hint">
                "Outside workouts: the minutes you usually sit and stand, the kilometres you usually move. It prices the day's NEAT."
            </p>
            <form class="usual-day" on:submit=save>
                <div class="row">
                    <DateField label="From" name="usual_from" value=from />
                </div>
                <div class="row thirds">
                    <FigureField label="Sitting (min)" name="usual_sitting" value=sitting />
                    <FigureField label="Standing (min)" name="usual_standing" value=standing />
                    <FigureField label="Moving (km)" name="usual_moving" value=moving />
                </div>
                <Recommend fill=fill said=said />
                <button class="cta" type="submit">
                    "Keep this usual day"
                </button>
                <ReferencesLink topic=Topic::Activity />
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
        </section>
    }
}

#[component]
fn BurnTargetCard() -> impl IntoView {
    let from = RwSignal::new(today());
    let kcal = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    let fill = Callback::new(move |advised: Recommendations| {
        if let Some(target) = advised.burn_target_kcal {
            kcal.set(typed_back(target));
        }
    });
    spawn_local(async move {
        if let Ok(Some(target)) = tauri::burn_target_on(&today()).await {
            kcal.set(typed_back(target));
        }
    });
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([typed]) = figures_typed([&kcal.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let Some(kcal) = typed else {
            said.set(Some(lines::NO_TARGET_TYPED.into()));
            return;
        };
        let form = BurnTargetForm {
            from: from.get_untracked(),
            kcal,
        };
        spawn_local(async move {
            said.set(Some(match tauri::set_burn_target(&form).await {
                Ok(_) => lines::burn_target_kept(&form.from),
                Err(failure) => lines::failure(&failure),
            }));
        });
    };
    view! {
        <section class="card">
            <h2>"The sweat the day owes"</h2>
            <p class="hint">{lines::BURN_TARGET_HINT}</p>
            <form class="burn-target" on:submit=save>
                <div class="row">
                    <DateField label="From" name="burn_target_from" value=from />
                    <FigureField label="Target (kcal a day)" name="burn_target" value=kcal />
                </div>
                <Recommend fill=fill said=said />
                <button class="cta" type="submit">
                    "Keep this target"
                </button>
                <ReferencesLink topic=Topic::Activity />
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
    let fill = Callback::new(move |advised: Recommendations| {
        protein.set(typed_back(advised.fuel.protein_g_per_kg));
        fat.set(typed_back(advised.fuel.fat_g_per_kg));
        fibre.set(typed_back(advised.fuel.fibre_g_per_kg));
    });
    spawn_local(async move {
        if let Ok(targets) = tauri::fuel_targets_on(&today()).await {
            protein.set(targets.protein_g_per_kg.to_string());
            fat.set(targets.fat_g_per_kg.to_string());
            fibre.set(targets.fibre_g_per_kg.to_string());
        }
    });
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([protein_g_per_kg, fat_g_per_kg, fibre_g_per_kg]) = figures_typed([
            &protein.get_untracked(),
            &fat.get_untracked(),
            &fibre.get_untracked(),
        ]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = FuelTargetsForm {
            from: from.get_untracked(),
            protein_g_per_kg: protein_g_per_kg.unwrap_or(0.0),
            fat_g_per_kg: fat_g_per_kg.unwrap_or(0.0),
            fibre_g_per_kg: fibre_g_per_kg.unwrap_or(0.0),
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
            <h2>"How to feed the forge"</h2>
            <p class="hint">
                "Grams per kilo of body mass a day: each is the floor of its band, which also asks for 20 to 33.3% of the day's energy (2.4 to 4% for fibre)."
            </p>
            <form class="fuel-targets" on:submit=save>
                <div class="row">
                    <DateField label="From" name="targets_from" value=from />
                </div>
                <div class="row thirds">
                    <FigureField label="Protein (g/kg)" name="target_protein" value=protein />
                    <FigureField label="Fat (g/kg)" name="target_fat_g" value=fat />
                    <FigureField label="Fibre (g/kg)" name="target_fibre" value=fibre />
                </div>
                <Recommend fill=fill said=said />
                <button class="cta" type="submit">
                    "Keep these targets"
                </button>
                <ReferencesLink topic=Topic::Macronutrients />
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
        let Some([height_cm, neck_cm, hip_cm]) = figures_typed([
            &height.get_untracked(),
            &neck.get_untracked(),
            &hip.get_untracked(),
        ]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        let form = ProfileForm {
            from: from.get_untracked(),
            birthday: birthday.get_untracked(),
            gender: gender.get_untracked(),
            height_cm: height_cm.unwrap_or(0.0),
            neck_cm: neck_cm.unwrap_or(0.0),
            hip_cm: hip_cm.filter(|_| gender.get_untracked() == Gender::Female),
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
            <h2>"The rusty you behind the numbers"</h2>
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
                    <FigureField
                        label="Hip (cm)"
                        name="hip"
                        value=hip
                        disabled=Signal::derive(move || gender.get() == Gender::Male)
                    />
                </div>
                <button class="cta" type="submit">
                    "Keep this profile"
                </button>
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
        </section>
        <GoalsCard />
        <FuelTargetsCard />
        <HydrateCountedCard />
        <UsualDayCard />
        <BurnTargetCard />
        <p class="hint disclaimer">{lines::RECOMMENDATIONS_DISCLAIMER}</p>
    }
}
