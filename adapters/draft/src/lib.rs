use chrono::NaiveDate;
use grind_core::burn_records::BurnRecords;
use grind_core::drink::Drink;
use grind_core::drinking::drinking;
use grind_core::exercise::{Exercise, exercise};
use grind_core::failure::Failure;
use grind_core::food::{Nutrients, Unit, food};
use grind_core::for_importing::Drafted;
use grind_core::fuel_records::{DEFAULT_FUEL_TARGETS, FuelRecords, FuelTargets};
use grind_core::gather_records::GatherRecords;
use grind_core::gender::Gender;
use grind_core::goals::Goals;
use grind_core::hydrate_records::HydrateRecords;
use grind_core::intake::Intake;
use grind_core::kept::Kept;
use grind_core::measurement::measurement;
use grind_core::profile::Profile;
use grind_core::refusal::Refusal;
use grind_core::version::version_in_force_on;
use grind_core::volitional::Volitional;
use grind_core::workout::workout;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DraftRefusal {
    Unreadable(String),
    NotAVersionWeRead(String),
    NotAMeasurement { date: String, refusal: Refusal },
    NotAnExercise { name: String, refusal: Refusal },
    NotAWorkout { date: String, refusal: Refusal },
    NotAFood { name: String, refusal: Refusal },
    NotADrinking { date: String, refusal: Refusal },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImportFailure {
    NoBackupFound(String),
    Draft(DraftRefusal),
    Failed(Failure),
}

#[derive(Deserialize)]
struct Backup {
    settings: Settings,
    measurement_bases: MeasurementBases,
    #[serde(default)]
    measurement_logs: Vec<MeasurementLog>,
    workout_bases: Option<DraftVolitional>,
    #[serde(default)]
    workout_catalogs: WorkoutCatalogs,
    #[serde(default)]
    workout_logs: Vec<WorkoutLog>,
    #[serde(default)]
    intake_catalogs: IntakeCatalogs,
    #[serde(default)]
    intake_logs: Vec<IntakeLog>,
}

#[derive(Default, Deserialize)]
struct IntakeCatalogs {
    #[serde(default)]
    foods: Vec<DraftFood>,
}

#[derive(Deserialize)]
struct DraftFood {
    name: String,
    qty: f64,
    unit: String,
    cal: f64,
    prot: f64,
    carbs: f64,
    fat: f64,
    fiber: f64,
}

#[derive(Deserialize)]
struct IntakeLog {
    date: String,
    prot_g_kg: f64,
    fat_g_kg: f64,
    #[serde(default)]
    foods: Vec<IntakeLine>,
}

#[derive(Deserialize)]
struct IntakeLine {
    name: String,
    qty: f64,
    unit: String,
    energy: f64,
    protein: f64,
    carb: f64,
    fat: f64,
    fiber: f64,
}

#[derive(Deserialize)]
struct Settings {
    version: String,
    started_at: Option<NaiveDate>,
}

#[derive(Clone, Copy, Deserialize)]
struct DraftVolitional {
    sitting_min: f64,
    standing_min: f64,
    moving_km: f64,
}

#[derive(Default, Deserialize)]
struct WorkoutCatalogs {
    #[serde(default)]
    activities: Vec<Activity>,
}

#[derive(Deserialize)]
struct Activity {
    name: String,
    met: f64,
}

#[derive(Deserialize)]
struct WorkoutLog {
    date: String,
    volitional: DraftVolitional,
    #[serde(default)]
    activities: Vec<WorkoutEntry>,
}

#[derive(Deserialize)]
struct WorkoutEntry {
    activity: String,
    minutes: f64,
    reps: u32,
    sets: u32,
    weight: f64,
    aee: f64,
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
    weight_target: f64,
    fat_target: f64,
}

fn gender(drafted: DraftGender) -> Gender {
    match drafted {
        DraftGender::Male => Gender::Male,
        DraftGender::Female => Gender::Female,
    }
}

fn logged_days(backup: &Backup) -> impl Iterator<Item = (NaiveDate, &MeasurementLog)> {
    backup.measurement_logs.iter().filter_map(|log| {
        NaiveDate::parse_from_str(&log.date, "%Y-%m-%d")
            .ok()
            .map(|day| (day, log))
    })
}

fn versions_where_they_change<V: PartialEq>(
    dated: impl Iterator<Item = (NaiveDate, V)>,
) -> Vec<(NaiveDate, V)> {
    dated.fold(Vec::new(), |mut versions, (day, version)| {
        if versions.last().is_none_or(|(_, last)| *last != version) {
            versions.push((day, version));
        }
        versions
    })
}

fn profile_versions(backup: &Backup) -> Vec<(NaiveDate, Profile)> {
    versions_where_they_change(logged_days(backup).map(|(day, log)| {
        (
            day,
            Profile {
                birthday: backup.measurement_bases.birthday,
                gender: gender(log.gender),
                height_cm: log.height,
                neck_cm: log.neck,
                hip_cm: log.hip,
            },
        )
    }))
}

fn goal_versions(backup: &Backup) -> Vec<(NaiveDate, Kept<Goals>)> {
    versions_where_they_change(logged_days(backup).map(|(day, log)| {
        (
            day,
            Kept::Own(Goals {
                body_mass_kg: log.weight_target,
                fat_ratio: log.fat_target / 100.0,
            }),
        )
    }))
}

pub fn gather_from_draft(backup: &str) -> Result<GatherRecords, DraftRefusal> {
    let backup = read(backup)?;
    Ok(GatherRecords {
        measurements: backup
            .measurement_logs
            .iter()
            .map(|log| {
                measurement(&log.date, log.weight, log.waist).map_err(|refusal| {
                    DraftRefusal::NotAMeasurement {
                        date: log.date.clone(),
                        refusal,
                    }
                })
            })
            .collect::<Result<_, _>>()?,
        profiles: profile_versions(&backup),
        goals: goal_versions(&backup),
        counted_from: vec![],
    })
}

fn read(backup: &str) -> Result<Backup, DraftRefusal> {
    let backup: Backup = serde_json::from_str(backup)
        .map_err(|error| DraftRefusal::Unreadable(error.to_string()))?;
    if backup.settings.version != "0.4.0" {
        return Err(DraftRefusal::NotAVersionWeRead(backup.settings.version));
    }
    Ok(backup)
}

fn off_the_clock(drafted: DraftVolitional) -> Volitional {
    Volitional {
        sitting_min: drafted.sitting_min,
        standing_min: drafted.standing_min,
        moving_km: drafted.moving_km,
    }
}

fn met_worked_back(entry: &WorkoutEntry, body_mass_kg: Option<f64>) -> f64 {
    let burnt_per_met_above_rest =
        body_mass_kg.map_or(0.0, |kg| 3.5 * (kg + entry.weight) / 200.0 * entry.minutes);
    if burnt_per_met_above_rest > 0.0 {
        ((entry.aee / burnt_per_met_above_rest + 1.0) * 10.0).round() / 10.0
    } else {
        1.0
    }
}

fn exercise_of(
    entry: &WorkoutEntry,
    day: NaiveDate,
    catalog: &[Exercise],
    masses: &[(NaiveDate, f64)],
) -> Exercise {
    catalog
        .iter()
        .find(|known| known.name == entry.activity)
        .cloned()
        .unwrap_or_else(|| Exercise {
            name: entry.activity.clone(),
            met: met_worked_back(entry, version_in_force_on(masses, day).copied()),
            description: String::new(),
        })
}

pub fn burn_from_draft(backup: &str) -> Result<BurnRecords, DraftRefusal> {
    let backup = read(backup)?;
    let catalog = backup
        .workout_catalogs
        .activities
        .iter()
        .map(|activity| {
            exercise(&activity.name, activity.met, "").map_err(|refusal| {
                DraftRefusal::NotAnExercise {
                    name: activity.name.clone(),
                    refusal,
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let masses: Vec<(NaiveDate, f64)> = logged_days(&backup)
        .map(|(day, log)| (day, log.weight))
        .collect();
    let mut workouts = Vec::new();
    let mut days_off_the_clock = Vec::new();
    for log in &backup.workout_logs {
        let refused = |refusal| DraftRefusal::NotAWorkout {
            date: log.date.clone(),
            refusal,
        };
        let day = NaiveDate::parse_from_str(&log.date, "%Y-%m-%d")
            .map_err(|_| refused(Refusal::NotADate))?;
        days_off_the_clock.push((day, off_the_clock(log.volitional)));
        for entry in &log.activities {
            let done = workout(
                &log.date,
                &exercise_of(entry, day, &catalog, &masses),
                entry.minutes,
                Some(entry.reps),
                entry.sets.max(1),
                entry.weight,
            )
            .map_err(refused)?;
            workouts.push((0, done));
        }
    }
    let started_at = backup
        .settings
        .started_at
        .or(days_off_the_clock.first().map(|(day, _)| *day));
    Ok(BurnRecords {
        exercises: catalog,
        workouts,
        days_off_the_clock,
        usual_days: started_at
            .zip(backup.workout_bases)
            .map(|(from, bases)| (from, Kept::Own(off_the_clock(bases))))
            .into_iter()
            .collect(),
        circuits: Vec::new(),
        targets: Vec::new(),
        counted_from: vec![],
    })
}

const LEFT_TO_HYDRATE: [&str; 2] = ["Coffee", "Tea"];

fn unit_of(drafted: &str) -> Unit {
    match drafted {
        "ml" => Unit::Millilitre,
        "piece" | "unit" => Unit::Piece,
        _ => Unit::Gram,
    }
}

pub fn fuel_from_draft(backup: &str) -> Result<FuelRecords, DraftRefusal> {
    let backup = read(backup)?;
    let foods = backup
        .intake_catalogs
        .foods
        .iter()
        .filter(|drafted| !LEFT_TO_HYDRATE.contains(&drafted.name.as_str()))
        .map(|drafted| {
            food(
                &drafted.name,
                unit_of(&drafted.unit),
                drafted.qty,
                Nutrients {
                    energy_kcal: drafted.cal,
                    protein_g: drafted.prot,
                    carbs_g: drafted.carbs,
                    fat_g: drafted.fat,
                    fibre_g: drafted.fiber,
                },
                "",
            )
            .map_err(|refusal| DraftRefusal::NotAFood {
                name: drafted.name.clone(),
                refusal,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut intakes = Vec::new();
    let mut targets = Vec::new();
    for log in &backup.intake_logs {
        let day = NaiveDate::parse_from_str(&log.date, "%Y-%m-%d").map_err(|_| {
            DraftRefusal::NotAFood {
                name: log.date.clone(),
                refusal: Refusal::NotADate,
            }
        })?;
        targets.push((
            day,
            FuelTargets {
                protein_g_per_kg: log.prot_g_kg,
                fat_g_per_kg: log.fat_g_kg,
                fibre_g_per_kg: DEFAULT_FUEL_TARGETS.fibre_g_per_kg,
            },
        ));
        intakes.extend(
            log.foods
                .iter()
                .filter(|line| !LEFT_TO_HYDRATE.contains(&line.name.as_str()))
                .map(|line| {
                    (
                        0,
                        Intake {
                            day,
                            food: line.name.clone(),
                            unit: unit_of(&line.unit),
                            quantity: line.qty,
                            nutrients: Nutrients {
                                energy_kcal: line.energy,
                                protein_g: line.protein,
                                carbs_g: line.carb,
                                fat_g: line.fat,
                                fibre_g: line.fiber,
                            },
                        },
                    )
                }),
        );
    }
    Ok(FuelRecords {
        foods,
        intakes,
        targets: versions_where_they_change(targets.into_iter())
            .into_iter()
            .map(|(from, aims)| (from, Kept::Own(aims)))
            .collect(),
        meals: vec![],
        counted_from: vec![],
    })
}

fn drafted_drinks() -> Vec<Drink> {
    vec![
        Drink {
            name: "Expresso".into(),
            volume_ml: 80.0,
            kind: grind_core::drink::DrinkKind::Other,
            description: String::new(),
        },
        Drink {
            name: "Tea".into(),
            volume_ml: 200.0,
            kind: grind_core::drink::DrinkKind::Other,
            description: String::new(),
        },
    ]
}

fn drink_drafted_as<'a>(line: &IntakeLine, catalog: &'a [Drink]) -> Option<&'a Drink> {
    let name = match line.name.as_str() {
        "Coffee" => "Expresso",
        "Tea" => "Tea",
        _ => return None,
    };
    catalog.iter().find(|drink| drink.name == name)
}

fn volume_drafted(line: &IntakeLine, drink: &Drink) -> Option<f64> {
    match line.unit.as_str() {
        "piece" => Some(line.qty * drink.volume_ml),
        "ml" => Some(line.qty),
        _ => None,
    }
}

pub fn hydrate_from_draft(backup: &str) -> Result<HydrateRecords, DraftRefusal> {
    let backup = read(backup)?;
    let drinks = drafted_drinks();
    let drinkings = backup
        .intake_logs
        .iter()
        .flat_map(|log| {
            log.foods.iter().filter_map(|line| {
                let drink = drink_drafted_as(line, &drinks)?;
                let volume_ml = volume_drafted(line, drink)?;
                Some(
                    drinking(&log.date, drink, volume_ml)
                        .map(|drunk| (0, drunk))
                        .map_err(|refusal| DraftRefusal::NotADrinking {
                            date: log.date.clone(),
                            refusal,
                        }),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HydrateRecords {
        drinks,
        drinkings,
        counted_from: vec![],
        targets: vec![],
    })
}

pub fn drafted(backup: &str) -> Result<Drafted, DraftRefusal> {
    Ok(Drafted {
        gather: gather_from_draft(backup)?,
        burn: burn_from_draft(backup)?,
        fuel: fuel_from_draft(backup)?,
        hydrate: hydrate_from_draft(backup)?,
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
                    Kept::Own(Goals {
                        body_mass_kg: 72.0,
                        fat_ratio: 0.12,
                    })
                ),
                (
                    day("2026-06-22"),
                    Kept::Own(Goals {
                        body_mass_kg: 70.0,
                        fat_ratio: 0.10,
                    })
                ),
            ])
        );
    }

    #[test]
    fn a_backup_of_another_version_is_refused_naming_it() {
        let older = BACKUP.replace(r#""version": "0.4.0""#, r#""version": "0.3.0""#);

        assert_eq!(
            gather_from_draft(&older),
            Err(DraftRefusal::NotAVersionWeRead("0.3.0".into()))
        );
    }

    #[test]
    fn a_backup_that_cannot_be_read_is_refused_saying_where_it_stopped() {
        let refused = gather_from_draft("{ \"settings\": nothing }");

        assert!(
            matches!(&refused, Err(DraftRefusal::Unreadable(where_it_stopped)) if where_it_stopped.contains("line 1 column")),
            "{refused:?}"
        );
    }

    #[test]
    fn a_logged_day_the_app_would_refuse_is_refused_naming_its_date() {
        let impossible = BACKUP.replace(r#""weight": 76.4"#, r#""weight": 0.0"#);

        assert_eq!(
            gather_from_draft(&impossible),
            Err(DraftRefusal::NotAMeasurement {
                date: "2026-06-22".into(),
                refusal: Refusal::NotAMass,
            })
        );
    }

    #[test]
    fn a_womans_logs_open_her_profile_with_her_hip() {
        let a_womans = BACKUP
            .replace(r#""gender": "male", "age""#, r#""gender": "female", "age""#)
            .replace(r#""hip": null"#, r#""hip": 98.0"#);

        let first_profile = gather_from_draft(&a_womans).map(|records| {
            records
                .profiles
                .first()
                .map(|(_, profile)| (profile.gender, profile.hip_cm))
        });

        assert_eq!(first_profile, Ok(Some((Gender::Female, Some(98.0)))));
    }

    const BURN_BACKUP: &str = r#"{
        "settings": { "version": "0.4.0", "started_at": "2026-03-01" },
        "measurement_bases": { "birthday": "1985-04-12" },
        "measurement_logs": [
            { "date": "2026-08-01", "gender": "male", "height": 180.0, "neck": 38.5, "hip": null,
              "weight": 80.0, "waist": null, "weight_target": 72.0, "fat_target": 12.0 }
        ],
        "workout_bases": { "moving_km": 4.23, "sitting_min": 300.0, "standing_min": 300.0 },
        "workout_catalogs": { "activities": [ { "id": "w", "created_at": 1, "name": "Walking", "met": 4.0 } ] },
        "workout_logs": [
            { "id": "l1", "created_at": 1, "date": "2026-08-10",
              "volitional": { "moving_km": 6.0, "sitting_min": 240.0, "standing_min": 360.0 },
              "activities": [
                { "id": "a", "created_at": 1, "activity": "Walking", "minutes": 30.0, "reps": 0, "sets": 1, "weight": 0.0, "aee": 999.0 },
                { "id": "b", "created_at": 2, "activity": "Star Plank", "minutes": 2.0, "reps": 10, "sets": 1, "weight": 0.0, "aee": 8.4 }
              ] }
        ]
    }"#;

    #[test]
    fn the_draft_activities_become_the_exercise_catalog() {
        assert_eq!(
            burn_from_draft(BURN_BACKUP).map(|records| records.exercises),
            Ok(vec![grind_core::exercise::Exercise {
                name: "Walking".into(),
                met: 4.0,
                description: String::new(),
            }])
        );
    }

    #[test]
    fn each_drafted_bout_becomes_a_workout_frozen_on_its_exercise_met_whatever_burn_the_draft_stored()
     {
        let workouts = burn_from_draft(BURN_BACKUP).map(|records| {
            records
                .workouts
                .into_iter()
                .map(|(_, done)| (done.day, done.exercise, done.met, done.minutes, done.reps))
                .collect::<Vec<_>>()
        });

        assert_eq!(
            workouts.as_ref().map(|done| done.first().cloned()),
            Ok(Some((day("2026-08-10"), "Walking".into(), 4.0, 30.0, None)))
        );
    }

    #[test]
    fn a_bout_of_an_activity_gone_from_the_catalog_works_its_met_back_from_its_net_burn() {
        let star_plank_met = burn_from_draft(BURN_BACKUP).ok().and_then(|records| {
            records
                .workouts
                .into_iter()
                .find(|(_, done)| done.exercise == "Star Plank")
                .map(|(_, done)| done.met)
        });

        assert_eq!(star_plank_met, Some(4.0));
    }

    #[test]
    fn each_drafted_day_keeps_its_day_off_the_clock_and_the_bases_become_the_usual_day() {
        let records = burn_from_draft(BURN_BACKUP).expect("burn records");
        let off_the_clock =
            |sitting_min, standing_min, moving_km| grind_core::volitional::Volitional {
                sitting_min,
                standing_min,
                moving_km,
            };

        assert_eq!(
            records.days_off_the_clock,
            vec![(day("2026-08-10"), off_the_clock(240.0, 360.0, 6.0))]
        );
        assert_eq!(
            records.usual_days,
            vec![(
                day("2026-03-01"),
                Kept::Own(off_the_clock(300.0, 300.0, 4.23))
            )]
        );
    }

    #[test]
    fn a_backup_drafts_the_records_of_every_section() {
        let drafted = drafted(BURN_BACKUP).expect("drafted records");

        assert_eq!(
            (
                drafted.gather.measurements.len(),
                drafted.burn.workouts.len()
            ),
            (1, 2)
        );
    }

    const FUEL_BACKUP: &str = r#"{
        "settings": { "version": "0.4.0", "started_at": "2026-03-01" },
        "measurement_bases": { "birthday": "1985-04-12" },
        "intake_catalogs": { "foods": [
            { "id": "s", "created_at": 1, "name": "Skyr", "qty": 100.0, "unit": "g", "cal": 54.0, "prot": 9.5, "carbs": 4.0, "fat": 0.0, "fiber": 0.0 },
            { "id": "b", "created_at": 2, "name": "Beer", "qty": 1000.0, "unit": "ml", "cal": 432.0, "prot": 3.0, "carbs": 30.0, "fat": 0.0, "fiber": 0.0 },
            { "id": "e", "created_at": 3, "name": "Egg", "qty": 1.0, "unit": "piece", "cal": 70.0, "prot": 6.0, "carbs": 0.5, "fat": 5.0, "fiber": 0.0 },
            { "id": "c", "created_at": 4, "name": "Coffee", "qty": 1.0, "unit": "piece", "cal": 0.0, "prot": 0.0, "carbs": 0.0, "fat": 0.0, "fiber": 0.0 }
        ] },
        "intake_logs": [
            { "id": "d1", "created_at": 1, "date": "2026-08-02", "prot_g_kg": 2.0, "fat_g_kg": 0.9, "foods": [
                { "id": "1", "created_at": 1, "name": "Skyr", "qty": 200.0, "unit": "g", "energy": 108.0, "protein": 19.0, "carb": 8.0, "fat": 0.0, "fiber": 0.0 },
                { "id": "2", "created_at": 2, "name": "Coffee", "qty": 1.0, "unit": "piece", "energy": 0.0, "protein": 0.0, "carb": 0.0, "fat": 0.0, "fiber": 0.0 }
            ] },
            { "id": "d2", "created_at": 2, "date": "2026-08-03", "prot_g_kg": 2.0, "fat_g_kg": 0.9, "foods": [
                { "id": "3", "created_at": 3, "name": "Huile (1 tbsp)", "qty": 15.0, "unit": "ml", "energy": 120.0, "protein": 0.0, "carb": 0.0, "fat": 13.5, "fiber": 0.0 }
            ] },
            { "id": "d3", "created_at": 3, "date": "2026-09-01", "prot_g_kg": 2.025, "fat_g_kg": 0.9, "foods": [] }
        ]
    }"#;

    #[test]
    fn the_draft_foods_become_the_food_catalog_with_their_portion_and_unit_but_coffee() {
        let foods = fuel_from_draft(FUEL_BACKUP).map(|records| {
            records
                .foods
                .into_iter()
                .map(|food| (food.name, food.unit, food.portion))
                .collect::<Vec<_>>()
        });

        assert_eq!(
            foods,
            Ok(vec![
                ("Skyr".into(), grind_core::food::Unit::Gram, 100.0),
                ("Beer".into(), grind_core::food::Unit::Millilitre, 1000.0),
                ("Egg".into(), grind_core::food::Unit::Piece, 1.0),
            ])
        );
    }

    #[test]
    fn each_drafted_line_but_coffee_and_tea_becomes_an_intake_frozen_on_its_stored_figures() {
        let intakes = fuel_from_draft(FUEL_BACKUP).map(|records| {
            records
                .intakes
                .into_iter()
                .map(|(_, eaten)| (eaten.day, eaten.food, eaten.quantity, eaten.nutrients.fat_g))
                .collect::<Vec<_>>()
        });

        assert_eq!(
            intakes,
            Ok(vec![
                (day("2026-08-02"), "Skyr".into(), 200.0, 0.0),
                (day("2026-08-03"), "Huile (1 tbsp)".into(), 15.0, 13.5),
            ])
        );
    }

    #[test]
    fn the_targets_frozen_on_the_drafted_days_open_a_version_wherever_they_change() {
        let targets = fuel_from_draft(FUEL_BACKUP).map(|records| {
            records
                .targets
                .into_iter()
                .filter_map(|(from, aims)| {
                    aims.own()
                        .map(|aims| (from, aims.protein_g_per_kg, aims.fibre_g_per_kg))
                })
                .collect::<Vec<_>>()
        });

        assert_eq!(
            targets,
            Ok(vec![
                (day("2026-08-02"), 2.0, 0.4),
                (day("2026-09-01"), 2.025, 0.4)
            ])
        );
    }

    #[test]
    fn a_drafted_day_the_calendar_lacks_is_refused_naming_it() {
        let no_such_intake_day =
            FUEL_BACKUP.replace(r#""date": "2026-08-03""#, r#""date": "2026-02-30""#);
        let no_such_workout_day =
            BURN_BACKUP.replace(r#""date": "2026-08-10""#, r#""date": "2026-02-30""#);

        assert_eq!(
            fuel_from_draft(&no_such_intake_day),
            Err(DraftRefusal::NotAFood {
                name: "2026-02-30".into(),
                refusal: Refusal::NotADate,
            })
        );
        assert_eq!(
            burn_from_draft(&no_such_workout_day),
            Err(DraftRefusal::NotAWorkout {
                date: "2026-02-30".into(),
                refusal: Refusal::NotADate,
            })
        );
    }

    #[test]
    fn a_bout_gone_from_the_catalog_with_no_weigh_in_to_work_from_burns_at_rest() {
        let unweighed = BURN_BACKUP
            .replace(r#""weight": 80.0"#, r#""weight": 80.0, "x": 0"#)
            .replace(
                r#""measurement_logs": ["#,
                r#""measurement_logs": [], "ignored": ["#,
            );

        let star_plank_met = burn_from_draft(&unweighed).ok().and_then(|records| {
            records
                .workouts
                .into_iter()
                .find(|(_, done)| done.exercise == "Star Plank")
                .map(|(_, done)| done.met)
        });

        assert_eq!(star_plank_met, Some(1.0));
    }

    #[test]
    fn an_activity_or_a_food_the_app_would_refuse_is_refused_naming_it() {
        let restful = BURN_BACKUP.replace(
            r#""name": "Walking", "met": 4.0"#,
            r#""name": "Walking", "met": 0.0"#,
        );
        let negative = FUEL_BACKUP.replace(r#""cal": 54.0"#, r#""cal": -54.0"#);

        assert_eq!(
            burn_from_draft(&restful),
            Err(DraftRefusal::NotAnExercise {
                name: "Walking".into(),
                refusal: Refusal::NotAMet,
            })
        );
        assert_eq!(
            fuel_from_draft(&negative),
            Err(DraftRefusal::NotAFood {
                name: "Skyr".into(),
                refusal: Refusal::NotANutrient,
            })
        );
    }

    const HYDRATE_BACKUP: &str = r#"{
        "settings": { "version": "0.4.0", "started_at": "2026-03-01" },
        "measurement_bases": { "birthday": "1985-04-12" },
        "intake_bases": { "fat_g_kg": 0.9, "prot_g_kg": 2.025 },
        "intake_logs": [
            { "id": "i1", "created_at": 1, "date": "2026-08-02", "fat_g_kg": 0.9, "prot_g_kg": 2.025, "foods": [
                { "id": "f1", "created_at": 1, "name": "Coffee", "qty": 2.0, "unit": "piece",
                  "energy": 0.0, "protein": 0.0, "fat": 0.0, "carb": 0.0, "fiber": 0.0 },
                { "id": "f2", "created_at": 2, "name": "Oeuf (~55g)", "qty": 6.0, "unit": "piece",
                  "energy": 450.0, "protein": 37.8, "fat": 31.8, "carb": 0.0, "fiber": 0.0 },
                { "id": "f3", "created_at": 3, "name": "Coffee", "qty": 15.0, "unit": "ml",
                  "energy": 0.0, "protein": 0.0, "fat": 0.0, "carb": 0.0, "fiber": 0.0 }
            ] },
            { "id": "i2", "created_at": 2, "date": "2026-08-03", "fat_g_kg": 0.9, "prot_g_kg": 2.025, "foods": [
                { "id": "f4", "created_at": 4, "name": "Tea", "qty": 1.0, "unit": "piece",
                  "energy": 0.0, "protein": 0.0, "fat": 0.0, "carb": 0.0, "fiber": 0.0 },
                { "id": "f5", "created_at": 5, "name": "Bubble Tea Mangue", "qty": 250.0, "unit": "ml",
                  "energy": 150.0, "protein": 0.0, "fat": 0.0, "carb": 37.0, "fiber": 0.0 },
                { "id": "f6", "created_at": 6, "name": "Tea", "qty": 250.0, "unit": "ml",
                  "energy": 0.0, "protein": 0.0, "fat": 0.0, "carb": 0.0, "fiber": 0.0 }
            ] }
        ]
    }"#;

    fn drunk_from(records: &HydrateRecords) -> Vec<(NaiveDate, String, f64)> {
        records
            .drinkings
            .iter()
            .map(|(_, drunk)| (drunk.day, drunk.drink.clone(), drunk.volume_ml))
            .collect()
    }

    #[test]
    fn the_drafted_drinks_are_an_expresso_of_80_ml_and_a_tea_of_200_ml_with_no_base() {
        let records = hydrate_from_draft(HYDRATE_BACKUP).expect("hydrate records");

        assert_eq!(
            records.drinks,
            vec![
                grind_core::drink::Drink {
                    name: "Expresso".into(),
                    volume_ml: 80.0,
                    kind: grind_core::drink::DrinkKind::Other,
                    description: String::new(),
                },
                grind_core::drink::Drink {
                    name: "Tea".into(),
                    volume_ml: 200.0,
                    kind: grind_core::drink::DrinkKind::Other,
                    description: String::new(),
                },
            ]
        );
        assert_eq!(records.counted_from, vec![]);
    }

    #[test]
    fn each_drafted_coffee_or_tea_becomes_a_drinking_by_the_piece_or_by_the_millilitre_and_nothing_else_does()
     {
        let records = hydrate_from_draft(HYDRATE_BACKUP).expect("hydrate records");

        assert_eq!(
            drunk_from(&records),
            vec![
                (day("2026-08-02"), "Expresso".into(), 160.0),
                (day("2026-08-02"), "Expresso".into(), 15.0),
                (day("2026-08-03"), "Tea".into(), 200.0),
                (day("2026-08-03"), "Tea".into(), 250.0),
            ]
        );
    }

    #[test]
    fn a_drafted_drink_on_a_day_the_calendar_lacks_is_refused_naming_its_date() {
        let impossible =
            HYDRATE_BACKUP.replace(r#""date": "2026-08-03""#, r#""date": "2026-02-30""#);

        assert_eq!(
            hydrate_from_draft(&impossible),
            Err(DraftRefusal::NotADrinking {
                date: "2026-02-30".into(),
                refusal: Refusal::NotADate,
            })
        );
    }

    #[test]
    fn a_coffee_weighed_in_grams_files_no_drinking() {
        let weighed =
            HYDRATE_BACKUP.replace(r#""qty": 15.0, "unit": "ml""#, r#""qty": 7.0, "unit": "g""#);

        let drunk = hydrate_from_draft(&weighed).map(|records| records.drinkings.len());

        assert_eq!(drunk, Ok(3));
    }

    #[test]
    fn the_drafts_coffee_and_tea_drinkings_are_other_drinks_keeping_their_servings() {
        let drinkings = hydrate_from_draft(HYDRATE_BACKUP).map(|records| {
            records
                .drinkings
                .into_iter()
                .map(|(_, drunk)| (drunk.kind, drunk.servings))
                .collect::<Vec<_>>()
        });

        assert_eq!(
            drinkings,
            Ok(vec![
                (grind_core::drink::DrinkKind::Other, 2.0),
                (grind_core::drink::DrinkKind::Other, 15.0 / 80.0),
                (grind_core::drink::DrinkKind::Other, 1.0),
                (grind_core::drink::DrinkKind::Other, 1.25),
            ])
        );
    }
}
