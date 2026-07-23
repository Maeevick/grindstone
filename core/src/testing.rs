use std::cell::RefCell;

use crate::failure::Broken;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::gather_records::GatherRecords;
use crate::goals::Goals;
use crate::measurement::Measurement;
use crate::profile::Profile;
use chrono::NaiveDate;

#[derive(Default)]
pub struct GatherRecordsInMemory(RefCell<GatherRecords>);

impl ForStoringGatherRecords for GatherRecordsInMemory {
    fn save_measurement(&self, measurement: &Measurement) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records
            .measurements
            .retain(|kept| kept.day != measurement.day);
        records.measurements.push(*measurement);
        Ok(())
    }

    fn remove_measurement(&self, day: NaiveDate) -> Result<(), Broken> {
        self.0
            .borrow_mut()
            .measurements
            .retain(|kept| kept.day != day);
        Ok(())
    }

    fn save_profile(&self, from: NaiveDate, profile: &Profile) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.profiles.retain(|(kept_from, _)| *kept_from != from);
        records.profiles.push((from, *profile));
        Ok(())
    }

    fn save_goals(&self, from: NaiveDate, goals: &Goals) -> Result<(), Broken> {
        let mut records = self.0.borrow_mut();
        records.goals.retain(|(kept_from, _)| *kept_from != from);
        records.goals.push((from, *goals));
        Ok(())
    }

    fn replace_gather_records(&self, records: &GatherRecords) -> Result<(), Broken> {
        *self.0.borrow_mut() = records.clone();
        Ok(())
    }

    fn gather_records(&self) -> Result<GatherRecords, Broken> {
        let mut gathered = self.0.borrow().clone();
        gathered.measurements.sort_by_key(|measured| measured.day);
        gathered.profiles.sort_by_key(|(from, _)| *from);
        gathered.goals.sort_by_key(|(from, _)| *from);
        Ok(gathered)
    }
}

pub struct BrokenGatherRecords;

impl ForStoringGatherRecords for BrokenGatherRecords {
    fn save_measurement(&self, _: &Measurement) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn remove_measurement(&self, _: NaiveDate) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_profile(&self, _: NaiveDate, _: &Profile) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn save_goals(&self, _: NaiveDate, _: &Goals) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn replace_gather_records(&self, _: &GatherRecords) -> Result<(), Broken> {
        Err(Broken("the disk is full".into()))
    }

    fn gather_records(&self) -> Result<GatherRecords, Broken> {
        Err(Broken("the disk is full".into()))
    }
}

pub mod honouring_for_storing_gather_records {
    use super::*;
    use crate::gender::Gender;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn measured(date: &str, body_mass_kg: f64, waist_cm: Option<f64>) -> Measurement {
        Measurement {
            day: day(date),
            body_mass_kg,
            waist_cm,
        }
    }

    fn a_man(neck_cm: f64) -> Profile {
        Profile {
            birthday: day("1985-04-12"),
            gender: Gender::Male,
            height_cm: 180.0,
            neck_cm,
            hip_cm: None,
        }
    }

    fn targeting(body_mass_kg: f64) -> Goals {
        Goals {
            body_mass_kg,
            fat_ratio: 0.10,
        }
    }

    pub fn gather_records_come_back_in_the_order_of_their_days(
        records: &impl ForStoringGatherRecords,
    ) {
        for measurement in [
            measured("2026-10-10", 80.0, None),
            measured("2026-10-08", 80.4, Some(84.0)),
            measured("2026-10-09", 80.2, None),
        ] {
            records
                .save_measurement(&measurement)
                .expect("a measurement saves");
        }
        records
            .save_profile(day("2026-09-01"), &a_man(38.0))
            .expect("a profile saves");
        records
            .save_profile(day("2026-03-01"), &a_man(38.5))
            .expect("a profile saves");
        records
            .save_goals(day("2026-09-01"), &targeting(70.0))
            .expect("goals save");
        records
            .save_goals(day("2026-03-01"), &targeting(72.0))
            .expect("goals save");

        assert_eq!(
            records.gather_records(),
            Ok(GatherRecords {
                measurements: vec![
                    measured("2026-10-08", 80.4, Some(84.0)),
                    measured("2026-10-09", 80.2, None),
                    measured("2026-10-10", 80.0, None),
                ],
                profiles: vec![
                    (day("2026-03-01"), a_man(38.5)),
                    (day("2026-09-01"), a_man(38.0))
                ],
                goals: vec![
                    (day("2026-03-01"), targeting(72.0)),
                    (day("2026-09-01"), targeting(70.0))
                ],
            })
        );
    }

    pub fn a_day_measured_twice_keeps_its_second_measurement(
        records: &impl ForStoringGatherRecords,
    ) {
        records
            .save_measurement(&measured("2026-10-10", 80.4, None))
            .expect("a measurement saves");

        records
            .save_measurement(&measured("2026-10-10", 80.0, Some(84.0)))
            .expect("a measurement saves");

        assert_eq!(
            records
                .gather_records()
                .map(|gathered| gathered.measurements),
            Ok(vec![measured("2026-10-10", 80.0, Some(84.0))])
        );
    }

    pub fn removing_a_measured_day_leaves_the_others(records: &impl ForStoringGatherRecords) {
        records
            .save_measurement(&measured("2026-10-09", 80.2, None))
            .expect("a measurement saves");
        records
            .save_measurement(&measured("2026-10-10", 80.0, None))
            .expect("a measurement saves");

        records
            .remove_measurement(day("2026-10-10"))
            .expect("a day is removed");

        assert_eq!(
            records
                .gather_records()
                .map(|gathered| gathered.measurements),
            Ok(vec![measured("2026-10-09", 80.2, None)])
        );
    }

    pub fn a_profile_or_goals_saved_again_from_the_same_day_replace_the_earlier_ones(
        records: &impl ForStoringGatherRecords,
    ) {
        records
            .save_profile(day("2026-03-01"), &a_man(38.5))
            .expect("a profile saves");
        records
            .save_goals(day("2026-03-01"), &targeting(72.0))
            .expect("goals save");

        records
            .save_profile(day("2026-03-01"), &a_man(38.0))
            .expect("a profile saves");
        records
            .save_goals(day("2026-03-01"), &targeting(70.0))
            .expect("goals save");

        let gathered = records.gather_records().expect("the records read");
        assert_eq!(gathered.profiles, vec![(day("2026-03-01"), a_man(38.0))]);
        assert_eq!(gathered.goals, vec![(day("2026-03-01"), targeting(70.0))]);
    }

    pub fn replacing_the_gather_records_leaves_only_the_new_ones(
        records: &impl ForStoringGatherRecords,
    ) {
        records
            .save_measurement(&measured("2026-10-10", 80.0, None))
            .expect("a measurement saves");
        records
            .save_profile(day("2026-03-01"), &a_man(38.5))
            .expect("a profile saves");
        let replacement = GatherRecords {
            measurements: vec![measured("2026-06-22", 76.4, Some(84.0))],
            profiles: vec![(day("2026-06-01"), a_man(38.0))],
            goals: vec![(day("2026-06-01"), targeting(70.0))],
        };

        records
            .replace_gather_records(&replacement)
            .expect("the records are replaced");

        assert_eq!(records.gather_records(), Ok(replacement));
    }

    pub fn a_womans_profile_comes_back_with_her_hip(records: &impl ForStoringGatherRecords) {
        let a_woman = Profile {
            birthday: day("1990-06-01"),
            gender: Gender::Female,
            height_cm: 165.0,
            neck_cm: 33.0,
            hip_cm: Some(98.0),
        };

        records
            .save_profile(day("2026-03-01"), &a_woman)
            .expect("a profile saves");

        assert_eq!(
            records.gather_records().map(|gathered| gathered.profiles),
            Ok(vec![(day("2026-03-01"), a_woman)])
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gather_records_come_back_in_the_order_of_their_days() {
        honouring_for_storing_gather_records::gather_records_come_back_in_the_order_of_their_days(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn a_day_measured_twice_keeps_its_second_measurement() {
        honouring_for_storing_gather_records::a_day_measured_twice_keeps_its_second_measurement(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn removing_a_measured_day_leaves_the_others() {
        honouring_for_storing_gather_records::removing_a_measured_day_leaves_the_others(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn a_profile_or_goals_saved_again_from_the_same_day_replace_the_earlier_ones() {
        honouring_for_storing_gather_records::a_profile_or_goals_saved_again_from_the_same_day_replace_the_earlier_ones(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn replacing_the_gather_records_leaves_only_the_new_ones() {
        honouring_for_storing_gather_records::replacing_the_gather_records_leaves_only_the_new_ones(
            &GatherRecordsInMemory::default(),
        );
    }

    #[test]
    fn a_womans_profile_comes_back_with_her_hip() {
        honouring_for_storing_gather_records::a_womans_profile_comes_back_with_her_hip(
            &GatherRecordsInMemory::default(),
        );
    }
}
