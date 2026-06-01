#![cfg_attr(not(test), expect(dead_code, reason = "the day's quota comes next"))]

fn bmr_by_katch_and_mcardle(age: u32, ffm: f64) -> f64 {
    (370.0 + 21.6 * ffm) * (1.0 - 0.0015 * f64::from(age.saturating_sub(20)))
}

fn organ_overhead_by_elia(age: u32, ffm: f64, fm: f64) -> f64 {
    (1.5 * ffm + 0.5 * fm) * (1.0 - 0.0025 * f64::from(age.saturating_sub(55)))
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

    #[test]
    fn up_to_fifty_five_the_organs_cost_one_and_a_half_kcal_per_lean_kilo_and_half_a_kcal_per_fat_kilo()
     {
        assert!(about(organ_overhead_by_elia(41, 65.0, 10.0), 102.5));
        assert!(about(organ_overhead_by_elia(55, 65.0, 10.0), 102.5));
    }

    #[test]
    fn every_year_past_fifty_five_takes_a_quarter_of_a_percent_off_the_organs() {
        assert!(about(organ_overhead_by_elia(65, 65.0, 10.0), 102.5 * 0.975));
    }
}
