use grind_core::drink::DrinkKind;
use grind_core::food::{Nutrients, Unit};
use grind_core::for_cataloguing::FoodForm;
use leptos::prelude::*;

use crate::screens::fields::FigureField;
use crate::typed::figures_typed;

pub(crate) fn unit_named(word: &str) -> Unit {
    match word {
        "Millilitre" => Unit::Millilitre,
        "Piece" => Unit::Piece,
        _ => Unit::Gram,
    }
}

pub(crate) const KINDS: [(DrinkKind, &str); 2] =
    [(DrinkKind::Water, "water"), (DrinkKind::Other, "other")];

pub(crate) fn kind_named(word: &str) -> DrinkKind {
    KINDS
        .into_iter()
        .find(|(named, _)| format!("{named:?}") == word)
        .map_or(DrinkKind::Other, |(named, _)| named)
}

/// The heading of the fields a name typed that nothing matches opens.
#[component]
pub fn NewEntry(name: RwSignal<String>) -> impl IntoView {
    view! { <p class="new">{move || format!("New {}", name.get().trim())}</p> }
}

#[component]
fn DescriptionField(name: String, value: RwSignal<String>) -> impl IntoView {
    view! {
        <label class="field grow2">
            "Description"
            <input
                name=name
                autocomplete="off"
                placeholder="Optional"
                prop:value=move || value.get()
                on:input=move |event| value.set(event_target_value(&event))
            />
        </label>
    }
}

/// What a food is typed with, its name aside.
#[derive(Clone, Copy)]
pub struct FoodFigures {
    pub description: RwSignal<String>,
    pub unit: RwSignal<String>,
    pub portion: RwSignal<String>,
    pub energy: RwSignal<String>,
    pub protein: RwSignal<String>,
    pub fat: RwSignal<String>,
    pub fibre: RwSignal<String>,
    pub carbs: RwSignal<String>,
}

impl Default for FoodFigures {
    fn default() -> Self {
        Self {
            description: RwSignal::new(String::new()),
            unit: RwSignal::new("Gram".into()),
            portion: RwSignal::new("100".into()),
            energy: RwSignal::new(String::new()),
            protein: RwSignal::new(String::new()),
            fat: RwSignal::new(String::new()),
            fibre: RwSignal::new(String::new()),
            carbs: RwSignal::new(String::new()),
        }
    }
}

impl FoodFigures {
    pub fn typed(self, name: &str) -> Option<FoodForm> {
        let [portion, energy_kcal, protein_g, carbs_g, fat_g, fibre_g] = figures_typed([
            &self.portion.get(),
            &self.energy.get(),
            &self.protein.get(),
            &self.carbs.get(),
            &self.fat.get(),
            &self.fibre.get(),
        ])?;
        Some(FoodForm {
            name: name.trim().into(),
            unit: unit_named(&self.unit.get()),
            portion: portion.unwrap_or(0.0),
            nutrients: Nutrients {
                energy_kcal: energy_kcal.unwrap_or(0.0),
                protein_g: protein_g.unwrap_or(0.0),
                carbs_g: carbs_g.unwrap_or(0.0),
                fat_g: fat_g.unwrap_or(0.0),
                fibre_g: fibre_g.unwrap_or(0.0),
            },
            description: self.description.get(),
        })
    }

    pub fn cleared(self) {
        for typed in [
            self.description,
            self.energy,
            self.protein,
            self.fat,
            self.fibre,
            self.carbs,
        ] {
            typed.set(String::new());
        }
        self.unit.set("Gram".into());
        self.portion.set("100".into());
    }
}

#[component]
pub fn FoodFields(prefix: &'static str, figures: FoodFigures) -> impl IntoView {
    view! {
        <div class="row">
            <DescriptionField name=format!("{prefix}_description") value=figures.description />
        </div>
        <div class="row thirds">
            <FigureField label="Per" name=format!("{prefix}_portion") value=figures.portion />
            <label class="field">
                "Unit"
                <select
                    name=format!("{prefix}_unit")
                    prop:value=move || figures.unit.get()
                    on:change=move |event| figures.unit.set(event_target_value(&event))
                >
                    <option value="Gram">"g"</option>
                    <option value="Millilitre">"mL"</option>
                    <option value="Piece">"piece"</option>
                </select>
            </label>
            <FigureField label="kcal" name=format!("{prefix}_energy") value=figures.energy />
        </div>
        <div class="row quarters">
            <FigureField label="Protein (g)" name=format!("{prefix}_protein") value=figures.protein />
            <FigureField label="Fat (g)" name=format!("{prefix}_fat") value=figures.fat />
            <FigureField label="Fibre (g)" name=format!("{prefix}_fibre") value=figures.fibre />
            <FigureField label="Carbs (g)" name=format!("{prefix}_carbs") value=figures.carbs />
        </div>
    }
}

/// What a drink is typed with, its name aside.
#[derive(Clone, Copy)]
pub struct DrinkFigures {
    pub description: RwSignal<String>,
    pub serving: RwSignal<String>,
    pub kind: RwSignal<DrinkKind>,
}

impl Default for DrinkFigures {
    fn default() -> Self {
        Self {
            description: RwSignal::new(String::new()),
            serving: RwSignal::new(String::new()),
            kind: RwSignal::new(DrinkKind::Water),
        }
    }
}

impl DrinkFigures {
    pub fn cleared(self) {
        self.description.set(String::new());
        self.serving.set(String::new());
        self.kind.set(DrinkKind::Water);
    }
}

#[component]
pub fn DrinkFields(prefix: &'static str, figures: DrinkFigures) -> impl IntoView {
    view! {
        <div class="row">
            <DescriptionField name=format!("{prefix}_description") value=figures.description />
            <FigureField label="Serving (mL)" name=format!("{prefix}_serving") value=figures.serving />
            <label class="field">
                "Kind"
                <select
                    name=format!("{prefix}_kind")
                    prop:value=move || format!("{:?}", figures.kind.get())
                    on:change=move |event| figures.kind.set(kind_named(&event_target_value(&event)))
                >
                    {KINDS
                        .into_iter()
                        .map(|(named, label)| {
                            view! { <option value=format!("{named:?}")>{label}</option> }
                        })
                        .collect_view()}
                </select>
            </label>
        </div>
    }
}

/// What an exercise is typed with, its name aside.
#[derive(Clone, Copy)]
pub struct ExerciseFigures {
    pub description: RwSignal<String>,
    pub met: RwSignal<String>,
}

impl Default for ExerciseFigures {
    fn default() -> Self {
        Self {
            description: RwSignal::new(String::new()),
            met: RwSignal::new(String::new()),
        }
    }
}

impl ExerciseFigures {
    pub fn cleared(self) {
        self.description.set(String::new());
        self.met.set(String::new());
    }
}

#[component]
pub fn ExerciseFields(prefix: &'static str, figures: ExerciseFigures) -> impl IntoView {
    view! {
        <div class="row">
            <DescriptionField name=format!("{prefix}_description") value=figures.description />
            <FigureField label="MET" name=format!("{prefix}_met") value=figures.met />
        </div>
    }
}
