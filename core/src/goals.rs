use crate::refusal::Refusal;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Goals {
    pub body_mass_kg: f64,
    pub fat_ratio: f64,
}

pub fn goals(body_mass_kg: f64, fat_percentage: f64) -> Result<Goals, Refusal> {
    Err(Refusal::NotAMass)
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
}
