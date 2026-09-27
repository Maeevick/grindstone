use std::path::Path;

use chrono::NaiveDate;
use grind_core::burn_records::BurnRecords;
use grind_core::circuit::{Circuit, CircuitStep};
use grind_core::drink::{Drink, DrinkKind};
use grind_core::drinking::Drinking;
use grind_core::exercise::Exercise;
use grind_core::failure::Broken;
use grind_core::food::{Food, Nutrients, Unit};
use grind_core::for_storing_burn_records::ForStoringBurnRecords;
use grind_core::for_storing_fuel_records::ForStoringFuelRecords;
use grind_core::for_storing_gather_records::ForStoringGatherRecords;
use grind_core::for_storing_hydrate_records::ForStoringHydrateRecords;
use grind_core::for_storing_switch_over::ForStoringSwitchOver;
use grind_core::fuel_records::{FuelRecords, FuelTargets};
use grind_core::gather_records::GatherRecords;
use grind_core::gender::Gender;
use grind_core::goals::Goals;
use grind_core::hydrate_records::HydrateRecords;
use grind_core::intake::Intake;
use grind_core::meal::{Ingredient, MASS_KEPT, Meal};
use grind_core::measurement::Measurement;
use grind_core::profile::Profile;
use grind_core::volitional::Volitional;
use grind_core::workout::Workout;
use rusqlite::{Connection, params};
use serde::Deserialize;

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
    "create table meals (name text primary key, meal text not null)",
    "create table switch_over (only integer primary key check (only = 1), day text not null)",
    "alter table workouts add column circuit text;
     create table circuit_steps (
         circuit text not null, position integer not null, exercise text not null, met real not null,
         reps integer, minutes real not null, load_kg real not null,
         primary key (circuit, position)
     );",
    "create table drinks (name text primary key, volume_ml real not null);
     create table drinkings (
         id integer primary key, day text not null, drink text not null, volume_ml real not null
     );
     create table hydrate_bases (effective_from text primary key, base_ml real not null);",
    "alter table drinks add column kind text not null default 'Other';
     alter table drinkings add column kind text not null default 'Other';
     alter table drinkings add column servings real not null default 1;
     update drinks set kind = 'Coffee' where name = 'Expresso';
     update drinks set kind = 'Tea' where name = 'Tea';
     update drinks set kind = 'Water' where lower(name) like '%water%' or lower(name) like 'eau%';
     update drinkings
         set kind = (select kind from drinks where drinks.name = drinkings.drink),
             servings = volume_ml / (select volume_ml from drinks where drinks.name = drinkings.drink)
         where drink in (select name from drinks);",
    "update drinks set kind = 'Other' where kind in ('Coffee', 'Tea');
     update drinkings set kind = 'Other' where kind in ('Coffee', 'Tea');",
    "select 1;",
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

    fn circuit_steps_inserted(&self, circuit: &Circuit) -> Result<(), Broken> {
        (1_i64..)
            .zip(&circuit.steps)
            .try_for_each(|(position, step)| {
                self.run(
                    "insert into circuit_steps (circuit, position, exercise, met, reps, minutes, load_kg)
                     values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        circuit.name,
                        position,
                        step.exercise,
                        step.met,
                        step.reps,
                        step.minutes,
                        step.load_kg
                    ],
                )
            })
    }
}

fn step_filed_in_its_circuit(
    mut circuits: Vec<Circuit>,
    (name, step): (String, CircuitStep),
) -> Vec<Circuit> {
    match circuits.last_mut() {
        Some(circuit) if circuit.name == name => circuit.steps.push(step),
        _ => circuits.push(Circuit {
            name,
            description: String::new(),
            steps: vec![step],
        }),
    }
    circuits
}

const EXERCISES: &str = "select name, met from exercises order by name";
const WORKOUTS: &str = "select id, day, exercise, met, minutes, reps, sets, load_kg, circuit from workouts order by day, id";
const DAYS_OFF_THE_CLOCK: &str =
    "select day, sitting_min, standing_min, moving_km from days_off_the_clock order by day";
const USUAL_DAYS: &str = "select effective_from, sitting_min, standing_min, moving_km from usual_days order by effective_from";

fn exercise_read(row: &rusqlite::Row) -> rusqlite::Result<Exercise> {
    Ok(Exercise {
        name: row.get(0)?,
        met: row.get(1)?,
    })
}

fn workout_read(row: &rusqlite::Row) -> rusqlite::Result<(i64, Workout)> {
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
            circuit: row.get(8)?,
        },
    ))
}

const CIRCUIT_STEPS: &str =
    "select circuit, exercise, met, reps, minutes, load_kg from circuit_steps
     order by circuit, position";

fn circuit_step_read(row: &rusqlite::Row) -> rusqlite::Result<(String, CircuitStep)> {
    Ok((
        row.get(0)?,
        CircuitStep {
            exercise: row.get(1)?,
            met: row.get(2)?,
            reps: row.get(3)?,
            minutes: row.get(4)?,
            load_kg: row.get(5)?,
        },
    ))
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
            "insert into workouts (day, exercise, met, minutes, reps, sets, load_kg, circuit)
             values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                workout.day,
                workout.exercise,
                workout.met,
                workout.minutes,
                workout.reps,
                workout.sets,
                workout.load_kg,
                workout.circuit
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
                 delete from days_off_the_clock; delete from usual_days; delete from circuit_steps;",
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
        for circuit in &records.circuits {
            self.circuit_steps_inserted(circuit)?;
        }
        replacing.commit().map_err(broken)
    }

    fn burn_records(&self) -> Result<BurnRecords, Broken> {
        Ok(BurnRecords {
            exercises: self.rows(EXERCISES, exercise_read)?,
            workouts: self.rows(WORKOUTS, workout_read)?,
            days_off_the_clock: self.rows(DAYS_OFF_THE_CLOCK, volitional_read)?,
            usual_days: self.rows(USUAL_DAYS, volitional_read)?,
            circuits: self
                .rows(CIRCUIT_STEPS, circuit_step_read)?
                .into_iter()
                .fold(Vec::new(), step_filed_in_its_circuit),
        })
    }

    fn save_circuit(&self, circuit: &Circuit) -> Result<(), Broken> {
        let saving = self.0.unchecked_transaction().map_err(broken)?;
        self.remove_circuit(&circuit.name)?;
        self.circuit_steps_inserted(circuit)?;
        saving.commit().map_err(broken)
    }

    fn remove_circuit(&self, name: &str) -> Result<(), Broken> {
        self.run(
            "delete from circuit_steps where circuit = ?1",
            params![name],
        )
    }
}

fn unit_word(unit: Unit) -> &'static str {
    match unit {
        Unit::Gram => "g",
        Unit::Millilitre => "mL",
        Unit::Piece => "piece",
    }
}

fn unit_read(word: &str) -> rusqlite::Result<Unit> {
    match word {
        "g" => Ok(Unit::Gram),
        "mL" => Ok(Unit::Millilitre),
        "piece" => Ok(Unit::Piece),
        unknown => Err(rusqlite::Error::InvalidParameterName(format!(
            "a unit the app does not know: {unknown}"
        ))),
    }
}

const FOODS: &str = "select name, unit, portion, energy_kcal, protein_g, carbs_g, fat_g, fibre_g from foods order by name";
const INTAKES: &str =
    "select id, day, food, unit, quantity, energy_kcal, protein_g, carbs_g, fat_g, fibre_g
     from intakes order by day, id";
const FUEL_TARGETS: &str = "select effective_from, protein_g_per_kg, fat_g_per_kg, fibre_g_per_kg
     from fuel_targets order by effective_from";
const MEALS: &str = "select meal from meals order by name";

fn food_read(row: &rusqlite::Row) -> rusqlite::Result<Food> {
    Ok(Food {
        name: row.get(0)?,
        unit: unit_read(&row.get::<_, String>(1)?)?,
        portion: row.get(2)?,
        nutrients: nutrients_read(row, 3)?,
    })
}

fn intake_read(row: &rusqlite::Row) -> rusqlite::Result<(i64, Intake)> {
    Ok((
        row.get(0)?,
        Intake {
            day: row.get(1)?,
            food: row.get(2)?,
            unit: unit_read(&row.get::<_, String>(3)?)?,
            quantity: row.get(4)?,
            nutrients: nutrients_read(row, 5)?,
        },
    ))
}

fn targets_read(row: &rusqlite::Row) -> rusqlite::Result<(NaiveDate, FuelTargets)> {
    Ok((
        row.get(0)?,
        FuelTargets {
            protein_g_per_kg: row.get(1)?,
            fat_g_per_kg: row.get(2)?,
            fibre_g_per_kg: row.get(3)?,
        },
    ))
}

/// A meal as kept: those cooked before each ingredient had its own ratio spread the pot's over them.
#[derive(Deserialize)]
struct KeptMeal {
    name: String,
    recipe: String,
    ingredients: Vec<KeptIngredient>,
    raw_g: f64,
    cooked_g: f64,
    totals: Nutrients,
}

#[derive(Deserialize)]
struct KeptIngredient {
    food: String,
    unit: Unit,
    quantity: f64,
    cooked_per_raw: Option<f64>,
    nutrients: Nutrients,
}

fn meal_read(row: &rusqlite::Row) -> rusqlite::Result<Meal> {
    let kept: KeptMeal = serde_json::from_str(&row.get::<_, String>(0)?)
        .map_err(|error| rusqlite::Error::InvalidParameterName(error.to_string()))?;
    let pot_ratio = if kept.raw_g > 0.0 {
        kept.cooked_g / kept.raw_g
    } else {
        MASS_KEPT
    };
    Ok(Meal {
        name: kept.name,
        recipe: kept.recipe,
        ingredients: kept
            .ingredients
            .into_iter()
            .map(|ingredient| Ingredient {
                food: ingredient.food,
                unit: ingredient.unit,
                quantity: ingredient.quantity,
                cooked_per_raw: ingredient.cooked_per_raw.unwrap_or(pot_ratio),
                nutrients: ingredient.nutrients,
            })
            .collect(),
        raw_g: kept.raw_g,
        cooked_g: kept.cooked_g,
        totals: kept.totals,
    })
}

fn nutrients_read(row: &rusqlite::Row, first: usize) -> rusqlite::Result<Nutrients> {
    Ok(Nutrients {
        energy_kcal: row.get(first)?,
        protein_g: row.get(first + 1)?,
        carbs_g: row.get(first + 2)?,
        fat_g: row.get(first + 3)?,
        fibre_g: row.get(first + 4)?,
    })
}

impl ForStoringFuelRecords for Sqlite {
    fn save_food(&self, food: &Food) -> Result<(), Broken> {
        let figures = food.nutrients;
        self.run(
            "insert into foods (name, unit, portion, energy_kcal, protein_g, carbs_g, fat_g, fibre_g)
             values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             on conflict (name) do update set unit = excluded.unit, portion = excluded.portion,
                 energy_kcal = excluded.energy_kcal, protein_g = excluded.protein_g,
                 carbs_g = excluded.carbs_g, fat_g = excluded.fat_g, fibre_g = excluded.fibre_g",
            params![
                food.name,
                unit_word(food.unit),
                food.portion,
                figures.energy_kcal,
                figures.protein_g,
                figures.carbs_g,
                figures.fat_g,
                figures.fibre_g
            ],
        )
    }

    fn remove_food(&self, name: &str) -> Result<(), Broken> {
        self.run("delete from foods where name = ?1", params![name])
    }

    fn save_intake(&self, intake: &Intake) -> Result<(), Broken> {
        let figures = intake.nutrients;
        self.run(
            "insert into intakes (day, food, unit, quantity, energy_kcal, protein_g, carbs_g, fat_g, fibre_g)
             values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                intake.day,
                intake.food,
                unit_word(intake.unit),
                intake.quantity,
                figures.energy_kcal,
                figures.protein_g,
                figures.carbs_g,
                figures.fat_g,
                figures.fibre_g
            ],
        )
    }

    fn remove_intake(&self, id: i64) -> Result<(), Broken> {
        self.run("delete from intakes where id = ?1", params![id])
    }

    fn save_targets(&self, from: NaiveDate, targets: &FuelTargets) -> Result<(), Broken> {
        self.run(
            "insert into fuel_targets (effective_from, protein_g_per_kg, fat_g_per_kg, fibre_g_per_kg)
             values (?1, ?2, ?3, ?4)
             on conflict (effective_from) do update set protein_g_per_kg = excluded.protein_g_per_kg,
                 fat_g_per_kg = excluded.fat_g_per_kg, fibre_g_per_kg = excluded.fibre_g_per_kg",
            params![
                from,
                targets.protein_g_per_kg,
                targets.fat_g_per_kg,
                targets.fibre_g_per_kg
            ],
        )
    }

    fn save_meal(&self, meal: &Meal) -> Result<(), Broken> {
        let written = serde_json::to_string(meal).map_err(|error| Broken(error.to_string()))?;
        self.run(
            "insert into meals (name, meal) values (?1, ?2)
             on conflict (name) do update set meal = excluded.meal",
            params![meal.name, written],
        )
    }

    fn remove_meal(&self, name: &str) -> Result<(), Broken> {
        self.run("delete from meals where name = ?1", params![name])
    }

    fn replace_fuel_records(&self, records: &FuelRecords) -> Result<(), Broken> {
        let replacing = self.0.unchecked_transaction().map_err(broken)?;
        replacing
            .execute_batch(
                "delete from foods; delete from intakes; delete from fuel_targets; delete from meals;",
            )
            .map_err(broken)?;
        for food in &records.foods {
            self.save_food(food)?;
        }
        for (_, intake) in &records.intakes {
            self.save_intake(intake)?;
        }
        for (from, targets) in &records.targets {
            self.save_targets(*from, targets)?;
        }
        for meal in &records.meals {
            self.save_meal(meal)?;
        }
        replacing.commit().map_err(broken)
    }

    fn fuel_records(&self) -> Result<FuelRecords, Broken> {
        Ok(FuelRecords {
            foods: self.rows(FOODS, food_read)?,
            intakes: self.rows(INTAKES, intake_read)?,
            targets: self.rows(FUEL_TARGETS, targets_read)?,
            meals: self.rows(MEALS, meal_read)?,
        })
    }
}

impl ForStoringSwitchOver for Sqlite {
    fn save_switch_over(&self, day: NaiveDate) -> Result<(), Broken> {
        self.run(
            "insert into switch_over (only, day) values (1, ?1)
             on conflict (only) do update set day = excluded.day",
            params![day],
        )
    }

    fn switch_over(&self) -> Result<Option<NaiveDate>, Broken> {
        Ok(self
            .rows("select day from switch_over", |row| row.get(0))?
            .into_iter()
            .next())
    }

    fn forget_switch_over(&self) -> Result<(), Broken> {
        self.run("delete from switch_over", [])
    }
}

const DRINKS: &str = "select name, volume_ml, kind from drinks order by name";
const DRINKINGS: &str =
    "select id, day, drink, volume_ml, kind, servings from drinkings order by day, id";

fn kind_word(kind: DrinkKind) -> &'static str {
    match kind {
        DrinkKind::Water => "Water",
        DrinkKind::Other => "Other",
    }
}

fn kind_read(word: &str) -> rusqlite::Result<DrinkKind> {
    match word {
        "Water" => Ok(DrinkKind::Water),
        "Other" => Ok(DrinkKind::Other),
        unknown => Err(rusqlite::Error::InvalidParameterName(format!(
            "a kind of drink the app does not know: {unknown}"
        ))),
    }
}
const HYDRATE_BASES: &str =
    "select effective_from, base_ml from hydrate_bases order by effective_from";

fn drink_read(row: &rusqlite::Row) -> rusqlite::Result<Drink> {
    Ok(Drink {
        name: row.get(0)?,
        volume_ml: row.get(1)?,
        kind: kind_read(&row.get::<_, String>(2)?)?,
    })
}

fn drinking_read(row: &rusqlite::Row) -> rusqlite::Result<(i64, Drinking)> {
    Ok((
        row.get(0)?,
        Drinking {
            day: row.get(1)?,
            drink: row.get(2)?,
            volume_ml: row.get(3)?,
            kind: kind_read(&row.get::<_, String>(4)?)?,
            servings: row.get(5)?,
        },
    ))
}

fn base_read(row: &rusqlite::Row) -> rusqlite::Result<(NaiveDate, f64)> {
    Ok((row.get(0)?, row.get(1)?))
}

impl ForStoringHydrateRecords for Sqlite {
    fn save_drink(&self, drink: &Drink) -> Result<(), Broken> {
        self.run(
            "insert into drinks (name, volume_ml, kind) values (?1, ?2, ?3)
             on conflict (name) do update set volume_ml = excluded.volume_ml, kind = excluded.kind",
            params![drink.name, drink.volume_ml, kind_word(drink.kind)],
        )
    }

    fn remove_drink(&self, name: &str) -> Result<(), Broken> {
        self.run("delete from drinks where name = ?1", params![name])
    }

    fn save_drinking(&self, drinking: &Drinking) -> Result<(), Broken> {
        self.run(
            "insert into drinkings (day, drink, volume_ml, kind, servings) values (?1, ?2, ?3, ?4, ?5)",
            params![
                drinking.day,
                drinking.drink,
                drinking.volume_ml,
                kind_word(drinking.kind),
                drinking.servings
            ],
        )
    }

    fn remove_drinking(&self, id: i64) -> Result<(), Broken> {
        self.run("delete from drinkings where id = ?1", params![id])
    }

    fn save_base(&self, from: NaiveDate, base_ml: f64) -> Result<(), Broken> {
        self.run(
            "insert into hydrate_bases (effective_from, base_ml) values (?1, ?2)
             on conflict (effective_from) do update set base_ml = excluded.base_ml",
            params![from, base_ml],
        )
    }

    fn replace_hydrate_records(&self, records: &HydrateRecords) -> Result<(), Broken> {
        let replacing = self.0.unchecked_transaction().map_err(broken)?;
        replacing
            .execute_batch("delete from drinks; delete from drinkings; delete from hydrate_bases;")
            .map_err(broken)?;
        for drink in &records.drinks {
            self.save_drink(drink)?;
        }
        for (_, drinking) in &records.drinkings {
            self.save_drinking(drinking)?;
        }
        for (from, base_ml) in &records.bases {
            self.save_base(*from, *base_ml)?;
        }
        replacing.commit().map_err(broken)
    }

    fn hydrate_records(&self) -> Result<HydrateRecords, Broken> {
        Ok(HydrateRecords {
            drinks: self.rows(DRINKS, drink_read)?,
            drinkings: self.rows(DRINKINGS, drinking_read)?,
            bases: self.rows(HYDRATE_BASES, base_read)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BEFORE_THE_DRINK_KINDS: usize = 11;
    const BEFORE_THE_TWO_DRINK_KINDS: usize = 12;
    const BEFORE_THE_CIRCUIT_DESCRIPTIONS: usize = 13;
    use grind_core::testing::honouring_for_storing_burn_records;
    use grind_core::testing::honouring_for_storing_fuel_records;
    use grind_core::testing::honouring_for_storing_gather_records;
    use grind_core::testing::honouring_for_storing_hydrate_records;
    use grind_core::testing::honouring_for_storing_switch_over;

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

    #[test]
    fn foods_and_intakes_keep_their_unit_whatever_it_is() {
        let records = database();
        for (name, unit) in [
            ("Skyr", Unit::Gram),
            ("Beer", Unit::Millilitre),
            ("Egg", Unit::Piece),
        ] {
            records
                .save_food(&Food {
                    name: name.into(),
                    unit,
                    portion: 1.0,
                    nutrients: Nutrients::default(),
                })
                .expect("a food saves");
        }

        let units: Vec<Unit> = records
            .fuel_records()
            .expect("the records read")
            .foods
            .iter()
            .map(|food| food.unit)
            .collect();

        assert_eq!(units, vec![Unit::Millilitre, Unit::Piece, Unit::Gram]);
    }

    #[test]
    fn a_food_whose_unit_the_app_does_not_know_reads_as_broken() {
        let records = database();
        records
            .0
            .execute(
                "insert into foods values ('Skyr', 'spoon', 1.0, 0.0, 0.0, 0.0, 0.0, 0.0)",
                [],
            )
            .expect("a food no app wrote");

        assert!(matches!(records.fuel_records(), Err(Broken(reason)) if reason.contains("spoon")));
    }

    #[test]
    fn meals_come_back_by_name_a_meal_cooked_again_replaces_the_earlier_and_a_removed_one_leaves() {
        honouring_for_storing_fuel_records::meals_come_back_by_name_a_meal_cooked_again_replaces_the_earlier_and_a_removed_one_leaves(
            &database(),
        );
    }

    #[test]
    fn a_meal_the_app_cannot_read_reads_as_broken() {
        let records = database();
        records
            .0
            .execute("insert into meals values ('Stew', 'not a meal')", [])
            .expect("a meal no app wrote");

        assert!(matches!(records.fuel_records(), Err(Broken(_))));
    }

    #[test]
    fn no_switch_over_until_one_is_saved_and_the_latest_one_stands() {
        honouring_for_storing_switch_over::no_switch_over_until_one_is_saved_and_the_latest_one_stands(
            &database(),
        );
    }

    #[test]
    fn crafted_circuits_come_back_by_name_with_their_steps_in_order() {
        honouring_for_storing_burn_records::crafted_circuits_come_back_by_name_with_their_steps_in_order(
            &database(),
        );
    }

    #[test]
    fn removing_a_circuit_leaves_the_others() {
        honouring_for_storing_burn_records::removing_a_circuit_leaves_the_others(&database());
    }

    #[test]
    fn a_circuit_crafted_again_under_its_name_replaces_the_earlier_one() {
        honouring_for_storing_burn_records::a_circuit_crafted_again_under_its_name_replaces_the_earlier_one(
            &database(),
        );
    }

    #[test]
    fn a_workout_done_in_a_circuit_comes_back_with_its_name() {
        honouring_for_storing_burn_records::a_workout_done_in_a_circuit_comes_back_with_its_name(
            &database(),
        );
    }

    #[test]
    fn a_circuit_that_cannot_be_kept_whole_leaves_the_earlier_one_as_it_was() {
        let records = database();
        let step = |exercise: &str, minutes: f64| CircuitStep {
            exercise: exercise.into(),
            met: 3.8,
            reps: Some(12),
            minutes,
            load_kg: 0.0,
        };
        let kept = Circuit {
            name: "Morning rust".into(),
            description: "Found on YouTube.".into(),
            steps: vec![step("Push up", 0.5)],
        };
        records.save_circuit(&kept).expect("a circuit saves");
        let half_valid = Circuit {
            steps: vec![step("Squat", 1.0), step("Burpee", f64::NAN)],
            ..kept.clone()
        };

        assert!(records.save_circuit(&half_valid).is_err());
        assert_eq!(
            records.burn_records().map(|burnt| burnt.circuits),
            Ok(vec![kept])
        );
    }

    #[test]
    fn hydrate_records_come_back_in_the_order_of_their_days() {
        honouring_for_storing_hydrate_records::hydrate_records_come_back_in_the_order_of_their_days(
            &database(),
        );
    }

    #[test]
    fn a_drink_or_a_base_saved_again_under_its_name_or_day_replaces_the_earlier_one() {
        honouring_for_storing_hydrate_records::a_drink_or_a_base_saved_again_under_its_name_or_day_replaces_the_earlier_one(
            &database(),
        );
    }

    #[test]
    fn removing_a_drinking_or_a_drink_leaves_the_others() {
        honouring_for_storing_hydrate_records::removing_a_drinking_or_a_drink_leaves_the_others(
            &database(),
        );
    }

    #[test]
    fn replacing_the_hydrate_records_leaves_only_the_new_ones() {
        honouring_for_storing_hydrate_records::replacing_the_hydrate_records_leaves_only_the_new_ones(
            &database(),
        );
    }

    #[test]
    fn drinks_and_drinkings_keep_their_kind_whatever_it_is() {
        let records = database();
        for (name, kind) in [
            ("Water", DrinkKind::Water),
            ("Expresso", DrinkKind::Other),
            ("Tea", DrinkKind::Other),
            ("Kombucha", DrinkKind::Other),
        ] {
            let drunk = Drink {
                name: name.into(),
                volume_ml: 250.0,
                kind,
            };
            records.save_drink(&drunk).expect("a drink saves");
            records
                .save_drinking(&Drinking {
                    day: NaiveDate::from_ymd_opt(2026, 10, 10).expect("a real day"),
                    drink: name.into(),
                    kind,
                    servings: 2.0,
                    volume_ml: 500.0,
                })
                .expect("a drinking saves");
        }

        let kept = records.hydrate_records().expect("the records read");

        assert_eq!(
            kept.drinks
                .iter()
                .map(|drink| drink.kind)
                .collect::<Vec<_>>(),
            vec![
                DrinkKind::Other,
                DrinkKind::Other,
                DrinkKind::Other,
                DrinkKind::Water
            ]
        );
        assert_eq!(
            kept.drinkings
                .iter()
                .map(|(_, drunk)| (drunk.kind, drunk.servings))
                .collect::<Vec<_>>(),
            vec![
                (DrinkKind::Water, 2.0),
                (DrinkKind::Other, 2.0),
                (DrinkKind::Other, 2.0),
                (DrinkKind::Other, 2.0)
            ]
        );
    }

    #[test]
    fn a_drink_whose_kind_the_app_does_not_know_reads_as_broken() {
        let records = database();
        records
            .0
            .execute(
                "insert into drinks (name, volume_ml, kind) values ('Mead', 250.0, 'Potion')",
                [],
            )
            .expect("a drink no app wrote");

        assert!(
            matches!(records.hydrate_records(), Err(Broken(reason)) if reason.contains("Potion"))
        );
    }

    #[test]
    fn the_drinks_already_on_the_phone_take_their_kind_and_their_drinkings_their_servings() {
        let file = std::env::temp_dir().join(format!(
            "grind-sqlite-drinks-carried-{}.sqlite",
            std::process::id()
        ));
        let before = Connection::open(&file).expect("a database before the drinks' kinds");
        for (version, migration) in (1_i64..).zip(&MIGRATIONS[..BEFORE_THE_DRINK_KINDS]) {
            before
                .execute_batch(migration)
                .expect("an earlier migration");
            before
                .pragma_update(None, "user_version", version)
                .expect("its version");
        }
        before
            .execute_batch(
                "insert into drinks (name, volume_ml) values ('Expresso', 80.0), ('Tea', 200.0), ('Water', 250.0);
                 insert into drinkings (day, drink, volume_ml) values
                     ('2026-10-10', 'Expresso', 160.0), ('2026-10-10', 'Tea', 200.0),
                     ('2026-10-10', 'Water', 500.0), ('2026-10-10', 'Kombucha', 330.0);",
            )
            .expect("the drinks filed before");
        drop(before);

        let carried = Sqlite::open(&file).and_then(|database| database.hydrate_records());

        std::fs::remove_file(&file).expect("the test database is removed");
        let carried = carried.expect("the records read");
        assert_eq!(
            carried
                .drinks
                .iter()
                .map(|drink| (drink.name.as_str(), drink.kind))
                .collect::<Vec<_>>(),
            vec![
                ("Expresso", DrinkKind::Other),
                ("Tea", DrinkKind::Other),
                ("Water", DrinkKind::Water)
            ]
        );
        assert_eq!(
            carried
                .drinkings
                .iter()
                .map(|(_, drunk)| (drunk.drink.as_str(), drunk.kind, drunk.servings))
                .collect::<Vec<_>>(),
            vec![
                ("Expresso", DrinkKind::Other, 2.0),
                ("Tea", DrinkKind::Other, 1.0),
                ("Water", DrinkKind::Water, 2.0),
                ("Kombucha", DrinkKind::Other, 1.0)
            ]
        );
    }

    #[test]
    fn a_forgotten_switch_over_leaves_none() {
        honouring_for_storing_switch_over::a_forgotten_switch_over_leaves_none(&database());
    }

    #[test]
    fn the_coffees_and_teas_already_on_the_phone_become_other_drinks() {
        let file = std::env::temp_dir().join(format!(
            "grind-sqlite-two-kinds-{}.sqlite",
            std::process::id()
        ));
        let before = Connection::open(&file).expect("a database before the two kinds");
        for (version, migration) in (1_i64..).zip(&MIGRATIONS[..BEFORE_THE_TWO_DRINK_KINDS]) {
            before
                .execute_batch(migration)
                .expect("an earlier migration");
            before
                .pragma_update(None, "user_version", version)
                .expect("its version");
        }
        before
            .execute_batch(
                "insert into drinks (name, volume_ml, kind) values
                     ('Expresso', 80.0, 'Coffee'), ('Tea', 200.0, 'Tea'), ('Water', 250.0, 'Water');
                 insert into drinkings (day, drink, volume_ml, kind, servings) values
                     ('2026-10-10', 'Expresso', 160.0, 'Coffee', 2.0), ('2026-10-10', 'Tea', 200.0, 'Tea', 1.0),
                     ('2026-10-10', 'Water', 500.0, 'Water', 2.0);",
            )
            .expect("the drinks filed before");
        drop(before);

        let carried = Sqlite::open(&file).and_then(|database| database.hydrate_records());

        std::fs::remove_file(&file).expect("the test database is removed");
        let carried = carried.expect("the records read");
        assert_eq!(
            carried
                .drinks
                .iter()
                .map(|drink| (drink.name.as_str(), drink.kind))
                .collect::<Vec<_>>(),
            vec![
                ("Expresso", DrinkKind::Other),
                ("Tea", DrinkKind::Other),
                ("Water", DrinkKind::Water)
            ]
        );
        assert_eq!(
            carried
                .drinkings
                .iter()
                .map(|(_, drunk)| (drunk.drink.as_str(), drunk.kind))
                .collect::<Vec<_>>(),
            vec![
                ("Expresso", DrinkKind::Other),
                ("Tea", DrinkKind::Other),
                ("Water", DrinkKind::Water)
            ]
        );
    }

    #[test]
    fn a_meal_kept_before_the_cooking_ratios_spreads_its_pot_ratio_over_its_ingredients() {
        let records = database();
        records
            .0
            .execute(
                r#"insert into meals values ('Stew', '{"name": "Stew", "recipe": "Simmer.",
                    "ingredients": [{"food": "Rice", "unit": "Gram", "quantity": 200.0,
                        "nutrients": {"energy_kcal": 700.0, "protein_g": 14.0, "carbs_g": 150.0, "fat_g": 1.0, "fibre_g": 2.0}}],
                    "raw_g": 200.0, "cooked_g": 500.0,
                    "totals": {"energy_kcal": 700.0, "protein_g": 14.0, "carbs_g": 150.0, "fat_g": 1.0, "fibre_g": 2.0}}')"#,
                [],
            )
            .expect("a meal kept by the earlier cooker");

        let kept = records.fuel_records().expect("the records read").meals;

        assert_eq!(
            kept.iter()
                .map(|meal| (meal.cooked_g, meal.ingredients[0].cooked_per_raw))
                .collect::<Vec<_>>(),
            vec![(500.0, 2.5)]
        );
    }

    #[test]
    fn a_meal_of_pieces_kept_before_the_cooking_ratios_keeps_its_ingredients_at_their_own_mass() {
        let records = database();
        records
            .0
            .execute(
                r#"insert into meals values ('Omelette', '{"name": "Omelette", "recipe": "",
                    "ingredients": [{"food": "Egg", "unit": "Piece", "quantity": 3.0,
                        "nutrients": {"energy_kcal": 210.0, "protein_g": 18.0, "carbs_g": 0.0, "fat_g": 15.0, "fibre_g": 0.0}}],
                    "raw_g": 0.0, "cooked_g": 150.0,
                    "totals": {"energy_kcal": 210.0, "protein_g": 18.0, "carbs_g": 0.0, "fat_g": 15.0, "fibre_g": 0.0}}')"#,
                [],
            )
            .expect("a meal kept by the earlier cooker");

        let kept = records.fuel_records().expect("the records read").meals;

        assert_eq!(kept[0].ingredients[0].cooked_per_raw, 1.0);
    }

    #[test]
    fn the_circuits_already_on_the_phone_keep_their_steps_with_no_description_yet() {
        let file = std::env::temp_dir().join(format!(
            "grind-sqlite-circuit-descriptions-{}.sqlite",
            std::process::id()
        ));
        let before = Connection::open(&file).expect("a database before the descriptions");
        for (version, migration) in (1_i64..).zip(&MIGRATIONS[..BEFORE_THE_CIRCUIT_DESCRIPTIONS]) {
            before
                .execute_batch(migration)
                .expect("an earlier migration");
            before
                .pragma_update(None, "user_version", version)
                .expect("its version");
        }
        before
            .execute_batch(
                "insert into circuit_steps (circuit, position, exercise, met, reps, minutes, load_kg)
                 values ('Morning rust', 1, 'Push up', 3.8, 12, 0.5, 0.0);",
            )
            .expect("a circuit crafted before");
        drop(before);

        let carried = Sqlite::open(&file).and_then(|database| database.burn_records());

        std::fs::remove_file(&file).expect("the test database is removed");
        let carried = carried.expect("the records read");
        assert_eq!(
            carried
                .circuits
                .iter()
                .map(|circuit| (
                    circuit.name.as_str(),
                    circuit.description.as_str(),
                    circuit.steps.len()
                ))
                .collect::<Vec<_>>(),
            vec![("Morning rust", "", 1)]
        );
    }
}
