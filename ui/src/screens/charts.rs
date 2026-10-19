use chrono::NaiveDate;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, bare, shown};
use grind_core::for_charting::{
    BandChart, BurnChart, BurnRow, FuelRow, GatherChart, GatherRow, HydrateRow, Month,
};
use grind_core::for_reviewing_progress::Window;
use grind_core::sharpening::sharpening_of;
use grind_core::streak::{Status, StreakDay};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::chart::{Axis, axis_of, days_along, height_of, line_of, spread};
use crate::screens::home::tick_class;
use crate::screens::lines;
use crate::tauri;

pub const WIDTH: f64 = 360.0;
pub const HEIGHT: f64 = 180.0;
const LABEL_GAP: f64 = 16.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Chip {
    Gather,
    Fuel,
    Hydrate,
    Burn,
}

const CHIPS: [(Chip, &str); 4] = [
    (Chip::Gather, "Gather"),
    (Chip::Fuel, "Fuel"),
    (Chip::Hydrate, "Hydrate"),
    (Chip::Burn, "Burn"),
];

const WINDOWS: [(Window, &str); 4] = [
    (Window::Days7, "7 days"),
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

#[derive(Clone)]
pub struct Label {
    at: f64,
    text: String,
    class: &'static str,
}

#[derive(Clone, Default)]
pub struct Plotted {
    pub lines: Vec<Line>,
    pub dots: Vec<(f64, f64)>,
    pub left: Vec<Label>,
    pub right: Vec<Label>,
    pub ticks: Vec<Label>,
    pub right_ticks: Vec<Label>,
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

fn level(value: f64, scale: (f64, f64)) -> String {
    let y = height_of(value, scale, HEIGHT);
    format!("0,{y:.1} {WIDTH},{y:.1}")
}

fn current(points: &[(NaiveDate, f64)]) -> Option<f64> {
    points.last().map(|(_, value)| *value)
}

fn labelled(
    value: Option<f64>,
    scale: (f64, f64),
    figure: fn(f64) -> Figure,
    class: &'static str,
) -> Option<Label> {
    value.map(|value| Label {
        at: height_of(value, scale, HEIGHT),
        text: shown(figure(value)),
        class,
    })
}

pub fn ticks_on(axis: Axis, figure: impl Fn(f64) -> Figure, class: &'static str) -> Vec<Label> {
    axis.ticks()
        .into_iter()
        .map(|value| Label {
            at: height_of(value, axis.scale(), HEIGHT),
            text: bare(figure(value)),
            class,
        })
        .collect()
}

fn spread_out(labels: Vec<Label>) -> Vec<Label> {
    let heights: Vec<f64> = labels.iter().map(|label| label.at).collect();
    labels
        .into_iter()
        .zip(spread(&heights, LABEL_GAP, HEIGHT))
        .map(|(label, at)| Label { at, ..label })
        .collect()
}

/// The ticks a series' own label leaves room for: the label says more than the tick it covers.
fn clear_of(ticks: Vec<Label>, labels: &[Label]) -> Vec<Label> {
    ticks
        .into_iter()
        .filter(|tick| {
            labels
                .iter()
                .all(|label| (label.at - tick.at).abs() >= LABEL_GAP)
        })
        .collect()
}

/// Two scales, two axes: the body mass on the left, the lean mass on the right, each with its goal, so that
/// the two lines fill the height and a small change shows over a short window.
fn gather_plot(chart: &GatherChart) -> Option<Plotted> {
    let span = span_of(chart.masses.iter().chain(&chart.lean_masses))?;
    let mass = axis_of(chart.masses.iter().map(|(_, kg)| *kg).chain(chart.goal_kg))?;
    let lean = axis_of(
        chart
            .lean_masses
            .iter()
            .map(|(_, kg)| *kg)
            .chain(chart.goal_lean_kg),
    );
    let mut lines = vec![Line {
        points: line_of(&chart.masses, span, mass.scale(), WIDTH, HEIGHT),
        class: "mass",
    }];
    lines.extend(chart.goal_kg.map(|kg| Line {
        points: level(kg, mass.scale()),
        class: "goal dashed",
    }));
    let mut right = vec![];
    let mut right_ticks = vec![];
    if let Some(lean) = lean {
        lines.push(Line {
            points: line_of(&chart.lean_masses, span, lean.scale(), WIDTH, HEIGHT),
            class: "lean",
        });
        lines.extend(chart.goal_lean_kg.map(|kg| Line {
            points: level(kg, lean.scale()),
            class: "lean dashed",
        }));
        right = [
            labelled(chart.goal_lean_kg, lean.scale(), Figure::Mass, "lean"),
            labelled(
                current(&chart.lean_masses),
                lean.scale(),
                Figure::Mass,
                "lean",
            ),
        ]
        .into_iter()
        .flatten()
        .collect();
        right_ticks = ticks_on(lean, Figure::Mass, "lean");
    }
    Some(Plotted {
        lines,
        dots: vec![],
        left: [
            labelled(chart.goal_kg, mass.scale(), Figure::Mass, "goal"),
            labelled(current(&chart.masses), mass.scale(), Figure::Mass, "mass"),
        ]
        .into_iter()
        .flatten()
        .collect(),
        right,
        ticks: ticks_on(mass, Figure::Mass, "mass"),
        days: days_along(span, WIDTH),
        right_ticks,
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
        dots: vec![],
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
        right_ticks: vec![],
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
        dots: vec![],
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
        right_ticks: vec![],
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
            let two_axes = !plotted.right_ticks.is_empty();
            view! {
                <div class="plot">
                    <svg viewBox="0 0 360 180" preserveAspectRatio="none">
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
                        {two_axes.then(|| view! { <line class="axis" x1="360" x2="360" y1="0" y2="180" /> })}
                        {plotted
                            .lines
                            .into_iter()
                            .map(|line| view! { <polyline class=line.class points=line.points /> })
                            .collect_view()}
                        {plotted
                            .dots
                            .into_iter()
                            .map(|(x, y)| view! { <circle cx=x cy=y r="3" /> })
                            .collect_view()}
                    </svg>
                    {clear_of(plotted.ticks, &left).into_iter().map(tick("left")).collect_view()}
                    {clear_of(plotted.right_ticks, &right).into_iter().map(tick("right")).collect_view()}
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
        let label = format!("Load {}", lines::month_name(&month));
        view! {
            <button class="cta ghost more" type="button" on:click=move |_| load.run(month.clone())>
                {label}
            </button>
        }
    })
}

fn streak_cell(status: Option<Status>) -> impl IntoView {
    view! {
        <td>
            {status
                .map(|status| {
                    view! {
                        <span class=format!("dot {status:?}") title=lines::status_word(status)></span>
                    }
                })}
        </td>
    }
}

fn header(name: &'static str, unit: &'static str) -> impl IntoView {
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

fn bare_or_blank(value: Option<f64>, figure: fn(f64) -> Figure) -> String {
    value.map(|value| bare(figure(value))).unwrap_or_default()
}

fn gather_cells(row: GatherRow) -> impl IntoView {
    view! {
            <td>{row.day.to_string()}</td>
            <td>{bare(Figure::Mass(row.body_mass_kg))}</td>
            <td>{bare_or_blank(row.lean_mass_kg, Figure::Mass)}</td>
            <td>{bare_or_blank(row.fat_mass_kg, Figure::Mass)}</td>
            <td>{bare_or_blank(row.fat_ratio, Figure::Ratio)}</td>
    }
}

fn fuel_cells(row: FuelRow) -> impl IntoView {
    view! {
            <td>{row.day.to_string()}</td>
            <td>{bare(Figure::Energy(row.eaten.energy_kcal))}</td>
            <td>{bare(Figure::Grams(row.eaten.protein_g))}</td>
            <td>{bare(Figure::Grams(row.eaten.fat_g))}</td>
            <td>{bare(Figure::Grams(row.eaten.carbs_g))}</td>
            <td>{bare(Figure::Grams(row.eaten.fibre_g))}</td>
    }
}

fn hydrate_cells(row: HydrateRow) -> impl IntoView {
    view! {
            <td>{row.day.to_string()}</td>
            <td>{bare(Figure::Volume(row.drunk_ml))}</td>
            <td>{bare(Figure::Ratio(row.water_ratio))}</td>
    }
}

fn burn_cells(row: BurnRow) -> impl IntoView {
    view! {
            <td>{row.day.to_string()}</td>
            <td>{bare_or_blank(row.burnt_kcal, Figure::Energy)}</td>
            <td>{bare(Figure::Duration(row.minutes))}</td>
            <td>{bare(Figure::Count(row.reps))}</td>
            <td>{row.top_exercise.unwrap_or_default()}</td>
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

#[component]
fn HistoryRow(streaked: Option<StreakDay>, columns: usize, children: Children) -> impl IntoView {
    let open = RwSignal::new(false);
    view! {
        <tr
            class:streaked=streaked.is_some()
            on:click=move |_| {
                if streaked.is_some() {
                    open.update(|open| *open = !*open);
                }
            }
        >
            {children()}
            {streak_cell(streaked.map(|day| day.status))}
        </tr>
        {move || {
            open.get()
                .then_some(streaked)
                .flatten()
                .map(|day| {
                    view! {
                        <tr class="ticks">
                            <td colspan=columns.to_string()>{ticks_of(day)}</td>
                        </tr>
                    }
                })
        }}
    }
}

macro_rules! history {
    ($name:ident, $row:ty, $load:path, $render:ident, [$(($head:literal, $unit:literal)),*]) => {
        #[component]
        fn $name(today: String, streaked: Signal<Vec<StreakDay>>) -> impl IntoView {
            let columns = [$($head),*].len() + 1;
            let months = RwSignal::new(Vec::<Month<$row>>::new());
            let said = RwSignal::new(None::<String>);
            let load = Callback::new(move |month: String| {
                spawn_local(async move {
                    match $load(&month).await {
                        Ok(loaded) => months.update(|kept| kept.push(loaded)),
                        Err(failure) => said.set(Some(lines::failure(&failure))),
                    }
                });
            });
            load.run(today.chars().take(7).collect());
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
                                        view! {
                                            <HistoryRow streaked=streaked columns=columns>
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
                {move || said.get().map(|line| view! { <p class="bad">{line}</p> })}
                {move || {
                    let earlier = months.get().last().and_then(|month| month.earlier.clone());
                    view! { <LoadMore earlier=earlier load=load /> }
                }}
            }
        }
    };
}

history!(
    GatherHistory,
    GatherRow,
    tauri::gather_month,
    gather_cells,
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
    [
        ("Day", ""),
        ("Energy", "kcal"),
        ("Prot", "g"),
        ("Fat", "g"),
        ("Carbs", "g"),
        ("Fib", "g")
    ]
);
history!(
    HydrateHistory,
    HydrateRow,
    tauri::hydrate_month,
    hydrate_cells,
    [("Day", ""), ("Drunk", "mL"), ("Water %", "")]
);
history!(
    BurnHistory,
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

#[component]
fn UnwantedTips(chip: RwSignal<Chip>, today: String, window: RwSignal<Window>) -> impl IntoView {
    let notes = LocalResource::new(move || {
        let (today, window) = (today.clone(), window.get());
        async move {
            let insights = tauri::insights(&today, window).await?;
            let card = tauri::gather_card(&today, window).await?;
            Ok::<_, Failure>(sharpening_of(&insights, &card))
        }
    });
    view! {
        <section class="card tips">
            <h2>"The unwanted tips"</h2>
            <Transition fallback=|| view! { <p class="empty">{lines::LOADING}</p> }>
                {move || {
                    notes
                        .get()
                        .map(|answer| match answer {
                            Ok(found) => {
                                let line = match chip.get() {
                                    Chip::Gather => lines::gather_note(&found.gather),
                                    Chip::Fuel => lines::fuel_note(&found.fuel),
                                    Chip::Hydrate => lines::hydrate_note(&found.hydrate),
                                    Chip::Burn => lines::burn_note(&found.burn),
                                };
                                view! { <p class="tip">{line}</p> }.into_any()
                            }
                            Err(failure) => {
                                view! { <p class="bad">{lines::failure(&failure)}</p> }.into_any()
                            }
                        })
                }}
            </Transition>
        </section>
    }
}

#[component]
pub fn Charts(today: String) -> impl IntoView {
    let chip = RwSignal::new(Chip::Gather);
    let window = RwSignal::new(Window::Days7);
    let today = StoredValue::new(today);
    let streak = LocalResource::new(move || {
        let today = today.get_value();
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
        <section class="card charts">
            <div class="pills chips">
                {CHIPS
                    .into_iter()
                    .map(|(named, label)| {
                        view! {
                            <button
                                type="button"
                                class=format!("pill chip-{named:?}")
                                class:chosen=move || chip.get() == named
                                on:click=move |_| chip.set(named)
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            <div class="pills">
                {WINDOWS
                    .into_iter()
                    .map(|(named, label)| {
                        view! {
                            <button
                                type="button"
                                class=format!("pill window-{named:?}")
                                class:chosen=move || window.get() == named
                                on:click=move |_| window.set(named)
                            >
                                {label}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            {move || view! { <ChipPlot chip=chip.get() today=today.get_value() window=window /> }}
        </section>
        <UnwantedTips chip=chip today=today.get_value() window=window />
        <section class="card history">
            <h2>"History"</h2>
            {move || match chip.get() {
                Chip::Gather => {
                    view! { <GatherHistory today=today.get_value() streaked=streaked /> }.into_any()
                }
                Chip::Fuel => {
                    view! { <FuelHistory today=today.get_value() streaked=streaked /> }.into_any()
                }
                Chip::Hydrate => {
                    view! { <HydrateHistory today=today.get_value() streaked=streaked /> }.into_any()
                }
                Chip::Burn => {
                    view! { <BurnHistory today=today.get_value() streaked=streaked /> }.into_any()
                }
            }}
            <ul class="legend">
                {[
                    (Status::Success, "every tick"),
                    (Status::Joker, "a joker"),
                    (Status::Break, "a break"),
                    (Status::Pending, "under way"),
                ]
                    .into_iter()
                    .map(|(status, meaning)| {
                        view! {
                            <li>
                                <span class=format!("dot {status:?}")></span>
                                {meaning}
                            </li>
                        }
                    })
                    .collect_view()}
            </ul>
        </section>
    }
}
