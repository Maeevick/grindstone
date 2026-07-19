use serde::{Deserialize, Serialize};

use crate::refusal::Refusal;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Goals {
    pub body_mass_kg: f64,
    pub fat_ratio: f64,
}

pub fn goals(body_mass_kg: f64, fat_percentage: f64) -> Result<Goals, Refusal> {
    if !(body_mass_kg.is_finite() && body_mass_kg > 0.0) {
        return Err(Refusal::NotAMass);
    }
    if !(fat_percentage > 0.0 && fat_percentage < 100.0) {
        return Err(Refusal::NotARatio);
    }
    Ok(Goals {
        body_mass_kg,
        fat_ratio: fat_percentage / 100.0,
    })
}

pub fn fat_percentage(goals: &Goals) -> f64 {
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fat_target_typed_in_percent_is_kept_as_a_ratio() {
        assert_eq!(
            goals(70.0, 10.0),
            Ok(Goals {
                body_mass_kg: 70.0,
                fat_ratio: 0.10,
            })
        );
    }

    #[test]
    fn a_target_mass_of_nothing_or_no_number_at_all_is_refused() {
        for kg in [0.0, -70.0, f64::NAN, f64::INFINITY] {
            assert_eq!(goals(kg, 10.0), Err(Refusal::NotAMass), "{kg}");
        }
    }

    #[test]
    fn a_fat_target_no_body_could_hold_is_refused() {
        for percent in [0.0, 100.0, -5.0, 150.0, f64::NAN] {
            assert_eq!(goals(70.0, percent), Err(Refusal::NotARatio), "{percent}");
        }
    }

    #[test]
    fn the_fat_target_reads_back_in_the_percent_it_was_typed_in() {
        let kept = goals(70.0, 12.0).expect("goals kept");

        assert_eq!(fat_percentage(&kept), 12.0);
    }
}
