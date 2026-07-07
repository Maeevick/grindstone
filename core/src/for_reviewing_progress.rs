use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::progress::{Progress, progress_towards};
use crate::refusal::Refusal;
use crate::trend::seven_day_trend;
use crate::version::version_in_force_on;

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
    let today = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .map_err(|_| Failure::Refused(Refusal::NotADate))?;
    let days_in_window = match window {
        Window::Days30 => 30,
    };
    let first_day = today - Days::new(days_in_window - 1);
    let gathered = records.gather_records()?;
    let mut masses: Vec<(NaiveDate, f64)> = gathered
        .measurements
        .iter()
        .filter(|measured| measured.day >= first_day && measured.day <= today)
        .map(|measured| (measured.day, measured.body_mass_kg))
        .collect();
    masses.sort_by_key(|(day, _)| *day);
    let every_mass: Vec<(NaiveDate, f64)> = gathered
        .measurements
        .iter()
        .map(|measured| (measured.day, measured.body_mass_kg))
        .collect();
    let trend: Vec<(NaiveDate, f64)> = seven_day_trend(&every_mass)
        .into_iter()
        .filter(|(day, _)| *day >= first_day && *day <= today)
        .collect();
    let target_kg = version_in_force_on(&gathered.goals, today).map(|goals| goals.body_mass_kg);
    let progress = progress_towards(target_kg, &trend);
    Ok(GatherCard {
        masses,
        trend,
        target_kg,
        progress,
        ..GatherCard::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::{Tapes, body_composition};
    use crate::for_setting_up_the_profile::{GoalsForm, ProfileForm, set_goals, set_profile};
    use crate::for_tracking::log_measurement;
    use crate::gender::Gender;
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

    #[test]
    fn the_card_traces_the_trend_of_those_masses_with_their_week_before_the_window() {
        let records = GatherRecordsInMemory::default();
        measured(
            &records,
            &[
                ("2026-09-08", 80.6),
                ("2026-09-12", 80.2),
                ("2026-09-14", 80.0),
            ],
        );

        let trend = gather_card(&records, "2026-10-10", Window::Days30)
            .map(|card| card.trend)
            .expect("a card");

        let expected = [
            (day("2026-09-12"), 80.4),
            (day("2026-09-14"), (80.6 + 80.2 + 80.0) / 3.0),
        ];
        assert_eq!(trend.len(), expected.len());
        assert!(
            trend
                .iter()
                .zip(expected)
                .all(|((day, mean), (expected_day, expected_mean))| {
                    *day == expected_day && (mean - expected_mean).abs() < 1e-9
                })
        );
    }

    #[test]
    fn the_card_reads_the_weekly_rate_towards_the_target_in_force_today() {
        let records = GatherRecordsInMemory::default();
        set_goals(
            &records,
            &GoalsForm {
                from: "2026-03-01".into(),
                body_mass_kg: 70.0,
                fat_percentage: 10.0,
            },
        )
        .expect("goals set");
        measured(
            &records,
            &[
                ("2026-09-19", 80.0),
                ("2026-09-26", 79.5),
                ("2026-10-03", 79.0),
                ("2026-10-10", 78.5),
            ],
        );

        let card = gather_card(&records, "2026-10-10", Window::Days30).expect("a card");

        assert_eq!(card.target_kg, Some(70.0));
        assert!(
            card.progress
                .is_some_and(|progress| (progress.per_week + 0.5).abs() < 1e-9)
        );
    }

    #[test]
    fn without_a_target_the_card_still_reads_the_weekly_rate() {
        let records = GatherRecordsInMemory::default();
        measured(
            &records,
            &[
                ("2026-09-19", 80.0),
                ("2026-09-26", 79.5),
                ("2026-10-03", 79.0),
                ("2026-10-10", 78.5),
            ],
        );

        let card = gather_card(&records, "2026-10-10", Window::Days30).expect("a card");

        assert!(
            card.progress
                .is_some_and(|progress| (progress.per_week + 0.5).abs() < 1e-9)
        );
        assert_eq!(card.progress.and_then(|progress| progress.reached_on), None);
    }

    #[test]
    fn the_card_tells_how_much_fat_and_lean_mass_changed_over_the_window() {
        let records = GatherRecordsInMemory::default();
        set_profile(
            &records,
            &ProfileForm {
                from: "2026-03-01".into(),
                birthday: "1985-04-12".into(),
                gender: Gender::Male,
                height_cm: 180.0,
                neck_cm: 38.5,
                hip_cm: None,
            },
        )
        .expect("a profile set");
        log_measurement(&records, "2026-09-20", 80.0, Some(80.0)).expect("a measurement logs");
        log_measurement(&records, "2026-10-10", 78.0, Some(78.0)).expect("a measurement logs");
        let tapes = |waist_cm| Tapes::Male {
            neck_cm: 38.5,
            waist_cm,
        };
        let before = body_composition(80.0, 180.0, tapes(80.0)).expect("a composition");
        let after = body_composition(78.0, 180.0, tapes(78.0)).expect("a composition");

        let card = gather_card(&records, "2026-10-10", Window::Days30).expect("a card");

        assert!(card.fat_change_kg.is_some_and(|change| {
            (change - (after.fat_mass_kg - before.fat_mass_kg)).abs() < 1e-9
        }));
        assert!(card.lean_change_kg.is_some_and(|change| {
            (change - (after.lean_mass_kg - before.lean_mass_kg)).abs() < 1e-9
        }));
    }
}
