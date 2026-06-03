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

pub fn fat_ratio_from_navy_formula(height_cm: f64, tapes: Tapes) -> Option<f64> {
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

    fn reads_about(reading: Option<f64>, expected: f64) -> bool {
        reading.is_some_and(|ratio| (ratio - expected).abs() < 1e-6)
    }

    #[test]
    fn a_mans_fat_ratio_reads_his_waist_against_his_neck_and_his_height() {
        let tapes = Tapes::Male {
            neck_cm: 38.5,
            waist_cm: 80.0,
        };

        assert!(reads_about(
            fat_ratio_from_navy_formula(180.0, tapes),
            0.116245
        ));
    }

    #[test]
    fn a_womans_fat_ratio_adds_her_hip_to_her_waist() {
        let tapes = Tapes::Female {
            neck_cm: 33.0,
            waist_cm: 75.0,
            hip_cm: 98.0,
        };

        assert!(reads_about(
            fat_ratio_from_navy_formula(165.0, tapes),
            0.284350
        ));
    }

    #[test]
    fn a_waist_no_wider_than_the_neck_gives_no_reading() {
        let tapes = Tapes::Male {
            neck_cm: 40.0,
            waist_cm: 40.0,
        };

        assert_eq!(fat_ratio_from_navy_formula(180.0, tapes), None);
    }

    #[test]
    fn a_height_of_nothing_gives_no_reading() {
        let tapes = Tapes::Male {
            neck_cm: 39.0,
            waist_cm: 85.0,
        };

        assert_eq!(fat_ratio_from_navy_formula(0.0, tapes), None);
    }

    #[test]
    fn a_ratio_no_body_could_have_gives_no_reading() {
        let tapes = Tapes::Male {
            neck_cm: 40.0,
            waist_cm: 41.0,
        };

        assert_eq!(fat_ratio_from_navy_formula(180.0, tapes), None);
    }
}
