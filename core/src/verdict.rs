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

pub fn within_grace(_amount: f64, _floor: f64, _ceiling: f64) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_amount_within_five_percent_beyond_either_bound_is_within_grace() {
        assert!(within_grace(1520.0, 1600.0, 2400.0));
        assert!(within_grace(2520.0, 1600.0, 2400.0));
    }
}
