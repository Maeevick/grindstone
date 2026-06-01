#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Girths {
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

struct DensityCoefficients {
    intercept: f64,
    girth_slope: f64,
    height_slope: f64,
}

const MALE_DENSITY: DensityCoefficients = DensityCoefficients {
    intercept: 1.0324,
    girth_slope: 0.19077,
    height_slope: 0.15456,
};

const FEMALE_DENSITY: DensityCoefficients = DensityCoefficients {
    intercept: 1.29579,
    girth_slope: 0.35004,
    height_slope: 0.22100,
};

fn body_density(girth_cm: f64, height_cm: f64, coefficients: &DensityCoefficients) -> f64 {
    coefficients.intercept - coefficients.girth_slope * girth_cm.log10()
        + coefficients.height_slope * height_cm.log10()
}

fn siri_fat_ratio(body_density: f64) -> f64 {
    (495.0 / body_density - 450.0) / 100.0
}

pub fn fat_ratio_from_navy_formula(height_cm: f64, girths: Girths) -> Option<f64> {
    let (girth_cm, coefficients) = match girths {
        Girths::Male { neck_cm, waist_cm } => (waist_cm - neck_cm, &MALE_DENSITY),
        Girths::Female {
            neck_cm,
            waist_cm,
            hip_cm,
        } => (waist_cm + hip_cm - neck_cm, &FEMALE_DENSITY),
    };
    let ratio = siri_fat_ratio(body_density(girth_cm, height_cm, coefficients));
    (ratio > 0.0 && ratio < 1.0).then_some(ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reads_about(reading: Option<f64>, expected: f64) -> bool {
        reading.is_some_and(|ratio| (ratio - expected).abs() < 1e-6)
    }

    #[test]
    fn a_mans_fat_ratio_reads_his_waist_against_his_neck_and_his_height() {
        let girths = Girths::Male {
            neck_cm: 39.0,
            waist_cm: 85.0,
        };

        assert!(reads_about(
            fat_ratio_from_navy_formula(178.0, girths),
            0.156542
        ));
    }

    #[test]
    fn a_womans_fat_ratio_adds_her_hip_to_her_waist() {
        let girths = Girths::Female {
            neck_cm: 33.0,
            waist_cm: 75.0,
            hip_cm: 98.0,
        };

        assert!(reads_about(
            fat_ratio_from_navy_formula(165.0, girths),
            0.284350
        ));
    }

    #[test]
    fn a_waist_no_wider_than_the_neck_gives_no_reading() {
        let girths = Girths::Male {
            neck_cm: 40.0,
            waist_cm: 40.0,
        };

        assert_eq!(fat_ratio_from_navy_formula(180.0, girths), None);
    }

    #[test]
    fn a_height_of_nothing_gives_no_reading() {
        let girths = Girths::Male {
            neck_cm: 39.0,
            waist_cm: 85.0,
        };

        assert_eq!(fat_ratio_from_navy_formula(0.0, girths), None);
    }

    #[test]
    fn a_ratio_no_body_could_have_gives_no_reading() {
        let girths = Girths::Male {
            neck_cm: 40.0,
            waist_cm: 41.0,
        };

        assert_eq!(fat_ratio_from_navy_formula(180.0, girths), None);
    }
}
