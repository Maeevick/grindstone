use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ticks {
    pub weighed: Option<bool>,
    pub fuelled: Option<bool>,
    pub worked_out: Option<bool>,
    pub hydrated: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Success,
    Joker,
    Break,
    Pending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreakDay {
    pub day: NaiveDate,
    pub ticks: Ticks,
    pub status: Status,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Streak {
    pub days: u32,
    pub at_risk: bool,
    pub recent: Vec<StreakDay>,
}

pub fn streak_of(ticked: &[(NaiveDate, Ticks)], today: NaiveDate) -> Streak {
    let _ = (ticked, today);
    Streak {
        days: 0,
        at_risk: false,
        recent: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Days;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn four(weighed: bool, fuelled: bool, worked_out: bool, hydrated: bool) -> Ticks {
        Ticks {
            weighed: Some(weighed),
            fuelled: Some(fuelled),
            worked_out: Some(worked_out),
            hydrated: Some(hydrated),
        }
    }

    fn three(weighed: bool, fuelled: bool, worked_out: bool) -> Ticks {
        Ticks {
            hydrated: None,
            ..four(weighed, fuelled, worked_out, false)
        }
    }

    const ALL: Ticks = Ticks {
        weighed: Some(true),
        fuelled: Some(true),
        worked_out: Some(true),
        hydrated: Some(true),
    };

    fn days_ending(today: &str, ticks: &[Ticks]) -> Vec<(NaiveDate, Ticks)> {
        let first = day(today) - Days::new(ticks.len() as u64 - 1);
        ticks
            .iter()
            .enumerate()
            .map(|(at, ticked)| (first + Days::new(at as u64), *ticked))
            .collect()
    }

    #[test]
    fn days_with_every_tick_in_a_row_make_the_streak_and_an_unfinished_today_does_not_break_it() {
        let ticked = days_ending(
            "2026-10-10",
            &[ALL, ALL, ALL, four(true, false, false, false)],
        );

        let streak = streak_of(&ticked, day("2026-10-10"));

        assert_eq!(streak.days, 3);
        assert_eq!(
            streak.recent.last().map(|today| today.status),
            Some(Status::Pending)
        );
    }

    #[test]
    fn today_counts_as_soon_as_every_tick_holds() {
        let streak = streak_of(&days_ending("2026-10-10", &[ALL, ALL]), day("2026-10-10"));

        assert_eq!(streak.days, 2);
    }

    #[test]
    fn a_day_one_tick_short_between_two_successes_is_a_joker_and_keeps_the_streak() {
        let ticked = days_ending(
            "2026-10-10",
            &[ALL, four(true, true, false, true), ALL, ALL],
        );

        let streak = streak_of(&ticked, day("2026-10-10"));

        assert_eq!(streak.days, 4);
        assert_eq!(
            streak.recent.get(1).map(|joker| joker.status),
            Some(Status::Joker)
        );
    }

    #[test]
    fn two_ticks_short_or_two_jokers_in_a_row_break_the_streak() {
        let two_short = days_ending("2026-10-10", &[ALL, four(true, true, false, false), ALL]);
        let two_jokers = days_ending(
            "2026-10-10",
            &[
                ALL,
                four(true, true, false, true),
                four(false, true, true, true),
                ALL,
            ],
        );

        assert_eq!(streak_of(&two_short, day("2026-10-10")).days, 1);
        assert_eq!(streak_of(&two_jokers, day("2026-10-10")).days, 1);
    }

    #[test]
    fn yesterday_one_tick_short_counts_while_today_is_pending_but_puts_the_streak_at_risk() {
        let ticked = days_ending(
            "2026-10-10",
            &[
                ALL,
                four(true, false, true, true),
                four(false, false, false, false),
            ],
        );

        let streak = streak_of(&ticked, day("2026-10-10"));

        assert_eq!((streak.days, streak.at_risk), (2, true));
    }

    #[test]
    fn days_before_hydrate_count_on_three_ticks_with_a_joker_at_two() {
        let ticked = days_ending(
            "2026-10-10",
            &[
                three(true, true, true),
                three(true, false, true),
                three(true, true, true),
                ALL,
            ],
        );

        assert_eq!(streak_of(&ticked, day("2026-10-10")).days, 4);
    }

    #[test]
    fn days_of_two_ticks_or_fewer_have_no_joker() {
        let two = |weighed, fuelled| Ticks {
            weighed: Some(weighed),
            fuelled: Some(fuelled),
            ..Ticks::default()
        };
        let ticked = days_ending(
            "2026-10-10",
            &[two(true, true), two(true, false), two(true, true)],
        );

        assert_eq!(streak_of(&ticked, day("2026-10-10")).days, 1);
    }

    #[test]
    fn the_recent_days_are_the_last_fourteen_up_to_today() {
        let ticked = days_ending("2026-10-10", &[ALL; 20]);

        let streak = streak_of(&ticked, day("2026-10-10"));

        assert_eq!(streak.days, 20);
        assert_eq!(streak.recent.len(), 14);
        assert_eq!(
            streak.recent.last().map(|today| today.day),
            Some(day("2026-10-10"))
        );
    }
}
