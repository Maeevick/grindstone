#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tapes {
    Male {
        neck_cm: f64,
        waist_cm: f64,
    },
    Female {
        neck_cm: f64,
        waist_cm: f64,
        hip_cm: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Composition {
    pub fat_ratio: f64,
    pub fat_mass_kg: f64,
    pub lean_mass_kg: f64,
}

pub fn body_composition(body_mass_kg: f64, height_cm: f64, tapes: Tapes) -> Option<Composition> {
    fat_ratio_from_navy_formula(height_cm, tapes).map(|fat_ratio| Composition {
        fat_ratio,
        fat_mass_kg: body_mass_kg * fat_ratio,
        lean_mass_kg: body_mass_kg * (1.0 - fat_ratio),
    })
}

fn fat_ratio_from_navy_formula(height_cm: f64, tapes: Tapes) -> Option<f64> {
    let fat_percentage = match tapes {
        Tapes::Male { neck_cm, waist_cm } => {
            495.0 / (1.0324 - 0.19077 * (waist_cm - neck_cm).log10() + 0.15456 * height_cm.log10())
                - 450.0
        }
        Tapes::Female {
            neck_cm,
            waist_cm,
            hip_cm,
        } => {
            495.0
                / (1.29579 - 0.35004 * (waist_cm + hip_cm - neck_cm).log10()
                    + 0.22100 * height_cm.log10())
                - 450.0
        }
    };
    let fat_ratio = fat_percentage / 100.0;
    (fat_ratio > 0.0 && fat_ratio < 1.0).then_some(fat_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    const A_MANS_TAPES: Tapes = Tapes::Male {
        neck_cm: 38.5,
        waist_cm: 80.0,
    };

    fn fat_ratio_of(height_cm: f64, tapes: Tapes) -> Option<f64> {
        body_composition(80.0, height_cm, tapes).map(|split| split.fat_ratio)
    }

    fn reads_about(reading: Option<f64>, expected: f64) -> bool {
        reading.is_some_and(|ratio| (ratio - expected).abs() < 1e-6)
    }

    #[test]
    fn a_body_splits_into_the_fat_its_navy_ratio_reads_and_the_lean_rest() {
        let split = body_composition(80.0, 180.0, A_MANS_TAPES).expect("a reading");

        assert!((split.fat_mass_kg - 80.0 * split.fat_ratio).abs() < 1e-9);
        assert!((split.fat_mass_kg + split.lean_mass_kg - 80.0).abs() < 1e-9);
    }

    #[test]
    fn a_mass_of_nothing_or_no_number_at_all_gives_no_composition() {
        for kg in [0.0, -80.0, f64::NAN, f64::INFINITY] {
            assert_eq!(body_composition(kg, 180.0, A_MANS_TAPES), None, "{kg}");
        }
    }

    #[test]
    fn a_mans_fat_ratio_reads_his_waist_against_his_neck_and_his_height() {
        assert!(reads_about(fat_ratio_of(180.0, A_MANS_TAPES), 0.116245));
    }

    #[test]
    fn a_womans_fat_ratio_adds_her_hip_to_her_waist() {
        let tapes = Tapes::Female {
            neck_cm: 33.0,
            waist_cm: 75.0,
            hip_cm: 98.0,
        };

        assert!(reads_about(fat_ratio_of(165.0, tapes), 0.284350));
    }

    #[test]
    fn a_waist_no_wider_than_the_neck_gives_no_reading() {
        let tapes = Tapes::Male {
            neck_cm: 40.0,
            waist_cm: 40.0,
        };

        assert_eq!(fat_ratio_of(180.0, tapes), None);
    }

    #[test]
    fn a_height_of_nothing_gives_no_reading() {
        let tapes = Tapes::Male {
            neck_cm: 39.0,
            waist_cm: 85.0,
        };

        assert_eq!(fat_ratio_of(0.0, tapes), None);
    }

    #[test]
    fn a_ratio_no_body_could_have_gives_no_reading() {
        let tapes = Tapes::Male {
            neck_cm: 40.0,
            waist_cm: 41.0,
        };

        assert_eq!(fat_ratio_of(180.0, tapes), None);
    }
}
