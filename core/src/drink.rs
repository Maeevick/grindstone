use serde::{Deserialize, Serialize};

use crate::refusal::Refusal;
use crate::volume::volume;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrinkKind {
    Water,
    #[default]
    Other,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Drink {
    pub name: String,
    pub volume_ml: f64,
    pub kind: DrinkKind,
}

pub fn drink(name: &str, volume_ml: f64, kind: DrinkKind) -> Result<Drink, Refusal> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Refusal::NotAName);
    }
    Ok(Drink {
        name: name.into(),
        volume_ml: volume(volume_ml)?,
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drink_keeps_its_trimmed_name_its_kind_and_the_volume_of_one_serving() {
        assert_eq!(
            drink("  Expresso ", 80.0, DrinkKind::Other),
            Ok(Drink {
                name: "Expresso".into(),
                volume_ml: 80.0,
                kind: DrinkKind::Other,
            })
        );
    }

    #[test]
    fn a_drink_without_a_name_is_refused() {
        assert_eq!(drink("   ", 80.0, DrinkKind::Other), Err(Refusal::NotAName));
    }

    #[test]
    fn a_serving_of_nothing_or_no_number_at_all_is_refused() {
        for volume_ml in [0.0, -80.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                drink("Expresso", volume_ml, DrinkKind::Other),
                Err(Refusal::NotAVolume),
                "{volume_ml}"
            );
        }
    }
}
