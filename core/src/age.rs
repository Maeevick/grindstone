use chrono::NaiveDate;

pub fn age_on(birthday: NaiveDate, day: NaiveDate) -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    #[test]
    fn a_birthday_adds_a_year_on_the_day_itself() {
        assert_eq!(age_on(day("1985-04-12"), day("2026-04-12")), 41);
    }
}
