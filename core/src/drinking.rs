use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::drink::{Drink, DrinkKind};
use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Drinking {
    pub day: NaiveDate,
    pub drink: String,
    pub kind: DrinkKind,
    pub servings: f64,
    pub volume_ml: f64,
}

pub fn drinking(date: &str, drink: &Drink, volume_ml: f64) -> Result<Drinking, Refusal> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Refusal::NotADate)?;
    if !(volume_ml.is_finite() && volume_ml > 0.0) {
        return Err(Refusal::NotAVolume);
    }
    Ok(Drinking {
        day,
        drink: drink.name.clone(),
        kind: drink.kind,
        servings: volume_ml / drink.volume_ml,
        volume_ml,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tea() -> Drink {
        Drink {
            name: "Tea".into(),
            volume_ml: 200.0,
            kind: DrinkKind::Tea,
        }
    }

    #[test]
    fn a_drinking_freezes_its_volume_and_how_many_servings_of_the_drink_it_makes() {
        assert_eq!(
            drinking("2026-10-10", &tea(), 300.0),
            Ok(Drinking {
                day: NaiveDate::from_ymd_opt(2026, 10, 10).expect("a real day"),
                drink: "Tea".into(),
                volume_ml: 300.0,
                kind: DrinkKind::Tea,
                servings: 1.5,
            })
        );
    }

    #[test]
    fn a_drinking_that_could_not_have_happened_is_refused() {
        assert_eq!(
            drinking("2026-02-30", &tea(), 200.0),
            Err(Refusal::NotADate)
        );
        for volume_ml in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                drinking("2026-10-10", &tea(), volume_ml),
                Err(Refusal::NotAVolume),
                "{volume_ml}"
            );
        }
    }

    #[test]
    fn a_drinking_freezes_its_drinks_kind() {
        let espresso = Drink {
            name: "Expresso".into(),
            volume_ml: 80.0,
            kind: DrinkKind::Coffee,
        };

        let drunk = drinking("2026-10-10", &espresso, 160.0).expect("a drinking");

        assert_eq!(
            (drunk.kind, drunk.servings, drunk.volume_ml),
            (DrinkKind::Coffee, 2.0, 160.0)
        );
    }
}
