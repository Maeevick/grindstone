use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::gather_records::GatherRecords;
use crate::ledger::{Day, ledger};
use crate::measurement::measurement;
use crate::refusal::Refusal;
use crate::tendency::{Tendency, tendency_of};
use crate::version::version_in_force_on;

const DAYS_BEFORE_IN_A_WEEK: u64 = 6;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GatherWeek {
    pub day: Option<Day>,
    pub days: Vec<Day>,
    pub tendency: Tendency,
}

fn day_of(date: &str) -> Result<NaiveDate, Failure> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Failure::Refused(Refusal::NotADate))
}

fn week_ending_on(gathered: &GatherRecords, day: NaiveDate) -> GatherWeek {
    let first = day - Days::new(DAYS_BEFORE_IN_A_WEEK);
    let days: Vec<Day> = ledger(&gathered.measurements, &gathered.profiles)
        .into_iter()
        .filter(|measured| (first..=day).contains(&measured.day))
        .rev()
        .collect();
    GatherWeek {
        day: days.first().filter(|measured| measured.day == day).copied(),
        tendency: tendency_of(&days, version_in_force_on(&gathered.goals, day)),
        days,
    }
}

pub fn gather_week(
    records: &impl ForStoringGatherRecords,
    date: &str,
) -> Result<GatherWeek, Failure> {
    let day = day_of(date)?;
    Ok(week_ending_on(&records.gather_records()?, day))
}

pub fn log_measurement(
    records: &impl ForStoringGatherRecords,
    date: &str,
    body_mass_kg: f64,
    waist_cm: Option<f64>,
) -> Result<GatherWeek, Failure> {
    let measured = measurement(date, body_mass_kg, waist_cm).map_err(Failure::Refused)?;
    records.save_measurement(&measured)?;
    Ok(week_ending_on(&records.gather_records()?, measured.day))
}

pub fn remove_measurement(
    records: &impl ForStoringGatherRecords,
    date: &str,
) -> Result<GatherWeek, Failure> {
    let day = day_of(date)?;
    records.remove_measurement(day)?;
    Ok(week_ending_on(&records.gather_records()?, day))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gender::Gender;
    use crate::goals::Goals;
    use crate::measurement::Measurement;
    use crate::profile::Profile;
    use crate::tendency::Heading;
    use crate::testing::{BrokenGatherRecords, GatherRecordsInMemory};

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn listed(days: &[Day]) -> Vec<(NaiveDate, f64, Option<f64>)> {
        days.iter()
            .map(|day| (day.day, day.body_mass_kg, day.waist_cm))
            .collect()
    }

    fn a_september_of_weigh_ins() -> GatherRecordsInMemory {
        let records = GatherRecordsInMemory::default();
        for date in 1..=20 {
            log_measurement(&records, &format!("2026-09-{date:02}"), 80.0, None)
                .expect("a measurement logs");
        }
        records
    }

    #[test]
    fn a_logged_measurement_comes_back_as_its_weeks_own_day() {
        let records = GatherRecordsInMemory::default();

        let week = log_measurement(&records, "2026-10-10", 80.0, Some(80.0)).expect("a week");

        assert_eq!(
            week.day
                .map(|own| (own.day, own.body_mass_kg, own.waist_cm)),
            Some((day("2026-10-10"), 80.0, Some(80.0)))
        );
        assert_eq!(
            listed(&week.days),
            vec![(day("2026-10-10"), 80.0, Some(80.0))]
        );
    }

    #[test]
    fn a_week_reads_its_composition_through_the_profile_in_force() {
        let records = GatherRecordsInMemory::default();
        let a_man = Profile {
            birthday: day("1985-04-12"),
            gender: Gender::Male,
            height_cm: 180.0,
            neck_cm: 38.5,
            hip_cm: None,
        };
        records
            .save_profile(day("2026-03-01"), &a_man)
            .expect("a profile saves");

        let week = log_measurement(&records, "2026-10-10", 80.0, Some(80.0));

        let fat_ratio = week
            .ok()
            .and_then(|week| week.days.first()?.composition)
            .map(|split| split.fat_ratio);
        assert!(fat_ratio.is_some_and(|ratio| (ratio - 0.116245).abs() < 1e-6));
    }

    #[test]
    fn a_week_holds_the_seven_days_ending_on_its_day_newest_first() {
        let records = a_september_of_weigh_ins();

        let week = gather_week(&records, "2026-09-14").expect("a week");

        assert_eq!(
            week.days
                .iter()
                .map(|measured| measured.day)
                .collect::<Vec<_>>(),
            (8..=14)
                .rev()
                .map(|date| day(&format!("2026-09-{date:02}")))
                .collect::<Vec<_>>()
        );
        assert_eq!(week.day.map(|own| own.day), Some(day("2026-09-14")));
    }

    #[test]
    fn a_day_not_gathered_yet_has_no_record_of_its_own_but_its_week_holds_the_days_before() {
        let records = a_september_of_weigh_ins();

        let week = gather_week(&records, "2026-09-22").expect("a week");

        assert_eq!(week.day, None);
        assert_eq!(
            week.days
                .iter()
                .map(|measured| measured.day)
                .collect::<Vec<_>>(),
            (16..=20)
                .rev()
                .map(|date| day(&format!("2026-09-{date:02}")))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn a_week_tells_its_tendency_against_the_goals_in_force() {
        let records = GatherRecordsInMemory::default();
        records
            .save_goals(
                day("2026-09-01"),
                &Goals {
                    body_mass_kg: 75.0,
                    fat_ratio: 0.15,
                },
            )
            .expect("goals save");
        log_measurement(&records, "2026-10-01", 80.0, None).expect("a measurement logs");
        log_measurement(&records, "2026-10-04", 80.5, None).expect("a measurement logs");

        let week = log_measurement(&records, "2026-10-07", 79.0, None).expect("a week");

        assert_eq!(
            week.tendency.body_mass.map(|change| change.heading),
            Some(Heading::Toward)
        );
        assert!(
            week.tendency
                .body_mass
                .is_some_and(|change| (change.by - -1.0).abs() < 1e-9)
        );
    }

    #[test]
    fn a_refused_measurement_leaves_the_records_as_they_were() {
        let records = GatherRecordsInMemory::default();

        let refused = log_measurement(&records, "2026-10-10", 0.0, None);

        assert_eq!(refused, Err(Failure::Refused(Refusal::NotAMass)));
        assert_eq!(
            records
                .gather_records()
                .map(|gathered| gathered.measurements),
            Ok(vec![])
        );
    }

    #[test]
    fn gather_records_that_cannot_be_written_come_back_broken() {
        assert_eq!(
            log_measurement(&BrokenGatherRecords, "2026-10-10", 80.0, None),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    #[test]
    fn removing_a_measured_day_answers_its_week_without_it() {
        let records = GatherRecordsInMemory::default();
        log_measurement(&records, "2026-10-09", 80.2, None).expect("a measurement logs");
        log_measurement(&records, "2026-10-10", 80.0, None).expect("a measurement logs");

        let week = remove_measurement(&records, "2026-10-10").expect("a week");

        assert_eq!(week.day, None);
        assert_eq!(listed(&week.days), vec![(day("2026-10-09"), 80.2, None)]);
    }

    #[test]
    fn a_week_lists_what_was_measured_without_logging_anything() {
        let records = GatherRecordsInMemory::default();
        records
            .save_measurement(&Measurement {
                day: day("2026-10-10"),
                body_mass_kg: 80.0,
                waist_cm: None,
            })
            .expect("a measurement saves");

        assert_eq!(
            gather_week(&records, "2026-10-10").map(|week| listed(&week.days)),
            Ok(vec![(day("2026-10-10"), 80.0, None)])
        );
    }

    #[test]
    fn a_week_or_a_removal_on_a_day_the_calendar_does_not_have_is_refused() {
        let records = GatherRecordsInMemory::default();

        assert_eq!(
            remove_measurement(&records, "2026-02-30"),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            gather_week(&records, "2026-02-30"),
            Err(Failure::Refused(Refusal::NotADate))
        );
    }

    #[test]
    fn gather_records_that_cannot_be_read_or_removed_from_come_back_broken() {
        let broken = Err(Failure::Broken("the disk is full".into()));

        assert_eq!(gather_week(&BrokenGatherRecords, "2026-10-10"), broken);
        assert_eq!(
            remove_measurement(&BrokenGatherRecords, "2026-10-10"),
            broken
        );
    }
}
