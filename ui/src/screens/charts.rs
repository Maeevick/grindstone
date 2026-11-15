use chrono::NaiveDate;
use grind_core::figures::{Figure, bare};
use grind_core::for_charting::{
    BandChart, BurnChart, BurnRow, FuelRow, GatherChart, GatherRow, HydrateRow, Month,
};
use grind_core::for_reviewing_progress::Window;
use grind_core::streak::{Status, StreakDay};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::chart::{
    HEIGHT, Label, WIDTH, axis_of, clear_of, days_along, labelled, level, line_of, spread_out,
    ticks_on,
};
use crate::lines;
use crate::screens::Recorded;
use crate::screens::burn::BurnEntries;
use crate::screens::cells::opening_cell;
use crate::screens::fuel::FuelEntries;
use crate::screens::header::tick_class;
use crate::screens::hydrate::HydrateEntries;
use crate::screens::pills::pill_buttons;
use crate::screens::said::refused_line;
use crate::screens::tips::{BurnOver, FuelOver, HydrateOver, PaceOver};
use crate::tauri;

/// Which tab a chart, its stats and its history belong to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Chip {
    Gather,
    Fuel,
    Hydrate,
    Burn,
}

const WINDOWS: [(Window, &str); 3] = [
    (Window::Days30, "30 days"),
    (Window::Days90, "90 days"),
    (Window::Everything, "∞"),
];

pub type Points = Vec<(NaiveDate, f64)>;

#[derive(Clone)]
pub struct Line {
    pub points: String,
    pub class: &'static str,
}

#[derive(Clone, Default)]
pub struct Plotted {
    pub lines: Vec<Line>,
    pub left: Vec<Label>,
    pub right: Vec<Label>,
    pub ticks: Vec<Label>,
    pub days: Vec<(f64, String)>,
    pub legend: Vec<(&'static str, &'static str)>,
}

pub fn span_of<'a>(
    series: impl Iterator<Item = &'a (NaiveDate, f64)> + Clone,
) -> Option<(NaiveDate, NaiveDate)> {
    let first = series.clone().map(|(day, _)| *day).min()?;
    let last = series.map(|(day, _)| *day).max()?;
    Some((first, last))
}

fn current(points: &[(NaiveDate, f64)]) -> Option<f64> {
    points.last().map(|(_, value)| *value)
}

/// Both masses on one axis, as both are kilograms, each with its goal.
fn gather_plot(chart: &GatherChart) -> Option<Plotted> {
    let span = span_of(chart.masses.iter().chain(&chart.lean_masses))?;
    let values = chart
        .masses
        .iter()
        .chain(&chart.lean_masses)
        .map(|(_, kg)| *kg)
        .chain(chart.goal_kg)
        .chain(chart.goal_lean_kg);
    let axis = axis_of(values)?;
    let scale = axis.scale();
    let mut lines = vec![
        Line {
            points: line_of(&chart.masses, span, scale, WIDTH, HEIGHT),
            class: "mass",
        },
        Line {
            points: line_of(&chart.lean_masses, span, scale, WIDTH, HEIGHT),
            class: "lean",
        },
    ];
    lines.extend(chart.goal_kg.map(|kg| Line {
        points: level(kg, scale),
        class: "goal dashed",
    }));
    lines.extend(chart.goal_lean_kg.map(|kg| Line {
        points: level(kg, scale),
        class: "lean dashed",
    }));
    Some(Plotted {
        lines,
        left: [
            labelled(chart.goal_kg, scale, Figure::Mass, "goal"),
            labelled(chart.goal_lean_kg, scale, Figure::Mass, "lean"),
        ]
        .into_iter()
        .flatten()
        .collect(),
        right: [
            labelled(current(&chart.masses), scale, Figure::Mass, "mass"),
            labelled(current(&chart.lean_masses), scale, Figure::Mass, "lean"),
        ]
        .into_iter()
        .flatten()
        .collect(),
        ticks: ticks_on(axis, Figure::Mass, ""),
        days: days_along(span, WIDTH),
        legend: vec![
            ("mass", "Body mass"),
            ("lean", "Lean mass"),
            ("goal dashed", "Body mass goal"),
            ("lean dashed", "Lean mass goal"),
        ],
    })
}

fn band_plot(
    chart: &BandChart,
    figure: fn(f64) -> Figure,
    amount: (&'static str, &'static str),
) -> Option<Plotted> {
    let span = span_of(chart.amounts.iter())?;
    let values = chart
        .amounts
        .iter()
        .chain(&chart.floors)
        .chain(&chart.caps)
        .map(|(_, value)| *value);
    let axis = axis_of(values)?;
    let scale = axis.scale();
    let (class, name) = amount;
    Some(Plotted {
        lines: vec![
            Line {
                points: line_of(&chart.amounts, span, scale, WIDTH, HEIGHT),
                class,
            },
            Line {
                points: line_of(&chart.floors, span, scale, WIDTH, HEIGHT),
                class: "floor",
            },
            Line {
                points: line_of(&chart.caps, span, scale, WIDTH, HEIGHT),
                class: "cap",
            },
        ],
        left: vec![],
        right: [
            labelled(current(&chart.amounts), scale, figure, class),
            labelled(current(&chart.floors), scale, figure, "floor"),
            labelled(current(&chart.caps), scale, figure, "cap"),
        ]
        .into_iter()
        .flatten()
        .collect(),
        ticks: ticks_on(axis, figure, ""),
        days: days_along(span, WIDTH),
        legend: vec![(class, name), ("floor", "Floor"), ("cap", "Cap")],
    })
}

/// The energy burnt, and the day's target as a dashed line on the same scale, from nothing up.
fn burn_plot(chart: &BurnChart) -> Option<Plotted> {
    let span = span_of(chart.burnt.iter())?;
    let burnt = axis_of(
        chart
            .burnt
            .iter()
            .chain(&chart.targets)
            .map(|(_, kcal)| *kcal)
            .chain([0.0]),
    )?;
    Some(Plotted {
        lines: vec![
            Line {
                points: line_of(&chart.burnt, span, burnt.scale(), WIDTH, HEIGHT),
                class: "burnt",
            },
            Line {
                points: line_of(&chart.targets, span, burnt.scale(), WIDTH, HEIGHT),
                class: "goal dashed",
            },
        ],
        left: vec![],
        right: [
            labelled(
                current(&chart.burnt),
                burnt.scale(),
                Figure::Energy,
                "burnt",
            ),
            labelled(
                current(&chart.targets),
                burnt.scale(),
                Figure::Energy,
                "goal",
            ),
        ]
        .into_iter()
        .flatten()
        .collect(),
        ticks: ticks_on(burnt, Figure::Energy, "burnt"),
        days: days_along(span, WIDTH),
        legend: vec![("burnt", "Burnt (kcal)"), ("goal dashed", "Target")],
    })
}

#[component]
pub fn Plot(plotted: Option<Plotted>) -> impl IntoView {
    match plotted {
        None => view! { <p class="empty">{lines::NOTHING_TO_CHART}</p> }.into_any(),
        Some(plotted) => {
            let place = |side: &'static str| {
                move |label: Label| {
                    view! {
                        <span
                            class=format!("label {side} {}", label.class)
                            style=format!("top: {:.1}%", label.at / HEIGHT * 100.0)
                        >
                            {label.text}
                        </span>
                    }
                }
            };
            let tick = |side: &'static str| {
                move |label: Label| {
                    view! {
                        <span
                            class=format!("label {side} tick {}", label.class)
                            style=format!("top: {:.1}%", label.at / HEIGHT * 100.0)
                        >
                            {label.text}
                        </span>
                    }
                }
            };
            let (left, right) = (spread_out(plotted.left), spread_out(plotted.right));
            let grid: Vec<f64> = plotted.ticks.iter().map(|tick| tick.at).collect();
            view! {
                <div class="plot">
                    <svg viewBox="0 0 360 180" preserveAspectRatio="none" aria-hidden="true">
                        {grid
                            .into_iter()
                            .map(|y| view! { <line class="grid" x1="0" x2="360" y1=y y2=y /> })
                            .collect_view()}
                        <line class="axis" x1="0" x2="0" y1="0" y2="180" />
                        <line class="x-axis" x1="0" x2="360" y1="180" y2="180" />
                        {plotted
                            .days
                            .iter()
                            .map(|(x, _)| view! { <line class="mark" x1=*x x2=*x y1="180" y2="186" /> })
                            .collect_view()}
                        {plotted
                            .lines
                            .into_iter()
                            .map(|line| view! { <polyline class=line.class points=line.points /> })
                            .collect_view()}
                    </svg>
                    {clear_of(plotted.ticks, &left).into_iter().map(tick("left")).collect_view()}
                    {left.into_iter().map(place("left")).collect_view()}
                    {right.into_iter().map(place("right")).collect_view()}
                    {plotted
                        .days
                        .into_iter()
                        .map(|(x, day)| {
                            view! {
                                <span class="label x" style=format!("left: {:.1}%", x / WIDTH * 100.0)>
                                    {day}
                                </span>
                            }
                        })
                        .collect_view()}
                </div>
                <ul class="legend">
                    {plotted
                        .legend
                        .into_iter()
                        .map(|(class, name)| {
                            view! {
                                <li>
                                    <span class=format!("swatch {class}")></span>
                                    {name}
                                </li>
                            }
                        })
                        .collect_view()}
                </ul>
            }
            .into_any()
        }
    }
}

#[component]
fn ChipPlot(chip: Chip, today: String, window: RwSignal<Window>) -> impl IntoView {
    let plotted = LocalResource::new(move || {
        let (today, window) = (today.clone(), window.get());
        async move {
            match chip {
                Chip::Gather => tauri::gather_chart(&today, window)
                    .await
                    .map(|chart| gather_plot(&chart)),
                Chip::Fuel => tauri::fuel_chart(&today, window)
                    .await
                    .map(|chart| band_plot(&chart, Figure::Energy, ("intake", "Energy eaten"))),
                Chip::Hydrate => tauri::hydrate_chart(&today, window)
                    .await
                    .map(|chart| band_plot(&chart, Figure::Volume, ("drunk", "Drunk"))),
                Chip::Burn => tauri::burn_chart(&today, window)
                    .await
                    .map(|chart| burn_plot(&chart)),
            }
        }
    });
    view! {
        <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
            {move || {
                plotted
                    .get()
                    .map(|answer| match answer {
                        Ok(plotted) => view! { <Plot plotted=plotted /> }.into_any(),
                        Err(failure) => {
                            view! { <p class="bad">{lines::failure(&failure)}</p> }.into_any()
                        }
                    })
            }}
        </Transition>
    }
}

fn months_of<Row: Clone + Send + Sync + 'static>(months: RwSignal<Vec<Month<Row>>>) -> Vec<Row> {
    months
        .get()
        .into_iter()
        .flat_map(|month| month.rows)
        .collect()
}

#[component]
fn LoadMore(earlier: Option<String>, load: Callback<String>) -> impl IntoView {
    earlier.map(|month| {
        let label = lines::load_the_month(&month);
        view! {
            <button class="cta ghost more" type="button" on:click=move |_| load.run(month.clone())>
                {label}
            </button>
        }
    })
}

/// A status's mark inside its dot, so it never rests on colour alone; under way keeps its hollow ring.
fn status_mark(status: Status) -> &'static str {
    match status {
        Status::Success => "✓",
        Status::Joker => "★",
        Status::Break => "✗",
        Status::Pending => "",
    }
}

fn streak_cell(status: Option<Status>) -> impl IntoView {
    view! {
        <td>
            {status
                .map(|status| {
                    view! {
                        <span class=format!("dot {status:?}") role="img" aria-label=lines::status_word(status)>
                            {status_mark(status)}
                        </span>
                    }
                })}
        </td>
    }
}

pub(crate) fn header(name: &'static str, unit: &'static str) -> impl IntoView {
    view! {
        {name}
        {(!unit.is_empty())
            .then(|| {
                view! {
                    " "
                    <span class="unit">{format!("({unit})")}</span>
                }
            })}
    }
}

pub(crate) fn bare_or_blank(value: Option<f64>, figure: fn(f64) -> Figure) -> String {
    value.map(|value| bare(figure(value))).unwrap_or_default()
}

/// A day's own figure, or a dash on a day it was not taken.
pub(crate) fn bare_or_dash(value: Option<f64>, figure: fn(f64) -> Figure) -> String {
    value.map_or_else(|| "–".to_string(), |value| bare(figure(value)))
}

fn gather_cells(row: GatherRow) -> impl IntoView {
    view! {
            <td>{bare_or_dash(row.body_mass_kg, Figure::Mass)}</td>
            <td>{bare_or_blank(row.lean_mass_kg, Figure::Mass)}</td>
            <td>{bare_or_blank(row.fat_mass_kg, Figure::Mass)}</td>
            <td>{bare_or_blank(row.fat_ratio, Figure::Ratio)}</td>
    }
}

pub(crate) fn fuel_cells(row: FuelRow) -> impl IntoView {
    view! {
            <td>{bare(Figure::Energy(row.eaten.energy_kcal))}</td>
            <td>{bare(Figure::Grams(row.eaten.protein_g))}</td>
            <td>{bare(Figure::Grams(row.eaten.fat_g))}</td>
            <td>{bare(Figure::Grams(row.eaten.fibre_g))}</td>
            <td>{bare(Figure::Grams(row.eaten.carbs_g))}</td>
            <td>{bare(Figure::Grams(row.quantity))}</td>
    }
}

pub(crate) fn hydrate_cells(row: HydrateRow) -> impl IntoView {
    view! {
            <td>{bare(Figure::Volume(row.drunk_ml))}</td>
            <td>{bare(Figure::Ratio(row.water_ratio))}</td>
    }
}

pub(crate) fn burn_cells(row: BurnRow) -> impl IntoView {
    view! {
            <td>{bare_or_blank(row.burnt_kcal, Figure::Energy)}</td>
            <td>{bare(Figure::Duration(row.minutes))}</td>
            <td>{bare(Figure::Count(row.reps))}</td>
    }
}

fn ticks_of(streaked: StreakDay) -> impl IntoView {
    let ticks = streaked.ticks;
    view! {
        <span class="status">{lines::status_word(streaked.status)}</span>
        {[
            ("⚖️", "weigh-in", ticks.weighed),
            ("⚡", "fuel", ticks.fuelled),
            ("💧", "hydrate", ticks.hydrated),
            ("🔥", "burn", ticks.worked_out),
        ]
            .into_iter()
            .map(|(icon, name, held)| {
                view! {
                    <span class=format!("tick {}", tick_class(held)) title=lines::tick_said(name, held)>
                        {format!("{icon} {}", lines::tick_mark(held))}
                    </span>
                }
            })
            .collect_view()}
    }
}

/// A day of the history: its cells and streak dot; opened, its ticks when it was streaked, then its details.
#[component]
fn HistoryRow(
    day: String,
    streaked: Option<StreakDay>,
    chosen: bool,
    columns: usize,
    open: RwSignal<Vec<String>>,
    details: ViewFn,
    children: Children,
) -> impl IntoView {
    let this = day.clone();
    let opened = Signal::derive(move || open.with(|open| open.contains(&this)));
    let toggled = day.clone();
    view! {
        <tr
            class:streaked=streaked.is_some()
            class:chosen=chosen
            class:open=opened
            on:click=move |_| {
                open.update(|open| {
                    if let Some(at) = open.iter().position(|day| *day == toggled) {
                        open.remove(at);
                    } else {
                        open.push(toggled.clone());
                    }
                })
            }
        >
            {opening_cell(day, opened)}
            {children()}
            {streak_cell(streaked.map(|day| day.status))}
        </tr>
        {move || {
            opened
                .get()
                .then(|| {
                    view! {
                        {streaked
                            .map(|day| {
                                view! {
                                    <tr class="ticks">
                                        <td colspan=columns.to_string()>{ticks_of(day)}</td>
                                    </tr>
                                }
                            })}
                        {details.run()}
                    }
                })
        }}
    }
}

macro_rules! history {
    ($name:ident, $row:ty, $load:path, $render:ident, $details:expr, [$(($head:literal, $unit:literal)),*]) => {
        #[component]
        fn $name(today: String, streaked: Signal<Vec<StreakDay>>) -> impl IntoView {
            let columns = [$($head),*].len() + 1;
            let months = RwSignal::new(Vec::<Month<$row>>::new());
            let open = RwSignal::new(Vec::<String>::new());
            let said = RwSignal::new(None::<String>);
            let recorded = Recorded::here();
            let picked = today.clone();
            let chosen = today.clone();
            let load = Callback::new(move |month: String| {
                let picked = picked.clone();
                spawn_local(async move {
                    match $load(&month, &picked).await {
                        Ok(loaded) => months.update(|kept| {
                            kept.retain(|month| month.month != loaded.month);
                            kept.push(loaded);
                            kept.sort_by(|newer, older| older.month.cmp(&newer.month));
                        }),
                        Err(failure) => said.set(Some(lines::failure(&failure))),
                    }
                });
            });
            let first: String = today.chars().take(7).collect();
            Effect::new(move |_| {
                recorded.heard();
                let shown: Vec<String> =
                    months.with_untracked(|kept| kept.iter().map(|month| month.month.clone()).collect());
                let again = if shown.is_empty() { vec![first.clone()] } else { shown };
                for month in again {
                    load.run(month);
                }
            });
            view! {
                <div class="scroll">
                    <table class="history-table">
                        <thead>
                            <tr>
                                $(<th>{header($head, $unit)}</th>)*
                                <th>"Streak"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || {
                                let days = streaked.get();
                                months_of(months)
                                    .into_iter()
                                    .map(|row| {
                                        let streaked = days
                                            .iter()
                                            .find(|streaked| streaked.day == row.day)
                                            .copied();
                                        let chosen = row.day.to_string() == chosen;
                                        let opened = row.clone();
                                        let details = ViewFn::from(move || $details(&opened));
                                        view! {
                                            <HistoryRow
                                                day=row.day.to_string()
                                                streaked=streaked
                                                chosen=chosen
                                                columns=columns
                                                open=open
                                                details=details
                                            >
                                                {$render(row)}
                                            </HistoryRow>
                                        }
                                    })
                                    .collect_view()
                            }}
                        </tbody>
                    </table>
                </div>
                {move || {
                    months_of(months)
                        .is_empty()
                        .then(|| view! { <p class="empty">{lines::NOTHING_THIS_MONTH}</p> })
                }}
                {refused_line(said)}
                {move || {
                    let earlier = months.get().last().and_then(|month| month.earlier.clone());
                    view! { <LoadMore earlier=earlier load=load /> }
                }}
            }
        }
    };
}

/// A gathered day's details: its waist, for now.
fn gathered_details(row: &GatherRow) -> AnyView {
    view! {
        <tr class="entry">
            <td>{header("Waist", "cm")}</td>
            <td>{bare_or_dash(row.waist_cm, Figure::Length)}</td>
            <td colspan="4"></td>
        </tr>
    }
    .into_any()
}

fn fuelled_details(row: &FuelRow) -> AnyView {
    view! { <FuelEntries date=row.day.to_string() /> }.into_any()
}

fn hydrated_details(row: &HydrateRow) -> AnyView {
    view! { <HydrateEntries date=row.day.to_string() /> }.into_any()
}

fn burnt_details(row: &BurnRow) -> AnyView {
    view! { <BurnEntries date=row.day.to_string() /> }.into_any()
}

history!(
    GatherHistory,
    GatherRow,
    tauri::gather_month,
    gather_cells,
    gathered_details,
    [
        ("Day", ""),
        ("BM", "kg"),
        ("FFM", "kg"),
        ("FM", "kg"),
        ("Fat %", "")
    ]
);
history!(
    FuelHistory,
    FuelRow,
    tauri::fuel_month,
    fuel_cells,
    fuelled_details,
    [
        ("Day", ""),
        ("Energy", "kcal"),
        ("Prot", "g"),
        ("Fat", "g"),
        ("Fib", "g"),
        ("Carbs", "g"),
        ("Qty", "g")
    ]
);
history!(
    HydrateHistory,
    HydrateRow,
    tauri::hydrate_month,
    hydrate_cells,
    hydrated_details,
    [("Day", ""), ("Drunk", "mL"), ("Water %", "")]
);
history!(
    BurnHistory,
    BurnRow,
    tauri::burn_month,
    burn_cells,
    burnt_details,
    [
        ("Day", ""),
        ("Burnt", "kcal"),
        ("Duration", "min"),
        ("Reps", "")
    ]
);

/// A tab's figures over the window picked.
fn figures_over(chip: Chip, today: String, window: RwSignal<Window>) -> AnyView {
    match chip {
        Chip::Gather => view! { <PaceOver today=today window=window /> }.into_any(),
        Chip::Fuel => view! { <FuelOver today=today window=window /> }.into_any(),
        Chip::Hydrate => view! { <HydrateOver today=today window=window /> }.into_any(),
        Chip::Burn => view! { <BurnOver today=today window=window /> }.into_any(),
    }
}

/// A tab's stats: its chart over the window picked, then its figures over the same days, up to today.
#[component]
pub fn Stats(chip: Chip) -> impl IntoView {
    let window = RwSignal::new(Window::Days30);
    let today = crate::screens::today();
    view! {
        <section class="card stats">
            <h2>"Stats"</h2>
            <div class="pills">
                {pill_buttons("window", WINDOWS, window)}
            </div>
            <ChipPlot chip=chip today=today.clone() window=window />
            {figures_over(chip, today, window)}
        </section>
    }
}

/// A tab's history, month by month back from today, each day's streak dot by it, a tap opening its ticks then
/// its entries.
#[component]
pub fn History(chip: Chip, today: String) -> impl IntoView {
    let recorded = Recorded::here();
    let on = today.clone();
    let streak = LocalResource::new(move || {
        recorded.heard();
        let today = on.clone();
        async move { tauri::streak(&today).await }
    });
    let streaked = Signal::derive(move || {
        streak
            .get()
            .and_then(Result::ok)
            .map(|streak| streak.history)
            .unwrap_or_default()
    });
    view! {
        <section class="card history">
            <h2>"History"</h2>
            {match chip {
                Chip::Gather => view! { <GatherHistory today=today streaked=streaked /> }.into_any(),
                Chip::Fuel => view! { <FuelHistory today=today streaked=streaked /> }.into_any(),
                Chip::Hydrate => view! { <HydrateHistory today=today streaked=streaked /> }.into_any(),
                Chip::Burn => view! { <BurnHistory today=today streaked=streaked /> }.into_any(),
            }}
            <ul class="legend">
                {[Status::Success, Status::Joker, Status::Break, Status::Pending]
                    .into_iter()
                    .map(|status| {
                        view! {
                            <li>
                                <span class=format!("dot {status:?}") aria-hidden="true">
                                    {status_mark(status)}
                                </span>
                                {lines::status_word(status)}
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </section>
    }
}
