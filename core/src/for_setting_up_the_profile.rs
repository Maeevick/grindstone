use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn::body_mass_in_force_on;
use crate::burn_target::target_on;
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::fuel_records::{DEFAULT_FUEL_TARGETS, FuelTargets, fuel_targets};
use crate::gender::Gender;
use crate::goals::{Goals, goals};
use crate::hydrate_counted::counted_on;
use crate::profile::{Profile, profile};
use crate::recommendations::{Recommendations, recommended};
use crate::refusal::Refusal;
use crate::version::version_in_force_on;
use crate::volitional::{Volitional, volitional};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileForm {
    pub from: String,
    pub birthday: String,
    pub gender: Gender,
    pub height_cm: f64,
    pub neck_cm: f64,
    pub hip_cm: Option<f64>,
}

fn day(date: &str) -> Result<NaiveDate, Failure> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Failure::Refused(Refusal::NotADate))
}

pub fn set_profile(
    records: &impl ForStoringGatherRecords,
    form: &ProfileForm,
) -> Result<Profile, Failure> {
    let from = day(&form.from)?;
    let profile = profile(
        &form.birthday,
        form.gender,
        form.height_cm,
        form.neck_cm,
        form.hip_cm,
    )
    .map_err(Failure::Refused)?;
    records.save_profile(from, &profile)?;
    Ok(profile)
}

pub fn profile_on(
    records: &impl ForStoringGatherRecords,
    date: &str,
) -> Result<Option<Profile>, Failure> {
    let day = day(date)?;
    let gathered = records.gather_records()?;
    Ok(version_in_force_on(&gathered.profiles, day).copied())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoalsForm {
    pub from: String,
    pub body_mass_kg: f64,
    pub fat_percentage: f64,
}

pub fn set_goals(
    records: &impl ForStoringGatherRecords,
    form: &GoalsForm,
) -> Result<Goals, Failure> {
    let from = day(&form.from)?;
    let goals = goals(form.body_mass_kg, form.fat_percentage).map_err(Failure::Refused)?;
    records.save_goals(from, &goals)?;
    Ok(goals)
}

pub fn goals_on(
    records: &impl ForStoringGatherRecords,
    date: &str,
) -> Result<Option<Goals>, Failure> {
    let day = day(date)?;
    let gathered = records.gather_records()?;
    Ok(version_in_force_on(&gathered.goals, day).copied())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UsualDayForm {
    pub from: String,
    pub sitting_min: f64,
    pub standing_min: f64,
    pub moving_km: f64,
}

pub fn set_usual_day(
    burn: &impl ForStoringBurnRecords,
    form: &UsualDayForm,
) -> Result<Volitional, Failure> {
    let from = day(&form.from)?;
    let usual = volitional(form.sitting_min, form.standing_min, form.moving_km)
        .map_err(Failure::Refused)?;
    burn.save_usual_day(from, &usual)?;
    Ok(usual)
}

pub fn usual_day_on(
    burn: &impl ForStoringBurnRecords,
    date: &str,
) -> Result<Option<Volitional>, Failure> {
    let day = day(date)?;
    Ok(version_in_force_on(&burn.burn_records()?.usual_days, day).copied())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelTargetsForm {
    pub from: String,
    pub protein_g_per_kg: f64,
    pub fat_g_per_kg: f64,
    pub fibre_g_per_kg: f64,
}

pub fn set_fuel_targets(
    fuel: &impl ForStoringFuelRecords,
    form: &FuelTargetsForm,
) -> Result<FuelTargets, Failure> {
    let from = day(&form.from)?;
    let targets = fuel_targets(
        form.protein_g_per_kg,
        form.fat_g_per_kg,
        form.fibre_g_per_kg,
    )
    .map_err(Failure::Refused)?;
    fuel.save_targets(from, &targets)?;
    Ok(targets)
}

pub fn fuel_targets_on(
    fuel: &impl ForStoringFuelRecords,
    date: &str,
) -> Result<FuelTargets, Failure> {
    let day = day(date)?;
    Ok(version_in_force_on(&fuel.fuel_records()?.targets, day)
        .copied()
        .unwrap_or(DEFAULT_FUEL_TARGETS))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HydrateCountedForm {
    pub from: String,
    pub counted: bool,
}

pub fn set_hydrate_counted(
    hydrate: &impl ForStoringHydrateRecords,
    form: &HydrateCountedForm,
) -> Result<bool, Failure> {
    hydrate.save_counted(day(&form.from)?, form.counted)?;
    Ok(form.counted)
}

pub fn hydrate_counted_on(
    hydrate: &impl ForStoringHydrateRecords,
    date: &str,
) -> Result<bool, Failure> {
    let day = day(date)?;
    Ok(counted_on(day, &hydrate.hydrate_records()?.counted_from))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BurnTargetForm {
    pub from: String,
    pub kcal: f64,
}

pub fn set_burn_target(
    burn: &impl ForStoringBurnRecords,
    form: &BurnTargetForm,
) -> Result<f64, Failure> {
    let from = day(&form.from)?;
    if !(form.kcal.is_finite() && form.kcal >= 0.0) {
        return Err(Failure::Refused(Refusal::NotAnEnergy));
    }
    burn.save_burn_target(from, form.kcal)?;
    Ok(form.kcal)
}

pub fn burn_target_on(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    date: &str,
) -> Result<Option<f64>, Failure> {
    let day = day(date)?;
    let measurements = gather.gather_records()?.measurements;
    Ok(target_on(
        day,
        &burn.burn_records()?.targets,
        body_mass_in_force_on(day, &measurements),
    ))
}

pub fn recommended_on(
    gather: &impl ForStoringGatherRecords,
    date: &str,
) -> Result<Recommendations, Failure> {
    let day = day(date)?;
    let gathered = gather.gather_records()?;
    let profile = version_in_force_on(&gathered.profiles, day)
        .ok_or(Failure::Refused(Refusal::NotAProfile))?;
    let masses: Vec<(NaiveDate, f64)> = gathered
        .measurements
        .iter()
        .map(|measured| (measured.day, measured.body_mass_kg))
        .collect();
    Ok(recommended(
        profile,
        day,
        version_in_force_on(&masses, day).copied(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{BrokenGatherRecords, GatherRecordsInMemory};

    fn a_mans_form(from: &str) -> ProfileForm {
        ProfileForm {
            from: from.into(),
            birthday: "1985-04-12".into(),
            gender: Gender::Male,
            height_cm: 180.0,
            neck_cm: 38.5,
            hip_cm: None,
        }
    }

    #[test]
    fn a_profile_set_from_a_day_is_in_force_from_that_day() {
        let records = GatherRecordsInMemory::default();

        let set = set_profile(&records, &a_mans_form("2026-10-10"));

        assert_eq!(profile_on(&records, "2026-10-12"), set.map(Some));
    }

    #[test]
    fn goals_set_from_a_day_are_in_force_from_that_day() {
        let records = GatherRecordsInMemory::default();

        let set = set_goals(
            &records,
            &GoalsForm {
                from: "2026-10-10".into(),
                body_mass_kg: 70.0,
                fat_percentage: 10.0,
            },
        );

        assert_eq!(goals_on(&records, "2026-10-12"), set.map(Some));
    }

    #[test]
    fn a_refused_profile_or_refused_goals_leave_what_was_set_as_it_was() {
        let records = GatherRecordsInMemory::default();
        let refused_profile = ProfileForm {
            height_cm: 0.0,
            ..a_mans_form("2026-10-10")
        };
        let refused_goals = GoalsForm {
            from: "2026-10-10".into(),
            body_mass_kg: 70.0,
            fat_percentage: 100.0,
        };

        assert_eq!(
            set_profile(&records, &refused_profile),
            Err(Failure::Refused(Refusal::NotAHeight))
        );
        assert_eq!(
            set_goals(&records, &refused_goals),
            Err(Failure::Refused(Refusal::NotARatio))
        );
        assert_eq!(profile_on(&records, "2026-10-12"), Ok(None));
        assert_eq!(goals_on(&records, "2026-10-12"), Ok(None));
    }

    #[test]
    fn a_day_the_calendar_lacks_is_refused_whether_setting_or_reading() {
        let records = GatherRecordsInMemory::default();
        let not_a_date = Failure::Refused(Refusal::NotADate);

        assert_eq!(
            set_profile(&records, &a_mans_form("2026-02-30")),
            Err(not_a_date.clone())
        );
        assert_eq!(profile_on(&records, "2026-02-30"), Err(not_a_date.clone()));
        assert_eq!(goals_on(&records, "2026-02-30"), Err(not_a_date));
    }

    #[test]
    fn records_that_cannot_be_written_or_read_come_back_broken() {
        let broken = Failure::Broken("the disk is full".into());
        let goals_form = GoalsForm {
            from: "2026-10-10".into(),
            body_mass_kg: 70.0,
            fat_percentage: 10.0,
        };

        assert_eq!(
            set_profile(&BrokenGatherRecords, &a_mans_form("2026-10-10")),
            Err(broken.clone())
        );
        assert_eq!(
            profile_on(&BrokenGatherRecords, "2026-10-10"),
            Err(broken.clone())
        );
        assert_eq!(
            set_goals(&BrokenGatherRecords, &goals_form),
            Err(broken.clone())
        );
        assert_eq!(goals_on(&BrokenGatherRecords, "2026-10-10"), Err(broken));
    }

    fn a_desk_day_from(from: &str) -> UsualDayForm {
        UsualDayForm {
            from: from.into(),
            sitting_min: 480.0,
            standing_min: 120.0,
            moving_km: 3.0,
        }
    }

    #[test]
    fn a_usual_day_set_from_a_date_is_the_one_in_force_from_that_date() {
        let burn = crate::testing::BurnRecordsInMemory::default();
        set_usual_day(&burn, &a_desk_day_from("2026-03-01")).expect("a usual day sets");

        assert_eq!(
            usual_day_on(&burn, "2026-10-10"),
            Ok(Some(Volitional {
                sitting_min: 480.0,
                standing_min: 120.0,
                moving_km: 3.0,
            }))
        );
    }

    #[test]
    fn a_usual_day_with_more_minutes_than_a_day_holds_is_refused() {
        let refused = set_usual_day(
            &crate::testing::BurnRecordsInMemory::default(),
            &UsualDayForm {
                sitting_min: 1500.0,
                ..a_desk_day_from("2026-03-01")
            },
        );

        assert_eq!(refused, Err(Failure::Refused(Refusal::NotADistance)));
        assert_eq!(
            usual_day_on(
                &crate::testing::BurnRecordsInMemory::default(),
                "2026-02-30"
            ),
            Err(Failure::Refused(Refusal::NotADate))
        );
    }

    #[test]
    fn fuel_targets_set_from_a_date_are_the_ones_in_force_and_the_defaults_stand_before() {
        let fuel = crate::testing::FuelRecordsInMemory::default();
        let defaults = fuel_targets_on(&fuel, "2026-10-10").expect("targets");
        set_fuel_targets(
            &fuel,
            &FuelTargetsForm {
                from: "2026-03-01".into(),
                protein_g_per_kg: 2.025,
                fat_g_per_kg: 0.9,
                fibre_g_per_kg: 0.4,
            },
        )
        .expect("targets set");

        assert_eq!(
            fuel_targets_on(&fuel, "2026-10-10"),
            Ok(FuelTargets {
                protein_g_per_kg: 2.025,
                fat_g_per_kg: 0.9,
                fibre_g_per_kg: 0.4,
            })
        );
        assert_eq!(defaults.protein_g_per_kg, 1.6);
    }

    #[test]
    fn fuel_targets_below_nothing_are_refused() {
        let refused = set_fuel_targets(
            &crate::testing::FuelRecordsInMemory::default(),
            &FuelTargetsForm {
                from: "2026-03-01".into(),
                protein_g_per_kg: -2.0,
                fat_g_per_kg: 0.9,
                fibre_g_per_kg: 0.4,
            },
        );

        assert_eq!(refused, Err(Failure::Refused(Refusal::NotANutrient)));
    }

    #[test]
    fn hydrate_counts_until_left_out_from_a_date() {
        let hydrate = crate::testing::HydrateRecordsInMemory::default();
        set_hydrate_counted(
            &hydrate,
            &HydrateCountedForm {
                from: "2026-10-12".into(),
                counted: false,
            },
        )
        .expect("a switch sets");

        assert_eq!(
            (
                hydrate_counted_on(&hydrate, "2026-10-11"),
                hydrate_counted_on(&hydrate, "2026-10-12")
            ),
            (Ok(true), Ok(false))
        );
    }

    #[test]
    fn a_burn_target_set_from_a_date_is_the_one_in_force_from_that_date() {
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            crate::testing::BurnRecordsInMemory::default(),
        );
        crate::for_tracking::log_measurement(&gather, "2026-10-01", 72.0, None)
            .expect("a measurement logs");
        set_burn_target(
            &burn,
            &BurnTargetForm {
                from: "2026-10-15".into(),
                kcal: 150.0,
            },
        )
        .expect("a target sets");

        assert_eq!(
            (
                burn_target_on(&gather, &burn, "2026-10-10"),
                burn_target_on(&gather, &burn, "2026-10-20")
            ),
            (
                Ok(crate::burn_target::target_on(
                    NaiveDate::from_ymd_opt(2026, 10, 10).expect("a day"),
                    &[],
                    Some(72.0)
                )),
                Ok(Some(150.0))
            )
        );
    }

    #[test]
    fn a_burn_target_below_nothing_a_day_the_calendar_lacks_or_broken_records_set_nothing() {
        let target = |from: &str, kcal: f64| BurnTargetForm {
            from: from.into(),
            kcal,
        };
        let burn = crate::testing::BurnRecordsInMemory::default();

        assert_eq!(
            set_burn_target(&burn, &target("2026-10-15", -10.0)),
            Err(Failure::Refused(Refusal::NotAnEnergy))
        );
        assert_eq!(
            set_burn_target(&burn, &target("2026-02-30", 150.0)),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            burn_target_on(&GatherRecordsInMemory::default(), &burn, "2026-02-30"),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            set_burn_target(
                &crate::testing::BrokenBurnRecords,
                &target("2026-10-15", 150.0)
            ),
            Err(Failure::Broken("the disk is full".into()))
        );
        assert_eq!(
            burn_target_on(&BrokenGatherRecords, &burn, "2026-10-15"),
            Err(Failure::Broken("the disk is full".into()))
        );
        assert_eq!(
            burn_target_on(
                &GatherRecordsInMemory::default(),
                &crate::testing::BrokenBurnRecords,
                "2026-10-15"
            ),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    #[test]
    fn a_hydrate_switch_on_a_day_the_calendar_lacks_or_broken_records_set_nothing() {
        let left_out_on = |from: &str| HydrateCountedForm {
            from: from.into(),
            counted: false,
        };

        assert_eq!(
            set_hydrate_counted(
                &crate::testing::HydrateRecordsInMemory::default(),
                &left_out_on("2026-02-30")
            ),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            hydrate_counted_on(
                &crate::testing::HydrateRecordsInMemory::default(),
                "2026-02-30"
            ),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            set_hydrate_counted(
                &crate::testing::BrokenHydrateRecords,
                &left_out_on("2026-03-01")
            ),
            Err(Failure::Broken("the disk is full".into()))
        );
        assert_eq!(
            hydrate_counted_on(&crate::testing::BrokenHydrateRecords, "2026-10-10"),
            Err(Failure::Broken("the disk is full".into()))
        );
    }

    #[test]
    fn recommendations_read_the_profile_and_the_body_mass_in_force_on_their_day() {
        let gather = GatherRecordsInMemory::default();
        set_profile(&gather, &a_mans_form("2026-03-01")).expect("a profile set");
        crate::for_tracking::log_measurement(&gather, "2026-10-01", 80.0, None)
            .expect("a measurement logs");

        let advised = recommended_on(&gather, "2026-10-11").expect("recommendations");

        assert_eq!(advised.fuel.protein_g_per_kg, 1.6);
        assert_eq!(advised.fuel.fibre_g_per_kg, 0.31);
    }

    #[test]
    fn without_a_profile_there_is_nothing_to_recommend() {
        assert_eq!(
            recommended_on(&GatherRecordsInMemory::default(), "2026-10-11"),
            Err(Failure::Refused(Refusal::NotAProfile))
        );
        assert_eq!(
            recommended_on(&BrokenGatherRecords, "2026-10-11"),
            Err(Failure::Broken("the disk is full".into()))
        );
    }
}
