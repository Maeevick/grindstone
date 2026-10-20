use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// What a card keeps from a day: the recommendation, followed as the body changes, or values of its own.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Kept<T> {
    Recommended,
    Own(T),
}

impl<T> Kept<T> {
    /// Its own values, if it keeps any.
    pub fn own(&self) -> Option<&T> {
        match self {
            Kept::Own(values) => Some(values),
            Kept::Recommended => None,
        }
    }
}

/// What applies on a day, whether it is the recommendation, and the day it is kept from: none while nothing
/// is kept, the recommendation followed from the start.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct InForce<T> {
    pub values: T,
    pub recommended: bool,
    pub from: Option<NaiveDate>,
}

/// What applies under the version kept, if any, with the day it is kept from: its own values, else the
/// recommendation when there is one to follow.
pub fn in_force<T: Copy>(
    kept: Option<&(NaiveDate, Kept<T>)>,
    recommendation: impl FnOnce() -> Option<T>,
) -> Option<InForce<T>> {
    let from = kept.map(|(from, _)| *from);
    match kept.map(|(_, kept)| kept) {
        Some(Kept::Own(values)) => Some(InForce {
            values: *values,
            recommended: false,
            from,
        }),
        _ => recommendation().map(|values| InForce {
            values,
            recommended: true,
            from,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn the_recommendation() -> Option<f64> {
        Some(77.0)
    }

    fn the_15th() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 15).expect("a real day")
    }

    fn from_the_15th<T>(kept: Kept<T>) -> (NaiveDate, Kept<T>) {
        (the_15th(), kept)
    }

    #[test]
    fn own_values_are_told_apart_from_the_recommendation() {
        assert_eq!(
            (Kept::Own(80.0).own(), Kept::<f64>::Recommended.own()),
            (Some(&80.0), None)
        );
    }

    #[test]
    fn own_values_kept_are_in_force_as_they_are() {
        assert_eq!(
            in_force(Some(&from_the_15th(Kept::Own(80.0))), the_recommendation),
            Some(InForce {
                values: 80.0,
                recommended: false,
                from: Some(the_15th()),
            })
        );
    }

    #[test]
    fn the_recommendation_kept_or_nothing_kept_brings_the_recommendation_in_force() {
        let recommended_from = |from| {
            Some(InForce {
                values: 77.0,
                recommended: true,
                from,
            })
        };

        assert_eq!(
            [
                in_force(Some(&from_the_15th(Kept::Recommended)), the_recommendation),
                in_force(None, the_recommendation)
            ],
            [recommended_from(Some(the_15th())), recommended_from(None)]
        );
    }

    #[test]
    fn own_values_stand_even_without_a_recommendation_but_nothing_else_does() {
        assert_eq!(
            [
                in_force(Some(&from_the_15th(Kept::Own(80.0))), || None),
                in_force(Some(&from_the_15th(Kept::Recommended)), || None),
                in_force(None, || None::<f64>)
            ],
            [
                Some(InForce {
                    values: 80.0,
                    recommended: false,
                    from: Some(the_15th()),
                }),
                None,
                None
            ]
        );
    }
}
