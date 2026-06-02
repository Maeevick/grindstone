#![cfg_attr(not(test), expect(dead_code, reason = "the day's quota comes next"))]

fn bmr_by_katch_and_mcardle(age: u32, ffm: f64) -> f64 {
    (370.0 + 21.6 * ffm) * (1.0 - 0.0015 * f64::from(age.saturating_sub(20)))
}

fn organ_overhead_by_elia(age: u32, ffm: f64, fm: f64) -> f64 {
    (1.5 * ffm + 0.5 * fm) * (1.0 - 0.0025 * f64::from(age.saturating_sub(55)))
}

fn tef_by_calcagno_et_al(age: u32, total_energy: f64, protein: f64, fat: f64, ffm: f64) -> f64 {
    let fibre = 0.014 * total_energy;
    let carbs = ((total_energy - 4.0 * protein - 9.0 * fat - 2.0 * fibre) / 4.0)
        .max(3.0 * ffm)
        .max(130.0);
    (protein + 0.32 * carbs + 0.225 * fat + 0.4 * fibre)
        * (1.0 - 0.002 * f64::from(age.saturating_sub(50)))
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

    #[test]
    fn digesting_2500_kcal_with_150_g_of_protein_and_70_g_of_fat_costs_275_75_kcal() {
        assert!(about(
            tef_by_calcagno_et_al(41, 2500.0, 150.0, 70.0, 65.0),
            275.75
        ));
    }

    #[test]
    fn a_day_too_small_for_its_protein_and_fat_still_counts_3_g_of_carbs_per_lean_kilo() {
        assert!(about(
            tef_by_calcagno_et_al(41, 1000.0, 150.0, 70.0, 65.0),
            233.75
        ));
    }

    #[test]
    fn under_43_lean_kilos_the_carbs_counted_never_fall_under_130_g() {
        assert!(about(
            tef_by_calcagno_et_al(41, 1000.0, 150.0, 70.0, 40.0),
            212.95
        ));
    }

    #[test]
    fn every_year_past_fifty_takes_two_tenths_of_a_percent_off_digestion() {
        assert!(about(
            tef_by_calcagno_et_al(50, 2500.0, 150.0, 70.0, 65.0),
            275.75
        ));
        assert!(about(
            tef_by_calcagno_et_al(60, 2500.0, 150.0, 70.0, 65.0),
            275.75 * 0.98
        ));
    }
}
