use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::progress::Progress;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Window {
    Days30,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GatherCard {
    pub masses: Vec<(NaiveDate, f64)>,
    pub trend: Vec<(NaiveDate, f64)>,
    pub target_kg: Option<f64>,
    pub progress: Option<Progress>,
    pub fat_change_kg: Option<f64>,
    pub lean_change_kg: Option<f64>,
}

pub fn gather_card(
    records: &impl ForStoringGatherRecords,
    today: &str,
    window: Window,
) -> Result<GatherCard, Failure> {
    Ok(GatherCard::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::for_tracking::log_measurement;
    use crate::testing::GatherRecordsInMemory;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn measured(records: &GatherRecordsInMemory, days_and_masses: &[(&str, f64)]) {
        for (date, kg) in days_and_masses {
            log_measurement(records, date, *kg, None).expect("a measurement logs");
        }
    }

    #[test]
    fn the_gather_card_draws_the_masses_of_the_window_ending_today() {
        let records = GatherRecordsInMemory::default();
        measured(
            &records,
            &[
                ("2026-08-01", 82.0),
                ("2026-09-20", 80.4),
                ("2026-10-10", 80.0),
            ],
        );

        let card = gather_card(&records, "2026-10-10", Window::Days30);

        assert_eq!(
            card.map(|card| card.masses),
            Ok(vec![(day("2026-09-20"), 80.4), (day("2026-10-10"), 80.0)])
        );
    }
}
