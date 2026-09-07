use crate::gender::Gender;
use crate::volitional::Volitional;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Deficit {
    pub trailing_deficit_kcal: f64,
    pub weeks_in_deficit: u32,
}

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

/// The day's energy budget before the burn: basal rate, organs, digestion and the day off the clock.
///
/// - Basal rate from lean mass (Katch-McArdle): McArdle W. D., Katch F. I., Katch V. L. (2015),
///   *Exercise physiology: nutrition, energy, and human performance*, 8th edition.
/// - Organs per kilo of lean and fat mass: Elia M. (1992), *Organ and tissue contribution to metabolic rate*,
///   in Kinney J. M., Tucker H. N. (eds), *Energy metabolism: tissue determinants and cellular corollaries*.
/// - Digestion priced on the grams eaten: Calcagno M. et al. (2019), *The thermic effect of food: a review*,
///   J Am Coll Nutr 38(6):547-551.
/// - The day off the clock (sitting, standing, moving, fidgeting): Levine J. A. (2002),
///   *Non-exercise activity thermogenesis (NEAT)*, Best Pract Res Clin Endocrinol Metab 16(4):679-702.
pub fn daily_energy_budget(
    age: u32,
    ffm: f64,
    fm: f64,
    gender: Gender,
    volitional: Volitional,
    deficit: Deficit,
    intake: Intake,
) -> f64 {
    let Macro {
        protein,
        fat,
        fib,
        carb,
    } = intake.macros;
    bmr_by_katch_and_mcardle(age, ffm)
        + organ_overhead_by_elia(age, ffm, fm)
        + tef_by_calcagno_et_al(age, protein, fat, fib, carb)
        + neat_by_mayo_clinic(age, ffm, fm, gender, volitional, deficit)
}

fn bmr_by_katch_and_mcardle(age: u32, ffm: f64) -> f64 {
    (370.0 + 21.6 * ffm.max(0.0)) * (1.0 - 0.0015 * f64::from(age.saturating_sub(20))).max(0.0)
}

fn organ_overhead_by_elia(age: u32, ffm: f64, fm: f64) -> f64 {
    (1.5 * ffm.max(0.0) + 0.5 * fm.max(0.0))
        * (1.0 - 0.0025 * f64::from(age.saturating_sub(55))).max(0.0)
}

fn tef_by_calcagno_et_al(age: u32, protein: f64, fat: f64, fib: f64, carb: f64) -> f64 {
    (protein.max(0.0) + 0.225 * fat.max(0.0) + 0.4 * fib.max(0.0) + 0.32 * carb.max(0.0))
        * (1.0 - 0.002 * f64::from(age.saturating_sub(50))).max(0.0)
}

fn neat_by_mayo_clinic(
    age: u32,
    ffm: f64,
    fm: f64,
    gender: Gender,
    volitional: Volitional,
    deficit: Deficit,
) -> f64 {
    let (ffm, fm) = (ffm.max(0.0), fm.max(0.0));
    let (male_standing_premium, female_walking_premium) = match gender {
        Gender::Male => (0.015, 0.0),
        Gender::Female => (0.0, 0.025),
    };
    let worn_down =
        1.0 - 0.0003 * deficit.trailing_deficit_kcal.max(0.0) * f64::from(deficit.weeks_in_deficit);
    let spontaneous = 0.0025 * ffm * 1440.0 * worn_down.max(0.55);
    let sitting = volitional.sitting_min / 60.0 * (0.66 + 0.048 * ffm + 0.006 * fm);
    let standing = volitional.standing_min
        * (0.022
            + 0.0021 * ffm
            + 0.0004 * fm
            + male_standing_premium
            + 0.0006 * f64::from(age.saturating_sub(55)));
    let walking = (ffm + fm)
        * volitional.moving_km
        * (0.485 + female_walking_premium + 0.0035 * f64::from(age.saturating_sub(60)));
    spontaneous + sitting + standing + walking
}

#[cfg(test)]
mod tests {
    use super::*;

    fn about(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-9
    }

    #[test]
    fn a_mans_budget_adds_up_his_resting_organ_digestion_and_non_exercise_burns() {
        let intake = Intake {
            energy: 2500.0,
            macros: Macro {
                protein: 150.0,
                fat: 70.0,
                fib: 35.0,
                carb: 300.0,
            },
        };

        assert!(about(
            daily_energy_budget(41, 65.0, 10.0, Gender::Male, A_DESK_DAY, NO_DEFICIT, intake),
            1774.0 * 0.9685 + 102.5 + 275.75 + 431.52
        ));
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
    fn a_composition_below_nothing_costs_the_organs_nothing() {
        assert!(about(organ_overhead_by_elia(41, -65.0, -10.0), 0.0));
    }

    #[test]
    fn a_mistyped_age_stops_the_organs_at_nothing_rather_than_below() {
        assert!(about(organ_overhead_by_elia(4100, 65.0, 10.0), 0.0));
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

    const NO_DEFICIT: Deficit = Deficit {
        trailing_deficit_kcal: 0.0,
        weeks_in_deficit: 0,
    };

    const A_DESK_DAY: Volitional = Volitional {
        sitting_min: 480.0,
        standing_min: 120.0,
        moving_km: 4.0,
    };

    const NOTHING_BUT_STANDING: Volitional = Volitional {
        sitting_min: 0.0,
        standing_min: 120.0,
        moving_km: 0.0,
    };

    const NOTHING_BUT_WALKING: Volitional = Volitional {
        sitting_min: 0.0,
        standing_min: 0.0,
        moving_km: 4.0,
    };

    const NOTHING_BUT_FIDGETING: Volitional = Volitional {
        sitting_min: 0.0,
        standing_min: 0.0,
        moving_km: 0.0,
    };

    fn a_mans_neat(age: u32, volitional: Volitional, deficit: Deficit) -> f64 {
        neat_by_mayo_clinic(age, 65.0, 10.0, Gender::Male, volitional, deficit)
    }

    #[test]
    fn eight_hours_sat_two_stood_and_four_kilometres_walked_burn_431_52_kcal_off_the_clock() {
        assert!(about(a_mans_neat(41, A_DESK_DAY, NO_DEFICIT), 431.52));
    }

    #[test]
    fn standing_costs_a_man_more_than_a_woman_of_the_same_build() {
        let woman = neat_by_mayo_clinic(
            41,
            65.0,
            10.0,
            Gender::Female,
            NOTHING_BUT_STANDING,
            NO_DEFICIT,
        );

        assert!(about(
            a_mans_neat(41, NOTHING_BUT_STANDING, NO_DEFICIT) - woman,
            1.8
        ));
    }

    #[test]
    fn walking_costs_a_woman_more_than_a_man_of_the_same_mass() {
        let woman = neat_by_mayo_clinic(
            41,
            65.0,
            10.0,
            Gender::Female,
            NOTHING_BUT_WALKING,
            NO_DEFICIT,
        );

        assert!(about(
            woman - a_mans_neat(41, NOTHING_BUT_WALKING, NO_DEFICIT),
            7.5
        ));
    }

    #[test]
    fn standing_gets_dearer_past_fifty_five_and_walking_past_sixty() {
        assert!(about(
            a_mans_neat(65, NOTHING_BUT_STANDING, NO_DEFICIT)
                - a_mans_neat(55, NOTHING_BUT_STANDING, NO_DEFICIT),
            0.72
        ));
        assert!(about(
            a_mans_neat(70, NOTHING_BUT_WALKING, NO_DEFICIT)
                - a_mans_neat(60, NOTHING_BUT_WALKING, NO_DEFICIT),
            10.5
        ));
    }

    #[test]
    fn a_deficit_held_for_weeks_wears_spontaneous_activity_down_by_its_depth_times_its_length() {
        let worn = |trailing_deficit_kcal, weeks_in_deficit| {
            a_mans_neat(
                41,
                NOTHING_BUT_FIDGETING,
                Deficit {
                    trailing_deficit_kcal,
                    weeks_in_deficit,
                },
            )
        };

        assert!(about(worn(400.0, 3), 234.0 * (1.0 - 0.0003 * 1200.0)));
        assert!(about(worn(400.0, 3), worn(300.0, 4)));
    }

    #[test]
    fn spontaneous_activity_wears_down_to_fifty_five_percent_and_no_further() {
        let worn = Deficit {
            trailing_deficit_kcal: 441.0,
            weeks_in_deficit: 40,
        };

        assert!(about(
            a_mans_neat(41, NOTHING_BUT_FIDGETING, worn),
            234.0 * 0.55
        ));
    }

    #[test]
    fn a_surplus_buys_no_extra_spontaneous_activity() {
        let surplus = Deficit {
            trailing_deficit_kcal: -441.0,
            weeks_in_deficit: 4,
        };

        assert!(about(
            a_mans_neat(41, NOTHING_BUT_FIDGETING, surplus),
            234.0
        ));
    }

    #[test]
    fn a_composition_below_nothing_leaves_only_the_postural_intercepts() {
        let burned = neat_by_mayo_clinic(41, -65.0, -10.0, Gender::Male, A_DESK_DAY, NO_DEFICIT);

        assert!(about(burned, 5.28 + 4.44));
    }
}
