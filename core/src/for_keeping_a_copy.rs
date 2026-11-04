use chrono::NaiveDate;

use crate::failure::Failure;
use crate::for_copying_records::{Counted, ForCopyingRecords, RecordsCopy};
use crate::refusal::Refusal;

/// A copy of every record, dated on the day it is kept.
pub fn keep_a_copy(store: &impl ForCopyingRecords, today: &str) -> Result<RecordsCopy, Failure> {
    let day = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .map_err(|_| Failure::Refused(Refusal::NotADate))?;
    Ok(store.copied(day)?)
}

/// Every record replaced by the copy's, telling how many tables and records came back.
pub fn restore_a_copy(store: &impl ForCopyingRecords, copy: &str) -> Result<Counted, Failure> {
    let _ = (store, copy);
    Err(Failure::Broken(String::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{BrokenCopies, CopiedRecords};

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
                counted: Counted {
                    tables: 18,
                    records: 2431,
                },
            })
        );
    }

    #[test]
    fn a_copy_the_store_cannot_make_is_a_failure() {
        assert_eq!(
            keep_a_copy(&BrokenCopies, "2026-11-04"),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    #[test]
    fn a_copy_dated_on_a_day_the_calendar_does_not_have_is_refused() {
        let store = CopiedRecords {
            tables: 18,
            records: 2431,
        };

        assert_eq!(
            keep_a_copy(&store, "2026-02-30"),
            Err(Failure::Refused(Refusal::NotADate))
        );
    }

    #[test]
    fn restoring_a_copy_tells_how_many_records_and_tables_it_brought_back() {
        let store = CopiedRecords {
            tables: 18,
            records: 3332,
        };

        assert_eq!(
            restore_a_copy(&store, "every record, copied on 2026-11-04"),
            Ok(Counted {
                tables: 18,
                records: 3332,
            })
        );
    }
}
