use crate::tendency::Heading;

/// A measure moving less than 0.15% of itself a week holds steady.
pub(crate) const STABLE_SHARE_A_WEEK: f64 = 0.0015;

/// What a measure aims at: a value to reach, or a floor to stay above.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Aim {
    Reach(f64),
    Floor(f64),
}

/// Where a measure moving so much a week heads: steady under 0.15% of itself, else toward its aim or away from
/// it, unaimed without one. A floor turns a measure away only once it falls to it: rising heads toward the
/// aim, and falling stays above the floor until then.
pub fn heading_of(per_week: f64, now: f64, aim: Option<Aim>) -> Heading {
    match aim {
        None => Heading::Unaimed,
        Some(_) if per_week.abs() < STABLE_SHARE_A_WEEK * now.abs() => Heading::Steady,
        Some(Aim::Reach(aim)) if (aim - now).signum() == per_week.signum() => Heading::Toward,
        Some(Aim::Reach(_)) => Heading::Away,
        Some(Aim::Floor(_)) if per_week > 0.0 => Heading::Toward,
        Some(Aim::Floor(floor)) if now > floor => Heading::AboveItsFloor,
        Some(Aim::Floor(_)) => Heading::Away,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_measure_moving_less_than_015_percent_of_itself_a_week_holds_steady_and_from_it_on_moves() {
        assert_eq!(
            [
                heading_of(0.1, 80.0, Some(Aim::Reach(70.0))),
                heading_of(STABLE_SHARE_A_WEEK * 80.0, 80.0, Some(Aim::Reach(70.0))),
                heading_of(-0.1, -80.0, Some(Aim::Reach(-70.0))),
            ],
            [Heading::Steady, Heading::Away, Heading::Steady]
        );
    }

    #[test]
    fn a_measure_moving_to_its_aim_heads_toward_it_and_from_it_away() {
        assert_eq!(
            [
                heading_of(-0.5, 80.0, Some(Aim::Reach(70.0))),
                heading_of(0.5, 80.0, Some(Aim::Reach(70.0))),
                heading_of(0.5, 60.0, Some(Aim::Reach(70.0))),
            ],
            [Heading::Toward, Heading::Away, Heading::Toward]
        );
    }

    #[test]
    fn a_measure_above_its_floor_rising_heads_toward_its_aim_falling_stays_above_it_until_it_reaches_it()
     {
        let floor = Some(Aim::Floor(61.6));

        assert_eq!(
            [
                heading_of(0.3, 63.0, floor),
                heading_of(0.3, 60.0, floor),
                heading_of(-0.3, 63.0, floor),
                heading_of(-0.3, 61.6, floor),
                heading_of(-0.3, 60.0, floor),
                heading_of(0.05, 63.0, floor),
            ],
            [
                Heading::Toward,
                Heading::Toward,
                Heading::AboveItsFloor,
                Heading::Away,
                Heading::Away,
                Heading::Steady
            ]
        );
    }

    #[test]
    fn a_measure_without_an_aim_is_unaimed_however_it_moves() {
        assert_eq!(heading_of(-0.5, 80.0, None), Heading::Unaimed);
    }
}
