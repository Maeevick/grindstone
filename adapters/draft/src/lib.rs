use chrono::NaiveDate;
use grind_core::gather_records::GatherRecords;
use grind_core::gender::Gender;
use grind_core::measurement::measurement;
use grind_core::profile::Profile;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub enum DraftRefusal {
    Unreadable(String),
}

#[derive(Deserialize)]
struct Backup {
    measurement_bases: MeasurementBases,
    #[serde(default)]
    measurement_logs: Vec<MeasurementLog>,
}

#[derive(Deserialize)]
struct MeasurementBases {
    birthday: NaiveDate,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DraftGender {
    Male,
    Female,
}

#[derive(Deserialize)]
struct MeasurementLog {
    date: String,
    gender: DraftGender,
    height: f64,
    neck: f64,
    hip: Option<f64>,
    weight: f64,
    waist: Option<f64>,
}

fn gender(drafted: DraftGender) -> Gender {
    match drafted {
        DraftGender::Male => Gender::Male,
        DraftGender::Female => Gender::Female,
    }
}

fn profile_versions(backup: &Backup) -> Vec<(NaiveDate, Profile)> {
    backup
        .measurement_logs
        .iter()
        .filter_map(|log| {
            let day = NaiveDate::parse_from_str(&log.date, "%Y-%m-%d").ok()?;
            Some((
                day,
                Profile {
                    birthday: backup.measurement_bases.birthday,
                    gender: gender(log.gender),
                    height_cm: log.height,
                    neck_cm: log.neck,
                    hip_cm: log.hip,
                },
            ))
        })
        .fold(
            Vec::new(),
            |mut versions: Vec<(NaiveDate, Profile)>, (day, profile)| {
                if versions.last().is_none_or(|(_, last)| *last != profile) {
                    versions.push((day, profile));
                }
                versions
            },
        )
}

pub fn gather_from_draft(backup: &str) -> Result<GatherRecords, DraftRefusal> {
    let backup: Backup =
        serde_json::from_str(backup).map_err(|_| DraftRefusal::Unreadable(String::new()))?;
    Ok(GatherRecords {
        measurements: backup
            .measurement_logs
            .iter()
            .filter_map(|log| measurement(&log.date, log.weight, log.waist).ok())
            .collect(),
        profiles: profile_versions(&backup),
        goals: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use grind_core::gender::Gender;
    use grind_core::goals::Goals;
    use grind_core::measurement::Measurement;
    use grind_core::profile::Profile;

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

    #[test]
    fn the_bases_frozen_on_the_logs_open_a_profile_version_wherever_they_change() {
        let a_man = |neck_cm| Profile {
            birthday: day("1985-04-12"),
            gender: Gender::Male,
            height_cm: 180.0,
            neck_cm,
            hip_cm: None,
        };

        assert_eq!(
            gather_from_draft(BACKUP).map(|records| records.profiles),
            Ok(vec![
                (day("2026-03-01"), a_man(38.5)),
                (day("2026-10-08"), a_man(38.0)),
            ])
        );
    }

    #[test]
    fn the_goals_frozen_on_the_logs_open_a_goal_version_wherever_they_change() {
        assert_eq!(
            gather_from_draft(BACKUP).map(|records| records.goals),
            Ok(vec![
                (
                    day("2026-03-01"),
                    Goals {
                        body_mass_kg: 72.0,
                        fat_ratio: 0.12,
                    }
                ),
                (
                    day("2026-06-22"),
                    Goals {
                        body_mass_kg: 70.0,
                        fat_ratio: 0.10,
                    }
                ),
            ])
        );
    }
}
