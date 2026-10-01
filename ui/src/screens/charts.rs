use chrono::NaiveDate;
use grind_core::failure::Failure;
use grind_core::figures::{Figure, shown};
use grind_core::for_charting::{
    BandChart, BurnChart, BurnRow, FuelRow, GatherChart, GatherRow, HydrateRow, Month,
};
use grind_core::for_reviewing_progress::Window;
use grind_core::sharpening::sharpening_of;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::chart::{height_of, line_of, scale_of, spread};
use crate::screens::lines;
use crate::tauri;

const WIDTH: f64 = 360.0;
const HEIGHT: f64 = 180.0;
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

const WINDOWS: [(Window, &str); 3] = [
    (Window::Days30, "30 days"),
    (Window::Days90, "90 days"),
    (Window::Everything, "Everything"),
];

type Points = Vec<(NaiveDate, f64)>;
type Series<'a> = (&'a Points, fn(f64) -> Figure, &'static str);

#[derive(Clone)]
struct Line {
    points: String,
    class: &'static str,
}

#[derive(Clone)]
struct Label {
    at: f64,
    text: String,
    class: &'static str,
}

#[derive(Clone)]
struct Plotted {
    lines: Vec<Line>,
    left: Vec<Label>,
    right: Vec<Label>,
    legend: Vec<(&'static str, &'static str)>,
}

fn span_of<'a>(
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

fn highest(points: &[(NaiveDate, f64)]) -> Option<f64> {
    points.iter().map(|(_, value)| *value).reduce(f64::max)
}

fn lowest(points: &[(NaiveDate, f64)]) -> Option<f64> {
    points.iter().map(|(_, value)| *value).reduce(f64::min)
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

fn spread_out(labels: Vec<Label>) -> Vec<Label> {
    let heights: Vec<f64> = labels.iter().map(|label| label.at).collect();
    labels
        .into_iter()
        .zip(spread(&heights, LABEL_GAP, HEIGHT))
        .map(|(label, at)| Label { at, ..label })
        .collect()
}

fn gather_plot(chart: &GatherChart) -> Option<Plotted> {
    let span = span_of(chart.masses.iter().chain(&chart.lean_masses))?;
    let values = chart
        .masses
        .iter()
        .chain(&chart.lean_masses)
        .map(|(_, kg)| *kg)
        .chain(chart.goal_kg)
        .chain(chart.goal_lean_kg);
    let scale = scale_of(values)?;
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
            labelled(highest(&chart.masses), scale, Figure::Mass, "mass"),
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
    let scale = scale_of(values)?;
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
        left: [
            labelled(highest(&chart.amounts), scale, figure, class),
            labelled(lowest(&chart.amounts), scale, figure, class),
        ]
        .into_iter()
        .flatten()
        .collect(),
        right: [
            labelled(current(&chart.amounts), scale, figure, class),
            labelled(current(&chart.floors), scale, figure, "floor"),
            labelled(current(&chart.caps), scale, figure, "cap"),
        ]
        .into_iter()
        .flatten()
        .collect(),
        legend: vec![(class, name), ("floor", "Floor"), ("cap", "Cap")],
    })
}

fn burn_plot(chart: &BurnChart) -> Option<Plotted> {
    let span = span_of(chart.burnt.iter().chain(&chart.minutes))?;
    let series: [Series; 2] = [
        (&chart.burnt, Figure::Energy, "burnt"),
        (&chart.minutes, Figure::Duration, "minutes"),
    ];
    let mut plotted = Plotted {
        lines: vec![],
        left: vec![],
        right: vec![],
        legend: vec![("burnt", "Burnt (kcal)"), ("minutes", "Minutes")],
    };
    for (points, figure, class) in series {
        let Some(scale) = scale_of(points.iter().map(|(_, value)| *value).chain([0.0])) else {
            continue;
        };
        plotted.lines.push(Line {
            points: line_of(points, span, scale, WIDTH, HEIGHT),
            class,
        });
        plotted
            .left
            .extend(labelled(highest(points), scale, figure, class));
        plotted
            .right
            .extend(labelled(current(points), scale, figure, class));
    }
    Some(plotted)
}

#[component]
fn Plot(plotted: Option<Plotted>) -> impl IntoView {
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
            view! {
                <div class="plot">
                    <svg viewBox="0 0 360 180" preserveAspectRatio="none">
                        {plotted
                            .lines
                            .into_iter()
                            .map(|line| view! { <polyline class=line.class points=line.points /> })
                            .collect_view()}
                    </svg>
                    {spread_out(plotted.left).into_iter().map(place("left")).collect_view()}
                    {spread_out(plotted.right).into_iter().map(place("right")).collect_view()}
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

#[component]
fn GatherFigures(today: String, window: RwSignal<Window>) -> impl IntoView {
    let card = LocalResource::new(move || {
        let (today, window) = (today.clone(), window.get());
        async move { tauri::gather_card(&today, window).await }
    });
    view! {
        <Transition fallback=|| ()>
            {move || {
                card.get()
                    .and_then(Result::ok)
                    .map(|card| {
                        view! {
                            <ul class="figures">
                                {lines::gather_figures(&card)
                                    .into_iter()
                                    .map(|line| view! { <li>{line}</li> })
                                    .collect_view()}
                            </ul>
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

fn gather_row(row: GatherRow) -> impl IntoView {
    let optional = |value: Option<f64>, figure: fn(f64) -> Figure| {
        value.map(|value| shown(figure(value))).unwrap_or_default()
    };
    view! {
        <tr>
            <td>{row.day.to_string()}</td>
            <td>{shown(Figure::Mass(row.body_mass_kg))}</td>
            <td>{optional(row.lean_mass_kg, Figure::Mass)}</td>
            <td>{optional(row.fat_mass_kg, Figure::Mass)}</td>
            <td>{optional(row.fat_ratio, Figure::Ratio)}</td>
        </tr>
    }
}

fn fuel_row(row: FuelRow) -> impl IntoView {
    view! {
        <tr>
            <td>{row.day.to_string()}</td>
            <td>{shown(Figure::Energy(row.eaten.energy_kcal))}</td>
            <td>{shown(Figure::Grams(row.eaten.protein_g))}</td>
            <td>{shown(Figure::Grams(row.eaten.fat_g))}</td>
            <td>{shown(Figure::Grams(row.eaten.carbs_g))}</td>
            <td>{shown(Figure::Grams(row.eaten.fibre_g))}</td>
        </tr>
    }
}

fn hydrate_row(row: HydrateRow) -> impl IntoView {
    view! {
        <tr>
            <td>{row.day.to_string()}</td>
            <td>{shown(Figure::Volume(row.drunk_ml))}</td>
            <td>{shown(Figure::Ratio(row.water_ratio))}</td>
        </tr>
    }
}

fn burn_row(row: BurnRow) -> impl IntoView {
    view! {
        <tr>
            <td>{row.day.to_string()}</td>
            <td>{row.burnt_kcal.map(|kcal| shown(Figure::Energy(kcal))).unwrap_or_default()}</td>
            <td>{shown(Figure::Duration(row.minutes))}</td>
            <td>{shown(Figure::Count(row.reps))}</td>
            <td>{row.top_exercise.unwrap_or_default()}</td>
        </tr>
    }
}

macro_rules! history {
    ($name:ident, $row:ty, $load:path, $render:ident, [$($head:literal),*]) => {
        #[component]
        fn $name(today: String) -> impl IntoView {
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
                <table class="history-table">
                    <thead>
                        <tr>
                            $(<th>{$head}</th>)*
                        </tr>
                    </thead>
                    <tbody>
                        {move || months_of(months).into_iter().map($render).collect_view()}
                    </tbody>
                </table>
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
    gather_row,
    ["Day", "Mass", "Lean", "Fat", "Fat %"]
);
history!(
    FuelHistory,
    FuelRow,
    tauri::fuel_month,
    fuel_row,
    ["Day", "kcal", "Protein", "Fat", "Carbs", "Fibre"]
);
history!(
    HydrateHistory,
    HydrateRow,
    tauri::hydrate_month,
    hydrate_row,
    ["Day", "Drunk", "Water"]
);
history!(
    BurnHistory,
    BurnRow,
    tauri::burn_month,
    burn_row,
    ["Day", "kcal", "Minutes", "Reps", "Top exercise"]
);

#[component]
fn VeilNotes(chip: RwSignal<Chip>, today: String, window: RwSignal<Window>) -> impl IntoView {
    let notes = LocalResource::new(move || {
        let (today, window) = (today.clone(), window.get());
        async move {
            let insights = tauri::insights(&today, window).await?;
            let card = tauri::gather_card(&today, window).await?;
            Ok::<_, Failure>(sharpening_of(&insights, &card))
        }
    });
    view! {
        <section class="card veil">
            <h2>"Veil's hunt"</h2>
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
                                view! { <p class="veil-note">{line}</p> }.into_any()
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
    let window = RwSignal::new(Window::Days30);
    let today = StoredValue::new(today);
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
            {move || {
                let chosen = chip.get();
                let plot = view! { <ChipPlot chip=chosen today=today.get_value() window=window /> };
                match chosen {
                    Chip::Gather => {
                        view! {
                            {plot}
                            <GatherFigures today=today.get_value() window=window />
                        }
                            .into_any()
                    }
                    _ => plot.into_any(),
                }
            }}
        </section>
        <VeilNotes chip=chip today=today.get_value() window=window />
        <section class="card history">
            <h2>"History"</h2>
            {move || match chip.get() {
                Chip::Gather => view! { <GatherHistory today=today.get_value() /> }.into_any(),
                Chip::Fuel => view! { <FuelHistory today=today.get_value() /> }.into_any(),
                Chip::Hydrate => view! { <HydrateHistory today=today.get_value() /> }.into_any(),
                Chip::Burn => view! { <BurnHistory today=today.get_value() /> }.into_any(),
            }}
        </section>
    }
}
