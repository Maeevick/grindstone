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
use grind_core::recommendations::Recommendations;
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

/// The recommendations for today, read once for every Profile card: their placeholders and notes.
#[derive(Clone, Copy)]
pub struct Advised(pub RwSignal<Option<Recommendations>>);

impl Advised {
    /// The recommendations the Profile shares, or none outside it.
    pub fn here() -> Self {
        use_context::<Self>().unwrap_or_else(|| Self(RwSignal::new(None)))
    }

    /// A figure of the recommendations, written as a field's placeholder.
    pub fn placeholder(self, figure: fn(&Recommendations) -> Option<f64>) -> Signal<String> {
        Signal::derive(move || {
            self.0
                .get()
                .as_ref()
                .and_then(figure)
                .map(typed_back)
                .unwrap_or_default()
        })
    }
}

/// What a card holds of what is in force: its own values, none while it follows the recommendation.
fn own<T>(in_force: InForce<T>) -> Option<T> {
    (!in_force.recommended).then_some(in_force.values)
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

/// What a card holds in force, in words: how to say it, and what it says now, for a confirmation to recall it.
pub type Held<T> = (fn(&T) -> String, RwSignal<String>);

/// A card's holding, in words: its own values said, or the recommendation.
fn held_now<T>(own: Option<&T>, (say, held): Held<T>) {
    held.set(
        own.map(say)
            .unwrap_or_else(|| lines::THE_RECOMMENDATION.into()),
    );
}

/// A card's choice kept: once filed, its own values fill the card, a recommendation followed leaving it empty,
/// and the confirmation recalls what the card held before.
pub fn keep_and_show<T: 'static>(
    keeping: impl Future<Output = Result<Option<InForce<T>>, Failure>> + 'static,
    show: impl Fn(Option<T>) + 'static,
    said: RwSignal<Option<String>>,
    (filed, recorded): (String, Recorded),
    held: Held<T>,
) {
    spawn_local(async move {
        said.set(Some(match keeping.await {
            Ok(Some(in_force)) => {
                let before = lines::before(&held.1.get_untracked());
                let own = own(in_force);
                held_now(own.as_ref(), held);
                show(own);
                recorded.told();
                format!("{filed} {before}")
            }
            Ok(None) => lines::NOTHING_TO_RECOMMEND_FOR.into(),
            Err(failure) => lines::failure(&failure),
        }));
    });
}

/// What is in force, its own values shown in the card.
pub fn shown<T>(in_force: InForce<T>, show: impl Fn(Option<T>), held: Held<T>) {
    let own = own(in_force);
    held_now(own.as_ref(), held);
    show(own);
}

/// What is in force today, shown in the card.
pub fn shown_in_force<T: 'static>(
    reading: impl Future<Output = Result<Option<InForce<T>>, Failure>> + 'static,
    show: impl Fn(Option<T>) + 'static,
    held: Held<T>,
) {
    spawn_local(async move {
        if let Ok(Some(in_force)) = reading.await {
            shown(in_force, show, held);
        }
    });
}

/// What a field gives once a card is saved with some of its figures typed: its own figure, or its placeholder.
fn typed_or_placeholder(typed: RwSignal<String>, placeholder: Signal<String>) -> String {
    let typed = typed.get_untracked();
    if typed.trim().is_empty() {
        placeholder.get_untracked()
    } else {
        typed
    }
}

/// Whether every field of a card was left empty.
fn all_empty(fields: &[RwSignal<String>]) -> bool {
    fields
        .iter()
        .all(|field| field.get_untracked().trim().is_empty())
}

/// A card's holding before anything is read: the recommendation.
pub fn holding<T>(say: fn(&T) -> String) -> Held<T> {
    (say, RwSignal::new(lines::THE_RECOMMENDATION.into()))
}

#[component]
fn GoalsCard() -> impl IntoView {
    let target_mass = RwSignal::new(String::new());
    let target_fat = RwSignal::new(String::new());
    let advised = Advised::here();
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let show = move |goals: Option<Goals>| {
        target_mass.set(
            goals
                .map(|goals| typed_back(goals.body_mass_kg))
                .unwrap_or_default(),
        );
        target_fat.set(
            goals
                .map(|goals| typed_back(fat_percentage(&goals)))
                .unwrap_or_default(),
        );
    };
    let held = holding(lines::goals_held);
    shown_in_force(async { tauri::goals_on(&today()).await }, show, held);
    let keep = move |kept: Kept<TypedGoals>, filed: fn(&str) -> String| {
        let form = GoalsForm {
            from: today(),
            kept,
        };
        let filed = filed(&form.from);
        keep_and_show(
            async move { tauri::set_goals(&form).await },
            show,
            said,
            (filed, recorded),
            held,
        );
    };
    let mass_placeholder = advised.placeholder(|advised| Some(advised.goal_mass_kg));
    let fat_placeholder = advised.placeholder(|advised| Some(100.0 * advised.goal_fat_ratio));
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if all_empty(&[target_mass, target_fat]) {
            keep(Kept::Recommended, lines::recommendation_kept);
            return;
        }
        let Some([body_mass_kg, fat_percentage]) = figures_typed([
            &typed_or_placeholder(target_mass, mass_placeholder),
            &typed_or_placeholder(target_fat, fat_placeholder),
        ]) else {
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
    let forget = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
    view! {
        <ProfileCard
            title="Where the grindstone is heading"
            form="goals"
            how=lines::GOALS_HOW
            why=Signal::derive(move || {
                advised.0.get().as_ref().map(lines::healthy_ranges).unwrap_or_default()
            })
            topic=Topic::HealthyRanges
            save="Engrave these goals"
            forget=forget
            said=said
            on_submit=Callback::new(save)
        >
            <div class="row">
                <FigureField
                    label="Target mass (kg)"
                    name="target_mass"
                    value=target_mass
                    placeholder=mass_placeholder
                />
                <FigureField
                    label="Target fat (%)"
                    name="target_fat"
                    value=target_fat
                    placeholder=fat_placeholder
                />
            </div>
        </ProfileCard>
    }
}

#[component]
fn UsualDayCard() -> impl IntoView {
    let sitting = RwSignal::new(String::new());
    let standing = RwSignal::new(String::new());
    let moving = RwSignal::new(String::new());
    let advised = Advised::here();
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let show = move |usual: Option<Volitional>| {
        let back = |figure: fn(&Volitional) -> f64| {
            usual
                .as_ref()
                .map(figure)
                .map(typed_back)
                .unwrap_or_default()
        };
        sitting.set(back(|usual| usual.sitting_min));
        standing.set(back(|usual| usual.standing_min));
        moving.set(back(|usual| usual.moving_km));
    };
    let held = holding(lines::usual_day_held);
    shown_in_force(async { tauri::usual_day_on(&today()).await }, show, held);
    let keep = move |kept: Kept<Volitional>, filed: fn(&str) -> String| {
        let form = UsualDayForm {
            from: today(),
            kept,
        };
        let filed = filed(&form.from);
        keep_and_show(
            async move { tauri::set_usual_day(&form).await },
            show,
            said,
            (filed, recorded),
            held,
        );
    };
    let recommend = Callback::new(move |()| keep(Kept::Recommended, lines::recommendation_kept));
    let sitting_placeholder = advised.placeholder(|advised| Some(advised.usual_day.sitting_min));
    let standing_placeholder = advised.placeholder(|advised| Some(advised.usual_day.standing_min));
    let moving_placeholder = advised.placeholder(|advised| Some(advised.usual_day.moving_km));
    let save = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        if all_empty(&[sitting, standing, moving]) {
            keep(Kept::Recommended, lines::recommendation_kept);
            return;
        }
        let Some([sitting_min, standing_min, moving_km]) = figures_typed([
            &typed_or_placeholder(sitting, sitting_placeholder),
            &typed_or_placeholder(standing, standing_placeholder),
            &typed_or_placeholder(moving, moving_placeholder),
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
                <FigureField
                    label="Sitting (min)"
                    name="usual_sitting"
                    value=sitting
                    placeholder=sitting_placeholder
                />
                <FigureField
                    label="Standing (min)"
                    name="usual_standing"
                    value=standing
                    placeholder=standing_placeholder
                />
                <FigureField
                    label="Moving (km)"
                    name="usual_moving"
                    value=moving
                    placeholder=moving_placeholder
                />
            </div>
        </ProfileCard>
    }
}

#[component]
fn BurnTargetCard() -> impl IntoView {
    let kcal = RwSignal::new(String::new());
    let unweighed = RwSignal::new(false);
    let advised = Advised::here();
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let show = move |target: Option<f64>| kcal.set(target.map(typed_back).unwrap_or_default());
    let held = holding(lines::burn_target_held);
    spawn_local(async move {
        match tauri::burn_target_on(&today()).await {
            Ok(Some(target)) => shown(target, show, held),
            Ok(None) => unweighed.set(true),
            Err(_) => {}
        }
    });
    let keep = move |kept: Kept<f64>, filed: fn(&str) -> String| {
        let form = BurnTargetForm {
            from: today(),
            kept,
        };
        let filed = filed(&form.from);
        keep_and_show(
            async move { tauri::set_burn_target(&form).await },
            show,
            said,
            (filed, recorded),
            held,
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
                <FigureField
                    label="Target (kcal a day)"
                    name="burn_target"
                    value=kcal
                    placeholder=advised.placeholder(|advised| advised.burn_target_kcal)
                />
            </div>
        </ProfileCard>
    }
}

#[component]
fn FuelTargetsCard() -> impl IntoView {
    let protein = RwSignal::new(String::new());
    let fat = RwSignal::new(String::new());
    let fibre = RwSignal::new(String::new());
    let carbs = RwSignal::new(String::new());
    let advised = Advised::here();
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let show = move |targets: Option<FuelTargets>| {
        let targets = targets.unwrap_or_default();
        let back = |figure: Option<f64>| figure.map(typed_back).unwrap_or_default();
        protein.set(back(targets.protein_g_per_kg));
        fat.set(back(targets.fat_g_per_kg));
        fibre.set(back(targets.fibre_g_per_kg));
        carbs.set(back(targets.carbs_g_per_kg));
    };
    let held = holding(lines::fuel_targets_held);
    shown_in_force(async { tauri::fuel_targets_on(&today()).await }, show, held);
    let keep = move |kept: Kept<FuelTargets>, filed: fn(&str) -> String| {
        let form = FuelTargetsForm {
            from: today(),
            kept,
        };
        let filed = filed(&form.from);
        keep_and_show(
            async move { tauri::set_fuel_targets(&form).await },
            show,
            said,
            (filed, recorded),
            held,
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
                <FigureField
                    label="Protein (g/kg)"
                    name="target_protein"
                    value=protein
                    placeholder=advised.placeholder(|advised| advised.fuel_floors_g_per_kg.protein_g_per_kg)
                />
                <FigureField
                    label="Fat (g/kg)"
                    name="target_fat_g"
                    value=fat
                    placeholder=advised.placeholder(|advised| advised.fuel_floors_g_per_kg.fat_g_per_kg)
                />
            </div>
            <div class="row">
                <FigureField
                    label="Fibre (g/kg)"
                    name="target_fibre"
                    value=fibre
                    placeholder=advised.placeholder(|advised| advised.fuel_floors_g_per_kg.fibre_g_per_kg)
                />
                <FigureField
                    label="Carbs (g/kg)"
                    name="target_carbs"
                    value=carbs
                    placeholder=advised.placeholder(|advised| advised.fuel_floors_g_per_kg.carbs_g_per_kg)
                />
            </div>
        </ProfileCard>
    }
}

#[component]
pub fn ProfileScreen() -> impl IntoView {
    let birthday = RwSignal::new(String::new());
    let gender = RwSignal::new(Gender::Male);
    let height = RwSignal::new(String::new());
    let neck = RwSignal::new(String::new());
    let hip = RwSignal::new(String::new());
    let said = RwSignal::new(None::<String>);
    let recorded = Recorded::here();
    let held = RwSignal::new(lines::NOTHING_HELD.to_string());
    let advised = Advised(RwSignal::new(None));
    provide_context(advised);
    spawn_local(async move {
        if let Ok(recommended) = tauri::recommended_on(&today()).await {
            advised.0.set(Some(recommended));
        }
    });
    spawn_local(async move {
        if let Ok(Some((_, profile))) = tauri::profile_on(&today()).await {
            held.set(lines::profile_held(&profile));
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
            from: today(),
            birthday: birthday.get_untracked(),
            gender: gender.get_untracked(),
            height_cm: height_cm.unwrap_or(0.0),
            neck_cm: neck_cm.unwrap_or(0.0),
            hip_cm: hip_cm.filter(|_| gender.get_untracked() == Gender::Female),
        };
        spawn_local(async move {
            said.set(Some(match tauri::set_profile(&form).await {
                Ok(profile) => {
                    let before = lines::before(&held.get_untracked());
                    held.set(lines::profile_held(&profile));
                    recorded.told();
                    format!("{} {before}", lines::profile_kept(&form.from))
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
