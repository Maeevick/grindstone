use serde::{Deserialize, Serialize};

use crate::hydrate::Hydrate;
use crate::week::{Averaged, Banded, averaged};

/// What the week drank, a day, over its logged days, against their bands.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct HydrateWeek {
    pub logged_days: usize,
    pub drunk: Averaged,
}

/// The week's drinks, from its logged days; nothing without one.
pub fn hydrate_week_of(days: &[Hydrate]) -> Option<HydrateWeek> {
    let drunk = averaged(
        &days
            .iter()
            .map(|day| Banded {
                amount: day.drunk_ml,
                floor: day.floor_ml,
                ceiling: day.cap_ml,
            })
            .collect::<Vec<_>>(),
    )?;
    Some(HydrateWeek {
        logged_days: days.len(),
        drunk,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verdict::verdict_of;
    use chrono::NaiveDate;

    fn a_day(drunk_ml: f64) -> Hydrate {
        Hydrate {
            day: NaiveDate::from_ymd_opt(2026, 10, 10).expect("a day"),
            drinkings: vec![],
            drunk_ml,
            target_ml: 2200.0,
            floor_ml: 2000.0,
            cap_ml: 2400.0,
            verdict: verdict_of(drunk_ml, 2000.0, 2400.0),
        }
    }

    #[test]
    fn a_hydrate_week_averages_the_drinks_of_its_logged_days_against_their_bands() {
        let days = [a_day(1800.0), a_day(2200.0), a_day(2600.0)];

        assert_eq!(
            hydrate_week_of(&days),
            Some(HydrateWeek {
                logged_days: 3,
                drunk: Averaged {
                    per_day: 2200.0,
                    floor: 2000.0,
                    ceiling: 2400.0,
                    in_band: 1,
                },
            })
        );
    }

    #[test]
    fn a_hydrate_week_without_a_logged_day_has_no_average() {
        assert_eq!(hydrate_week_of(&[]), None);
    }
}
