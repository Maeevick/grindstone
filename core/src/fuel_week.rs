use serde::{Deserialize, Serialize};

use crate::fuel::{Band, Fuel, MacroRow, Macros};
use crate::week::{Averaged, Banded, averaged};

/// What the week ate, a day, over the days a band judged: the energy and each of the four macros against
/// their bands.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelWeek {
    pub logged_days: usize,
    pub energy: Averaged,
    pub protein: Averaged,
    pub fat: Averaged,
    pub fibre: Averaged,
    pub carbs: Averaged,
}

fn banded(row: MacroRow) -> Banded {
    Banded {
        amount: row.eaten_g,
        floor: row.floor_g,
        ceiling: row.ceiling_g,
        goal: row.painted.then_some(row.goal_g),
    }
}

/// The week's fuel, from its logged days; nothing when no band judged any of them.
pub fn fuel_week_of(days: &[Fuel]) -> Option<FuelWeek> {
    let judged: Vec<(&Fuel, Band, Macros)> = days
        .iter()
        .filter_map(|day| Some((day, day.band?, day.macros?)))
        .collect();
    let row = |of: fn(&Fuel, &Band, &Macros) -> Banded| {
        averaged(
            &judged
                .iter()
                .map(|(day, band, macros)| of(day, band, macros))
                .collect::<Vec<_>>(),
        )
    };
    Some(FuelWeek {
        logged_days: judged.len(),
        energy: row(|day, band, _| Banded {
            amount: day.eaten.energy_kcal,
            floor: band.floor_kcal,
            ceiling: Some(band.cap_kcal),
            goal: Some(band.goal_kcal),
        })?,
        protein: row(|_, _, macros| banded(macros.protein))?,
        fat: row(|_, _, macros| banded(macros.fat))?,
        fibre: row(|_, _, macros| banded(macros.fibre))?,
        carbs: row(|_, _, macros| banded(macros.carbs))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::food::Nutrients;
    use crate::fuel::{Band, MacroRow, Macros};
    use crate::verdict::verdict_of;
    use chrono::NaiveDate;

    fn row(eaten_g: f64, floor_g: f64, ceiling_g: f64) -> MacroRow {
        MacroRow {
            eaten_g,
            floor_g,
            ceiling_g: Some(ceiling_g),
            goal_g: 0.0,
            painted: false,
            verdict: verdict_of(eaten_g, floor_g, ceiling_g),
        }
    }

    fn a_day(energy_kcal: f64, protein_g: f64, fat_g: f64, fibre_g: f64, carbs_g: f64) -> Fuel {
        Fuel {
            day: NaiveDate::from_ymd_opt(2026, 10, 10).expect("a day"),
            intakes: vec![],
            eaten: Nutrients {
                energy_kcal,
                protein_g,
                carbs_g,
                fat_g,
                fibre_g,
            },
            band: Some(Band {
                minimum_kcal_per_kg_of_lean_mass: 25.0,
                minimum_kcal: 1500.0,
                floor_kcal: 1800.0,
                cap_kcal: 2400.0,
                goal_kcal: 1800.0,
                maintenance_kcal: 2400.0,
            }),
            energy: Some(verdict_of(energy_kcal, 1800.0, 2400.0)),
            macros: Some(Macros {
                protein: row(protein_g, 115.0, 190.0),
                fat: row(fat_g, 48.0, 57.0),
                fibre: row(fibre_g, 25.0, 60.0),
                carbs: row(carbs_g, 200.0, 310.0),
            }),
            budget: None,
        }
    }

    fn unjudged(day: Fuel) -> Fuel {
        Fuel {
            band: None,
            energy: None,
            macros: None,
            ..day
        }
    }

    fn averaged(per_day: f64, floor: f64, ceiling: f64, in_band: usize) -> Averaged {
        aimed(per_day, floor, ceiling, None, in_band)
    }

    fn aimed(
        per_day: f64,
        floor: f64,
        ceiling: f64,
        goal: Option<f64>,
        in_band: usize,
    ) -> Averaged {
        Averaged {
            per_day,
            floor,
            ceiling: Some(ceiling),
            goal,
            in_band,
        }
    }

    #[test]
    fn a_fuel_week_averages_the_energy_and_each_macro_against_their_bands_over_the_days_a_band_judged()
     {
        let days = [
            a_day(2000.0, 100.0, 50.0, 20.0, 250.0),
            a_day(2200.0, 120.0, 60.0, 30.0, 230.0),
            unjudged(a_day(900.0, 10.0, 10.0, 0.0, 100.0)),
        ];

        assert_eq!(
            fuel_week_of(&days),
            Some(FuelWeek {
                logged_days: 2,
                energy: aimed(2100.0, 1800.0, 2400.0, Some(1800.0), 2),
                protein: averaged(110.0, 115.0, 190.0, 1),
                fat: averaged(55.0, 48.0, 57.0, 1),
                fibre: averaged(25.0, 25.0, 60.0, 1),
                carbs: averaged(240.0, 200.0, 310.0, 2),
            })
        );
    }

    #[test]
    fn a_week_without_a_day_a_band_judged_has_no_average() {
        assert_eq!(
            [
                fuel_week_of(&[unjudged(a_day(2000.0, 100.0, 50.0, 20.0, 250.0))]),
                fuel_week_of(&[])
            ],
            [None, None]
        );
    }

    #[test]
    fn a_fuel_week_averages_the_goal_tick_as_the_energys_goal() {
        let days = [
            a_day(2000.0, 100.0, 50.0, 20.0, 250.0),
            a_day(2200.0, 120.0, 60.0, 30.0, 230.0),
        ];

        assert_eq!(
            fuel_week_of(&days).and_then(|week| week.energy.goal),
            Some(1800.0)
        );
    }
}
