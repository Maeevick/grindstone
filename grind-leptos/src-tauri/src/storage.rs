use grind_core::weigh_in::WeighIn;
use rusqlite::{Connection, params};

const MIGRATIONS: &[&str] = &["create table weigh_ins (date text primary key, kg real not null)"];

pub fn migrate(connection: &Connection) -> rusqlite::Result<()> {
    let applied: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (version, migration) in (1_i64..).zip(MIGRATIONS) {
        if version > applied {
            connection.execute_batch(migration)?;
            connection.pragma_update(None, "user_version", version)?;
        }
    }
    Ok(())
}

pub fn save_weigh_in(connection: &Connection, weigh_in: &WeighIn) -> rusqlite::Result<()> {
    connection.execute(
        "insert into weigh_ins (date, kg) values (?1, ?2)
         on conflict (date) do update set kg = excluded.kg",
        params![weigh_in.date, weigh_in.kg],
    )?;
    Ok(())
}

pub fn recent_weigh_ins(connection: &Connection, count: u32) -> rusqlite::Result<Vec<WeighIn>> {
    connection
        .prepare("select date, kg from weigh_ins order by date desc limit ?1")?
        .query_map([count], |row| {
            Ok(WeighIn {
                date: row.get(0)?,
                kg: row.get(1)?,
            })
        })?
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use grind_core::weigh_in::weigh_in;

    fn migrated() -> Connection {
        let connection = Connection::open_in_memory().expect("an in-memory database opens");
        migrate(&connection).expect("a fresh database migrates");
        connection
    }

    fn weighed(date: &str, kg: f64) -> WeighIn {
        weigh_in(date, kg).expect("a valid weigh-in")
    }

    #[test]
    fn recent_weigh_ins_come_newest_first_and_no_more_than_asked() {
        let connection = migrated();
        for (date, kg) in [
            ("2026-10-01", 72.0),
            ("2026-10-03", 72.1),
            ("2026-10-02", 72.2),
        ] {
            save_weigh_in(&connection, &weighed(date, kg)).unwrap();
        }

        assert_eq!(
            recent_weigh_ins(&connection, 2).unwrap(),
            vec![weighed("2026-10-03", 72.1), weighed("2026-10-02", 72.2)]
        );
    }

    #[test]
    fn a_day_weighed_twice_keeps_its_second_weigh_in() {
        let connection = migrated();
        save_weigh_in(&connection, &weighed("2026-10-03", 72.4)).unwrap();

        save_weigh_in(&connection, &weighed("2026-10-03", 72.1)).unwrap();

        assert_eq!(
            recent_weigh_ins(&connection, 7).unwrap(),
            vec![weighed("2026-10-03", 72.1)]
        );
    }

    #[test]
    fn migrating_a_migrated_database_changes_nothing() {
        let connection = migrated();
        save_weigh_in(&connection, &weighed("2026-10-03", 72.1)).unwrap();

        migrate(&connection).unwrap();

        assert_eq!(
            recent_weigh_ins(&connection, 7).unwrap(),
            vec![weighed("2026-10-03", 72.1)]
        );
    }
}
