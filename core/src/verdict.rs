use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Low,
    InBand,
    High,
}

pub fn verdict_of(amount: f64, floor: f64, ceiling: f64) -> Verdict {
    let (amount, floor, ceiling) = (amount.round(), floor.round(), ceiling.round());
    if amount < floor {
        Verdict::Low
    } else if amount > ceiling {
        Verdict::High
    } else {
        Verdict::InBand
    }
}

const GRACE: f64 = 0.05;

/// Whether an amount holds its band once each bound gives way by 5% of itself: the streak's tolerance,
/// the gauges keeping the strict band.
pub fn within_grace(amount: f64, floor: f64, ceiling: f64) -> bool {
    verdict_of(amount, floor * (1.0 - GRACE), ceiling * (1.0 + GRACE)) == Verdict::InBand
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_amount_within_five_percent_beyond_either_bound_is_within_grace() {
        assert!(within_grace(1520.0, 1600.0, 2400.0));
        assert!(within_grace(2520.0, 1600.0, 2400.0));
    }

    #[test]
    fn an_amount_further_than_five_percent_beyond_a_bound_is_not_within_grace() {
        assert!(!within_grace(1519.0, 1600.0, 2400.0));
        assert!(!within_grace(2521.0, 1600.0, 2400.0));
    }
}
