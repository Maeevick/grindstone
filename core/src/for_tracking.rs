use crate::failure::Failure;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_weigh_ins::ForStoringWeighIns;
use crate::ledger::{Day, ledger};
use crate::measurement::measurement;
use crate::weigh_in::{WeighIn, weigh_in};

const RECENT_WEIGH_INS: u32 = 14;

pub fn recent_weigh_ins(weigh_ins: &impl ForStoringWeighIns) -> Result<Vec<WeighIn>, Failure> {
    Ok(weigh_ins.recent(RECENT_WEIGH_INS)?)
}

pub fn log_weigh_in(
    weigh_ins: &impl ForStoringWeighIns,
    date: &str,
    kg: f64,
) -> Result<Vec<WeighIn>, Failure> {
    let weighed = weigh_in(date, kg).map_err(Failure::Refused)?;
    weigh_ins.save(&weighed)?;
    recent_weigh_ins(weigh_ins)
}

pub fn log_measurement(
    records: &impl ForStoringGatherRecords,
    date: &str,
    body_mass_kg: f64,
    waist_cm: Option<f64>,
) -> Result<Vec<Day>, Failure> {
    if let Ok(measured) = measurement(date, body_mass_kg, waist_cm) {
        let _ = records.save_measurement(&measured);
    }
    let gathered = records.gather_records().unwrap_or_default();
    Ok(ledger(&gathered.measurements, &[]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::failure::Broken;
    use crate::gender::Gender;
    use crate::profile::Profile;
    use crate::refusal::Refusal;
    use crate::testing::{GatherRecordsInMemory, WeighInsInMemory};
    use chrono::NaiveDate;

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

    struct UnwritableRecords;

    impl ForStoringWeighIns for UnwritableRecords {
        fn save(&self, _: &WeighIn) -> Result<(), Broken> {
            Err(Broken("the disk is full".into()))
        }

        fn recent(&self, _: u32) -> Result<Vec<WeighIn>, Broken> {
            Err(Broken("the disk is full".into()))
        }
    }

    fn weighed(date: &str, kg: f64) -> WeighIn {
        weigh_in(date, kg).expect("a valid weigh-in")
    }

    #[test]
    fn a_logged_weigh_in_comes_back_first_among_the_recent_ones() {
        let records = WeighInsInMemory::default();
        log_weigh_in(&records, "2026-10-02", 72.4).expect("a first weigh-in logs");

        let recent = log_weigh_in(&records, "2026-10-03", 72.1);

        assert_eq!(
            recent,
            Ok(vec![
                weighed("2026-10-03", 72.1),
                weighed("2026-10-02", 72.4)
            ])
        );
    }

    #[test]
    fn only_the_last_fourteen_weigh_ins_come_back() {
        let records = WeighInsInMemory::default();
        for day in 1..=20 {
            log_weigh_in(&records, &format!("2026-09-{day:02}"), 72.0).expect("a weigh-in logs");
        }

        let recent = recent_weigh_ins(&records).expect("the recent weigh-ins read back");

        assert_eq!(recent.len(), 14);
        assert_eq!(recent.first(), Some(&weighed("2026-09-20", 72.0)));
    }

    #[test]
    fn a_refused_weigh_in_leaves_the_records_as_they_were() {
        let records = WeighInsInMemory::default();

        let refused = log_weigh_in(&records, "2026-10-03", 0.0);

        assert_eq!(refused, Err(Failure::Refused(Refusal::NotAMass)));
        assert_eq!(recent_weigh_ins(&records), Ok(vec![]));
    }

    #[test]
    fn records_that_cannot_be_written_come_back_broken() {
        assert_eq!(
            log_weigh_in(&UnwritableRecords, "2026-10-03", 72.1),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    #[test]
    fn records_that_cannot_be_read_come_back_broken() {
        assert_eq!(
            recent_weigh_ins(&UnwritableRecords),
            Err(Failure::Broken("the disk is full".into()))
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
}
