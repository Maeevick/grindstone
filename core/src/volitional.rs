use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Volitional {
    pub sitting_min: f64,
    pub standing_min: f64,
    pub moving_km: f64,
}

pub fn volitional(
    sitting_min: f64,
    standing_min: f64,
    moving_km: f64,
) -> Result<Volitional, crate::refusal::Refusal> {
    let _ = (sitting_min, standing_min, moving_km);
    Err(crate::refusal::Refusal::NotADistance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refusal::Refusal;

    #[test]
    fn a_day_off_the_clock_keeps_its_sitting_and_standing_minutes_and_its_moving_kilometres() {
        assert_eq!(
            volitional(300.0, 300.0, 4.23),
            Ok(Volitional {
                sitting_min: 300.0,
                standing_min: 300.0,
                moving_km: 4.23,
            })
        );
    }

    #[test]
    fn a_day_spent_lying_down_is_a_day_off_the_clock_too() {
        assert_eq!(
            volitional(0.0, 0.0, 0.0),
            Ok(Volitional::default())
        );
    }

    #[test]
    fn negative_or_unreadable_figures_or_more_minutes_than_a_day_holds_are_refused() {
        for (sitting_min, standing_min, moving_km) in [
            (-1.0, 300.0, 4.0),
            (300.0, f64::NAN, 4.0),
            (300.0, 300.0, -0.5),
            (1000.0, 500.0, 4.0),
        ] {
            assert_eq!(
                volitional(sitting_min, standing_min, moving_km),
                Err(Refusal::NotADistance),
                "{sitting_min} {standing_min} {moving_km}"
            );
        }
    }
}
