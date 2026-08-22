use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::drink::Drink;
use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Drinking {
    pub day: NaiveDate,
    pub drink: String,
    pub volume_ml: f64,
}

pub fn drinking(date: &str, drink: &Drink, servings: f64) -> Result<Drinking, Refusal> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Refusal::NotADate)?;
    if !(servings.is_finite() && servings > 0.0) {
        return Err(Refusal::NotAServing);
    }
    Ok(Drinking {
        day,
        drink: drink.name.clone(),
        volume_ml: servings * drink.volume_ml,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tea() -> Drink {
        Drink {
            name: "Tea".into(),
            volume_ml: 200.0,
        }
    }

    #[test]
    fn a_drinking_freezes_its_volume_as_its_servings_of_the_drink() {
        assert_eq!(
            drinking("2026-10-10", &tea(), 1.5),
            Ok(Drinking {
                day: NaiveDate::from_ymd_opt(2026, 10, 10).expect("a real day"),
                drink: "Tea".into(),
                volume_ml: 300.0,
            })
        );
    }

    #[test]
    fn a_drinking_that_could_not_have_happened_is_refused() {
        assert_eq!(drinking("2026-02-30", &tea(), 1.0), Err(Refusal::NotADate));
        for servings in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                drinking("2026-10-10", &tea(), servings),
                Err(Refusal::NotAServing),
                "{servings}"
            );
        }
    }
}
