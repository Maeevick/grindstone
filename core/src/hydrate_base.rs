use crate::gender::Gender;

const DRINKS_PERCENT_OF_TOTAL_WATER: f64 = 80.0;

fn adequate_intake_of_total_water_ml(gender: Option<Gender>) -> f64 {
    match gender {
        Some(Gender::Female) => 2000.0,
        Some(Gender::Male) | None => 2500.0,
    }
}

/// The drinks a day takes before any lean mass is known: four fifths of EFSA's adequate intake of
/// total water, 2.5 L for men and 2.0 L for women, food bringing the rest.
///
/// EFSA Panel on Dietetic Products, Nutrition and Allergies (2010), *Scientific Opinion on Dietary
/// Reference Values for water*, EFSA Journal 8(3):1459.
pub fn default_base_ml(gender: Option<Gender>) -> f64 {
    adequate_intake_of_total_water_ml(gender) * DRINKS_PERCENT_OF_TOTAL_WATER / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_man_drinks_four_fifths_of_his_adequate_intake_of_total_water() {
        assert_eq!(default_base_ml(Some(Gender::Male)), 2000.0);
    }

    #[test]
    fn a_woman_drinks_four_fifths_of_her_adequate_intake_of_total_water() {
        assert_eq!(default_base_ml(Some(Gender::Female)), 1600.0);
    }

    #[test]
    fn with_no_profile_yet_the_default_is_two_litres() {
        assert_eq!(default_base_ml(None), 2000.0);
    }
}
