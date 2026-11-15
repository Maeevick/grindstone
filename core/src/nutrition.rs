use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Macro {
    pub protein: f64,
    pub fat: f64,
    pub fib: f64,
    pub carb: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Intake {
    pub energy: f64,
    pub macros: Macro,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    pub resting_kcal: f64,
    pub digestion_kcal: f64,
}

impl Budget {
    pub fn kcal(&self) -> f64 {
        self.resting_kcal + self.digestion_kcal
    }
}

/// The day's energy budget before the burn: basal rate and digestion. Sitting, standing, walking and fidgeting
/// (NEAT) are left out, the user's choice: they only deepen the deficit below the band.
///
/// - Basal rate from lean mass (Katch-McArdle), the organs' cost inside it [10].
/// - Digestion priced on the grams eaten [11].
pub fn daily_energy_budget(age: u32, ffm: f64, intake: Intake) -> Budget {
    let Macro {
        protein,
        fat,
        fib,
        carb,
    } = intake.macros;
    Budget {
        resting_kcal: bmr_by_katch_and_mcardle(age, ffm),
        digestion_kcal: tef_by_calcagno_et_al(age, protein, fat, fib, carb),
    }
}

fn bmr_by_katch_and_mcardle(age: u32, ffm: f64) -> f64 {
    (370.0 + 21.6 * ffm.max(0.0)) * (1.0 - 0.0015 * f64::from(age.saturating_sub(20))).max(0.0)
}

fn tef_by_calcagno_et_al(age: u32, protein: f64, fat: f64, fib: f64, carb: f64) -> f64 {
    (protein.max(0.0) + 0.225 * fat.max(0.0) + 0.4 * fib.max(0.0) + 0.32 * carb.max(0.0))
        * (1.0 - 0.002 * f64::from(age.saturating_sub(50))).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn about(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    #[test]
    fn a_days_budget_is_its_resting_rate_and_digestion_alone() {
        let intake = Intake {
            energy: 2500.0,
            macros: Macro {
                protein: 150.0,
                fat: 70.0,
                fib: 35.0,
                carb: 300.0,
            },
        };

        let budget = daily_energy_budget(41, 65.0, intake);

        assert!(about(budget.kcal(), 1774.0 * 0.9685 + 275.75));
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
    fn a_lean_mass_below_nothing_leaves_only_the_intercept() {
        assert!(about(bmr_by_katch_and_mcardle(20, -10.0), 370.0));
    }

    #[test]
    fn a_mistyped_age_stops_the_basal_rate_at_nothing_rather_than_below() {
        assert!(about(bmr_by_katch_and_mcardle(4100, 65.0), 0.0));
    }

    #[test]
    fn digesting_150_g_of_protein_70_of_fat_35_of_fibre_and_300_of_carbs_costs_275_75_kcal() {
        assert!(about(
            tef_by_calcagno_et_al(41, 150.0, 70.0, 35.0, 300.0),
            275.75
        ));
    }

    #[test]
    fn every_year_past_fifty_takes_two_tenths_of_a_percent_off_digestion() {
        assert!(about(
            tef_by_calcagno_et_al(50, 150.0, 70.0, 35.0, 300.0),
            275.75
        ));
        assert!(about(
            tef_by_calcagno_et_al(60, 150.0, 70.0, 35.0, 300.0),
            275.75 * 0.98
        ));
    }

    #[test]
    fn grams_below_nothing_cost_nothing_to_digest() {
        assert!(about(
            tef_by_calcagno_et_al(41, -150.0, -70.0, -35.0, -300.0),
            0.0
        ));
    }

    #[test]
    fn a_mistyped_age_stops_digestion_at_nothing_rather_than_below() {
        assert!(about(
            tef_by_calcagno_et_al(4100, 150.0, 70.0, 35.0, 300.0),
            0.0
        ));
    }

    #[test]
    fn the_budget_tells_apart_its_resting_and_digestion_parts() {
        let intake = Intake {
            energy: 2500.0,
            macros: Macro {
                protein: 150.0,
                fat: 70.0,
                fib: 35.0,
                carb: 300.0,
            },
        };

        let budget = daily_energy_budget(41, 65.0, intake);

        assert!(about(budget.resting_kcal, 1774.0 * 0.9685));
        assert!(about(budget.digestion_kcal, 275.75));
    }
}
