use grind_core::figures::{Figure, bare};
use grind_core::food::Nutrients;
use grind_core::week::Averaged;
use leptos::prelude::*;

use crate::screens::lines;

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

/// A measure's row in a week's table: its name and unit, its average, and the figure it reads in.
pub type WeekRow = (&'static str, Averaged, fn(f64) -> Figure);

/// A week's averages over its logged days: a row per measure, a day on average against the average of their
/// bands and coloured by it, with how many days stood in theirs.
pub fn week_table(logged_days: usize, rows: Vec<WeekRow>) -> impl IntoView {
    view! {
        <p class="hint">{lines::over_logged_days(logged_days)}</p>
        <table class="macros">
            <thead>
                <tr>
                    <th></th>
                    <th>"A day"</th>
                    <th>"Band"</th>
                    <th>"In band"</th>
                </tr>
            </thead>
            <tbody>
                {rows
                    .into_iter()
                    .map(|(name, row, figure)| {
                        view! {
                            <tr>
                                <td>{name}</td>
                                <td class=lines::verdict_class(row.verdict())>{bare(figure(row.per_day))}</td>
                                <td>{format!("{}–{}", bare(figure(row.floor)), bare(figure(row.ceiling)))}</td>
                                <td>{format!("{}/{logged_days}", row.in_band)}</td>
                            </tr>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}
