#![cfg_attr(not(test), expect(dead_code, reason = "the day's quota comes next"))]

fn bmr_by_katch_and_mcardle(age: u32, ffm: f64) -> f64 {
    (370.0 + 21.6 * ffm) * (1.0 - 0.0015 * f64::from(age.saturating_sub(20)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn about(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    #[test]
    fn at_twenty_the_basal_rate_is_katch_and_mcardle_with_nothing_taken_off() {
        assert!(about(bmr_by_katch_and_mcardle(20, 65.0), 1774.0));
    }

    #[test]
    fn every_decade_past_twenty_takes_one_and_a_half_percent_off_the_basal_rate() {
        assert!(about(bmr_by_katch_and_mcardle(30, 65.0), 1774.0 * 0.985));
        assert!(about(bmr_by_katch_and_mcardle(40, 65.0), 1774.0 * 0.97));
    }

    #[test]
    fn the_years_before_twenty_add_nothing_back_to_the_basal_rate() {
        assert!(about(bmr_by_katch_and_mcardle(15, 65.0), 1774.0));
    }
}
