use std::path::Path;

use chrono::NaiveDate;
use grind_core::failure::Broken;
use grind_core::for_storing_gather_records::ForStoringGatherRecords;
use grind_core::for_storing_weigh_ins::ForStoringWeighIns;
use grind_core::gather_records::GatherRecords;
use grind_core::goals::Goals;
use grind_core::measurement::Measurement;
use grind_core::profile::Profile;
use grind_core::weigh_in::WeighIn;
use rusqlite::{Connection, params};

const MIGRATIONS: &[&str] = &[
    "create table weigh_ins (date text primary key, kg real not null)",
    "create table measurements (day text primary key, body_mass_kg real not null, waist_cm real);
     insert into measurements (day, body_mass_kg) select date, kg from weigh_ins;",
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

    fn remove_measurement(&self, _day: NaiveDate) -> Result<(), Broken> {
        Err(Broken("not kept yet".into()))
    }

    fn save_profile(&self, _from: NaiveDate, _profile: &Profile) -> Result<(), Broken> {
        Err(Broken("not kept yet".into()))
    }

    fn save_goals(&self, _from: NaiveDate, _goals: &Goals) -> Result<(), Broken> {
        Err(Broken("not kept yet".into()))
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
        Ok(GatherRecords {
            measurements,
            profiles: Vec::new(),
            goals: Vec::new(),
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
}
