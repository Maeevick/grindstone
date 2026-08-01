use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn_records::BurnRecords;
use crate::food::Nutrients;
use crate::fuel_records::FuelRecords;
use crate::gather_records::GatherRecords;
use crate::intake::Intake;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Low,
    InBand,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Band {
    pub minimum_kcal: f64,
    pub floor_kcal: f64,
    pub cap_kcal: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct MacroRow {
    pub eaten_g: f64,
    pub floor_g: f64,
    pub ceiling_g: f64,
    pub verdict: Verdict,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Macros {
    pub protein: MacroRow,
    pub fat: MacroRow,
    pub fibre: MacroRow,
    pub carbs: MacroRow,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fuel {
    pub day: NaiveDate,
    pub intakes: Vec<(i64, Intake)>,
    pub eaten: Nutrients,
    pub band: Option<Band>,
    pub energy: Option<Verdict>,
    pub macros: Option<Macros>,
}

pub fn fuel_over(
    days: &[NaiveDate],
    gather: &GatherRecords,
    burn: &BurnRecords,
    fuel: &FuelRecords,
) -> Vec<Fuel> {
    let _ = (gather, burn, fuel);
    days.iter()
        .map(|day| Fuel {
            day: *day,
            intakes: vec![],
            eaten: Nutrients::default(),
            band: None,
            energy: None,
            macros: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exercise::Exercise;
    use crate::food::{Food, Unit};
    use crate::fuel_records::FuelTargets;
    use crate::gender::Gender;
    use crate::intake::intake;
    use crate::measurement::Measurement;
    use crate::nutrition::{Deficit, Intake as Eaten, Macro, daily_energy_budget};
    use crate::profile::Profile;
    use crate::volitional::Volitional;
    use crate::workout::{burnt_by, workout};

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    const A_DESK_DAY: Volitional = Volitional {
        sitting_min: 480.0,
        standing_min: 120.0,
        moving_km: 3.0,
    };

    const AIMS: FuelTargets = FuelTargets {
        protein_g_per_kg: 2.0,
        fat_g_per_kg: 0.9,
        fibre_g_per_kg: 0.4,
    };

    fn a_mans_records(waist_cm: f64) -> GatherRecords {
        GatherRecords {
            measurements: vec![Measurement {
                day: day("2026-10-01"),
                body_mass_kg: 80.0,
                waist_cm: Some(waist_cm),
            }],
            profiles: vec![(
                day("2026-03-01"),
                Profile {
                    birthday: day("1985-04-12"),
                    gender: Gender::Male,
                    height_cm: 180.0,
                    neck_cm: 38.5,
                    hip_cm: None,
                },
            )],
            goals: vec![],
        }
    }

    fn a_desk_job() -> BurnRecords {
        BurnRecords {
            usual_days: vec![(day("2026-03-01"), A_DESK_DAY)],
            ..BurnRecords::default()
        }
    }

    fn aiming() -> FuelRecords {
        FuelRecords {
            targets: vec![(day("2026-03-01"), AIMS)],
            ..FuelRecords::default()
        }
    }

    fn a_food(protein_g: f64, carbs_g: f64, fat_g: f64, fibre_g: f64) -> Food {
        Food {
            name: "Plate".into(),
            unit: Unit::Piece,
            portion: 1.0,
            nutrients: Nutrients {
                energy_kcal: 4.0 * protein_g + 4.0 * carbs_g + 9.0 * fat_g + 2.0 * fibre_g,
                protein_g,
                carbs_g,
                fat_g,
                fibre_g,
            },
        }
    }

    fn eating(fuel: &mut FuelRecords, date: &str, plate: &Food) {
        let id = fuel.intakes.len() as i64 + 1;
        fuel.intakes
            .push((id, intake(date, plate, 1.0).expect("an intake")));
    }

    fn fuel_on(date: &str, gather: &GatherRecords, burn: &BurnRecords, fuel: &FuelRecords) -> Fuel {
        fuel_over(&[day(date)], gather, burn, fuel)
            .into_iter()
            .next()
            .expect("a fuelled day")
    }

    fn the_body() -> (f64, f64) {
        let composition = crate::composition::body_composition(
            80.0,
            180.0,
            crate::composition::Tapes::Male {
                neck_cm: 38.5,
                waist_cm: 80.0,
            },
        )
        .expect("a composition");
        (composition.lean_mass_kg, composition.fat_mass_kg)
    }

    fn about(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-6
    }

    #[test]
    fn before_anything_is_eaten_the_cap_is_the_budget_priced_on_the_targets_plus_the_burn() {
        let (lean_kg, fat_kg) = the_body();
        let mut burn = a_desk_job();
        let walking = Exercise {
            name: "Walking".into(),
            met: 4.0,
        };
        let walked = workout("2026-10-10", &walking, 60.0, None, 1, 0.0).expect("a workout");
        burn.workouts.push((1, walked.clone()));

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &burn, &aiming());

        let budget = daily_energy_budget(
            41,
            lean_kg,
            fat_kg,
            Gender::Male,
            A_DESK_DAY,
            Deficit {
                trailing_deficit_kcal: 0.0,
                weeks_in_deficit: 0,
            },
            Eaten {
                energy: 0.0,
                macros: Macro {
                    protein: 2.0 * 80.0,
                    fat: 0.9 * 80.0,
                    fib: 28.0,
                    carb: (3.0 * lean_kg).max(130.0),
                },
            },
        );
        let burnt = burnt_by(&walked, 80.0);
        assert!(
            fuelled
                .band
                .is_some_and(|band| about(band.cap_kcal, budget + burnt))
        );
        assert!(fuelled.band.is_some_and(|band| about(
            band.floor_kcal,
            (budget + burnt - 48.5 * fat_kg).max(25.0 * lean_kg + burnt)
        )));
    }

    #[test]
    fn once_eaten_digestion_is_priced_on_the_grams_eaten() {
        let mut fuel = aiming();
        let before = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);
        eating(&mut fuel, "2026-10-10", &a_food(400.0, 300.0, 72.0, 28.0));

        let after = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        let cap = |fuelled: &Fuel| fuelled.band.map(|band| band.cap_kcal).unwrap_or_default();
        assert!(cap(&after) > cap(&before) + 200.0);
    }

    #[test]
    fn a_fat_body_floors_its_band_at_the_lean_mass_minimum_rather_than_the_alpert_limit() {
        let fuelled = fuel_on(
            "2026-10-10",
            &a_mans_records(110.0),
            &a_desk_job(),
            &aiming(),
        );

        assert!(
            fuelled
                .band
                .is_some_and(|band| about(band.floor_kcal, band.minimum_kcal))
        );
    }

    #[test]
    fn without_a_profile_or_a_waist_there_is_no_band_rather_than_a_stand_in_body() {
        let no_profile = GatherRecords {
            profiles: vec![],
            ..a_mans_records(80.0)
        };

        let fuelled = fuel_on("2026-10-10", &no_profile, &a_desk_job(), &aiming());

        assert_eq!((fuelled.band, fuelled.energy), (None, None));
    }

    #[test]
    fn a_day_eaten_inside_its_band_with_each_macro_inside_its_own_reads_in_band() {
        let mut fuel = aiming();
        eating(&mut fuel, "2026-10-10", &a_food(170.0, 230.0, 80.0, 35.0));

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        assert_eq!(fuelled.energy, Some(Verdict::InBand));
        let macros = fuelled.macros.expect("macros");
        assert_eq!(
            [
                macros.protein.verdict,
                macros.fat.verdict,
                macros.fibre.verdict,
                macros.carbs.verdict
            ],
            [Verdict::InBand; 4]
        );
    }

    #[test]
    fn a_day_eaten_under_its_floor_reads_low_and_over_its_cap_reads_high() {
        let mut light = aiming();
        eating(&mut light, "2026-10-10", &a_food(100.0, 50.0, 20.0, 10.0));
        let mut heavy = aiming();
        eating(&mut heavy, "2026-10-10", &a_food(250.0, 600.0, 150.0, 40.0));

        let under = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &light);
        let over = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &heavy);

        assert_eq!(under.energy, Some(Verdict::Low));
        assert_eq!(over.energy, Some(Verdict::High));
    }

    #[test]
    fn the_protein_floor_is_its_target_per_kilo_and_its_ceiling_a_third_of_the_energy_eaten() {
        let mut fuel = aiming();
        eating(&mut fuel, "2026-10-10", &a_food(100.0, 50.0, 20.0, 10.0));

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        let protein = fuelled.macros.expect("macros").protein;
        assert!(about(protein.floor_g, 160.0));
        assert!(about(
            protein.ceiling_g,
            0.333 * fuelled.eaten.energy_kcal / 4.0
        ));
        assert_eq!(protein.verdict, Verdict::Low);
    }

    #[test]
    fn weeks_eaten_well_under_the_cap_wear_the_spontaneous_activity_down() {
        let mut dieting = aiming();
        for date in 1..=28 {
            eating(
                &mut dieting,
                &format!("2026-09-{:02}", date + 2),
                &a_food(150.0, 100.0, 40.0, 20.0),
            );
        }

        let rested = fuel_on(
            "2026-10-01",
            &a_mans_records(80.0),
            &a_desk_job(),
            &aiming(),
        );
        let worn = fuel_on("2026-10-01", &a_mans_records(80.0), &a_desk_job(), &dieting);

        let cap = |fuelled: &Fuel| fuelled.band.map(|band| band.cap_kcal).unwrap_or_default();
        assert!(cap(&worn) < cap(&rested) - 100.0);
    }

    #[test]
    fn a_day_keeps_its_own_intakes_and_adds_up_what_they_brought() {
        let mut fuel = aiming();
        eating(&mut fuel, "2026-10-09", &a_food(50.0, 50.0, 10.0, 5.0));
        eating(&mut fuel, "2026-10-10", &a_food(30.0, 20.0, 10.0, 5.0));
        eating(&mut fuel, "2026-10-10", &a_food(20.0, 30.0, 0.0, 5.0));

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        assert_eq!(fuelled.intakes.len(), 2);
        assert!(about(fuelled.eaten.protein_g, 50.0));
        assert!(about(
            fuelled.eaten.energy_kcal,
            4.0 * 50.0 + 4.0 * 50.0 + 9.0 * 10.0 + 2.0 * 10.0
        ));
    }
}
