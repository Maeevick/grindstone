use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::failure::Broken;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_weigh_ins::ForStoringWeighIns;
use crate::gather_records::GatherRecords;
use crate::measurement::Measurement;
use crate::weigh_in::{WeighIn, weigh_in};

#[derive(Default)]
pub struct GatherRecordsInMemory(RefCell<GatherRecords>);

impl ForStoringGatherRecords for GatherRecordsInMemory {
    fn save_measurement(&self, measurement: &Measurement) -> Result<(), Broken> {
        Ok(())
    }

    fn gather_records(&self) -> Result<GatherRecords, Broken> {
        Ok(GatherRecords::default())
    }
}

#[derive(Default)]
pub struct WeighInsInMemory(RefCell<BTreeMap<String, WeighIn>>);

impl ForStoringWeighIns for WeighInsInMemory {
    fn save(&self, weigh_in: &WeighIn) -> Result<(), Broken> {
        self.0
            .borrow_mut()
            .insert(weigh_in.date.clone(), weigh_in.clone());
        Ok(())
    }

    fn recent(&self, count: u32) -> Result<Vec<WeighIn>, Broken> {
        Ok(self
            .0
            .borrow()
            .values()
            .rev()
            .take(count as usize)
            .cloned()
            .collect())
    }
}

pub mod honouring_for_storing_weigh_ins {
    use super::*;

    fn weighed(date: &str, kg: f64) -> WeighIn {
        weigh_in(date, kg).expect("a valid weigh-in")
    }

    pub fn recent_weigh_ins_come_newest_first_and_no_more_than_asked(
        records: &impl ForStoringWeighIns,
    ) {
        for (date, kg) in [
            ("2026-10-01", 72.0),
            ("2026-10-03", 72.1),
            ("2026-10-02", 72.2),
        ] {
            records.save(&weighed(date, kg)).expect("a weigh-in saves");
        }

        assert_eq!(
            records.recent(2),
            Ok(vec![
                weighed("2026-10-03", 72.1),
                weighed("2026-10-02", 72.2)
            ])
        );
    }

    pub fn a_day_weighed_twice_keeps_its_second_weigh_in(records: &impl ForStoringWeighIns) {
        records
            .save(&weighed("2026-10-03", 72.4))
            .expect("a weigh-in saves");

        records
            .save(&weighed("2026-10-03", 72.1))
            .expect("a weigh-in saves");

        assert_eq!(records.recent(7), Ok(vec![weighed("2026-10-03", 72.1)]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recent_weigh_ins_come_newest_first_and_no_more_than_asked() {
        honouring_for_storing_weigh_ins::recent_weigh_ins_come_newest_first_and_no_more_than_asked(
            &WeighInsInMemory::default(),
        );
    }

    #[test]
    fn a_day_weighed_twice_keeps_its_second_weigh_in() {
        honouring_for_storing_weigh_ins::a_day_weighed_twice_keeps_its_second_weigh_in(
            &WeighInsInMemory::default(),
        );
    }
}
