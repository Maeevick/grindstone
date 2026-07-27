use crate::exercise::{Exercise, exercise};
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;

pub fn exercises(burn: &impl ForStoringBurnRecords) -> Result<Vec<Exercise>, Failure> {
    Ok(burn.burn_records()?.exercises)
}

pub fn add_exercise(
    burn: &impl ForStoringBurnRecords,
    name: &str,
    met: f64,
) -> Result<Vec<Exercise>, Failure> {
    burn.save_exercise(&exercise(name, met).map_err(Failure::Refused)?)?;
    exercises(burn)
}

pub fn remove_exercise(
    burn: &impl ForStoringBurnRecords,
    name: &str,
) -> Result<Vec<Exercise>, Failure> {
    burn.remove_exercise(name)?;
    exercises(burn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::refusal::Refusal;
    use crate::testing::{BrokenBurnRecords, BurnRecordsInMemory};

    fn named(name: &str, met: f64) -> Exercise {
        Exercise {
            name: name.into(),
            met,
        }
    }

    #[test]
    fn added_exercises_come_back_in_the_catalog_by_name() {
        let burn = BurnRecordsInMemory::default();
        add_exercise(&burn, "Walking", 4.0).expect("an exercise adds");

        let catalog = add_exercise(&burn, "Biking", 8.0);

        assert_eq!(
            catalog,
            Ok(vec![named("Biking", 8.0), named("Walking", 4.0)])
        );
        assert_eq!(exercises(&burn), catalog);
    }

    #[test]
    fn an_exercise_added_again_under_its_name_takes_its_new_met() {
        let burn = BurnRecordsInMemory::default();
        add_exercise(&burn, "Walking", 3.5).expect("an exercise adds");

        assert_eq!(
            add_exercise(&burn, "Walking", 4.0),
            Ok(vec![named("Walking", 4.0)])
        );
    }

    #[test]
    fn a_removed_exercise_leaves_the_catalog() {
        let burn = BurnRecordsInMemory::default();
        add_exercise(&burn, "Walking", 4.0).expect("an exercise adds");

        assert_eq!(remove_exercise(&burn, "Walking"), Ok(vec![]));
    }

    #[test]
    fn a_nameless_exercise_or_broken_records_change_nothing() {
        assert_eq!(
            add_exercise(&BurnRecordsInMemory::default(), " ", 4.0),
            Err(Failure::Refused(Refusal::NotAName))
        );
        assert_eq!(
            remove_exercise(&BrokenBurnRecords, "Walking"),
            Err(Failure::Broken("the disk is full".into()))
        );
    }
}
