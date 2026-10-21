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
