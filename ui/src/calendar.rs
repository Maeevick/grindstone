use chrono::{Datelike, NaiveDate};

pub fn weeks_of<Day: Clone>(days: &[(NaiveDate, Day)]) -> Vec<Vec<Option<Day>>> {
    let mut weeks: Vec<(chrono::IsoWeek, Vec<Option<Day>>)> = Vec::new();
    for (date, day) in days {
        let week = date.iso_week();
        if weeks.last().is_none_or(|(kept, _)| *kept != week) {
            weeks.push((week, vec![None; 7]));
        }
        if let Some((_, cells)) = weeks.last_mut() {
            cells[date.weekday().num_days_from_monday() as usize] = Some(day.clone());
        }
    }
    weeks.into_iter().rev().map(|(_, cells)| cells).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn named(dates: &[&str]) -> Vec<(NaiveDate, String)> {
        dates
            .iter()
            .map(|date| (day(date), date.to_string()))
            .collect()
    }

    #[test]
    fn days_fall_into_weeks_from_monday_to_sunday_the_latest_week_first() {
        let weeks = weeks_of(&named(&[
            "2026-10-02",
            "2026-10-03",
            "2026-10-05",
            "2026-10-06",
        ]));

        assert_eq!(
            weeks,
            vec![
                vec![
                    Some("2026-10-05".to_string()),
                    Some("2026-10-06".to_string()),
                    None,
                    None,
                    None,
                    None,
                    None
                ],
                vec![
                    None,
                    None,
                    None,
                    None,
                    Some("2026-10-02".to_string()),
                    Some("2026-10-03".to_string()),
                    None
                ],
            ]
        );
    }

    #[test]
    fn no_day_is_no_week() {
        assert_eq!(weeks_of::<String>(&[]), Vec::<Vec<Option<String>>>::new());
    }
}
