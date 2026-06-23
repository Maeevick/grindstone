use grind_core::gather_records::GatherRecords;

#[derive(Clone, Debug, PartialEq)]
pub enum DraftRefusal {
    Unreadable(String),
}

pub fn gather_from_draft(backup: &str) -> Result<GatherRecords, DraftRefusal> {
    Err(DraftRefusal::Unreadable(String::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use grind_core::measurement::Measurement;

    const BACKUP: &str = r#"{
        "settings": { "version": "0.4.0", "started_at": "2026-03-01" },
        "measurement_bases": { "birthday": "1985-04-12", "gender": "male", "height": 180.0, "neck": 38.0, "ffm_fallback": 63.0 },
        "measurement_goals": { "weight_target": 70.0, "fat_target": 10.0 },
        "measurement_logs": [
            { "id": "a", "created_at": 1, "date": "2026-03-01", "gender": "male", "age": 40, "height": 180.0, "neck": 38.5, "ffm_fallback": 63.0,
              "weight": 79.5, "waist": null, "hip": null, "weight_target": 72.0, "fat_target": 12.0 },
            { "id": "b", "created_at": 2, "date": "2026-06-22", "gender": "male", "age": 41, "height": 180.0, "neck": 38.5, "ffm_fallback": 63.0,
              "weight": 76.4, "waist": 84.0, "hip": null, "weight_target": 70.0, "fat_target": 10.0 },
            { "id": "c", "created_at": 3, "date": "2026-10-08", "gender": "male", "age": 41, "height": 180.0, "neck": 38.0, "ffm_fallback": 63.0,
              "weight": 72.15, "waist": 80.5, "hip": null, "weight_target": 70.0, "fat_target": 10.0 }
        ]
    }"#;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    fn measured(date: &str, body_mass_kg: f64, waist_cm: Option<f64>) -> Measurement {
        Measurement {
            day: day(date),
            body_mass_kg,
            waist_cm,
        }
    }

    #[test]
    fn each_logged_day_becomes_a_measurement_with_its_mass_and_its_waist() {
        let records = gather_from_draft(BACKUP).map(|records| records.measurements);

        assert_eq!(
            records,
            Ok(vec![
                measured("2026-03-01", 79.5, None),
                measured("2026-06-22", 76.4, Some(84.0)),
                measured("2026-10-08", 72.15, Some(80.5)),
            ])
        );
    }
}
