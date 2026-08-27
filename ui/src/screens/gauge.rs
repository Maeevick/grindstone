use grind_core::figures::{Figure, shown};
use leptos::prelude::*;

use crate::chart::gauge_of;

#[component]
pub fn Gauge(
    amount: f64,
    floor: f64,
    optimal: f64,
    cap: f64,
    shown_as: fn(f64) -> Figure,
) -> impl IntoView {
    let placed = gauge_of(amount, (floor, optimal, cap), 360.0);
    view! {
        <div class="gauge">
            <svg viewBox="0 0 360 24" preserveAspectRatio="none">
                <rect class="track" x="0" y="9" width="360" height="6" />
                <rect class="band" x=placed.band_from y="9" width=placed.band_to - placed.band_from height="6" />
                <line class="optimal" x1=placed.optimal_at x2=placed.optimal_at y1="4" y2="20" />
                <circle class="amount" cx=placed.amount_at cy="12" r="5" />
            </svg>
            <div class="gauge-labels">
                <span class="low">{format!("min {}", shown(shown_as(floor)))}</span>
                <span>{format!("optimal {}", shown(shown_as(optimal)))}</span>
                <span class="high">{format!("max {}", shown(shown_as(cap)))}</span>
            </div>
        </div>
    }
}
