use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::aims::aims_on;
use crate::gather_records::GatherRecords;
use crate::heading::{Aim, heading_of};
use crate::ledger::{Day, ledger};
use crate::progress::progress_towards;
use crate::tendency::Change;
use crate::trend::seven_day_trend;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pace {
    pub body_mass: Change,
    pub lean_mass: Option<Change>,
    pub fat_mass: Option<Change>,
    pub fat_ratio: Option<Change>,
    /// The weigh-ins of the window the pace reads.
    pub weigh_ins: usize,
}

/// The weigh-ins a pace reads: at least 4, and spread over a fortnight unless there are 8 of them, so a day's swing
/// of the scale, about 0.5 kg with water and salt, cannot fake a slope from a few weigh-ins close together. A design
/// choice, not a published figure.
const FEWEST_WEIGH_INS: usize = 4;
const WEIGH_INS_ENOUGH_ANYWHERE: usize = 8;
const FEWEST_DAYS_SPANNED: i64 = 14;
/// How finely a mass and a ratio are shown, so a change that shows as nothing holds steady.
const A_TENTH_OF_A_KILO: f64 = 0.1;
const A_TENTH_OF_A_PERCENT: f64 = 0.001;

/// Whether weigh-ins are enough for a slope: 8 of them, or at least 4 spread over a fortnight.
fn reads_a_slope(weighed: &[NaiveDate]) -> bool {
    let spanned = weighed
        .iter()
        .min()
        .zip(weighed.iter().max())
        .map_or(0, |(first, last)| (*last - *first).num_days() + 1);
    weighed.len() >= WEIGH_INS_ENOUGH_ANYWHERE
        || (weighed.len() >= FEWEST_WEIGH_INS && spanned >= FEWEST_DAYS_SPANNED)
}

/// The slope, per week, of the seven-day average of a measure from a first day to the last, as Home's chart
/// reads it over its window, with the latest of those averages.
fn weekly(first: NaiveDate, day: NaiveDate, readings: &[(NaiveDate, f64)]) -> Option<(f64, f64)> {
    let in_the_week = |read: NaiveDate| (first..=day).contains(&read);
    let weighed_in_the_week: Vec<NaiveDate> = readings
        .iter()
        .map(|(read, _)| *read)
        .filter(|read| in_the_week(*read))
        .collect();
    if !reads_a_slope(&weighed_in_the_week) {
        return None;
    }
    let up_to_the_day: Vec<(NaiveDate, f64)> = readings
        .iter()
        .filter(|(read, _)| *read <= day)
        .copied()
        .collect();
    let trend: Vec<(NaiveDate, f64)> = seven_day_trend(&up_to_the_day)
        .into_iter()
        .filter(|(averaged, _)| in_the_week(*averaged))
        .collect();
    let progress = progress_towards(None, &trend)?;
    Some((progress.per_week, trend.last()?.1))
}

/// The pace from a first day to the last, read per week as on the last day of a week.
pub fn pace_between(first: NaiveDate, day: NaiveDate, gathered: &GatherRecords) -> Option<Pace> {
    let days = ledger(&gathered.measurements, &gathered.profiles);
    let readings = |measure: fn(&Day) -> Option<f64>| -> Vec<(NaiveDate, f64)> {
        days.iter()
            .filter_map(|measured| measure(measured).map(|value| (measured.day, value)))
            .collect()
    };
    let (per_week, body_mass_kg) = weekly(
        first,
        day,
        &readings(|measured| Some(measured.body_mass_kg)),
    )?;
    let aims = aims_on(day, gathered);
    let change = |measure: fn(&Day) -> Option<f64>, aim: Option<Aim>, shown_to: f64| {
        weekly(first, day, &readings(measure)).map(|(per_week, now)| Change {
            by: per_week,
            heading: heading_of(per_week, now, aim, shown_to),
        })
    };
    Some(Pace {
        body_mass: Change {
            by: per_week,
            heading: heading_of(
                per_week,
                body_mass_kg,
                aims.map(|aims| aims.body_mass()),
                A_TENTH_OF_A_KILO,
            ),
        },
        lean_mass: change(
            |measured| measured.composition.map(|split| split.lean_mass_kg),
            aims.map(|aims| aims.lean_mass()),
            A_TENTH_OF_A_KILO,
        ),
        fat_mass: change(
            |measured| measured.composition.map(|split| split.fat_mass_kg),
            aims.map(|aims| Aim::Reach(aims.fat_mass_kg())),
            A_TENTH_OF_A_KILO,
        ),
        fat_ratio: change(
            |measured| measured.composition.map(|split| split.fat_ratio),
            aims.map(|aims| Aim::Reach(aims.fat_ratio())),
            A_TENTH_OF_A_PERCENT,
        ),
        weigh_ins: days
            .iter()
            .filter(|measured| (first..=day).contains(&measured.day))
            .count(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kept::Kept;
    use chrono::Days;

    use crate::gender::Gender;
    use crate::measurement::Measurement;
    use crate::profile::Profile;
    use crate::tendency::Heading;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    /// The pace over the fortnight up to the day.
    fn pace_on(day: NaiveDate, gathered: &GatherRecords) -> Option<Pace> {
        pace_between(day - Days::new(13), day, gathered)
    }

    fn a_man_weighed(from_kg: f64, kg_a_day: f64) -> GatherRecords {
        let last = day("2026-10-10");
        GatherRecords {
            profiles: vec![(
                day("2026-03-01"),
                Profile {
                    birthday: day("1985-04-12"),
                    gender: Gender::Male,
                    height_cm: 180.0,
                    neck_cm: 38.5,
                    hip_cm: None,
                },
            )],
            measurements: (0..21)
                .map(|at| Measurement {
                    day: last - Days::new(20 - at),
                    body_mass_kg: from_kg + kg_a_day * at as f64,
                    waist_cm: Some(82.0),
                })
                .collect(),
            goals: vec![],
        }
    }

    #[test]
    fn a_weeks_pace_is_the_slope_of_the_seven_day_average_over_the_last_fourteen_days() {
        let pace = pace_on(day("2026-10-10"), &a_man_weighed(80.0, -0.05)).expect("a pace");

        assert!(
            (pace.body_mass.by - -0.35).abs() < 1e-9,
            "{}",
            pace.body_mass.by
        );
    }

    fn aiming_at(body_mass_kg: f64, gathered: GatherRecords) -> GatherRecords {
        GatherRecords {
            goals: vec![(
                day("2026-03-01"),
                Kept::Own(crate::goals::Goals {
                    body_mass_kg,
                    fat_ratio: 0.12,
                }),
            )],
            ..gathered
        }
    }

    #[test]
    fn each_measure_heads_toward_or_away_from_its_aim_holds_steady_or_stays_above_its_floor() {
        let pace = pace_on(
            day("2026-10-10"),
            &aiming_at(70.0, a_man_weighed(81.0, -0.05)),
        )
        .expect("a pace");

        assert_eq!(
            [
                Some(pace.body_mass.heading),
                pace.lean_mass.map(|change| change.heading),
                pace.fat_mass.map(|change| change.heading),
                pace.fat_ratio.map(|change| change.heading)
            ],
            [
                Some(Heading::Toward),
                Some(Heading::AboveItsFloor),
                Some(Heading::Steady),
                Some(Heading::Steady)
            ]
        );
    }

    #[test]
    fn a_lean_mass_rising_above_its_goal_heads_toward_the_aim_its_goal_being_a_floor() {
        let pace = pace_on(
            day("2026-10-10"),
            &aiming_at(70.0, a_man_weighed(74.0, 0.05)),
        )
        .expect("a pace");

        assert_eq!(
            pace.lean_mass.map(|change| change.heading),
            Some(Heading::Toward)
        );
    }

    #[test]
    fn a_weigh_in_on_the_day_counts_in_its_pace() {
        let flat = a_man_weighed(80.0, 0.0);
        let dropping_on_the_day = GatherRecords {
            measurements: flat
                .measurements
                .iter()
                .map(|measured| Measurement {
                    body_mass_kg: if measured.day == day("2026-10-10") {
                        78.0
                    } else {
                        measured.body_mass_kg
                    },
                    ..*measured
                })
                .collect(),
            ..flat
        };

        let pace = pace_on(day("2026-10-10"), &dropping_on_the_day).expect("a pace");

        assert!(pace.body_mass.by < 0.0, "{}", pace.body_mass.by);
    }

    #[test]
    fn weigh_ins_after_the_day_leave_its_pace_as_it_was() {
        let weighed = a_man_weighed(80.0, -0.05);
        let and_after = GatherRecords {
            measurements: weighed
                .measurements
                .iter()
                .copied()
                .chain((1..5).map(|later| Measurement {
                    day: day("2026-10-10") + Days::new(later),
                    body_mass_kg: 90.0,
                    waist_cm: Some(82.0),
                }))
                .collect(),
            ..weighed.clone()
        };

        assert_eq!(
            pace_on(day("2026-10-10"), &and_after),
            pace_on(day("2026-10-10"), &weighed)
        );
    }

    #[test]
    fn weigh_ins_older_than_the_week_do_not_make_up_for_those_missing_in_it() {
        let weighed = a_man_weighed(80.0, -0.05);
        let three_in_the_week = GatherRecords {
            measurements: weighed.measurements[..6]
                .iter()
                .chain(&weighed.measurements[18..])
                .copied()
                .collect(),
            ..weighed
        };

        assert_eq!(pace_on(day("2026-10-10"), &three_in_the_week), None);
    }

    #[test]
    fn the_fortnights_pace_reads_the_slope_of_the_seven_day_average_over_its_fourteen_days() {
        let weighed = [
            71.4, 71.0, 71.1, 71.2, 71.4, 71.4, 70.9, 71.5, 71.0, 71.7, 71.7, 71.4, 71.5, 71.7,
            72.4, 72.4, 72.1, 72.2, 72.0,
        ];
        let gathered = GatherRecords {
            measurements: weighed
                .iter()
                .zip(0..)
                .map(|(kg, back)| Measurement {
                    day: day("2026-10-19") - Days::new(back),
                    body_mass_kg: *kg,
                    waist_cm: None,
                })
                .collect(),
            ..a_man_weighed(80.0, 0.0)
        };

        let pace = pace_on(day("2026-10-19"), &gathered).expect("a pace");

        assert!((pace.body_mass.by - -0.542).abs() < 1e-3);
    }

    #[test]
    fn four_weigh_ins_on_four_days_running_give_no_pace_a_days_swing_could_fake() {
        let weighed = a_man_weighed(80.0, -0.05);
        let four_days_running = GatherRecords {
            measurements: weighed.measurements[17..].to_vec(),
            ..weighed
        };

        assert_eq!(pace_on(day("2026-10-10"), &four_days_running), None);
    }

    #[test]
    fn four_weigh_ins_from_the_first_day_of_the_fortnight_to_its_last_read_a_pace() {
        let weighed = a_man_weighed(80.0, -0.05);
        let spread_over_the_fortnight = GatherRecords {
            measurements: [7, 11, 16, 20]
                .iter()
                .map(|at| weighed.measurements[*at])
                .collect(),
            ..weighed
        };

        assert!(pace_on(day("2026-10-10"), &spread_over_the_fortnight).is_some());
    }

    #[test]
    fn fewer_than_four_weigh_ins_in_the_week_give_no_pace() {
        let weighed = a_man_weighed(80.0, -0.05);
        let three_weigh_ins = GatherRecords {
            measurements: weighed.measurements[18..].to_vec(),
            ..weighed
        };

        assert_eq!(pace_on(day("2026-10-10"), &three_weigh_ins), None);
    }
}
