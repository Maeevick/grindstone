use crate::failure::Failure;
use crate::for_storing_weigh_ins::ForStoringWeighIns;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::failure::Broken;
    use crate::testing::WeighInsInMemory;
    use crate::weigh_in::Refusal;

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
}
