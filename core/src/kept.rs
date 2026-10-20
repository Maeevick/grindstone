use serde::{Deserialize, Serialize};

/// What a card keeps from a day: the recommendation, followed as the body changes, or values of its own.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Kept<T> {
    Recommended,
    Own(T),
}

/// What applies on a day, and whether it is the recommendation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct InForce<T> {
    pub values: T,
    pub recommended: bool,
}

/// What applies under the version kept, if any: its own values, else the recommendation when there is one to
/// follow.
pub fn in_force<T: Copy>(
    kept: Option<&Kept<T>>,
    recommendation: impl FnOnce() -> Option<T>,
) -> Option<InForce<T>> {
    let _ = (kept, recommendation);
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn the_recommendation() -> Option<f64> {
        Some(77.0)
    }

    #[test]
    fn own_values_kept_are_in_force_as_they_are() {
        assert_eq!(
            in_force(Some(&Kept::Own(80.0)), the_recommendation),
            Some(InForce {
                values: 80.0,
                recommended: false
            })
        );
    }

    #[test]
    fn the_recommendation_kept_or_nothing_kept_brings_the_recommendation_in_force() {
        let recommended = Some(InForce {
            values: 77.0,
            recommended: true,
        });

        assert_eq!(
            [
                in_force(Some(&Kept::Recommended), the_recommendation),
                in_force(None, the_recommendation)
            ],
            [recommended, recommended]
        );
    }

    #[test]
    fn own_values_stand_even_without_a_recommendation_but_nothing_else_does() {
        assert_eq!(
            [
                in_force(Some(&Kept::Own(80.0)), || None),
                in_force(Some(&Kept::Recommended), || None),
                in_force(None, || None::<f64>)
            ],
            [
                Some(InForce {
                    values: 80.0,
                    recommended: false
                }),
                None,
                None
            ]
        );
    }
}
