use serde::{Deserialize, Serialize};

use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeighIn {
    pub date: String,
    pub kg: f64,
}

fn is_a_date(text: &str) -> bool {
    chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d").is_ok()
}

fn to_two_decimals(figure: f64) -> f64 {
    (figure * 100.0).round() / 100.0
}

pub fn weigh_in(date: &str, kg: f64) -> Result<WeighIn, Refusal> {
    if !is_a_date(date) {
        return Err(Refusal::NotADate);
    }
    if !(kg.is_finite() && kg > 0.0) {
        return Err(Refusal::NotAMass);
    }
    Ok(WeighIn {
        date: date.into(),
        kg: to_two_decimals(kg),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_weigh_in_keeps_its_date_and_its_mass_to_two_decimals() {
        assert_eq!(
            weigh_in("2026-10-03", 72.149),
            Ok(WeighIn {
                date: "2026-10-03".into(),
                kg: 72.15
            })
        );
    }

    #[test]
    fn a_date_that_does_not_exist_is_refused() {
        assert_eq!(weigh_in("2026-02-30", 72.1), Err(Refusal::NotADate));
        assert_eq!(weigh_in("yesterday", 72.1), Err(Refusal::NotADate));
    }

    #[test]
    fn a_mass_of_nothing_or_no_number_at_all_is_refused() {
        for kg in [0.0, -72.1, f64::NAN, f64::INFINITY] {
            assert_eq!(weigh_in("2026-10-03", kg), Err(Refusal::NotAMass), "{kg}");
        }
    }
}
