use chrono::NaiveDate;

use crate::version::version_in_force_on;

/// Whether Hydrate counts in the streak on a day: as switched on or off from the latest day before
/// it, counting until it is left out.
pub fn counted_on(day: NaiveDate, counted_from: &[(NaiveDate, bool)]) -> bool {
    version_in_force_on(counted_from, day)
        .copied()
        .unwrap_or(true)
}
