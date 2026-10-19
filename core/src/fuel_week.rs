use serde::{Deserialize, Serialize};

use crate::fuel::{Band, Fuel, MacroRow, Macros};
use crate::sharpening::Habit;
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

/// How far a habit's average runs from its band, a day: negative short of it, positive over it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gap {
    pub habit: Habit,
    pub per_day: f64,
}

impl FuelWeek {
    /// What runs short of its band, furthest first by the share of its floor it misses.
    pub fn short(&self) -> Vec<Gap> {
        self.off_band(|gap| gap < 0.0, |row| row.floor)
    }

    /// What runs over its band, furthest first by the share of its ceiling it passes.
    pub fn over(&self) -> Vec<Gap> {
        self.off_band(|gap| gap > 0.0, |row| row.ceiling)
    }

    fn off_band(&self, side: fn(f64) -> bool, bound: fn(&Averaged) -> f64) -> Vec<Gap> {
        let mut off: Vec<(Gap, f64)> = [
            (Habit::Energy, self.energy),
            (Habit::Protein, self.protein),
            (Habit::Fat, self.fat),
            (Habit::Fibre, self.fibre),
            (Habit::Carbs, self.carbs),
        ]
        .into_iter()
        .filter_map(|(habit, row)| {
            let per_day = row.gap().filter(|gap| side(*gap))?;
            Some((Gap { habit, per_day }, per_day.abs() / bound(&row)))
        })
        .collect();
        off.sort_by(|(_, one), (_, other)| other.total_cmp(one));
        off.into_iter().map(|(gap, _)| gap).collect()
    }
}

fn banded(row: MacroRow) -> Banded {
    Banded {
        amount: row.eaten_g,
        floor: row.floor_g,
        ceiling: row.ceiling_g,
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
            ceiling: band.cap_kcal,
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
            ceiling_g,
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
        Averaged {
            per_day,
            floor,
            ceiling,
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
                energy: averaged(2100.0, 1800.0, 2400.0, 2),
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
    fn what_runs_short_and_what_runs_over_are_named_furthest_first() {
        let week = FuelWeek {
            logged_days: 6,
            energy: averaged(2100.0, 1800.0, 2400.0, 4),
            protein: averaged(96.0, 115.0, 190.0, 1),
            fat: averaged(71.0, 48.0, 57.0, 2),
            fibre: averaged(19.0, 25.0, 60.0, 0),
            carbs: averaged(330.0, 200.0, 310.0, 2),
        };

        assert_eq!(
            (week.short(), week.over()),
            (
                vec![
                    Gap {
                        habit: Habit::Fibre,
                        per_day: -6.0
                    },
                    Gap {
                        habit: Habit::Protein,
                        per_day: -19.0
                    }
                ],
                vec![
                    Gap {
                        habit: Habit::Fat,
                        per_day: 14.0
                    },
                    Gap {
                        habit: Habit::Carbs,
                        per_day: 20.0
                    }
                ]
            )
        );
    }
}
