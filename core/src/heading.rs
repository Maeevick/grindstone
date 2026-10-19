use crate::tendency::Heading;

/// A measure moving less than 0.15% of itself a week holds steady.
pub(crate) const STABLE_SHARE_A_WEEK: f64 = 0.0015;

/// Where a measure moving so much a week heads: steady under 0.15% of itself, else toward its aim or away from
/// it, unaimed without one.
pub fn heading_of(per_week: f64, now: f64, aim: Option<f64>) -> Heading {
    match aim {
        None => Heading::Unaimed,
        Some(_) if per_week.abs() < STABLE_SHARE_A_WEEK * now.abs() => Heading::Steady,
        Some(aim) if (aim - now).signum() == per_week.signum() => Heading::Toward,
        Some(_) => Heading::Away,
    }
}
