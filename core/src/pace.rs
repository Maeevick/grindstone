use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::aims::aims_on;
use crate::body::body_on;
use crate::fuel::SPARED_KCAL_PER_KG_OF_FAT;
use crate::gather_records::GatherRecords;
use crate::heading::heading_of;
use crate::judgement::{Judgement, Week, judgement_of};
use crate::ledger::{Day, ledger};
use crate::progress::progress_towards;
use crate::recommendations::recommended;
use crate::tendency::Change;
use crate::trend::seven_day_trend;
use crate::version::version_in_force_on;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pace {
    pub body_mass: Change,
    pub lean_mass: Option<Change>,
    pub fat_mass: Option<Change>,
    pub fat_ratio: Option<Change>,
    pub judgement: Judgement,
    pub safe_loss_kg: f64,
}

const DAYS_OF_THE_SLOPE: u64 = 13;
const FEWEST_WEIGH_INS: usize = 4;
const DAYS_A_WEEK: f64 = 7.0;
const LOSS_SHARE_A_WEEK_AT_MOST: f64 = 0.01;
const LOSS_KG_A_WEEK_AT_MOST: f64 = 1.0;
const KCAL_PER_KG_OF_FAT_TISSUE: f64 = 7700.0;

/// The most a body can lose in a week without eating into its lean mass: 1% of its mass, 1 kg at most, and
/// no more than its fat store can spare.
///
/// - 0.5 to 1% of the body mass a week keeps the muscle: Helms E. R., Aragon A. A., Fitschen P. J. (2014),
///   J Int Soc Sports Nutr 11:20; 0.7% kept more lean mass than 1.4%: Garthe I. et al. (2011), Int J Sport Nutr
///   Exerc Metab 21(2):97-104.
/// - No more than 0.5 to 1 kg a week: NICE (2025), *Overweight and obesity management*, NG246.
/// - What the fat store can spare, about 48.5 kcal per kilo of fat a day: Alpert S. S. (2005), J Theor Biol
///   233(1):1-13; about 7,700 kcal a kilo of fat tissue: Wishnofsky M. (1958), *Caloric equivalent of gained or lost weight*, Am J Clin Nutr 6(5):542-546.
fn safe_loss_kg(body_mass_kg: f64, fat_kg: Option<f64>) -> f64 {
    let spared = fat_kg.map_or(f64::INFINITY, |fat_kg| {
        SPARED_KCAL_PER_KG_OF_FAT * fat_kg * DAYS_A_WEEK / KCAL_PER_KG_OF_FAT_TISSUE
    });
    (LOSS_SHARE_A_WEEK_AT_MOST * body_mass_kg)
        .min(LOSS_KG_A_WEEK_AT_MOST)
        .min(spared)
}

/// The slope, per week, of the seven-day average of a measure over the fourteen days ending on a day,
/// with the latest of those averages.
fn weekly(day: NaiveDate, readings: &[(NaiveDate, f64)]) -> Option<(f64, f64)> {
    let in_the_fortnight =
        |read: NaiveDate| read <= day && read + Days::new(DAYS_OF_THE_SLOPE) >= day;
    let weighed_in_the_fortnight = readings
        .iter()
        .filter(|(read, _)| in_the_fortnight(*read))
        .count();
    if weighed_in_the_fortnight < FEWEST_WEIGH_INS {
        return None;
    }
    let up_to_the_day: Vec<(NaiveDate, f64)> = readings
        .iter()
        .filter(|(read, _)| *read <= day)
        .copied()
        .collect();
    let trend: Vec<(NaiveDate, f64)> = seven_day_trend(&up_to_the_day)
        .into_iter()
        .filter(|(averaged, _)| in_the_fortnight(*averaged))
        .collect();
    let progress = progress_towards(None, &trend)?;
    Some((progress.per_week, trend.last()?.1))
}

/// The week's pace on a day: how each measure moved per week, heading toward its aim or away from it, and a
/// judgement of the body mass's pace, health first.
pub fn pace_on(day: NaiveDate, gathered: &GatherRecords) -> Option<Pace> {
    let days = ledger(&gathered.measurements, &gathered.profiles);
    let readings = |measure: fn(&Day) -> Option<f64>| -> Vec<(NaiveDate, f64)> {
        days.iter()
            .filter_map(|measured| measure(measured).map(|value| (measured.day, value)))
            .collect()
    };
    let (per_week, body_mass_kg) = weekly(day, &readings(|measured| Some(measured.body_mass_kg)))?;
    let aims = aims_on(day, gathered);
    let change = |measure: fn(&Day) -> Option<f64>, aim: Option<f64>| {
        weekly(day, &readings(measure)).map(|(per_week, now)| Change {
            by: per_week,
            heading: heading_of(per_week, now, aim),
        })
    };
    let body = body_on(day, &days, &gathered.profiles);
    let safe_loss_kg = safe_loss_kg(body_mass_kg, body.as_ref().map(|body| body.fat_kg));
    let healthy = version_in_force_on(&gathered.profiles, day)
        .map(|profile| recommended(profile, day, Some(body_mass_kg)));
    let judgement = judgement_of(&Week {
        per_week,
        body_mass_kg,
        fat_ratio: body.as_ref().map(|body| body.fat_kg / body.mass_kg),
        healthy_mass_from_kg: healthy.map(|healthy| healthy.healthy_mass_kg.0),
        healthy_fat_ratio_from: healthy.map(|healthy| healthy.healthy_fat_ratio.0),
        safe_loss_kg,
        aim_kg: aims.map(|aims| aims.body_mass_kg),
    });
    Some(Pace {
        body_mass: Change {
            by: per_week,
            heading: heading_of(per_week, body_mass_kg, aims.map(|aims| aims.body_mass_kg)),
        },
        lean_mass: change(
            |measured| measured.composition.map(|split| split.lean_mass_kg),
            aims.map(|aims| aims.lean_mass_kg),
        ),
        fat_mass: change(
            |measured| measured.composition.map(|split| split.fat_mass_kg),
            aims.map(|aims| aims.fat_mass_kg()),
        ),
        fat_ratio: change(
            |measured| measured.composition.map(|split| split.fat_ratio),
            aims.map(|aims| aims.fat_ratio()),
        ),
        judgement,
        safe_loss_kg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::gender::Gender;
    use crate::measurement::Measurement;
    use crate::profile::Profile;
    use crate::tendency::Heading;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
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
                crate::goals::Goals {
                    body_mass_kg,
                    fat_ratio: 0.12,
                },
            )],
            ..gathered
        }
    }

    fn fat_kg_of(gathered: &GatherRecords) -> f64 {
        crate::ledger::ledger(&gathered.measurements, &gathered.profiles)
            .last()
            .and_then(|measured| measured.composition)
            .map(|composition| composition.fat_mass_kg)
            .expect("a fat mass")
    }

    #[test]
    fn losing_faster_than_the_fat_store_can_spare_a_week_is_dangerous() {
        let losing_a_kilo_a_week = aiming_at(70.0, a_man_weighed(81.0, -0.14));

        let pace = pace_on(day("2026-10-10"), &losing_a_kilo_a_week).expect("a pace");

        assert_eq!(pace.judgement, Judgement::LosingTooFast);
        assert!(
            (pace.safe_loss_kg - 290.0 / 4.184 * fat_kg_of(&losing_a_kilo_a_week) * 7.0 / 7700.0)
                .abs()
                < 1e-9
        );
    }

    fn judged(gathered: &GatherRecords) -> Judgement {
        pace_on(day("2026-10-10"), gathered)
            .expect("a pace")
            .judgement
    }

    fn a_man_with_a_waist_weighed(waist_cm: f64, from_kg: f64, kg_a_day: f64) -> GatherRecords {
        let weighed = a_man_weighed(from_kg, kg_a_day);
        GatherRecords {
            measurements: weighed
                .measurements
                .iter()
                .map(|measured| Measurement {
                    waist_cm: Some(waist_cm),
                    ..*measured
                })
                .collect(),
            ..weighed
        }
    }

    #[test]
    fn losing_under_the_healthy_mass_or_fat_is_dangerous_whatever_the_aim() {
        let under_the_healthy_mass = aiming_at(55.0, a_man_with_a_waist_weighed(72.0, 59.0, -0.02));
        let under_the_healthy_fat = aiming_at(70.0, a_man_with_a_waist_weighed(70.0, 76.0, -0.02));

        assert_eq!(
            (
                judged(&under_the_healthy_mass),
                judged(&under_the_healthy_fat)
            ),
            (Judgement::UnderHealthyMass, Judgement::UnderHealthyFat)
        );
    }

    fn losing_at(share_of_the_safe_loss: f64) -> GatherRecords {
        let safe_loss_kg = pace_on(
            day("2026-10-10"),
            &aiming_at(70.0, a_man_weighed(81.0, -0.05)),
        )
        .expect("a pace")
        .safe_loss_kg;
        aiming_at(
            70.0,
            a_man_weighed(81.0, -share_of_the_safe_loss * safe_loss_kg / 7.0),
        )
    }

    #[test]
    fn losing_between_half_the_safe_loss_and_all_of_it_towards_the_aim_is_very_good() {
        assert_eq!(judged(&losing_at(0.75)), Judgement::VeryGood);
    }

    #[test]
    fn losing_slower_than_half_the_safe_loss_towards_the_aim_is_good() {
        assert_eq!(judged(&losing_at(0.3)), Judgement::Good);
    }

    #[test]
    fn gaining_away_from_a_lower_aim_is_the_wrong_way_and_moving_less_than_015_percent_is_stable() {
        assert_eq!(
            (
                judged(&aiming_at(70.0, a_man_weighed(80.0, 0.03))),
                judged(&aiming_at(70.0, a_man_weighed(80.0, 0.0)))
            ),
            (Judgement::WrongWay, Judgement::Stable)
        );
    }

    #[test]
    fn towards_a_higher_aim_gaining_a_quarter_to_half_a_percent_a_week_is_very_good_slower_good_faster_too_fast()
     {
        let gaining = |share_a_week: f64| {
            judged(&aiming_at(
                85.0,
                a_man_weighed(75.0, share_a_week * 75.0 / 7.0),
            ))
        };

        assert_eq!(
            [
                gaining(0.0035),
                gaining(0.002),
                gaining(0.008),
                gaining(-0.004)
            ],
            [
                Judgement::VeryGood,
                Judgement::Good,
                Judgement::GainingTooFast,
                Judgement::WrongWay
            ]
        );
    }

    #[test]
    fn within_1_percent_of_the_aim_and_steady_is_holding() {
        assert_eq!(
            judged(&aiming_at(70.0, a_man_weighed(70.5, 0.0))),
            Judgement::Holding
        );
    }

    #[test]
    fn each_measure_heads_toward_or_away_from_its_aim_or_holds_steady() {
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
                Some(Heading::Toward),
                Some(Heading::Toward),
                Some(Heading::Steady)
            ]
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
    fn weigh_ins_older_than_the_fortnight_do_not_make_up_for_those_missing_in_it() {
        let weighed = a_man_weighed(80.0, -0.05);
        let three_in_the_fortnight = GatherRecords {
            measurements: weighed.measurements[..6]
                .iter()
                .chain(&weighed.measurements[18..])
                .copied()
                .collect(),
            ..weighed
        };

        assert_eq!(pace_on(day("2026-10-10"), &three_in_the_fortnight), None);
    }

    #[test]
    fn a_light_body_loses_no_more_than_1_percent_of_its_mass_a_week() {
        let light_and_round = a_man_with_a_waist_weighed(100.0, 60.0, 0.0);

        let pace = pace_on(day("2026-10-10"), &light_and_round).expect("a pace");

        assert!(
            (pace.safe_loss_kg - 0.6).abs() < 1e-9,
            "{}",
            pace.safe_loss_kg
        );
    }

    #[test]
    fn fewer_than_four_weigh_ins_in_the_fortnight_give_no_pace() {
        let weighed = a_man_weighed(80.0, -0.05);
        let three_weigh_ins = GatherRecords {
            measurements: weighed.measurements[18..].to_vec(),
            ..weighed
        };

        assert_eq!(pace_on(day("2026-10-10"), &three_weigh_ins), None);
    }
}
