use crate::burn_records::BurnRecords;
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::fuel_records::FuelRecords;
use crate::gather_records::GatherRecords;
use crate::hydrate_records::HydrateRecords;

pub fn erase_everything(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    hydrate: &impl ForStoringHydrateRecords,
) -> Result<(), Failure> {
    gather.replace_gather_records(&GatherRecords::default())?;
    burn.replace_burn_records(&BurnRecords::default())?;
    fuel.replace_fuel_records(&FuelRecords::default())?;
    hydrate.replace_hydrate_records(&HydrateRecords::default())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drink::DrinkKind;
    use crate::for_cataloguing::{add_drink, add_exercise};
    use crate::for_tracking::log_measurement;
    use crate::testing::{
        BrokenFuelRecords, BurnRecordsInMemory, FuelRecordsInMemory, GatherRecordsInMemory,
        HydrateRecordsInMemory,
    };

    #[test]
    fn erasing_everything_leaves_every_record_empty() {
        let (gather, burn, fuel, hydrate) = (
            GatherRecordsInMemory::default(),
            BurnRecordsInMemory::default(),
            FuelRecordsInMemory::default(),
            HydrateRecordsInMemory::default(),
        );
        log_measurement(&gather, "2026-10-10", 80.0, None).expect("a measurement logs");
        add_exercise(&burn, "Walking", 4.0, "").expect("an exercise adds");
        add_drink(&hydrate, "Water", 250.0, DrinkKind::Water, "").expect("a drink adds");

        erase_everything(&gather, &burn, &fuel, &hydrate).expect("everything erased");

        assert_eq!(gather.gather_records(), Ok(GatherRecords::default()));
        assert_eq!(burn.burn_records(), Ok(BurnRecords::default()));
        assert_eq!(fuel.fuel_records(), Ok(FuelRecords::default()));
        assert_eq!(hydrate.hydrate_records(), Ok(HydrateRecords::default()));
    }

    #[test]
    fn records_that_cannot_be_erased_say_so() {
        assert_eq!(
            erase_everything(
                &GatherRecordsInMemory::default(),
                &BurnRecordsInMemory::default(),
                &BrokenFuelRecords,
                &HydrateRecordsInMemory::default(),
            ),
            Err(Failure::Broken("the disk is full".into()))
        );
    }
}
