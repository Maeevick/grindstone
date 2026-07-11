use std::path::Path;

use chrono::NaiveDate;
use grind_core::failure::Broken;
use grind_core::for_storing_gather_records::ForStoringGatherRecords;
use grind_core::for_storing_weigh_ins::ForStoringWeighIns;
use grind_core::gather_records::GatherRecords;
use grind_core::gender::Gender;
use grind_core::goals::Goals;
use grind_core::measurement::Measurement;
use grind_core::profile::Profile;
use grind_core::weigh_in::WeighIn;
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

impl ForStoringWeighIns for Sqlite {
    fn save(&self, weigh_in: &WeighIn) -> Result<(), Broken> {
        self.0
            .execute(
                "insert into weigh_ins (date, kg) values (?1, ?2)
                 on conflict (date) do update set kg = excluded.kg",
                params![weigh_in.date, weigh_in.kg],
            )
            .map(|_| ())
            .map_err(broken)
    }

    fn recent(&self, count: u32) -> Result<Vec<WeighIn>, Broken> {
        self.0
            .prepare("select date, kg from weigh_ins order by date desc limit ?1")
            .and_then(|mut statement| {
                statement
                    .query_map([count], |row| {
                        Ok(WeighIn {
                            date: row.get(0)?,
                            kg: row.get(1)?,
                        })
                    })?
                    .collect()
            })
            .map_err(broken)
    }
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

    fn replace_gather_records(&self, _records: &GatherRecords) -> Result<(), Broken> {
        Err(Broken("not kept yet".into()))
    }

    fn gather_records(&self) -> Result<GatherRecords, Broken> {
        let measurements = self
            .0
            .prepare("select day, body_mass_kg, waist_cm from measurements")
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
                "select effective_from, birthday, gender, height_cm, neck_cm, hip_cm from profiles",
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
            .prepare("select effective_from, body_mass_kg, fat_ratio from goals")
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

#[cfg(test)]
mod tests {
    use super::*;
    use grind_core::testing::honouring_for_storing_gather_records;
    use grind_core::testing::honouring_for_storing_weigh_ins;
    use grind_core::weigh_in::weigh_in;

    fn database() -> Sqlite {
        Sqlite::in_memory().expect("an in-memory database opens and migrates")
    }

    #[test]
    fn recent_weigh_ins_come_newest_first_and_no_more_than_asked() {
        honouring_for_storing_weigh_ins::recent_weigh_ins_come_newest_first_and_no_more_than_asked(
            &database(),
        );
    }

    #[test]
    fn a_day_weighed_twice_keeps_its_second_weigh_in() {
        honouring_for_storing_weigh_ins::a_day_weighed_twice_keeps_its_second_weigh_in(&database());
    }

    #[test]
    fn a_database_opened_again_keeps_its_weigh_ins_and_migrates_nothing_twice() {
        let file = std::env::temp_dir().join(format!(
            "grind-sqlite-opened-again-{}.sqlite",
            std::process::id()
        ));
        let weighed = weigh_in("2026-10-03", 72.1).expect("a valid weigh-in");
        Sqlite::open(&file)
            .and_then(|database| database.save(&weighed))
            .expect("a first opening saves a weigh-in");

        let reopened = Sqlite::open(&file).and_then(|database| database.recent(7));

        std::fs::remove_file(&file).expect("the test database is removed");
        assert_eq!(reopened, Ok(vec![weighed]));
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
}
