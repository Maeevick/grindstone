use grind_core::figures::{Figure, bare, shown};
use grind_core::fuel::Band;
use grind_core::heading::Aim;
use leptos::prelude::*;

use crate::chart::{aim_gauge_of, burn_gauge_of, energy_gauge_of, gauge_of};
use crate::screens::lines;

/// A tick's label under it, in gold: anchored at the share of itself its tick takes of the line, so it stays
/// under its tick without leaving the gauge.
#[component]
fn TickLabel(at: f64, label: String) -> impl IntoView {
    let share = 100.0 * at / 360.0;
    view! {
        <div class="gauge-under-ticks">
            <span
                class="acc tick-label"
                style=format!("left: {share}%; transform: translateX(-{share}%)")
            >
                {label}
            </span>
        </div>
    }
}

#[component]
pub fn Gauge(
    amount: f64,
    floor: f64,
    optimal: f64,
    cap: f64,
    shown_as: fn(f64) -> Figure,
) -> impl IntoView {
    let placed = gauge_of(amount, (floor, optimal, cap), None, 360.0);
    view! {
        <div class="gauge">
            <svg viewBox="0 0 360 24" preserveAspectRatio="none">
                <rect class="track" x="0" y="9" width="360" height="6" />
                <rect class="band" x=placed.band_from y="9" width=placed.band_to - placed.band_from height="6" />
                <line class="optimal" x1=placed.optimal_at x2=placed.optimal_at y1="4" y2="20" />
                <circle class="amount" cx=placed.amount_at cy="12" r="5" />
            </svg>
            <TickLabel at=placed.optimal_at label=format!("Optimal {}", shown(shown_as(optimal))) />
            <div class="gauge-labels">
                <span class="low">{format!("Min {}", shown(shown_as(floor)))}</span>
                <span class="high">{format!("Max {}", shown(shown_as(cap)))}</span>
            </div>
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
            <TickLabel at=placed.aim_at label=format!("{aimed} {}", shown(Figure::Mass(value))) />
            <div class="gauge-labels">
                <span>{shown(Figure::Mass(placed.from))}</span>
                <span>{shown(Figure::Mass(placed.up_to))}</span>
            </div>
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
            <TickLabel at=placed.target_at label=format!("Target {}", shown(Figure::Energy(target))) />
            <div class="gauge-labels">
                <span>{shown(Figure::Energy(0.0))}</span>
                <span>{shown(Figure::Energy(placed.up_to_kcal))}</span>
            </div>
        </div>
    }
}

#[component]
pub fn FuelGauge(eaten: f64, band: Band, ghost: Option<f64>) -> impl IntoView {
    let bounds = (
        band.minimum_kcal,
        band.floor_kcal,
        band.maintenance_kcal,
        band.cap_kcal,
    );
    let placed = energy_gauge_of(eaten, bounds, None, 360.0);
    let ghost_at = ghost.map(|kcal| energy_gauge_of(kcal, bounds, None, 360.0).eaten_at);
    view! {
        <div class="gauge fuel-gauge">
            <svg viewBox="0 0 360 24" preserveAspectRatio="none">
                <rect class="track" x="0" y="9" width="360" height="6" />
                <rect class="band" x=placed.floor_at y="9" width=placed.cap_at - placed.floor_at height="6" />
                <line class="minimum" x1=placed.minimum_at x2=placed.minimum_at y1="3" y2="21" />
                <line
                    class="maintenance"
                    x1=placed.maintenance_at
                    x2=placed.maintenance_at
                    y1="3"
                    y2="21"
                />
                {ghost_at.map(|at| view! { <circle class="ghost" cx=at cy="12" r="5" /> })}
                <circle class="eaten" cx=placed.eaten_at cy="12" r="5" />
            </svg>
            <TickLabel
                at=placed.maintenance_at
                label=format!("Maintenance {}", bare(Figure::Energy(band.maintenance_kcal)))
            />
            <div class="gauge-labels">
                <span class="low">
                    {format!(
                        "EA{} {}",
                        bare(Figure::Count(band.minimum_kcal_per_kg_of_lean_mass)),
                        bare(Figure::Energy(band.minimum_kcal)),
                    )}
                </span>
            </div>
        </div>
    }
}
