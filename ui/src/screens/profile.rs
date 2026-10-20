use std::future::Future;

use grind_core::bibliography::Topic;
use grind_core::failure::Failure;
use grind_core::figures::typed_back;
use grind_core::for_setting_up_the_profile::{
    BurnTargetForm, FuelTargetsForm, GoalsForm, ProfileForm, Scope, TypedGoals, UsualDayForm,
};
use grind_core::fuel_records::FuelTargets;
use grind_core::gender::Gender;
use grind_core::goals::{Goals, fat_percentage};
use grind_core::kept::{InForce, Kept};
use grind_core::volitional::Volitional;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::screens::fields::{DateField, FigureField};
use crate::screens::hydrate::HydrateTargetCard;
use crate::screens::references::ReferencesLink;
use crate::screens::settings::Settings;
use crate::screens::{Recorded, lines, today};
use crate::tauri;
use crate::typed::figures_typed;

/// Whose values a card shows: the recommendation followed, or values of its own.
#[component]
pub fn Whose(whose: RwSignal<Option<bool>>) -> impl IntoView {
    move || {
        whose
            .get()
            .map(|recommended| view! { <p class="hint whose">{lines::whose(recommended)}</p> })
    }
}

#[component]
pub fn Recommend(keep: Callback<()>) -> impl IntoView {
    view! {
        <button class="cta ghost recommend" type="button" on:click=move |_| keep.run(())>
            "Use the recommendation"
        </button>
    }
}

const SWITCHES: [(bool, &str, &str); 2] = [
    (true, "Counted", "counted"),
    (false, "Left out", "left-out"),
];

/// Whether a card's tick counts in the streak, kept with its values from the card's day.
#[component]
pub fn Counted(counted: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="field streak">
            "Streak"
            <div class="pills">
                {SWITCHES
                    .into_iter()
                    .map(|(on, label, named)| {
                        view! {
                            <button
                                type="button"
                                class=format!("pill {named}")
                                class:chosen=move || counted.get() == on
                                on:click=move |_| counted.set(on)
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

/// Whether the card's tick counts today, read from the streak's scope.
pub fn shown_counted(counted: RwSignal<bool>, tick: fn(&Scope) -> bool) {
    spawn_local(async move {
        if let Ok(scope) = tauri::scope_on(&today()).await {
            counted.set(tick(&scope));
        }
    });
}

/// A card's choice kept: once filed, what is in force fills the card and says whose it is.
pub fn keep_and_show<T: 'static>(
    keeping: impl Future<Output = Result<Option<InForce<T>>, Failure>> + 'static,
    show: impl Fn(T) + 'static,
    whose: RwSignal<Option<bool>>,
    said: RwSignal<Option<String>>,
    (filed, recorded): (String, Recorded),
) {
    spawn_local(async move {
        said.set(Some(match keeping.await {
            Ok(Some(in_force)) => {
                show(in_force.values);
                whose.set(Some(in_force.recommended));
                recorded.told();
                filed
            }
            Ok(None) => lines::NOTHING_TO_RECOMMEND_FOR.into(),
            Err(failure) => lines::failure(&failure),
        }));
    });
}

/// What is in force, shown in the card with whose it is and, once kept, the day it was kept from.
pub fn shown<T>(
    in_force: InForce<T>,
    show: impl Fn(T),
    whose: RwSignal<Option<bool>>,
    from: RwSignal<String>,
) {
    show(in_force.values);
    whose.set(Some(in_force.recommended));
    if let Some(kept_from) = in_force.from {
        from.set(kept_from.to_string());
    }
}

/// What is in force today, shown in the card.
pub fn shown_in_force<T: 'static>(
    reading: impl Future<Output = Result<Option<InForce<T>>, Failure>> + 'static,
    show: impl Fn(T) + 'static,
    whose: RwSignal<Option<bool>>,
    from: RwSignal<String>,
) {
    spawn_local(async move {
        if let Ok(Some(in_force)) = reading.await {
            shown(in_force, show, whose, from);
        }
    });
}

#[component]
fn GoalsCard() -> impl IntoView {
    let from = RwSignal::new(today());
    let target_mass = RwSignal::new(String::new());
    let target_fat = RwSignal::new(String::new());
    let healthy = RwSignal::new(None::<String>);
    let whose = RwSignal::new(None::<bool>);
    let counted = RwSignal::new(true);
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let show = move |goals: Goals| {
        target_mass.set(typed_back(goals.body_mass_kg));
        target_fat.set(typed_back(fat_percentage(&goals)));
    };
    shown_in_force(async { tauri::goals_on(&today()).await }, show, whose, from);
    shown_counted(counted, |scope| scope.weighed);
    spawn_local(async move {
        if let Ok(advised) = tauri::recommended_on(&today()).await {
            healthy.set(Some(lines::healthy_ranges(&advised)));
        }
    });
    let keep = move |kept: Kept<TypedGoals>, filed: fn(&str) -> String| {
        let form = GoalsForm {
            from: from.get_untracked(),
            kept,
            counted: counted.get_untracked(),
        };
        let filed = lines::filed_with_its_tick(filed(&form.from), form.counted);
        keep_and_show(
            async move { tauri::set_goals(&form).await },
            show,
            whose,
            said,
            (filed, recorded),
        );
    };
    let recommend = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([body_mass_kg, fat_percentage]) =
            figures_typed([&target_mass.get_untracked(), &target_fat.get_untracked()])
        else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        keep(
            Kept::Own(TypedGoals {
                body_mass_kg: body_mass_kg.unwrap_or(0.0),
                fat_percentage: fat_percentage.unwrap_or(0.0),
            }),
            lines::goals_kept,
        );
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
                <Whose whose=whose />
                {move || healthy.get().map(|line| view! { <p class="hint healthy">{line}</p> })}
                <Counted counted=counted />
                <Recommend keep=recommend />
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
    let whose = RwSignal::new(None::<bool>);
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let show = move |usual: Volitional| {
        sitting.set(typed_back(usual.sitting_min));
        standing.set(typed_back(usual.standing_min));
        moving.set(typed_back(usual.moving_km));
    };
    shown_in_force(
        async { tauri::usual_day_on(&today()).await },
        show,
        whose,
        from,
    );
    let keep = move |kept: Kept<Volitional>, filed: fn(&str) -> String| {
        let form = UsualDayForm {
            from: from.get_untracked(),
            kept,
        };
        let filed = filed(&form.from);
        keep_and_show(
            async move { tauri::set_usual_day(&form).await },
            show,
            whose,
            said,
            (filed, recorded),
        );
    };
    let recommend = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
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
        keep(
            Kept::Own(Volitional {
                sitting_min: sitting_min.unwrap_or(0.0),
                standing_min: standing_min.unwrap_or(0.0),
                moving_km: moving_km.unwrap_or(0.0),
            }),
            lines::usual_day_kept,
        );
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
                <Whose whose=whose />
                <Recommend keep=recommend />
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
    let unweighed = RwSignal::new(false);
    let whose = RwSignal::new(None::<bool>);
    let counted = RwSignal::new(true);
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let show = move |target: f64| kcal.set(typed_back(target));
    spawn_local(async move {
        match tauri::burn_target_on(&today()).await {
            Ok(Some(target)) => shown(target, show, whose, from),
            Ok(None) => unweighed.set(true),
            Err(_) => {}
        }
    });
    shown_counted(counted, |scope| scope.worked_out);
    let keep = move |kept: Kept<f64>, filed: fn(&str) -> String| {
        let form = BurnTargetForm {
            from: from.get_untracked(),
            kept,
            counted: counted.get_untracked(),
        };
        let filed = lines::filed_with_its_tick(filed(&form.from), form.counted);
        keep_and_show(
            async move { tauri::set_burn_target(&form).await },
            show,
            whose,
            said,
            (filed, recorded),
        );
    };
    let recommend = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
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
        keep(Kept::Own(kcal), lines::burn_target_kept);
    };
    view! {
        <section class="card">
            <h2>"The aches the day owes"</h2>
            <p class="hint">{lines::BURN_TARGET_HINT}</p>
            <form class="burn-target" on:submit=save>
                {move || {
                    unweighed
                        .get()
                        .then(|| view! { <p class="hint">{lines::NO_WEIGH_IN_TO_PRICE_A_WORKOUT}</p> })
                }}
                <div class="row">
                    <DateField label="From" name="burn_target_from" value=from />
                    <FigureField label="Target (kcal a day)" name="burn_target" value=kcal />
                </div>
                <Whose whose=whose />
                <Counted counted=counted />
                <Recommend keep=recommend />
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
    let whose = RwSignal::new(None::<bool>);
    let counted = RwSignal::new(true);
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let show = move |targets: FuelTargets| {
        protein.set(typed_back(targets.protein_g_per_kg));
        fat.set(typed_back(targets.fat_g_per_kg));
        fibre.set(typed_back(targets.fibre_g_per_kg));
    };
    shown_in_force(
        async { tauri::fuel_targets_on(&today()).await },
        show,
        whose,
        from,
    );
    shown_counted(counted, |scope| scope.fuelled);
    let keep = move |kept: Kept<FuelTargets>, filed: fn(&str) -> String| {
        let form = FuelTargetsForm {
            from: from.get_untracked(),
            kept,
            counted: counted.get_untracked(),
        };
        let filed = lines::filed_with_its_tick(filed(&form.from), form.counted);
        keep_and_show(
            async move { tauri::set_fuel_targets(&form).await },
            show,
            whose,
            said,
            (filed, recorded),
        );
    };
    let recommend = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
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
        keep(
            Kept::Own(FuelTargets {
                protein_g_per_kg: protein_g_per_kg.unwrap_or(0.0),
                fat_g_per_kg: fat_g_per_kg.unwrap_or(0.0),
                fibre_g_per_kg: fibre_g_per_kg.unwrap_or(0.0),
            }),
            lines::fuel_targets_kept,
        );
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
                <Whose whose=whose />
                <Counted counted=counted />
                <Recommend keep=recommend />
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
    let recorded = Recorded::here();
    spawn_local(async move {
        if let Ok(Some((kept_from, profile))) = tauri::profile_on(&today()).await {
            from.set(kept_from.to_string());
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
                Ok(_) => {
                    recorded.told();
                    lines::profile_kept(&form.from)
                }
                Err(failure) => lines::failure(&failure),
            }));
        });
    };

    view! {
        <header class="screen-top">
            <h2>"Profile"</h2>
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
        <HydrateTargetCard />
        <BurnTargetCard />
        <UsualDayCard />
        <p class="hint disclaimer">{lines::RECOMMENDATIONS_DISCLAIMER}</p>
        <Settings />
    }
}
