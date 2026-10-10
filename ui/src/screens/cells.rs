use grind_core::figures::{Figure, bare};
use grind_core::food::Nutrients;
use leptos::prelude::*;

/// Energy and macros in the order the food tables head them: kcal, protein, fat, fibre, carbs.
pub fn nutrient_cells(nutrients: Nutrients) -> impl IntoView {
    view! {
        <td>{bare(Figure::Energy(nutrients.energy_kcal))}</td>
        <td>{bare(Figure::Grams(nutrients.protein_g))}</td>
        <td>{bare(Figure::Grams(nutrients.fat_g))}</td>
        <td>{bare(Figure::Grams(nutrients.fibre_g))}</td>
        <td>{bare(Figure::Grams(nutrients.carbs_g))}</td>
    }
}
