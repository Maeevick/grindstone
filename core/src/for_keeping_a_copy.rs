use chrono::NaiveDate;

use crate::failure::Failure;
use crate::for_copying_records::{ForCopyingRecords, RecordsCopy};

/// A copy of every record, dated on the day it is kept.
pub fn keep_a_copy(store: &impl ForCopyingRecords, today: &str) -> Result<RecordsCopy, Failure> {
    let day = NaiveDate::parse_from_str(today, "%Y-%m-%d").expect("a real day");
    Ok(store.copied(day).expect("a copy"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::CopiedRecords;

    #[test]
    fn keeping_a_copy_tells_how_many_records_and_tables_it_holds() {
        let store = CopiedRecords {
            tables: 18,
            records: 2431,
        };

        assert_eq!(
            keep_a_copy(&store, "2026-11-04"),
            Ok(RecordsCopy {
                text: "every record, copied on 2026-11-04".into(),
                tables: 18,
                records: 2431,
            })
        );
    }
}
