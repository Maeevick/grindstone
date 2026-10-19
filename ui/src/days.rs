use chrono::NaiveDate;

/// The days a tab's last days show, from the first one up to the tab's day.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shown {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl Shown {
    /// At first: the week up to the day, the day included.
    pub fn a_week_up_to(day: NaiveDate) -> Self {
        Self { from: day, to: day }
    }

    /// One more load: back to the first of the month shown first, then to the first of the latest
    /// month with records before it; nothing further once none is left.
    pub fn further_back(self, _earlier: Option<&str>) -> Option<Self> {
        None
    }

    /// The months the days shown run across, newest first, named as the app names them.
    pub fn months(self) -> Vec<String> {
        Vec::new()
    }

    pub fn holds(self, _day: NaiveDate) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn shown(from: &str, to: &str) -> Shown {
        Shown {
            from: day(from),
            to: day(to),
        }
    }

    #[test]
    fn the_last_days_show_the_week_up_to_the_day_the_day_included() {
        let week = Shown::a_week_up_to(day("2026-10-03"));

        assert_eq!(week, shown("2026-09-27", "2026-10-03"));
        assert_eq!(
            ["2026-09-26", "2026-09-27", "2026-10-03", "2026-10-04"]
                .map(|date| week.holds(day(date))),
            [false, true, true, false]
        );
    }

    #[test]
    fn loading_more_reaches_the_first_of_the_month_then_the_first_of_the_month_before_with_records()
    {
        let week = shown("2026-10-04", "2026-10-10");

        let october = week.further_back(Some("2026-08"));
        let august = october.and_then(|month| month.further_back(Some("2026-08")));

        assert_eq!(october, Some(shown("2026-10-01", "2026-10-10")));
        assert_eq!(august, Some(shown("2026-08-01", "2026-10-10")));
        assert_eq!(shown("2026-08-01", "2026-10-10").further_back(None), None);
    }

    #[test]
    fn the_days_shown_run_across_their_months_newest_first() {
        assert_eq!(
            shown("2026-08-01", "2026-10-03").months(),
            ["2026-10", "2026-09", "2026-08"]
        );
        assert_eq!(shown("2026-10-04", "2026-10-10").months(), ["2026-10"]);
    }
}
