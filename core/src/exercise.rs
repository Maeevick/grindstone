use serde::{Deserialize, Serialize};

use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Exercise {
    pub name: String,
    pub met: f64,
    #[serde(default)]
    pub description: String,
}

pub fn exercise(name: &str, met: f64, description: &str) -> Result<Exercise, Refusal> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Refusal::NotAName);
    }
    if !(met.is_finite() && met > 0.0) {
        return Err(Refusal::NotAMet);
    }
    Ok(Exercise {
        name: name.into(),
        met,
        description: description.trim().into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_exercise_keeps_its_trimmed_name_and_its_met() {
        assert_eq!(
            exercise("  Push up ", 3.8, ""),
            Ok(Exercise {
                name: "Push up".into(),
                met: 3.8,
                description: String::new(),
            })
        );
    }

    #[test]
    fn an_exercise_without_a_name_is_refused() {
        assert_eq!(exercise("   ", 3.8, ""), Err(Refusal::NotAName));
    }

    #[test]
    fn a_met_of_nothing_or_no_number_at_all_is_refused() {
        for met in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(exercise("Push up", met, ""), Err(Refusal::NotAMet), "{met}");
        }
    }
}
