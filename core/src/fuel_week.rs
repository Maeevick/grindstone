use serde::{Deserialize, Serialize};

use crate::fuel::{Band, Fuel, MacroRow, Macros};
use crate::verdict::Verdict;
use crate::week::{Averaged, Banded, averaged};

/// What the week ate, a day, over the days a band judged: the energy and each of the four macros against
/// their bands.
/// A habit of Fuel the average tells of: the energy or one of the four macros.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Habit {
    Energy,
    Protein,
    Fat,
    Fibre,
    Carbs,
}

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

/// How far from its goal an average may stand and still be on it, a tenth either side.
const ON_THE_GOAL_SHARE: f64 = 0.1;

impl FuelWeek {
    /// What runs short: the energy under its band, each macro more than a tenth under its goal; furthest first
    /// by the share it misses.
    pub fn short(&self) -> Vec<Gap> {
        self.off(Verdict::Low)
    }

    /// What runs over: the energy past its band, each macro more than a tenth past its goal; furthest first by
    /// the share it passes.
    pub fn over(&self) -> Vec<Gap> {
        self.off(Verdict::High)
    }

    /// Whether any macro has a goal painted to be told against.
    pub fn has_goals(&self) -> bool {
        [self.protein, self.fat, self.fibre, self.carbs]
            .iter()
            .any(|row| row.goal.is_some())
    }

    fn off(&self, side: Verdict) -> Vec<Gap> {
        let energy = (self.energy.verdict() == side)
            .then(|| {
                let bound = match side {
                    Verdict::High => self.energy.ceiling.unwrap_or(f64::INFINITY),
                    _ => self.energy.floor,
                };
                self.energy.gap().map(|per_day| {
                    (
                        Gap {
                            habit: Habit::Energy,
                            per_day,
                        },
                        per_day.abs() / bound,
                    )
                })
            })
            .flatten();
        let macros = [
            (Habit::Protein, self.protein),
            (Habit::Fat, self.fat),
            (Habit::Fibre, self.fibre),
            (Habit::Carbs, self.carbs),
        ]
        .into_iter()
        .filter_map(|(habit, row)| {
            let goal = row.goal?;
            let per_day = row.per_day - goal;
            let off_by = per_day / goal;
            let is_off = match side {
                Verdict::Low => off_by < -ON_THE_GOAL_SHARE,
                _ => off_by > ON_THE_GOAL_SHARE,
            };
            is_off.then_some((Gap { habit, per_day }, off_by.abs()))
        });
        let mut off: Vec<(Gap, f64)> = energy.into_iter().chain(macros).collect();
        off.sort_by(|(_, one), (_, other)| other.total_cmp(one));
        off.into_iter().map(|(gap, _)| gap).collect()
    }
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
            goal: None,
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
    fn the_energy_off_its_band_is_named_with_the_macros_by_the_share_of_its_bound() {
        let aimless = averaged(100.0, 50.0, 200.0, 1);
        let week = |energy: Averaged| FuelWeek {
            logged_days: 2,
            energy,
            protein: aimless,
            fat: aimless,
            fibre: aimless,
            carbs: aimless,
        };

        assert_eq!(
            (
                week(averaged(1500.0, 1800.0, 2400.0, 0)).short(),
                week(averaged(2600.0, 1800.0, 2400.0, 0)).over()
            ),
            (
                vec![Gap {
                    habit: Habit::Energy,
                    per_day: -300.0
                }],
                vec![Gap {
                    habit: Habit::Energy,
                    per_day: 200.0
                }]
            )
        );
    }

    #[test]
    fn the_energy_and_the_macros_are_ranked_together_by_the_share_they_miss() {
        let on_the_goal = aimed(100.0, 50.0, 200.0, Some(100.0), 1);
        let week = FuelWeek {
            logged_days: 2,
            energy: averaged(2600.0, 1800.0, 2400.0, 0),
            protein: aimed(90.0, 50.0, 200.0, Some(100.0), 1),
            fat: aimed(71.0, 48.0, 90.0, Some(64.0), 1),
            fibre: on_the_goal,
            carbs: aimed(110.0, 50.0, 200.0, Some(100.0), 1),
        };

        assert_eq!(
            (week.short(), week.over()),
            (
                vec![],
                vec![
                    Gap {
                        habit: Habit::Fat,
                        per_day: 7.0
                    },
                    Gap {
                        habit: Habit::Energy,
                        per_day: 200.0
                    }
                ]
            )
        );
    }

    #[test]
    fn a_week_has_goals_once_a_macro_has_one() {
        let aimless = averaged(100.0, 50.0, 200.0, 1);
        let week = |fibre: Averaged| FuelWeek {
            logged_days: 2,
            energy: averaged(2100.0, 1800.0, 2400.0, 2),
            protein: aimless,
            fat: aimless,
            fibre,
            carbs: aimless,
        };

        assert_eq!(
            [
                week(aimless).has_goals(),
                week(aimed(25.0, 20.0, 60.0, Some(30.0), 1)).has_goals()
            ],
            [false, true]
        );
    }

    #[test]
    fn a_macro_short_of_its_goal_or_over_it_by_more_than_a_tenth_is_named_furthest_first() {
        let week = FuelWeek {
            logged_days: 6,
            energy: averaged(2100.0, 1800.0, 2400.0, 4),
            protein: aimed(96.0, 66.0, 190.0, Some(128.0), 1),
            fat: aimed(71.0, 48.0, 90.0, Some(64.0), 2),
            fibre: aimed(19.0, 15.0, 60.0, Some(32.0), 0),
            carbs: aimed(260.0, 130.0, 400.0, Some(240.0), 2),
        };

        assert_eq!(
            (week.short(), week.over()),
            (
                vec![
                    Gap {
                        habit: Habit::Fibre,
                        per_day: -13.0
                    },
                    Gap {
                        habit: Habit::Protein,
                        per_day: -32.0
                    }
                ],
                vec![Gap {
                    habit: Habit::Fat,
                    per_day: 7.0
                }]
            )
        );
    }
}
