use chrono::NaiveDate;

/// Whether Hydrate counts in the streak on a day: as switched on or off from the latest day up to it,
/// counting before any switch and until it is left out.
pub fn counted_on(day: NaiveDate, counted_from: &[(NaiveDate, bool)]) -> bool {
    counted_from
        .iter()
        .filter(|(from, _)| *from <= day)
        .max_by_key(|(from, _)| *from)
        .is_none_or(|(_, counted)| *counted)
}
