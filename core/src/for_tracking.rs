use chrono::NaiveDate;

use crate::failure::Failure;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::ledger::{Day, ledger};
use crate::measurement::measurement;
use crate::refusal::Refusal;

const RECENT_DAYS: usize = 14;

pub fn recent_days(records: &impl ForStoringGatherRecords) -> Result<Vec<Day>, Failure> {
    let gathered = records.gather_records()?;
    Ok(ledger(&gathered.measurements, &gathered.profiles)
        .into_iter()
        .rev()
        .take(RECENT_DAYS)
        .collect())
}

pub fn log_measurement(
    records: &impl ForStoringGatherRecords,
    date: &str,
    body_mass_kg: f64,
    waist_cm: Option<f64>,
) -> Result<Vec<Day>, Failure> {
    let measured = measurement(date, body_mass_kg, waist_cm).map_err(Failure::Refused)?;
    records.save_measurement(&measured)?;
    recent_days(records)
}

pub fn remove_measurement(
    records: &impl ForStoringGatherRecords,
    date: &str,
) -> Result<Vec<Day>, Failure> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|_| Failure::Refused(Refusal::NotADate))?;
    records.remove_measurement(day)?;
    recent_days(records)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gender::Gender;
    use crate::measurement::Measurement;
    use crate::profile::Profile;
    use crate::testing::{BrokenGatherRecords, GatherRecordsInMemory};

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn listed(days: &[Day]) -> Vec<(NaiveDate, f64, Option<f64>)> {
        days.iter()
            .map(|day| (day.day, day.body_mass_kg, day.waist_cm))
            .collect()
    }

    #[test]
    fn a_logged_measurement_comes_back_among_the_recent_days() {
        let records = GatherRecordsInMemory::default();

        let recent = log_measurement(&records, "2026-10-10", 80.0, Some(80.0));

        assert_eq!(
            recent.map(|days| listed(&days)),
            Ok(vec![(day("2026-10-10"), 80.0, Some(80.0))])
        );
    }

    #[test]
    fn the_recent_days_read_their_composition_through_the_profile_in_force() {
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

        let recent = log_measurement(&records, "2026-10-10", 80.0, Some(80.0));

        let fat_ratio = recent
            .ok()
            .and_then(|days| days.first()?.composition)
            .map(|split| split.fat_ratio);
        assert!(fat_ratio.is_some_and(|ratio| (ratio - 0.116245).abs() < 1e-6));
    }

    #[test]
    fn only_the_fourteen_most_recent_days_come_back_newest_first() {
        let records = GatherRecordsInMemory::default();
        for date in 1..=20 {
            log_measurement(&records, &format!("2026-09-{date:02}"), 80.0, None)
                .expect("a measurement logs");
        }

        let recent =
            log_measurement(&records, "2026-09-21", 79.8, None).expect("a measurement logs");

        assert_eq!(recent.len(), 14);
        assert_eq!(recent.first().map(|day| day.day), Some(day("2026-09-21")));
        assert_eq!(recent.last().map(|day| day.day), Some(day("2026-09-08")));
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
    fn removing_a_measured_day_leaves_the_other_days() {
        let records = GatherRecordsInMemory::default();
        log_measurement(&records, "2026-10-09", 80.2, None).expect("a measurement logs");
        log_measurement(&records, "2026-10-10", 80.0, None).expect("a measurement logs");

        let recent = remove_measurement(&records, "2026-10-10");

        assert_eq!(
            recent.map(|days| listed(&days)),
            Ok(vec![(day("2026-10-09"), 80.2, None)])
        );
    }

    #[test]
    fn the_recent_days_list_what_was_measured_without_logging_anything() {
        let records = GatherRecordsInMemory::default();
        records
            .save_measurement(&Measurement {
                day: day("2026-10-10"),
                body_mass_kg: 80.0,
                waist_cm: None,
            })
            .expect("a measurement saves");

        assert_eq!(
            recent_days(&records).map(|days| listed(&days)),
            Ok(vec![(day("2026-10-10"), 80.0, None)])
        );
    }

    #[test]
    fn removing_a_day_the_calendar_does_not_have_is_refused() {
        assert_eq!(
            remove_measurement(&GatherRecordsInMemory::default(), "2026-02-30"),
            Err(Failure::Refused(Refusal::NotADate))
        );
    }

    #[test]
    fn gather_records_that_cannot_be_read_or_removed_from_come_back_broken() {
        let broken = Err(Failure::Broken("the disk is full".into()));

        assert_eq!(recent_days(&BrokenGatherRecords), broken);
        assert_eq!(
            remove_measurement(&BrokenGatherRecords, "2026-10-10"),
            broken
        );
    }
}
