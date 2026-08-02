use std::path::Path;

use chrono::NaiveDate;
use grind_core::burn_records::BurnRecords;
use grind_core::exercise::Exercise;
use grind_core::failure::Broken;
use grind_core::food::{Food, Nutrients, Unit};
use grind_core::for_storing_burn_records::ForStoringBurnRecords;
use grind_core::for_storing_fuel_records::ForStoringFuelRecords;
use grind_core::for_storing_gather_records::ForStoringGatherRecords;
use grind_core::fuel_records::{FuelRecords, FuelTargets};
use grind_core::gather_records::GatherRecords;
use grind_core::gender::Gender;
use grind_core::goals::Goals;
use grind_core::intake::Intake;
use grind_core::measurement::Measurement;
use grind_core::profile::Profile;
use grind_core::volitional::Volitional;
use grind_core::workout::Workout;
use rusqlite::{Connection, params};

const MIGRATIONS: &[&str] = &[
    "create table weigh_ins (date text primary key, kg real not null)",
    "create table measurements (day text primary key, body_mass_kg real not null, waist_cm real);
     insert into measurements (day, body_mass_kg) select date, kg from weigh_ins;",
    "create table profiles (
         effective_from text primary key, birthday text not null, gender text not null,
         height_cm real not null, neck_cm real not null, hip_cm real
     )",
    "create table goals (effective_from text primary key, body_mass_kg real not null, fat_ratio real not null)",
    "drop table weigh_ins",
    "create table exercises (name text primary key, met real not null);
     create table workouts (
         id integer primary key, day text not null, exercise text not null, met real not null,
         minutes real not null, reps integer, sets integer not null, load_kg real not null
     );
     create table days_off_the_clock (
         day text primary key, sitting_min real not null, standing_min real not null, moving_km real not null
     );
     create table usual_days (
         effective_from text primary key, sitting_min real not null, standing_min real not null,
         moving_km real not null
     );",
    "create table foods (
         name text primary key, unit text not null, portion real not null, energy_kcal real not null,
         protein_g real not null, carbs_g real not null, fat_g real not null, fibre_g real not null
     );
     create table intakes (
         id integer primary key, day text not null, food text not null, unit text not null,
         quantity real not null, energy_kcal real not null, protein_g real not null,
         carbs_g real not null, fat_g real not null, fibre_g real not null
     );
     create table fuel_targets (
         effective_from text primary key, protein_g_per_kg real not null, fat_g_per_kg real not null,
         fibre_g_per_kg real not null
     );",
];

pub struct Sqlite(Connection);

impl Sqlite {
    pub fn open(file: &Path) -> Result<Self, Broken> {
        Connection::open(file).map_err(broken).and_then(migrated)
    }

    pub fn in_memory() -> Result<Self, Broken> {
        Connection::open_in_memory()
            .map_err(broken)
            .and_then(migrated)
    }
}

fn broken(error: rusqlite::Error) -> Broken {
    Broken(error.to_string())
}

fn gender_word(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "male",
        Gender::Female => "female",
    }
}

fn gender_read(word: &str) -> rusqlite::Result<Gender> {
    match word {
        "male" => Ok(Gender::Male),
        "female" => Ok(Gender::Female),
        unknown => Err(rusqlite::Error::InvalidParameterName(format!(
            "a gender the app does not know: {unknown}"
        ))),
    }
}

fn migrated(connection: Connection) -> Result<Sqlite, Broken> {
    migrate(&connection).map_err(broken)?;
    Ok(Sqlite(connection))
}

fn migrate(connection: &Connection) -> rusqlite::Result<()> {
    let applied: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (version, migration) in (1_i64..).zip(MIGRATIONS) {
        if version > applied {
            connection.execute_batch(migration)?;
            connection.pragma_update(None, "user_version", version)?;
        }
    }
    Ok(())
}

impl ForStoringGatherRecords for Sqlite {
    fn save_measurement(&self, measurement: &Measurement) -> Result<(), Broken> {
        self.0
            .execute(
                "insert into measurements (day, body_mass_kg, waist_cm) values (?1, ?2, ?3)
                 on conflict (day) do update
                 set body_mass_kg = excluded.body_mass_kg, waist_cm = excluded.waist_cm",
                params![
                    measurement.day,
                    measurement.body_mass_kg,
                    measurement.waist_cm
                ],
            )
            .map(|_| ())
            .map_err(broken)
    }

    fn remove_measurement(&self, day: NaiveDate) -> Result<(), Broken> {
        self.0
            .execute("delete from measurements where day = ?1", params![day])
            .map(|_| ())
            .map_err(broken)
    }

    fn save_profile(&self, from: NaiveDate, profile: &Profile) -> Result<(), Broken> {
        self.0
            .execute(
                "insert into profiles (effective_from, birthday, gender, height_cm, neck_cm, hip_cm)
                 values (?1, ?2, ?3, ?4, ?5, ?6)
                 on conflict (effective_from) do update
                 set birthday = excluded.birthday, gender = excluded.gender,
                     height_cm = excluded.height_cm, neck_cm = excluded.neck_cm, hip_cm = excluded.hip_cm",
                params![
                    from,
                    profile.birthday,
                    gender_word(profile.gender),
                    profile.height_cm,
                    profile.neck_cm,
                    profile.hip_cm
                ],
            )
            .map(|_| ())
            .map_err(broken)
    }

    fn save_goals(&self, from: NaiveDate, goals: &Goals) -> Result<(), Broken> {
        self.0
            .execute(
                "insert into goals (effective_from, body_mass_kg, fat_ratio) values (?1, ?2, ?3)
                 on conflict (effective_from) do update
                 set body_mass_kg = excluded.body_mass_kg, fat_ratio = excluded.fat_ratio",
                params![from, goals.body_mass_kg, goals.fat_ratio],
            )
            .map(|_| ())
            .map_err(broken)
    }

    fn replace_gather_records(&self, records: &GatherRecords) -> Result<(), Broken> {
        let replacing = self.0.unchecked_transaction().map_err(broken)?;
        replacing
            .execute_batch("delete from measurements; delete from profiles; delete from goals;")
            .map_err(broken)?;
        for measured in &records.measurements {
            self.save_measurement(measured)?;
        }
        for (from, profile) in &records.profiles {
            self.save_profile(*from, profile)?;
        }
        for (from, goals) in &records.goals {
            self.save_goals(*from, goals)?;
        }
        replacing.commit().map_err(broken)
    }

    fn gather_records(&self) -> Result<GatherRecords, Broken> {
        let measurements = self
            .0
            .prepare("select day, body_mass_kg, waist_cm from measurements order by day")
            .and_then(|mut statement| {
                statement
                    .query_map([], |row| {
                        Ok(Measurement {
                            day: row.get(0)?,
                            body_mass_kg: row.get(1)?,
                            waist_cm: row.get(2)?,
                        })
                    })?
                    .collect()
            })
            .map_err(broken)?;
        let profiles = self
            .0
            .prepare(
                "select effective_from, birthday, gender, height_cm, neck_cm, hip_cm from profiles
                 order by effective_from",
            )
            .and_then(|mut statement| {
                statement
                    .query_map([], |row| {
                        Ok((
                            row.get(0)?,
                            Profile {
                                birthday: row.get(1)?,
                                gender: gender_read(&row.get::<_, String>(2)?)?,
                                height_cm: row.get(3)?,
                                neck_cm: row.get(4)?,
                                hip_cm: row.get(5)?,
                            },
                        ))
                    })?
                    .collect()
            })
            .map_err(broken)?;
        let goals = self
            .0
            .prepare(
                "select effective_from, body_mass_kg, fat_ratio from goals order by effective_from",
            )
            .and_then(|mut statement| {
                statement
                    .query_map([], |row| {
                        Ok((
                            row.get(0)?,
                            Goals {
                                body_mass_kg: row.get(1)?,
                                fat_ratio: row.get(2)?,
                            },
                        ))
                    })?
                    .collect()
            })
            .map_err(broken)?;
        Ok(GatherRecords {
            measurements,
            profiles,
            goals,
        })
    }
}

impl Sqlite {
    fn run(&self, statement: &str, values: impl rusqlite::Params) -> Result<(), Broken> {
        self.0
            .execute(statement, values)
            .map(|_| ())
            .map_err(broken)
    }

    fn rows<Row>(
        &self,
        query: &str,
        read: impl FnMut(&rusqlite::Row) -> rusqlite::Result<Row>,
    ) -> Result<Vec<Row>, Broken> {
        self.0
            .prepare(query)
            .and_then(|mut statement| statement.query_map([], read)?.collect())
            .map_err(broken)
    }
}

fn volitional_read(row: &rusqlite::Row) -> rusqlite::Result<(NaiveDate, Volitional)> {
    Ok((
        row.get(0)?,
        Volitional {
            sitting_min: row.get(1)?,
            standing_min: row.get(2)?,
            moving_km: row.get(3)?,
        },
    ))
}

impl ForStoringBurnRecords for Sqlite {
    fn save_exercise(&self, exercise: &Exercise) -> Result<(), Broken> {
        self.run(
            "insert into exercises (name, met) values (?1, ?2)
             on conflict (name) do update set met = excluded.met",
            params![exercise.name, exercise.met],
        )
    }

    fn remove_exercise(&self, name: &str) -> Result<(), Broken> {
        self.run("delete from exercises where name = ?1", params![name])
    }

    fn save_workout(&self, workout: &Workout) -> Result<(), Broken> {
        self.run(
            "insert into workouts (day, exercise, met, minutes, reps, sets, load_kg)
             values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                workout.day,
                workout.exercise,
                workout.met,
                workout.minutes,
                workout.reps,
                workout.sets,
                workout.load_kg
            ],
        )
    }

    fn remove_workout(&self, id: i64) -> Result<(), Broken> {
        self.run("delete from workouts where id = ?1", params![id])
    }

    fn save_day_off_the_clock(
        &self,
        day: NaiveDate,
        volitional: &Volitional,
    ) -> Result<(), Broken> {
        self.run(
            "insert into days_off_the_clock (day, sitting_min, standing_min, moving_km)
             values (?1, ?2, ?3, ?4)
             on conflict (day) do update set sitting_min = excluded.sitting_min,
                 standing_min = excluded.standing_min, moving_km = excluded.moving_km",
            params![
                day,
                volitional.sitting_min,
                volitional.standing_min,
                volitional.moving_km
            ],
        )
    }

    fn save_usual_day(&self, from: NaiveDate, volitional: &Volitional) -> Result<(), Broken> {
        self.run(
            "insert into usual_days (effective_from, sitting_min, standing_min, moving_km)
             values (?1, ?2, ?3, ?4)
             on conflict (effective_from) do update set sitting_min = excluded.sitting_min,
                 standing_min = excluded.standing_min, moving_km = excluded.moving_km",
            params![
                from,
                volitional.sitting_min,
                volitional.standing_min,
                volitional.moving_km
            ],
        )
    }

    fn replace_burn_records(&self, records: &BurnRecords) -> Result<(), Broken> {
        let replacing = self.0.unchecked_transaction().map_err(broken)?;
        replacing
            .execute_batch(
                "delete from exercises; delete from workouts;
                 delete from days_off_the_clock; delete from usual_days;",
            )
            .map_err(broken)?;
        for exercise in &records.exercises {
            self.save_exercise(exercise)?;
        }
        for (_, workout) in &records.workouts {
            self.save_workout(workout)?;
        }
        for (day, volitional) in &records.days_off_the_clock {
            self.save_day_off_the_clock(*day, volitional)?;
        }
        for (from, volitional) in &records.usual_days {
            self.save_usual_day(*from, volitional)?;
        }
        replacing.commit().map_err(broken)
    }

    fn burn_records(&self) -> Result<BurnRecords, Broken> {
        Ok(BurnRecords {
            exercises: self.rows("select name, met from exercises order by name", |row| {
                Ok(Exercise {
                    name: row.get(0)?,
                    met: row.get(1)?,
                })
            })?,
            workouts: self.rows(
                "select id, day, exercise, met, minutes, reps, sets, load_kg from workouts
                 order by day, id",
                |row| {
                    Ok((
                        row.get(0)?,
                        Workout {
                            day: row.get(1)?,
                            exercise: row.get(2)?,
                            met: row.get(3)?,
                            minutes: row.get(4)?,
                            reps: row.get(5)?,
                            sets: row.get(6)?,
                            load_kg: row.get(7)?,
                        },
                    ))
                },
            )?,
            days_off_the_clock: self.rows(
                "select day, sitting_min, standing_min, moving_km from days_off_the_clock order by day",
                volitional_read,
            )?,
            usual_days: self.rows(
                "select effective_from, sitting_min, standing_min, moving_km from usual_days
                 order by effective_from",
                volitional_read,
            )?,
        })
    }
}

impl ForStoringFuelRecords for Sqlite {
    fn save_food(&self, _: &Food) -> Result<(), Broken> {
        Err(Broken("not yet".into()))
    }

    fn remove_food(&self, _: &str) -> Result<(), Broken> {
        Err(Broken("not yet".into()))
    }

    fn save_intake(&self, _: &Intake) -> Result<(), Broken> {
        Err(Broken("not yet".into()))
    }

    fn remove_intake(&self, _: i64) -> Result<(), Broken> {
        Err(Broken("not yet".into()))
    }

    fn save_targets(&self, _: NaiveDate, _: &FuelTargets) -> Result<(), Broken> {
        Err(Broken("not yet".into()))
    }

    fn replace_fuel_records(&self, _: &FuelRecords) -> Result<(), Broken> {
        Err(Broken("not yet".into()))
    }

    fn fuel_records(&self) -> Result<FuelRecords, Broken> {
        let _ = (Nutrients::default(), Unit::Gram);
        Err(Broken("not yet".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use grind_core::testing::honouring_for_storing_burn_records;
    use grind_core::testing::honouring_for_storing_fuel_records;
    use grind_core::testing::honouring_for_storing_gather_records;

    fn database() -> Sqlite {
        Sqlite::in_memory().expect("an in-memory database opens and migrates")
    }

    #[test]
    fn a_database_opened_again_keeps_its_measurements_and_migrates_nothing_twice() {
        let file = std::env::temp_dir().join(format!(
            "grind-sqlite-opened-again-{}.sqlite",
            std::process::id()
        ));
        let measured = Measurement {
            day: NaiveDate::from_ymd_opt(2026, 10, 10).expect("a real day"),
            body_mass_kg: 80.0,
            waist_cm: Some(84.0),
        };
        Sqlite::open(&file)
            .and_then(|database| database.save_measurement(&measured))
            .expect("a first opening saves a measurement");

        let reopened = Sqlite::open(&file)
            .and_then(|database| database.gather_records())
            .map(|gathered| gathered.measurements);

        std::fs::remove_file(&file).expect("the test database is removed");
        assert_eq!(reopened, Ok(vec![measured]));
    }

    #[test]
    fn a_database_that_cannot_be_opened_says_it_is_broken() {
        let nowhere = std::env::temp_dir().join("grind-sqlite-no-such-folder/grindstone.sqlite");

        assert!(matches!(Sqlite::open(&nowhere), Err(Broken(_))));
    }

    #[test]
    fn a_day_measured_twice_keeps_its_second_measurement() {
        honouring_for_storing_gather_records::a_day_measured_twice_keeps_its_second_measurement(
            &database(),
        );
    }

    #[test]
    fn the_weigh_ins_already_on_the_phone_become_measurements() {
        let file = std::env::temp_dir().join(format!(
            "grind-sqlite-weigh-ins-carried-{}.sqlite",
            std::process::id()
        ));
        let before = Connection::open(&file).expect("a database at version 1");
        before
            .execute_batch(
                "create table weigh_ins (date text primary key, kg real not null);
                 insert into weigh_ins (date, kg) values ('2026-10-03', 72.1), ('2026-10-04', 71.9);
                 pragma user_version = 1;",
            )
            .expect("the version 1 weigh-ins");
        drop(before);

        let carried = Sqlite::open(&file).and_then(|database| database.gather_records());

        std::fs::remove_file(&file).expect("the test database is removed");
        let carried: Vec<(String, f64, Option<f64>)> = carried
            .expect("the records read")
            .measurements
            .iter()
            .map(|measured| {
                (
                    measured.day.to_string(),
                    measured.body_mass_kg,
                    measured.waist_cm,
                )
            })
            .collect();
        assert!(carried.contains(&("2026-10-03".into(), 72.1, None)));
        assert!(carried.contains(&("2026-10-04".into(), 71.9, None)));
    }

    #[test]
    fn removing_a_measured_day_leaves_the_others() {
        honouring_for_storing_gather_records::removing_a_measured_day_leaves_the_others(&database());
    }

    #[test]
    fn a_profile_or_goals_saved_again_from_the_same_day_replace_the_earlier_ones() {
        honouring_for_storing_gather_records::a_profile_or_goals_saved_again_from_the_same_day_replace_the_earlier_ones(
            &database(),
        );
    }

    #[test]
    fn gather_records_come_back_in_the_order_of_their_days() {
        honouring_for_storing_gather_records::gather_records_come_back_in_the_order_of_their_days(
            &database(),
        );
    }

    #[test]
    fn replacing_the_gather_records_leaves_only_the_new_ones() {
        honouring_for_storing_gather_records::replacing_the_gather_records_leaves_only_the_new_ones(
            &database(),
        );
    }

    #[test]
    fn a_replacement_failing_half_way_leaves_the_records_as_they_were() {
        let records = database();
        let kept = Measurement {
            day: NaiveDate::from_ymd_opt(2026, 10, 10).expect("a real day"),
            body_mass_kg: 80.0,
            waist_cm: None,
        };
        records
            .save_measurement(&kept)
            .expect("a measurement saves");
        let half_valid = GatherRecords {
            measurements: vec![Measurement {
                body_mass_kg: 76.4,
                ..kept
            }],
            profiles: vec![(
                kept.day,
                Profile {
                    birthday: kept.day,
                    gender: Gender::Male,
                    height_cm: f64::NAN,
                    neck_cm: 38.5,
                    hip_cm: None,
                },
            )],
            goals: vec![],
        };

        assert!(records.replace_gather_records(&half_valid).is_err());
        assert_eq!(
            records
                .gather_records()
                .map(|gathered| gathered.measurements),
            Ok(vec![kept])
        );
    }

    #[test]
    fn a_profile_whose_gender_the_app_does_not_know_reads_as_broken() {
        let file = std::env::temp_dir().join(format!(
            "grind-sqlite-unknown-gender-{}.sqlite",
            std::process::id()
        ));
        drop(Sqlite::open(&file).expect("a migrated database"));
        Connection::open(&file)
            .and_then(|raw| {
                raw.execute(
                    "insert into profiles (effective_from, birthday, gender, height_cm, neck_cm)
                     values ('2026-03-01', '1985-04-12', 'goblin', 180.0, 38.5)",
                    [],
                )
            })
            .expect("a profile no app wrote");

        let read = Sqlite::open(&file).and_then(|database| database.gather_records());

        std::fs::remove_file(&file).expect("the test database is removed");
        assert!(matches!(read, Err(Broken(reason)) if reason.contains("goblin")));
    }

    #[test]
    fn a_womans_profile_comes_back_with_her_hip() {
        honouring_for_storing_gather_records::a_womans_profile_comes_back_with_her_hip(&database());
    }

    #[test]
    fn burn_records_come_back_in_the_order_of_their_days() {
        honouring_for_storing_burn_records::burn_records_come_back_in_the_order_of_their_days(
            &database(),
        );
    }

    #[test]
    fn what_is_saved_again_under_its_name_or_day_replaces_the_earlier_one() {
        honouring_for_storing_burn_records::what_is_saved_again_under_its_name_or_day_replaces_the_earlier_one(
            &database(),
        );
    }

    #[test]
    fn removing_a_workout_or_an_exercise_leaves_the_others() {
        honouring_for_storing_burn_records::removing_a_workout_or_an_exercise_leaves_the_others(
            &database(),
        );
    }

    #[test]
    fn replacing_the_burn_records_leaves_only_the_new_ones() {
        honouring_for_storing_burn_records::replacing_the_burn_records_leaves_only_the_new_ones(
            &database(),
        );
    }

    #[test]
    fn fuel_records_come_back_in_the_order_of_their_days() {
        honouring_for_storing_fuel_records::fuel_records_come_back_in_the_order_of_their_days(
            &database(),
        );
    }

    #[test]
    fn a_food_or_targets_saved_again_replace_the_earlier_ones() {
        honouring_for_storing_fuel_records::a_food_or_targets_saved_again_replace_the_earlier_ones(
            &database(),
        );
    }

    #[test]
    fn removing_an_intake_or_a_food_leaves_the_others() {
        honouring_for_storing_fuel_records::removing_an_intake_or_a_food_leaves_the_others(
            &database(),
        );
    }

    #[test]
    fn replacing_the_fuel_records_leaves_only_the_new_ones() {
        honouring_for_storing_fuel_records::replacing_the_fuel_records_leaves_only_the_new_ones(
            &database(),
        );
    }
}
