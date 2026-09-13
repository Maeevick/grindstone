//! What is worth sharpening on each chip over a window: the weakest habit against its band, the
//! course of the body mass, the rhythm of the workouts. A habit, not a verdict.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::for_charting::{Insights, Tally};
use crate::for_reviewing_progress::GatherCard;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Habit {
    Energy,
    Protein,
    Fat,
    Fibre,
    Carbs,
    Hydrate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Leaning {
    Under,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BandNote {
    Unlogged,
    Sharp {
        weakest: Habit,
        in_band: usize,
        logged: usize,
    },
    Blunt {
        habit: Habit,
        in_band: usize,
        logged: usize,
        leaning: Leaning,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum GatherNote {
    Unweighed,
    Irregular {
        weighed_days: usize,
        days: usize,
    },
    TooSoon,
    Aimless {
        per_week_kg: f64,
    },
    OnCourse {
        per_week_kg: f64,
        reached_on: NaiveDate,
    },
    OffCourse {
        per_week_kg: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BurnNote {
    Idle,
    Sparse {
        per_week: f64,
    },
    Steady {
        per_week: f64,
        burnt_per_active_day: Option<f64>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sharpening {
    pub gather: GatherNote,
    pub fuel: BandNote,
    pub hydrate: BandNote,
    pub burn: BurnNote,
}

pub fn sharpening_of(_insights: &Insights, _card: &GatherCard) -> Sharpening {
    Sharpening {
        gather: GatherNote::TooSoon,
        fuel: BandNote::Unlogged,
        hydrate: BandNote::Unlogged,
        burn: BurnNote::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::Progress;

    fn tally(low: usize, in_band: usize, high: usize) -> Tally {
        Tally { low, in_band, high }
    }

    fn fed(energy: Tally, protein: Tally, fat: Tally, fibre: Tally, carbs: Tally) -> Insights {
        Insights {
            days: 10,
            fuelled_days: 10,
            energy,
            protein,
            fat,
            fibre,
            carbs,
            ..Insights::default()
        }
    }

    fn weighed(weighed_days: usize, days: usize) -> Insights {
        Insights {
            days,
            weighed_days,
            ..Insights::default()
        }
    }

    fn heading(target_kg: Option<f64>, reached_on: Option<NaiveDate>) -> GatherCard {
        GatherCard {
            target_kg,
            progress: Some(Progress {
                per_week: -0.5,
                share_per_week: -0.00625,
                reached_on,
            }),
            ..GatherCard::default()
        }
    }

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    #[test]
    fn with_nothing_logged_there_is_nothing_to_sharpen_yet() {
        assert_eq!(
            sharpening_of(&Insights::default(), &GatherCard::default()),
            Sharpening {
                gather: GatherNote::Unweighed,
                fuel: BandNote::Unlogged,
                hydrate: BandNote::Unlogged,
                burn: BurnNote::Idle,
            }
        );
    }

    #[test]
    fn the_fuel_note_names_the_weakest_habit_and_the_side_of_its_band_it_misses() {
        let short_of_fibre = fed(
            tally(1, 8, 1),
            tally(4, 5, 1),
            tally(0, 9, 1),
            tally(7, 3, 0),
            tally(0, 9, 1),
        );
        let heavy_on_carbs = fed(
            tally(1, 8, 1),
            tally(4, 5, 1),
            tally(0, 9, 1),
            tally(5, 5, 0),
            tally(0, 2, 8),
        );

        assert_eq!(
            sharpening_of(&short_of_fibre, &GatherCard::default()).fuel,
            BandNote::Blunt {
                habit: Habit::Fibre,
                in_band: 3,
                logged: 10,
                leaning: Leaning::Under
            }
        );
        assert_eq!(
            sharpening_of(&heavy_on_carbs, &GatherCard::default()).fuel,
            BandNote::Blunt {
                habit: Habit::Carbs,
                in_band: 2,
                logged: 10,
                leaning: Leaning::Over
            }
        );
    }

    #[test]
    fn when_even_the_weakest_habit_holds_its_band_four_days_in_five_the_fuel_is_sharp() {
        let held = fed(
            tally(1, 9, 0),
            tally(2, 8, 0),
            tally(0, 10, 0),
            tally(2, 8, 0),
            tally(0, 9, 1),
        );

        assert_eq!(
            sharpening_of(&held, &GatherCard::default()).fuel,
            BandNote::Sharp {
                weakest: Habit::Protein,
                in_band: 8,
                logged: 10
            }
        );
    }

    #[test]
    fn the_hydrate_note_reads_the_hydrate_band_alone() {
        let dry = Insights {
            days: 10,
            hydrated_days: 6,
            hydrate: tally(4, 2, 0),
            ..Insights::default()
        };

        assert_eq!(
            sharpening_of(&dry, &GatherCard::default()).hydrate,
            BandNote::Blunt {
                habit: Habit::Hydrate,
                in_band: 2,
                logged: 6,
                leaning: Leaning::Under
            }
        );
    }

    #[test]
    fn the_gather_note_asks_for_a_weigh_in_every_other_day_at_least_then_follows_the_course() {
        let regular = weighed(20, 30);

        assert_eq!(
            sharpening_of(&weighed(10, 30), &heading(Some(75.0), None)).gather,
            GatherNote::Irregular {
                weighed_days: 10,
                days: 30
            }
        );
        assert_eq!(
            sharpening_of(&regular, &GatherCard::default()).gather,
            GatherNote::TooSoon
        );
        assert_eq!(
            sharpening_of(&regular, &heading(None, None)).gather,
            GatherNote::Aimless { per_week_kg: -0.5 }
        );
        assert_eq!(
            sharpening_of(&regular, &heading(Some(75.0), Some(day("2026-12-24")))).gather,
            GatherNote::OnCourse {
                per_week_kg: -0.5,
                reached_on: day("2026-12-24")
            }
        );
        assert_eq!(
            sharpening_of(&regular, &heading(Some(85.0), None)).gather,
            GatherNote::OffCourse { per_week_kg: -0.5 }
        );
    }

    #[test]
    fn two_active_days_a_week_keep_the_burn_steady_fewer_leave_it_sparse() {
        let active = |active_days| Insights {
            days: 28,
            active_days,
            burnt_per_active_day: Some(300.0),
            ..Insights::default()
        };

        assert_eq!(
            sharpening_of(&active(8), &GatherCard::default()).burn,
            BurnNote::Steady {
                per_week: 2.0,
                burnt_per_active_day: Some(300.0)
            }
        );
        assert_eq!(
            sharpening_of(&active(4), &GatherCard::default()).burn,
            BurnNote::Sparse { per_week: 1.0 }
        );
    }
}
