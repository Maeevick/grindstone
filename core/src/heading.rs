use crate::tendency::Heading;

/// A measure moving less than 0.15% of itself a week holds steady.
pub(crate) const STABLE_SHARE_A_WEEK: f64 = 0.0015;

/// Where a measure moving so much a week heads: steady under 0.15% of itself, else toward its aim or away from
/// it, unaimed without one.
pub fn heading_of(per_week: f64, now: f64, aim: Option<f64>) -> Heading {
    match aim {
        None => Heading::Unaimed,
        Some(_) if per_week.abs() < STABLE_SHARE_A_WEEK * now.abs() => Heading::Steady,
        Some(aim) if (aim - now).signum() == per_week.signum() => Heading::Toward,
        Some(_) => Heading::Away,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_measure_moving_less_than_015_percent_of_itself_a_week_holds_steady_and_from_it_on_moves() {
        assert_eq!(
            [
                heading_of(0.1, 80.0, Some(70.0)),
                heading_of(STABLE_SHARE_A_WEEK * 80.0, 80.0, Some(70.0)),
                heading_of(-0.1, -80.0, Some(-70.0)),
            ],
            [Heading::Steady, Heading::Away, Heading::Steady]
        );
    }

    #[test]
    fn a_measure_moving_to_its_aim_heads_toward_it_and_from_it_away() {
        assert_eq!(
            [
                heading_of(-0.5, 80.0, Some(70.0)),
                heading_of(0.5, 80.0, Some(70.0)),
                heading_of(0.5, 60.0, Some(70.0)),
            ],
            [Heading::Toward, Heading::Away, Heading::Toward]
        );
    }

    #[test]
    fn a_measure_without_an_aim_is_unaimed_however_it_moves() {
        assert_eq!(heading_of(-0.5, 80.0, None), Heading::Unaimed);
    }
}
