use std::cmp::Reverse;

use chrono::{Datelike, NaiveDate};
use grind_core::figures::{Figure, bare};
use grind_core::for_charting::{BurnRow, FuelRow, GatherRow, HydrateRow, Month};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::days::Shown;
use crate::screens::charts::{bare_or_blank, burn_cells, fuel_cells, header, hydrate_cells};
use crate::screens::{Recorded, lines};
use crate::tauri;

fn gathered_cells(row: GatherRow) -> impl IntoView {
    view! {
        <td>{row.day.to_string()}</td>
        <td>{bare(Figure::Mass(row.body_mass_kg))}</td>
        <td>{bare_or_blank(row.waist_cm, Figure::Length)}</td>
        <td>{bare_or_blank(row.fat_ratio, Figure::Ratio)}</td>
        <td>{bare_or_blank(row.lean_mass_kg, Figure::Mass)}</td>
    }
}

/// What loading more reads: the rest of the first month shown, else the month before it.
fn more_said(shown: Shown, earlier: Option<&str>) -> Option<String> {
    if shown.from.day() > 1 {
        Some(lines::load_the_rest_of(
            &shown.from.format("%Y-%m").to_string(),
        ))
    } else {
        earlier.map(lines::load_the_month)
    }
}

macro_rules! last_days {
    ($name:ident, $row:ty, $load:path, $render:path, [$(($head:literal, $unit:literal)),*]) => {
        /// The days up to the tab's day, a week at first, then further back a load at a time.
        #[component]
        pub fn $name(day: RwSignal<String>) -> impl IntoView {
            let recorded = Recorded::here();
            let shown = RwSignal::new(None::<Shown>);
            let months = RwSignal::new(Vec::<Month<$row>>::new());
            let said = RwSignal::new(None::<String>);
            Effect::new(move |_| {
                recorded.heard();
                let chosen = NaiveDate::parse_from_str(&day.get(), "%Y-%m-%d").ok();
                months.set(Vec::new());
                shown.set(chosen.map(Shown::a_week_up_to));
            });
            Effect::new(move |_| {
                let Some(span) = shown.get() else {
                    return;
                };
                let read: Vec<String> =
                    months.with_untracked(|kept| kept.iter().map(|month| month.month.clone()).collect());
                for month in span.months().into_iter().filter(|month| !read.contains(month)) {
                    spawn_local(async move {
                        match $load(&month).await {
                            Ok(loaded) => months.update(|kept| {
                                kept.retain(|month| month.month != loaded.month);
                                kept.push(loaded);
                            }),
                            Err(failure) => said.set(Some(lines::failure(&failure))),
                        }
                    });
                }
            });
            let rows = move || {
                let Some(span) = shown.get() else {
                    return Vec::new();
                };
                let mut rows: Vec<$row> = months
                    .get()
                    .into_iter()
                    .flat_map(|month| month.rows)
                    .filter(|row| span.holds(row.day))
                    .collect();
                rows.sort_by_key(|row| Reverse(row.day));
                rows
            };
            let earlier = move || {
                let oldest = shown.get()?.months().last().cloned()?;
                months
                    .get()
                    .into_iter()
                    .find(|month| month.month == oldest)
                    .map(|month| month.earlier)
            };
            let more = move || {
                let span = shown.get()?;
                let earlier = earlier()?;
                Some((span.further_back(earlier.as_deref())?, more_said(span, earlier.as_deref())?))
            };
            view! {
                <section class="card last-days">
                    <h2>"The last days"</h2>
                    <div class="scroll">
                        <table>
                            <thead>
                                <tr>
                                    $(<th>{header($head, $unit)}</th>)*
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    let chosen = day.get();
                                    rows()
                                        .into_iter()
                                        .map(|row| {
                                            let is_chosen = row.day.to_string() == chosen;
                                            view! { <tr class:chosen=is_chosen>{$render(row)}</tr> }
                                        })
                                        .collect_view()
                                }}
                            </tbody>
                        </table>
                    </div>
                    {move || rows().is_empty().then(|| view! { <p class="empty">{lines::NOTHING_THESE_DAYS}</p> })}
                    {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
                    {move || {
                        more()
                            .map(|(further, label)| {
                                view! {
                                    <button
                                        class="cta ghost more"
                                        type="button"
                                        on:click=move |_| shown.set(Some(further))
                                    >
                                        {label}
                                    </button>
                                }
                            })
                    }}
                </section>
            }
        }
    };
}

last_days!(
    GatherLastDays,
    GatherRow,
    tauri::gather_month,
    gathered_cells,
    [
        ("Day", ""),
        ("Mass", "kg"),
        ("Waist", "cm"),
        ("Fat", "%"),
        ("Lean", "kg")
    ]
);
last_days!(
    FuelLastDays,
    FuelRow,
    tauri::fuel_month,
    fuel_cells,
    [
        ("Day", ""),
        ("Energy", "kcal"),
        ("Prot", "g"),
        ("Fat", "g"),
        ("Carbs", "g"),
        ("Fib", "g")
    ]
);
last_days!(
    HydrateLastDays,
    HydrateRow,
    tauri::hydrate_month,
    hydrate_cells,
    [("Day", ""), ("Drunk", "mL"), ("Water %", "")]
);
last_days!(
    BurnLastDays,
    BurnRow,
    tauri::burn_month,
    burn_cells,
    [
        ("Day", ""),
        ("Burnt", "kcal"),
        ("Duration", "min"),
        ("Reps", ""),
        ("Top exercise", "")
    ]
);
