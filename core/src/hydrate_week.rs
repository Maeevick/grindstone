use serde::{Deserialize, Serialize};

use crate::drink::DrinkKind;
use crate::hydrate::Hydrate;
use crate::week::{Averaged, Banded, averaged};

/// What the week drank, a day, over its logged days, against their bands.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct HydrateWeek {
    pub logged_days: usize,
    pub drunk: Averaged,
    /// A logged day's share of water among its drinks, in percent, against three quarters at least.
    pub water: Averaged,
}

const WATER_SHARE_FLOOR_PERCENT: f64 = 75.0;

/// A day's share of water among its drinks, in percent, judged against three quarters at least.
fn water_share_of(day: &Hydrate) -> Banded {
    let water_ml: f64 = day
        .drinkings
        .iter()
        .filter(|(_, drunk)| drunk.kind == DrinkKind::Water)
        .map(|(_, drunk)| drunk.volume_ml)
        .sum();
    Banded {
        amount: if day.drunk_ml > 0.0 {
            100.0 * water_ml / day.drunk_ml
        } else {
            0.0
        },
        floor: WATER_SHARE_FLOOR_PERCENT,
        ceiling: 100.0,
    }
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
        water: averaged(&days.iter().map(water_share_of).collect::<Vec<_>>())?,
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
            hydrate_week_of(&days).map(|week| (week.logged_days, week.drunk)),
            Some((
                3,
                Averaged {
                    per_day: 2200.0,
                    floor: 2000.0,
                    ceiling: 2400.0,
                    in_band: 1,
                }
            ))
        );
    }

    #[test]
    fn a_day_with_nothing_drunk_has_no_share_of_water() {
        assert_eq!(
            hydrate_week_of(&[a_day(0.0)]).map(|week| week.water.per_day),
            Some(0.0)
        );
    }

    #[test]
    fn a_hydrate_week_without_a_logged_day_has_no_average() {
        assert_eq!(hydrate_week_of(&[]), None);
    }
}
