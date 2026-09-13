use chrono::{Days, NaiveDate};
use serde::{Deserialize, Serialize};

use crate::age::age_on;
use crate::burn::burn_on;
use crate::burn_records::BurnRecords;
use crate::food::{Nutrients, summed};
use crate::fuel_records::{DEFAULT_FUEL_TARGETS, FuelRecords, FuelTargets};
use crate::gather_records::GatherRecords;
use crate::gender::Gender;
use crate::intake::Intake;
use crate::ledger::{Day, ledger};
use crate::nutrition::{Budget, Deficit, Intake as Eaten, Macro, daily_energy_budget};
use crate::profile::Profile;
pub use crate::verdict::Verdict;
use crate::verdict::verdict_of;
use crate::version::version_in_force_on;
use crate::volitional::Volitional;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Band {
    pub minimum_kcal_per_kg_of_lean_mass: f64,
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

impl MacroRow {
    /// What was eaten beyond the goal, which is the floor; short of it, the gap is negative.
    pub fn gap_g(&self) -> f64 {
        self.eaten_g - self.floor_g
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnergyShares {
    pub protein: f64,
    pub fat: f64,
    pub fibre: f64,
    pub carbs: f64,
    pub total: f64,
}

/// Each macro's share of the energy eaten, at 4 kcal a gram of protein and of carbs, 9 of fat
/// and 2 of fibre; their total tells how much of that energy the four account for.
pub fn energy_shares(eaten: &Nutrients) -> Option<EnergyShares> {
    let energy = eaten.energy_kcal;
    (energy > 0.0).then(|| {
        let protein = eaten.protein_g * PROTEIN_KCAL_PER_G / energy;
        let fat = eaten.fat_g * FAT_KCAL_PER_G / energy;
        let fibre = eaten.fibre_g * FIBRE_KCAL_PER_G / energy;
        let carbs = eaten.carbs_g * CARBS_KCAL_PER_G / energy;
        EnergyShares {
            protein,
            fat,
            fibre,
            carbs,
            total: protein + fat + fibre + carbs,
        }
    })
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
    pub budget: Option<Budget>,
}

const PROTEIN_KCAL_PER_G: f64 = 4.0;
const CARBS_KCAL_PER_G: f64 = 4.0;
const FAT_KCAL_PER_G: f64 = 9.0;
const FIBRE_KCAL_PER_G: f64 = 2.0;
const SPARED_KCAL_PER_KG_OF_FAT: f64 = 48.5;
const FIBRE_BEFORE_EATING_G: f64 = 28.0;
const CARBS_FLOOR_G: f64 = 130.0;
const CARBS_FLOOR_G_PER_KG_OF_LEAN_MASS: f64 = 3.0;
const DAYS_OF_ADAPTATION: u64 = 28;

struct Body {
    mass_kg: f64,
    lean_kg: f64,
    fat_kg: f64,
    age: u32,
    gender: Gender,
}

fn body_on(day: NaiveDate, ledger: &[Day], profiles: &[(NaiveDate, Profile)]) -> Option<Body> {
    let profile = version_in_force_on(profiles, day)?;
    let so_far = || ledger.iter().filter(move |measured| measured.day <= day);
    let weighed = so_far().next_back().or(ledger.first())?;
    let lean_kg = so_far()
        .filter_map(|measured| measured.composition)
        .next_back()
        .or_else(|| ledger.iter().find_map(|measured| measured.composition))?
        .lean_mass_kg;
    Some(Body {
        mass_kg: weighed.body_mass_kg,
        lean_kg,
        fat_kg: (weighed.body_mass_kg - lean_kg).max(0.0),
        age: age_on(profile.birthday, day),
        gender: profile.gender,
    })
}

fn added_up(intakes: &[(i64, Intake)]) -> Nutrients {
    summed(intakes.iter().map(|(_, eaten)| eaten.nutrients))
}

fn carbs_floor_g(body: &Body) -> f64 {
    (CARBS_FLOOR_G_PER_KG_OF_LEAN_MASS * body.lean_kg).max(CARBS_FLOOR_G)
}

fn digested(eaten: Option<Nutrients>, body: &Body, targets: FuelTargets) -> Eaten {
    let macros = match eaten {
        Some(eaten) => Macro {
            protein: eaten.protein_g,
            fat: eaten.fat_g,
            fib: eaten.fibre_g,
            carb: eaten.carbs_g,
        },
        None => Macro {
            protein: targets.protein_g_per_kg * body.mass_kg,
            fat: targets.fat_g_per_kg * body.mass_kg,
            fib: FIBRE_BEFORE_EATING_G,
            carb: carbs_floor_g(body),
        },
    };
    Eaten {
        energy: eaten.map_or(0.0, |eaten| eaten.energy_kcal),
        macros,
    }
}

/// The day's band: the budget plus the burn caps it; the floor is what the fat store can spare
/// below the cap, never under a minimum per kilo of lean mass plus the burn.
///
/// - What the fat store can spare, about 48.5 kcal per kilo of fat a day: Alpert S. S. (2005),
///   *A limit on the energy transfer rate from the human fat store in hypophagia*, J Theor Biol 233(1):1-13.
/// - The minimum per kilo of lean mass: Loucks A. B., Kiens B., Wright H. H. (2011),
///   *Energy availability in athletes*, J Sports Sci 29(sup1):S7-S15.
fn band_for(
    body: &Body,
    burnt_kcal: f64,
    off_the_clock: Volitional,
    deficit: Deficit,
    digested: Eaten,
) -> (Band, Budget) {
    let minimum_kcal_per_kg_of_lean_mass = match body.gender {
        Gender::Male => 25.0,
        Gender::Female => 30.0,
    };
    let minimum_kcal = minimum_kcal_per_kg_of_lean_mass * body.lean_kg + burnt_kcal;
    let budget = daily_energy_budget(
        body.age,
        body.lean_kg,
        body.fat_kg,
        body.gender,
        off_the_clock,
        deficit,
        digested,
    );
    let cap_kcal = (budget.kcal() + burnt_kcal).max(minimum_kcal);
    let band = Band {
        minimum_kcal_per_kg_of_lean_mass: 0.0,
        minimum_kcal,
        floor_kcal: (cap_kcal - SPARED_KCAL_PER_KG_OF_FAT * body.fat_kg).max(minimum_kcal),
        cap_kcal,
    };
    (band, budget)
}

fn mean(gaps: &[f64]) -> Option<f64> {
    (!gaps.is_empty()).then(|| gaps.iter().sum::<f64>() / gaps.len() as f64)
}

fn deficit_before(day: NaiveDate, gaps: &[(NaiveDate, f64)]) -> Deficit {
    let between = |nearest: u64, furthest: u64| -> Vec<f64> {
        gaps.iter()
            .filter(|(logged, _)| {
                (day - Days::new(furthest)..=day - Days::new(nearest)).contains(logged)
            })
            .map(|(_, gap)| *gap)
            .collect()
    };
    let weeks_in_deficit = (0..DAYS_OF_ADAPTATION / 7)
        .take_while(|week| mean(&between(1 + 7 * week, 7 + 7 * week)).is_some_and(|gap| gap > 0.0))
        .count();
    Deficit {
        trailing_deficit_kcal: mean(&between(1, DAYS_OF_ADAPTATION)).unwrap_or(0.0),
        weeks_in_deficit: weeks_in_deficit as u32,
    }
}

fn row(eaten_g: f64, floor_g: f64, ceiling_g: f64) -> MacroRow {
    MacroRow {
        eaten_g,
        floor_g,
        ceiling_g,
        verdict: verdict_of(eaten_g, floor_g, ceiling_g),
    }
}

/// Each macro's floor and ceiling for the day.
///
/// - Protein and fat floors from the targets per kilo, each at least 20% of the energy eaten and at most 33.3%,
///   within EFSA's 20 to 35% for fat (2010) and the ISSN's 1.4 to 2.0 g/kg for protein: Jäger R. et al. (2017),
///   J Int Soc Sports Nutr 14:20.
/// - Fibre between 2.4% and 4% of the energy eaten, after the Institute of Medicine's 14 g per 1000 kcal (2005).
/// - Carbohydrates fill the band, never under 3 g per kilo of lean mass or the Institute of Medicine's 130 g (2005).
fn macros_for(eaten: Nutrients, body: &Body, targets: FuelTargets, band: Band) -> Macros {
    let energy = eaten.energy_kcal;
    let protein = row(
        eaten.protein_g,
        (targets.protein_g_per_kg * body.mass_kg).max(0.20 * energy / PROTEIN_KCAL_PER_G),
        0.333 * energy / PROTEIN_KCAL_PER_G,
    );
    let fat = row(
        eaten.fat_g,
        (targets.fat_g_per_kg * body.mass_kg).max(0.20 * energy / FAT_KCAL_PER_G),
        0.333 * energy / FAT_KCAL_PER_G,
    );
    let fibre = row(
        eaten.fibre_g,
        (targets.fibre_g_per_kg * body.mass_kg).max(0.024 * energy / FIBRE_KCAL_PER_G),
        0.04 * energy / FIBRE_KCAL_PER_G,
    );
    let spent_kcal = protein.floor_g.max(eaten.protein_g) * PROTEIN_KCAL_PER_G
        + fat.floor_g.max(eaten.fat_g) * FAT_KCAL_PER_G
        + fibre.floor_g.max(eaten.fibre_g) * FIBRE_KCAL_PER_G;
    let carbs_floor = ((band.floor_kcal - spent_kcal) / CARBS_KCAL_PER_G).max(carbs_floor_g(body));
    let carbs_ceiling = ((band.cap_kcal - spent_kcal) / CARBS_KCAL_PER_G).max(carbs_floor);
    Macros {
        protein,
        fat,
        fibre,
        carbs: row(eaten.carbs_g, carbs_floor, carbs_ceiling),
    }
}

struct Ledgers<'records> {
    days: Vec<Day>,
    gather: &'records GatherRecords,
    burn: &'records BurnRecords,
    fuel: &'records FuelRecords,
}

impl Ledgers<'_> {
    fn intakes_on(&self, day: NaiveDate) -> Vec<(i64, Intake)> {
        self.fuel
            .intakes
            .iter()
            .filter(|(_, eaten)| eaten.day == day)
            .cloned()
            .collect()
    }

    fn fuel_on(&self, day: NaiveDate, gaps: &[(NaiveDate, f64)]) -> Fuel {
        let intakes = self.intakes_on(day);
        let eaten = added_up(&intakes);
        let targets = version_in_force_on(&self.fuel.targets, day)
            .copied()
            .unwrap_or(DEFAULT_FUEL_TARGETS);
        let banded = body_on(day, &self.days, &self.gather.profiles).map(|body| {
            let burnt = burn_on(day, self.burn, &self.gather.measurements);
            let (band, budget) = band_for(
                &body,
                burnt.burnt_kcal.unwrap_or(0.0),
                burnt.off_the_clock.unwrap_or_default(),
                deficit_before(day, gaps),
                digested((!intakes.is_empty()).then_some(eaten), &body, targets),
            );
            (band, macros_for(eaten, &body, targets, band), budget)
        });
        Fuel {
            day,
            intakes,
            eaten,
            band: banded.map(|(band, _, _)| band),
            energy: banded
                .map(|(band, _, _)| verdict_of(eaten.energy_kcal, band.floor_kcal, band.cap_kcal)),
            macros: banded.map(|(_, macros, _)| macros),
            budget: banded.map(|(_, _, budget)| budget),
        }
    }
}

impl<'records> Ledgers<'records> {
    fn of(
        gather: &'records GatherRecords,
        burn: &'records BurnRecords,
        fuel: &'records FuelRecords,
    ) -> Self {
        Ledgers {
            days: ledger(&gather.measurements, &gather.profiles),
            gather,
            burn,
            fuel,
        }
    }

    fn gaps(&self) -> Vec<(NaiveDate, f64)> {
        let mut logged: Vec<NaiveDate> = self
            .fuel
            .intakes
            .iter()
            .map(|(_, eaten)| eaten.day)
            .collect();
        logged.sort();
        logged.dedup();
        logged.iter().fold(Vec::new(), |mut gaps, logged| {
            let fuelled = self.fuel_on(*logged, &gaps);
            if let Some(band) = fuelled.band {
                gaps.push((*logged, band.cap_kcal - fuelled.eaten.energy_kcal));
            }
            gaps
        })
    }
}

pub fn fuel_on(
    day: NaiveDate,
    gather: &GatherRecords,
    burn: &BurnRecords,
    fuel: &FuelRecords,
) -> Fuel {
    let ledgers = Ledgers::of(gather, burn, fuel);
    ledgers.fuel_on(day, &ledgers.gaps())
}

pub fn fuel_over(
    days: &[NaiveDate],
    gather: &GatherRecords,
    burn: &BurnRecords,
    fuel: &FuelRecords,
) -> Vec<Fuel> {
    let ledgers = Ledgers::of(gather, burn, fuel);
    let gaps = ledgers.gaps();
    days.iter()
        .map(|day| ledgers.fuel_on(*day, &gaps))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exercise::Exercise;
    use crate::food::{Food, Unit};
    use crate::intake::intake;
    use crate::measurement::Measurement;
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
                .is_some_and(|band| about(band.cap_kcal, budget.kcal() + burnt))
        );
        assert!(fuelled.band.is_some_and(|band| about(
            band.floor_kcal,
            (budget.kcal() + burnt - 48.5 * fat_kg).max(25.0 * lean_kg + burnt)
        )));
        assert_eq!(fuelled.budget, Some(budget));
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

    #[test]
    fn a_womans_floor_never_falls_below_thirty_kcal_per_kg_of_lean_mass_plus_the_burn() {
        let a_woman = GatherRecords {
            measurements: vec![Measurement {
                day: day("2026-10-01"),
                body_mass_kg: 75.0,
                waist_cm: Some(95.0),
            }],
            profiles: vec![(
                day("2026-03-01"),
                Profile {
                    birthday: day("1990-06-01"),
                    gender: Gender::Female,
                    height_cm: 165.0,
                    neck_cm: 33.0,
                    hip_cm: Some(110.0),
                },
            )],
            goals: vec![],
        };

        let mut burn = a_desk_job();
        let walking = Exercise {
            name: "Walking".into(),
            met: 4.0,
        };
        let walked = workout("2026-10-10", &walking, 60.0, None, 1, 0.0).expect("a workout");
        burn.workouts.push((1, walked.clone()));

        let fuelled = fuel_on("2026-10-10", &a_woman, &burn, &aiming());

        let lean_kg = crate::ledger::ledger(&a_woman.measurements, &a_woman.profiles)
            .first()
            .and_then(|measured| measured.composition)
            .map(|split| split.lean_mass_kg)
            .expect("a lean mass");
        let burnt = burnt_by(&walked, 75.0);
        assert!(
            fuelled
                .band
                .is_some_and(|band| about(band.minimum_kcal, 30.0 * lean_kg + burnt))
        );
        assert_eq!(
            fuelled
                .band
                .map(|band| band.minimum_kcal_per_kg_of_lean_mass),
            Some(30.0)
        );
        assert!(
            fuelled
                .band
                .is_some_and(|band| band.floor_kcal >= band.minimum_kcal)
        );
    }

    fn deficit_of(trailing_deficit_kcal: f64, weeks_in_deficit: u32) -> Deficit {
        Deficit {
            trailing_deficit_kcal,
            weeks_in_deficit,
        }
    }

    fn cap_with(deficit: Deficit) -> f64 {
        let (lean_kg, _) = the_body();
        cap_eating(
            deficit,
            (2.0 * 80.0, (3.0 * lean_kg).max(130.0), 0.9 * 80.0, 28.0),
        )
    }

    fn cap_eating(deficit: Deficit, (protein, carb, fat, fib): (f64, f64, f64, f64)) -> f64 {
        let (lean_kg, fat_kg) = the_body();
        daily_energy_budget(
            41,
            lean_kg,
            fat_kg,
            Gender::Male,
            A_DESK_DAY,
            deficit,
            Eaten {
                energy: 0.0,
                macros: Macro {
                    protein,
                    fat,
                    fib,
                    carb,
                },
            },
        )
        .kcal()
    }

    fn gap_on(date: &str, fuel: &FuelRecords) -> f64 {
        let fuelled = fuel_on(date, &a_mans_records(80.0), &a_desk_job(), fuel);
        cap_of(&fuelled) - fuelled.eaten.energy_kcal
    }

    #[test]
    fn two_days_logged_in_the_last_week_wear_the_next_by_their_mean_gap() {
        let mut fuel = aiming();
        eating(&mut fuel, "2026-10-08", &a_food(170.0, 230.0, 80.0, 35.0));
        eating(&mut fuel, "2026-10-09", &a_food(170.0, 230.0, 80.0, 35.0));
        let mean_gap = (gap_on("2026-10-08", &fuel) + gap_on("2026-10-09", &fuel)) / 2.0;

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        assert!(about(cap_of(&fuelled), cap_with(deficit_of(mean_gap, 1))));
    }

    #[test]
    fn a_week_without_a_logged_day_ends_the_run_of_weeks_in_deficit() {
        let mut fuel = aiming();
        eating(&mut fuel, "2026-10-02", &a_food(170.0, 230.0, 80.0, 35.0));
        eating(&mut fuel, "2026-10-10", &a_food(170.0, 230.0, 80.0, 35.0));
        let gap = gap_on("2026-10-02", &fuel);

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        assert!(about(
            cap_of(&fuelled),
            cap_eating(deficit_of(gap, 0), (170.0, 230.0, 80.0, 35.0))
        ));
    }

    #[test]
    fn no_more_than_four_weeks_count_in_deficit() {
        let mut fuel = aiming();
        let days: Vec<String> = (1..=35)
            .map(|back| (day("2026-10-10") - chrono::Days::new(back)).to_string())
            .collect();
        for date in &days {
            eating(&mut fuel, date, &a_food(170.0, 230.0, 80.0, 35.0));
        }
        let last_four_weeks: Vec<f64> = days[..28].iter().map(|date| gap_on(date, &fuel)).collect();
        let mean_gap = last_four_weeks.iter().sum::<f64>() / 28.0;

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        assert!(about(cap_of(&fuelled), cap_with(deficit_of(mean_gap, 4))));
    }

    fn cap_of(fuelled: &Fuel) -> f64 {
        fuelled.band.map(|band| band.cap_kcal).unwrap_or_default()
    }

    #[test]
    fn days_eaten_over_the_cap_do_not_wear_the_spontaneous_activity_down() {
        let mut feasting = aiming();
        for date in 1..=28 {
            eating(
                &mut feasting,
                &format!("2026-09-{:02}", date + 2),
                &a_food(250.0, 600.0, 150.0, 40.0),
            );
        }

        let fuelled = fuel_on(
            "2026-10-01",
            &a_mans_records(80.0),
            &a_desk_job(),
            &feasting,
        );

        assert!(about(cap_of(&fuelled), cap_with(deficit_of(0.0, 0))));
    }

    #[test]
    fn a_single_logged_day_wears_the_next_by_its_gap_over_one_week() {
        let mut fuel = aiming();
        eating(&mut fuel, "2026-10-09", &a_food(150.0, 100.0, 40.0, 20.0));
        let eve = fuel_on("2026-10-09", &a_mans_records(80.0), &a_desk_job(), &fuel);
        let gap = cap_of(&eve) - eve.eaten.energy_kcal;

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        assert!(about(cap_of(&fuelled), cap_with(deficit_of(gap, 1))));
    }

    #[test]
    fn days_logged_in_two_weeks_wear_the_next_by_their_mean_gap_over_two_weeks() {
        let mut fuel = aiming();
        eating(&mut fuel, "2026-10-02", &a_food(150.0, 100.0, 40.0, 20.0));
        eating(&mut fuel, "2026-10-09", &a_food(120.0, 80.0, 30.0, 10.0));
        let gap_on = |date: &str| {
            let fuelled = fuel_on(date, &a_mans_records(80.0), &a_desk_job(), &fuel);
            cap_of(&fuelled) - fuelled.eaten.energy_kcal
        };
        let mean_gap = (gap_on("2026-10-02") + gap_on("2026-10-09")) / 2.0;

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        assert!(about(cap_of(&fuelled), cap_with(deficit_of(mean_gap, 2))));
    }

    #[test]
    fn a_macro_eaten_right_at_its_floor_or_at_its_ceiling_is_in_its_band() {
        let mut fuel = aiming();
        eating(&mut fuel, "2026-10-10", &a_food(160.0, 227.5, 90.0, 35.0));

        let macros = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel)
            .macros
            .expect("macros");

        assert_eq!(macros.protein.floor_g.round(), 160.0);
        assert_eq!(macros.fat.ceiling_g.round(), 90.0);
        assert_eq!(
            (macros.protein.verdict, macros.fat.verdict),
            (Verdict::InBand, Verdict::InBand)
        );
    }

    #[test]
    fn each_macro_reads_its_floor_and_ceiling_from_its_target_and_the_energy_eaten() {
        let (lean_kg, _) = the_body();
        let mut fuel = aiming();
        eating(&mut fuel, "2026-10-10", &a_food(170.0, 230.0, 80.0, 35.0));

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        let band = fuelled.band.expect("a band");
        let macros = fuelled.macros.expect("macros");
        let energy = fuelled.eaten.energy_kcal;
        let spent = 170.0 * 4.0 + 80.0 * 9.0 + 35.0 * 2.0;
        let rows = [macros.protein, macros.fat, macros.fibre, macros.carbs];
        let expected = [
            (
                (2.0 * 80.0f64).max(0.2 * energy / 4.0),
                0.333 * energy / 4.0,
            ),
            (
                (0.9 * 80.0f64).max(0.2 * energy / 9.0),
                0.333 * energy / 9.0,
            ),
            (
                (0.4 * 80.0f64).max(0.024 * energy / 2.0),
                0.04 * energy / 2.0,
            ),
            (
                ((band.floor_kcal - spent) / 4.0)
                    .max(3.0 * lean_kg)
                    .max(130.0),
                (band.cap_kcal - spent) / 4.0,
            ),
        ];
        for (row, (floor, ceiling)) in rows.iter().zip(expected) {
            assert!(about(row.floor_g, floor), "{row:?} {floor}");
            assert!(about(row.ceiling_g, ceiling), "{row:?} {ceiling}");
        }
    }

    #[test]
    fn the_body_in_force_is_the_last_weighed_on_or_before_the_day() {
        let mut records = a_mans_records(80.0);
        records.measurements.push(Measurement {
            day: day("2026-10-20"),
            body_mass_kg: 70.0,
            waist_cm: Some(70.0),
        });

        let fuelled = fuel_on("2026-10-10", &records, &a_desk_job(), &aiming());

        let floor = |fuelled: &Fuel| fuelled.macros.map(|macros| macros.protein.floor_g);
        assert_eq!(floor(&fuelled), Some(160.0));
    }

    #[test]
    fn with_low_targets_per_kilo_the_floors_follow_the_energy_eaten_and_carbs_fill_the_calorie_floor()
     {
        let (lean_kg, _) = the_body();
        let mut fuel = FuelRecords {
            targets: vec![(
                day("2026-03-01"),
                FuelTargets {
                    protein_g_per_kg: 0.1,
                    fat_g_per_kg: 0.1,
                    fibre_g_per_kg: 0.01,
                },
            )],
            ..FuelRecords::default()
        };
        eating(&mut fuel, "2026-10-10", &a_food(40.0, 400.0, 20.0, 5.0));

        let fuelled = fuel_on("2026-10-10", &a_mans_records(80.0), &a_desk_job(), &fuel);

        let band = fuelled.band.expect("a band");
        let macros = fuelled.macros.expect("macros");
        let energy = fuelled.eaten.energy_kcal;
        let floors = [0.2 * energy / 4.0, 0.2 * energy / 9.0, 0.024 * energy / 2.0];
        let spent =
            floors[0].max(40.0) * 4.0 + floors[1].max(20.0) * 9.0 + floors[2].max(5.0) * 2.0;
        let carbs_floor = (band.floor_kcal - spent) / 4.0;
        assert!(carbs_floor > 3.0 * lean_kg);
        for (row, floor) in [macros.protein, macros.fat, macros.fibre, macros.carbs]
            .iter()
            .zip([floors[0], floors[1], floors[2], carbs_floor])
        {
            assert!(about(row.floor_g, floor), "{row:?} {floor}");
        }
    }

    #[test]
    fn a_macros_gap_is_what_was_eaten_beyond_its_floor_the_goal() {
        let short = MacroRow {
            eaten_g: 120.0,
            floor_g: 130.0,
            ceiling_g: 180.0,
            verdict: Verdict::Low,
        };

        assert!(about(short.gap_g(), -10.0));
    }

    #[test]
    fn each_macro_takes_its_share_of_the_energy_eaten_and_together_they_account_for_it() {
        let eaten = Nutrients {
            energy_kcal: 2000.0,
            protein_g: 150.0,
            fat_g: 70.0,
            fibre_g: 30.0,
            carbs_g: 177.5,
        };

        let shares = energy_shares(&eaten).expect("shares of what was eaten");

        assert!(about(shares.protein, 0.3));
        assert!(about(shares.fat, 0.315));
        assert!(about(shares.fibre, 0.03));
        assert!(about(shares.carbs, 0.355));
        assert!(about(shares.total, 1.0));
        assert_eq!(
            energy_shares(&Nutrients {
                energy_kcal: 0.0,
                ..eaten
            }),
            None
        );
    }

    #[test]
    fn a_mans_band_tells_the_25_kcal_per_kilo_of_lean_mass_its_minimum_rests_on() {
        let fuelled = fuel_on(
            "2026-10-10",
            &a_mans_records(80.0),
            &a_desk_job(),
            &aiming(),
        );

        assert_eq!(
            fuelled
                .band
                .map(|band| band.minimum_kcal_per_kg_of_lean_mass),
            Some(25.0)
        );
    }
}
