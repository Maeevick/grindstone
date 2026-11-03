use grind_core::figures::{Figure, bare};
use grind_core::fuel::Band;
use grind_core::heading::Aim;
use leptos::prelude::*;

use crate::chart::{aim_gauge_of, burn_gauge_of, energy_gauge_of, gauge_of};
use crate::screens::lines;

/// Where a label stands against its place on the line: under a tick, anchored at the share of itself its tick
/// takes of the line; or ending at the left edge of a zone, sliding over the edge rather than out of the gauge
/// when it has no room.
#[derive(Clone, Copy)]
enum Anchor {
    Tick,
    EndsAt,
}

/// A label on a row under the line: where it stands, what it says, its colour and how it is anchored.
type Placed = (f64, String, &'static str, Anchor);

/// A row of labels under the line, each where its tick or its zone's edge falls, in its colour.
#[component]
fn TickRow(labels: Vec<Placed>) -> impl IntoView {
    view! {
        <div class="gauge-under-ticks">
            {labels
                .into_iter()
                .map(|(at, label, class, anchor)| {
                    let share = 100.0 * at / 360.0;
                    let (anchored, style) = match anchor {
                        Anchor::Tick => ("", format!("left: {share}%; transform: translateX(-{share}%)")),
                        Anchor::EndsAt => (" ends-at", format!("left: 0%; width: {share}%")),
                    };
                    view! {
                        <span class=format!("{class} tick-label{anchored}") style=style>
                            {label}
                        </span>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// A tick's label under it, in gold.
#[component]
fn TickLabel(at: f64, label: String) -> impl IntoView {
    view! { <TickRow labels=vec![(at, label, "acc", Anchor::Tick)] /> }
}

#[component]
pub fn Gauge(
    amount: f64,
    floor: f64,
    optimal: f64,
    cap: f64,
    shown_as: fn(f64) -> Figure,
    #[prop(default = None)] ghost: Option<f64>,
) -> impl IntoView {
    let placed = gauge_of(amount, (floor, optimal, cap), ghost, 360.0);
    view! {
        <div class="gauge">
            <svg viewBox="0 0 360 24" preserveAspectRatio="none">
                <rect class="track" x="0" y="9" width="360" height="6" />
                <rect class="band" x=placed.band_from y="9" width=placed.band_to - placed.band_from height="6" />
                <line class="optimal" x1=placed.optimal_at x2=placed.optimal_at y1="4" y2="20" />
                {placed.ghost_at.map(|at| view! { <circle class="ghost" cx=at cy="12" r="5" /> })}
                <circle class="amount" cx=placed.amount_at cy="12" r="5" />
            </svg>
            <TickLabel at=placed.optimal_at label=format!("Opt. {}", bare(shown_as(optimal))) />
        </div>
    }
}

#[component]
pub fn AimGauge(label: &'static str, amount: f64, aim: Aim, recommended: bool) -> impl IntoView {
    let placed = aim_gauge_of(amount, aim, 360.0);
    let (Aim::Reach(value) | Aim::Floor(value)) = aim;
    let aimed = if recommended { "Healthy" } else { "Goal" };
    view! {
        <div class="gauge aim-gauge">
            <p class="gauge-title">{lines::aim_line(label, amount, aim, recommended)}</p>
            <svg viewBox="0 0 360 24" preserveAspectRatio="none">
                <rect class="track" x="0" y="9" width="360" height="6" />
                <rect class="band" x=placed.band_from y="9" width=placed.band_to - placed.band_from height="6" />
                <line class="optimal" x1=placed.aim_at x2=placed.aim_at y1="4" y2="20" />
                <circle class="amount" cx=placed.amount_at cy="12" r="5" />
            </svg>
            <TickLabel at=placed.aim_at label=format!("{aimed} {}", bare(Figure::Mass(value))) />
        </div>
    }
}

#[component]
pub fn BurnGauge(burnt: f64, target: f64, ghost: Option<f64>) -> impl IntoView {
    let placed = burn_gauge_of(burnt, target, ghost, 360.0);
    view! {
        <div class="gauge burn-gauge">
            <svg viewBox="0 0 360 24" preserveAspectRatio="none">
                <rect class="track" x="0" y="9" width="360" height="6" />
                <rect class="band" x=placed.band_from y="9" width=360.0 - placed.band_from height="6" />
                <line class="optimal" x1=placed.target_at x2=placed.target_at y1="4" y2="20" />
                {placed.ghost_at.map(|at| view! { <circle class="ghost" cx=at cy="12" r="5" /> })}
                <circle class="amount" cx=placed.burnt_at cy="12" r="5" />
            </svg>
            <TickLabel at=placed.target_at label=format!("Target {}", bare(Figure::Energy(target))) />
        </div>
    }
}

#[component]
pub fn FuelGauge(eaten: f64, band: Band, ghost: Option<(f64, Band)>) -> impl IntoView {
    let bounds = (
        band.minimum_kcal,
        band.floor_kcal,
        band.goal_kcal,
        band.maintenance_kcal,
        band.cap_kcal,
    );
    let placed = energy_gauge_of(
        eaten,
        bounds,
        ghost.map(|(kcal, after)| {
            (
                kcal,
                (after.floor_kcal, after.maintenance_kcal, after.cap_kcal),
            )
        }),
        360.0,
    );
    view! {
        <div class="gauge fuel-gauge">
            <svg viewBox="0 0 360 24" preserveAspectRatio="none">
                <rect class="track" x="0" y="9" width="360" height="6" />
                <rect class="band" x=placed.floor_at y="9" width=placed.cap_at - placed.floor_at height="6" />
                {placed
                    .ghost
                    .map(|ghost| {
                        view! {
                            <rect
                                class="ghost-band"
                                x=ghost.floor_at
                                y="6"
                                width=ghost.cap_at - ghost.floor_at
                                height="12"
                            />
                            <line
                                class="ghost-maintenance"
                                x1=ghost.maintenance_at
                                x2=ghost.maintenance_at
                                y1="2"
                                y2="22"
                            />
                        }
                    })}
                <line class="minimum" x1=placed.minimum_at x2=placed.minimum_at y1="3" y2="21" />
                <line class="goal" x1=placed.goal_at x2=placed.goal_at y1="3" y2="21" />
                <line
                    class="maintenance"
                    x1=placed.maintenance_at
                    x2=placed.maintenance_at
                    y1="3"
                    y2="21"
                />
                {placed.ghost.map(|ghost| view! { <circle class="ghost" cx=ghost.eaten_at cy="12" r="5" /> })}
                <circle class="eaten" cx=placed.eaten_at cy="12" r="5" />
            </svg>
            <TickRow labels=vec![
                (
                    placed.minimum_at,
                    format!(
                        "EA{} {}",
                        bare(Figure::Count(band.minimum_kcal_per_kg_of_lean_mass)),
                        bare(Figure::Energy(band.minimum_kcal)),
                    ),
                    "low",
                    Anchor::EndsAt,
                ),
                (placed.goal_at, format!("Goal {}", bare(Figure::Energy(band.goal_kcal))), "acc", Anchor::Tick),
            ] />
            <TickRow labels=(band.floor_kcal > band.minimum_kcal)
                .then(|| {
                    (
                        placed.floor_at,
                        format!("Alpert {}", bare(Figure::Energy(band.floor_kcal))),
                        "good",
                        Anchor::EndsAt,
                    )
                })
                .into_iter()
                .chain([
                    (
                        placed.maintenance_at,
                        format!("Maintenance {}", bare(Figure::Energy(band.maintenance_kcal))),
                        "acc",
                        Anchor::Tick,
                    ),
                ])
                .collect() />
        </div>
    }
}
