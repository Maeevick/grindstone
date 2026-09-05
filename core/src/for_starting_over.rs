use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::for_storing_switch_over::ForStoringSwitchOver;

pub fn erase_everything(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    fuel: &impl ForStoringFuelRecords,
    hydrate: &impl ForStoringHydrateRecords,
    switch_over: &impl ForStoringSwitchOver,
) -> Result<(), Failure> {
    let _ = (gather, burn, fuel, hydrate, switch_over);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::burn_records::BurnRecords;
    use crate::drink::DrinkKind;
    use crate::for_cataloguing::{add_drink, add_exercise};
    use crate::for_tracking::log_measurement;
    use crate::fuel_records::FuelRecords;
    use crate::gather_records::GatherRecords;
    use crate::hydrate_records::HydrateRecords;
    use crate::testing::{
        BrokenFuelRecords, BurnRecordsInMemory, FuelRecordsInMemory, GatherRecordsInMemory,
        HydrateRecordsInMemory, SwitchOverInMemory,
    };
    use chrono::NaiveDate;

    #[test]
    fn erasing_everything_leaves_every_record_empty_and_no_switch_over() {
        let (gather, burn, fuel, hydrate, switch_over) = (
            GatherRecordsInMemory::default(),
            BurnRecordsInMemory::default(),
            FuelRecordsInMemory::default(),
            HydrateRecordsInMemory::default(),
            SwitchOverInMemory::default(),
        );
        log_measurement(&gather, "2026-10-10", 80.0, None).expect("a measurement logs");
        add_exercise(&burn, "Walking", 4.0).expect("an exercise adds");
        add_drink(&hydrate, "Water", 250.0, DrinkKind::Water).expect("a drink adds");
        switch_over
            .save_switch_over(NaiveDate::from_ymd_opt(2026, 10, 10).expect("a real day"))
            .expect("a switch-over saves");

        erase_everything(&gather, &burn, &fuel, &hydrate, &switch_over).expect("everything erased");

        assert_eq!(gather.gather_records(), Ok(GatherRecords::default()));
        assert_eq!(burn.burn_records(), Ok(BurnRecords::default()));
        assert_eq!(fuel.fuel_records(), Ok(FuelRecords::default()));
        assert_eq!(hydrate.hydrate_records(), Ok(HydrateRecords::default()));
        assert_eq!(switch_over.switch_over(), Ok(None));
    }

    #[test]
    fn records_that_cannot_be_erased_say_so() {
        assert_eq!(
            erase_everything(
                &GatherRecordsInMemory::default(),
                &BurnRecordsInMemory::default(),
                &BrokenFuelRecords,
                &HydrateRecordsInMemory::default(),
                &SwitchOverInMemory::default(),
            ),
            Err(Failure::Broken("the disk is full".into()))
        );
    }
}
