pub fn activity_induced_energy_expenditure_from_compendium(
    bodymass: f64,
    met: f64,
    duration: f64,
    weight: f64,
) -> f64 {
    (met - 1.0).max(0.0) * 3.5 * (bodymass + weight).max(0.0) / 200.0 * duration.max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn about(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    #[test]
    fn an_hour_at_two_mets_burns_one_met_above_rest_about_a_kcal_per_kilo() {
        assert!(about(
            activity_induced_energy_expenditure_from_compendium(70.0, 2.0, 60.0, 0.0),
            73.5
        ));
    }

    #[test]
    fn an_hour_at_rest_burns_nothing_since_the_day_already_pays_for_it() {
        assert!(about(
            activity_induced_energy_expenditure_from_compendium(70.0, 1.0, 60.0, 0.0),
            0.0
        ));
    }

    #[test]
    fn a_load_adds_its_kilos_to_the_working_share_and_a_band_takes_them_off() {
        let unloaded = activity_induced_energy_expenditure_from_compendium(75.0, 7.5, 10.0, 0.0);
        let loaded = activity_induced_energy_expenditure_from_compendium(75.0, 7.5, 10.0, 10.0);
        let helped = activity_induced_energy_expenditure_from_compendium(75.0, 7.5, 10.0, -10.0);
        let ten_kilos_of_working_share = 6.5 * 3.5 * 10.0 / 200.0 * 10.0;

        assert!(about(loaded - unloaded, ten_kilos_of_working_share));
        assert!(about(unloaded - helped, ten_kilos_of_working_share));
    }

    #[test]
    fn a_band_helping_more_than_the_body_weighs_burns_nothing_rather_than_below() {
        assert!(about(
            activity_induced_energy_expenditure_from_compendium(75.0, 7.5, 10.0, -200.0),
            0.0
        ));
    }

    #[test]
    fn an_activity_below_rest_burns_nothing_rather_than_below() {
        assert!(about(
            activity_induced_energy_expenditure_from_compendium(75.0, 0.5, 30.0, 0.0),
            0.0
        ));
    }

    #[test]
    fn a_negative_duration_burns_nothing_rather_than_below() {
        assert!(about(
            activity_induced_energy_expenditure_from_compendium(75.0, 7.5, -30.0, 0.0),
            0.0
        ));
    }
}
