use serde::{Deserialize, Serialize};

use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::hydrate::Hydrate;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DrinkingForm {
    pub date: String,
    pub drink: String,
    pub servings: f64,
}

pub fn hydrate_day(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    hydrate: &impl ForStoringHydrateRecords,
    date: &str,
) -> Result<Hydrate, Failure> {
    let _ = (gather, burn, hydrate, date);
    Err(Failure::Broken("not yet".into()))
}

pub fn log_drink(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    hydrate: &impl ForStoringHydrateRecords,
    form: &DrinkingForm,
) -> Result<Hydrate, Failure> {
    let _ = (gather, burn, hydrate, form);
    Err(Failure::Broken("not yet".into()))
}

pub fn remove_drinking(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    hydrate: &impl ForStoringHydrateRecords,
    date: &str,
    id: i64,
) -> Result<Hydrate, Failure> {
    let _ = (gather, burn, hydrate, date, id);
    Err(Failure::Broken("not yet".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drink::Drink;
    use crate::exercise::Exercise;
    use crate::for_burning::{WorkoutForm, log_workout};
    use crate::for_tracking::log_measurement;
    use crate::refusal::Refusal;
    use crate::testing::{
        BrokenBurnRecords, BrokenGatherRecords, BrokenHydrateRecords, BurnRecordsInMemory,
        GatherRecordsInMemory, HydrateRecordsInMemory,
    };

    fn a_catalog_with_tea() -> HydrateRecordsInMemory {
        let hydrate = HydrateRecordsInMemory::default();
        hydrate
            .save_drink(&Drink {
                name: "Tea".into(),
                volume_ml: 200.0,
            })
            .expect("a drink saves");
        hydrate
    }

    fn tea(date: &str, servings: f64) -> DrinkingForm {
        DrinkingForm {
            date: date.into(),
            drink: "Tea".into(),
            servings,
        }
    }

    fn volumes(hydrate: &Hydrate) -> Vec<f64> {
        hydrate
            .drinkings
            .iter()
            .map(|(_, drunk)| drunk.volume_ml)
            .collect()
    }

    #[test]
    fn a_logged_drink_comes_back_in_its_days_hydrate_at_its_servings_volume() {
        let hydrate = a_catalog_with_tea();

        let hydrated = log_drink(
            &GatherRecordsInMemory::default(),
            &BurnRecordsInMemory::default(),
            &hydrate,
            &tea("2026-10-10", 1.5),
        )
        .expect("a hydrate");

        assert_eq!(volumes(&hydrated), vec![300.0]);
        assert_eq!(hydrated.drunk_ml, 300.0);
    }

    #[test]
    fn a_drink_missing_from_the_catalog_is_refused() {
        let refused = log_drink(
            &GatherRecordsInMemory::default(),
            &BurnRecordsInMemory::default(),
            &HydrateRecordsInMemory::default(),
            &tea("2026-10-10", 1.0),
        );

        assert_eq!(refused, Err(Failure::Refused(Refusal::NotADrink)));
    }

    #[test]
    fn removing_a_drinking_leaves_the_others_of_its_day() {
        let gather = GatherRecordsInMemory::default();
        let burn = BurnRecordsInMemory::default();
        let hydrate = a_catalog_with_tea();
        log_drink(&gather, &burn, &hydrate, &tea("2026-10-10", 1.0)).expect("a drink logs");
        let logged =
            log_drink(&gather, &burn, &hydrate, &tea("2026-10-10", 0.5)).expect("a hydrate");
        let first = logged
            .drinkings
            .first()
            .map(|(id, _)| *id)
            .expect("a first drinking");

        let hydrated =
            remove_drinking(&gather, &burn, &hydrate, "2026-10-10", first).expect("a hydrate");

        assert_eq!(volumes(&hydrated), vec![100.0]);
    }

    #[test]
    fn a_days_hydrate_reads_back_what_was_drunk_on_it() {
        let gather = GatherRecordsInMemory::default();
        let burn = BurnRecordsInMemory::default();
        let hydrate = a_catalog_with_tea();
        log_drink(&gather, &burn, &hydrate, &tea("2026-10-10", 2.0)).expect("a drink logs");

        let hydrated = hydrate_day(&gather, &burn, &hydrate, "2026-10-10").expect("a hydrate");

        assert_eq!(volumes(&hydrated), vec![400.0]);
    }

    #[test]
    fn a_days_target_grows_by_a_millilitre_per_kcal_its_workouts_burnt() {
        let gather = GatherRecordsInMemory::default();
        log_measurement(&gather, "2026-10-10", 80.0, None).expect("a measurement logs");
        let burn = BurnRecordsInMemory::default();
        burn.save_exercise(&Exercise {
            name: "Walking".into(),
            met: 4.0,
        })
        .expect("an exercise saves");
        log_workout(
            &gather,
            &burn,
            &WorkoutForm {
                date: "2026-10-10".into(),
                exercise: "Walking".into(),
                minutes: 60.0,
                reps: None,
                sets: 1,
                load_kg: 0.0,
            },
        )
        .expect("a workout logs");

        let hydrated = hydrate_day(&gather, &burn, &HydrateRecordsInMemory::default(), "2026-10-10")
            .expect("a hydrate");

        assert!((hydrated.target_ml - (2000.0 + 252.0)).abs() < 1e-9);
    }

    #[test]
    fn a_day_the_calendar_lacks_a_drinking_of_no_serving_or_broken_records_give_no_hydrate() {
        let gather = GatherRecordsInMemory::default();
        let burn = BurnRecordsInMemory::default();
        let broken = Failure::Broken("the disk is full".into());

        assert_eq!(
            hydrate_day(&gather, &burn, &a_catalog_with_tea(), "2026-02-30"),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            log_drink(&gather, &burn, &a_catalog_with_tea(), &tea("2026-10-10", 0.0)),
            Err(Failure::Refused(Refusal::NotAServing))
        );
        assert_eq!(
            hydrate_day(&gather, &burn, &BrokenHydrateRecords, "2026-10-10"),
            Err(broken.clone())
        );
        assert_eq!(
            hydrate_day(&BrokenGatherRecords, &burn, &a_catalog_with_tea(), "2026-10-10"),
            Err(broken.clone())
        );
        assert_eq!(
            hydrate_day(&gather, &BrokenBurnRecords, &a_catalog_with_tea(), "2026-10-10"),
            Err(broken.clone())
        );
        assert_eq!(
            remove_drinking(&gather, &burn, &BrokenHydrateRecords, "2026-10-10", 1),
            Err(broken)
        );
    }
}
