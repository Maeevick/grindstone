#![cfg_attr(not(test), expect(dead_code, reason = "the day's quota comes next"))]

use crate::gender::Gender;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Volitional {
    sitting_min: f64,
    standing_min: f64,
    moving_km: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Deficit {
    trailing_deficit_kcal: f64,
    weeks_in_deficit: u32,
}

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

fn neat_by_mayo_clinic(
    age: u32,
    ffm: f64,
    fm: f64,
    gender: Gender,
    volitional: Volitional,
    deficit: Deficit,
) -> f64 {
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
}
