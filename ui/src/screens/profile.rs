use std::future::Future;

use grind_core::bibliography::Topic;
use grind_core::failure::Failure;
use grind_core::figures::typed_back;
use grind_core::for_setting_up_the_profile::{
    BurnTargetForm, FuelTargetsForm, GoalsForm, ProfileForm, TypedGoals, UsualDayForm,
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

/// A Profile card: its title, how to fill it, its fields, why its recommendation stands with the references
/// behind it, then its buttons and what was said.
#[component]
pub fn ProfileCard(
    title: &'static str,
    form: &'static str,
    how: &'static str,
    #[prop(into)] why: Signal<String>,
    topic: Topic,
    save: &'static str,
    #[prop(optional)] forget: Option<Callback<()>>,
    said: RwSignal<Option<String>>,
    on_submit: Callback<leptos::ev::SubmitEvent>,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="card profile-card">
            <h2>{title}</h2>
            <p class="hint how">{how}</p>
            <form class=form on:submit=move |event| on_submit.run(event)>
                {children()}
                <p class="hint why">{move || why.get()} " " <ReferencesLink topic=topic /></p>
                <button class="cta" type="submit">
                    {save}
                </button>
                {forget
                    .map(|forget| {
                        view! {
                            <button class="cta ghost forget" type="button" on:click=move |_| forget.run(())>
                                "Forget it 🔄"
                            </button>
                        }
                    })}
                {move || said.get().map(|line| view! { <p class="said">{line}</p> })}
            </form>
        </section>
    }
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
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let show = move |goals: Goals| {
        target_mass.set(typed_back(goals.body_mass_kg));
        target_fat.set(typed_back(fat_percentage(&goals)));
    };
    shown_in_force(async { tauri::goals_on(&today()).await }, show, whose, from);
    spawn_local(async move {
        if let Ok(advised) = tauri::recommended_on(&today()).await {
            healthy.set(Some(lines::healthy_ranges(&advised)));
        }
    });
    let keep = move |kept: Kept<TypedGoals>, filed: fn(&str) -> String| {
        let form = GoalsForm {
            from: from.get_untracked(),
            kept,
        };
        let filed = filed(&form.from);
        keep_and_show(
            async move { tauri::set_goals(&form).await },
            show,
            whose,
            said,
            (filed, recorded),
        );
    };
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([body_mass_kg, fat_percentage]) =
            figures_typed([&target_mass.get_untracked(), &target_fat.get_untracked()])
        else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        match (body_mass_kg, fat_percentage) {
            (None, None) => keep(Kept::Recommended, lines::recommendation_kept),
            _ => keep(
                Kept::Own(TypedGoals {
                    body_mass_kg: body_mass_kg.unwrap_or(0.0),
                    fat_percentage: fat_percentage.unwrap_or(0.0),
                }),
                lines::goals_kept,
            ),
        }
    };
    let forget = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
    view! {
        <ProfileCard
            title="Where the grindstone is heading"
            form="goals"
            how=lines::GOALS_HOW
            why=Signal::derive(move || healthy.get().unwrap_or_default())
            topic=Topic::HealthyRanges
            save="Engrave these goals"
            forget=forget
            said=said
            on_submit=Callback::new(save)
        >
            <div class="row">
                <FigureField label="Target mass (kg)" name="target_mass" value=target_mass />
                <FigureField label="Target fat (%)" name="target_fat" value=target_fat />
            </div>
            <Whose whose=whose />
        </ProfileCard>
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
        <ProfileCard
            title="The forgotten bucket of sweat"
            form="usual-day"
            how=lines::USUAL_DAY_HOW
            why=lines::USUAL_DAY_WHY.to_string()
            topic=Topic::Activity
            save="Keep this usual day"
            forget=recommend
            said=said
            on_submit=Callback::new(save)
        >
            <div class="row thirds">
                <FigureField label="Sitting (min)" name="usual_sitting" value=sitting />
                <FigureField label="Standing (min)" name="usual_standing" value=standing />
                <FigureField label="Moving (km)" name="usual_moving" value=moving />
            </div>
            <Whose whose=whose />
        </ProfileCard>
    }
}

#[component]
fn BurnTargetCard() -> impl IntoView {
    let from = RwSignal::new(today());
    let kcal = RwSignal::new(String::new());
    let unweighed = RwSignal::new(false);
    let whose = RwSignal::new(None::<bool>);
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
    let keep = move |kept: Kept<f64>, filed: fn(&str) -> String| {
        let form = BurnTargetForm {
            from: from.get_untracked(),
            kept,
        };
        let filed = filed(&form.from);
        keep_and_show(
            async move { tauri::set_burn_target(&form).await },
            show,
            whose,
            said,
            (filed, recorded),
        );
    };
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some([typed]) = figures_typed([&kcal.get_untracked()]) else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        match typed {
            Some(kcal) => keep(Kept::Own(kcal), lines::burn_target_kept),
            None => keep(Kept::Recommended, lines::recommendation_kept),
        }
    };
    let forget = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
    let why = Signal::derive(move || {
        if unweighed.get() {
            format!(
                "{} {}",
                lines::BURN_TARGET_WHY,
                lines::NO_WEIGH_IN_TO_PRICE_A_WORKOUT
            )
        } else {
            lines::BURN_TARGET_WHY.to_string()
        }
    });
    view! {
        <ProfileCard
            title="The aches the day owes"
            form="burn-target"
            how=lines::BURN_TARGET_HOW
            why=why
            topic=Topic::Activity
            save="Fire this ambition"
            forget=forget
            said=said
            on_submit=Callback::new(save)
        >
            <div class="row">
                <FigureField label="Target (kcal a day)" name="burn_target" value=kcal />
            </div>
            <Whose whose=whose />
        </ProfileCard>
    }
}

#[component]
fn FuelTargetsCard() -> impl IntoView {
    let from = RwSignal::new(today());
    let protein = RwSignal::new(String::new());
    let fat = RwSignal::new(String::new());
    let fibre = RwSignal::new(String::new());
    let carbs = RwSignal::new(String::new());
    let whose = RwSignal::new(None::<bool>);
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let show = move |targets: FuelTargets| {
        let back = |figure: Option<f64>| figure.map(typed_back).unwrap_or_default();
        protein.set(back(targets.protein_g_per_kg));
        fat.set(back(targets.fat_g_per_kg));
        fibre.set(back(targets.fibre_g_per_kg));
        carbs.set(back(targets.carbs_g_per_kg));
    };
    shown_in_force(
        async { tauri::fuel_targets_on(&today()).await },
        show,
        whose,
        from,
    );
    let keep = move |kept: Kept<FuelTargets>, filed: fn(&str) -> String| {
        let form = FuelTargetsForm {
            from: from.get_untracked(),
            kept,
        };
        let filed = filed(&form.from);
        keep_and_show(
            async move { tauri::set_fuel_targets(&form).await },
            show,
            whose,
            said,
            (filed, recorded),
        );
    };
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let Some(
            [
                protein_g_per_kg,
                fat_g_per_kg,
                fibre_g_per_kg,
                carbs_g_per_kg,
            ],
        ) = figures_typed([
            &protein.get_untracked(),
            &fat.get_untracked(),
            &fibre.get_untracked(),
            &carbs.get_untracked(),
        ])
        else {
            said.set(Some(lines::UNREADABLE.into()));
            return;
        };
        keep(
            Kept::Own(FuelTargets {
                protein_g_per_kg,
                fat_g_per_kg,
                fibre_g_per_kg,
                carbs_g_per_kg,
            }),
            lines::fuel_targets_kept,
        );
    };
    let forget = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
    view! {
        <ProfileCard
            title="How to feed the forge"
            form="fuel-targets"
            how=lines::FUEL_TARGETS_HOW
            why=lines::FUEL_TARGETS_WHY.to_string()
            topic=Topic::Macronutrients
            save="Paint these targets"
            forget=forget
            said=said
            on_submit=Callback::new(save)
        >
            <div class="row">
                <FigureField label="Protein (g/kg)" name="target_protein" value=protein />
                <FigureField label="Fat (g/kg)" name="target_fat_g" value=fat />
            </div>
            <div class="row">
                <FigureField label="Fibre (g/kg)" name="target_fibre" value=fibre />
                <FigureField label="Carbs (g/kg)" name="target_carbs" value=carbs />
            </div>
            <Whose whose=whose />
        </ProfileCard>
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
        <ProfileCard
            title="The rusty you behind the numbers"
            form="profile"
            how=lines::PROFILE_HOW
            why=lines::PROFILE_WHY.to_string()
            topic=Topic::BodyComposition
            save="Keep this profile"
            said=said
            on_submit=Callback::new(save)
        >
            <div class="row">
                <DateField label="Birthday" name="birthday" value=birthday />
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
            </div>
            <div class="row thirds">
                <FigureField label="Height (cm)" name="height" value=height />
                <FigureField label="Neck (cm)" name="neck" value=neck />
                <FigureField
                    label="Hip (cm)"
                    name="hip"
                    value=hip
                    disabled=Signal::derive(move || gender.get() == Gender::Male)
                />
            </div>
        </ProfileCard>
        <GoalsCard />
        <FuelTargetsCard />
        <HydrateTargetCard />
        <BurnTargetCard />
        <UsualDayCard />
        <p class="hint disclaimer">{lines::RECOMMENDATIONS_DISCLAIMER}</p>
        <Settings />
    }
}
