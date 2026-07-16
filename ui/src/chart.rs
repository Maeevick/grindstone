use grind_core::for_reviewing_progress::GatherCard;

#[derive(Clone, Debug, PartialEq)]
pub struct Chart {
    pub masses: Vec<(f64, f64)>,
    pub trend: String,
    pub target: Option<f64>,
    pub highest_kg: f64,
    pub lowest_kg: f64,
}

pub fn chart_of(card: &GatherCard, _width: f64, _height: f64) -> Option<Chart> {
    if card.masses.is_empty() {
        return None;
    }
    Some(Chart {
        masses: Vec::new(),
        trend: String::new(),
        target: None,
        highest_kg: 0.0,
        lowest_kg: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_card_without_masses_draws_no_chart() {
        assert_eq!(chart_of(&GatherCard::default(), 360.0, 180.0), None);
    }
}
