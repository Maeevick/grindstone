use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::burn::{body_mass_in_force_on, usual_day_in_force};
use crate::burn_target::target_on;
use crate::failure::Failure;
use crate::for_storing_burn_records::ForStoringBurnRecords;
use crate::for_storing_fuel_records::ForStoringFuelRecords;
use crate::for_storing_gather_records::ForStoringGatherRecords;
use crate::for_storing_hydrate_records::ForStoringHydrateRecords;
use crate::fuel::fuel_targets_in_force;
use crate::fuel_records::{FuelTargets, fuel_targets};
use crate::gender::Gender;
use crate::goals::{Goals, goals};
use crate::hydrate_counted::counted_on;
use crate::kept::{InForce, Kept};
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

/// Goals as typed: a body mass and a fat percentage.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TypedGoals {
    pub body_mass_kg: f64,
    pub fat_percentage: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GoalsForm {
    pub from: String,
    pub kept: Kept<TypedGoals>,
}

pub fn set_goals(
    records: &impl ForStoringGatherRecords,
    form: &GoalsForm,
) -> Result<Option<InForce<Goals>>, Failure> {
    let from = day(&form.from)?;
    let kept = match form.kept {
        Kept::Own(typed) => {
            Kept::Own(goals(typed.body_mass_kg, typed.fat_percentage).map_err(Failure::Refused)?)
        }
        Kept::Recommended => Kept::Recommended,
    };
    records.save_goals(from, &kept)?;
    goals_on(records, &form.from)
}

pub fn goals_on(
    records: &impl ForStoringGatherRecords,
    date: &str,
) -> Result<Option<InForce<Goals>>, Failure> {
    let day = day(date)?;
    let gathered = records.gather_records()?;
    Ok(version_in_force_on(&gathered.goals, day)
        .and_then(Kept::own)
        .map(|goals| InForce {
            values: *goals,
            recommended: false,
        }))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UsualDayForm {
    pub from: String,
    pub kept: Kept<Volitional>,
}

pub fn set_usual_day(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    form: &UsualDayForm,
) -> Result<Option<InForce<Volitional>>, Failure> {
    let from = day(&form.from)?;
    let kept = match form.kept {
        Kept::Own(usual) => Kept::Own(
            volitional(usual.sitting_min, usual.standing_min, usual.moving_km)
                .map_err(Failure::Refused)?,
        ),
        Kept::Recommended => Kept::Recommended,
    };
    burn.save_usual_day(from, &kept)?;
    usual_day_on(gather, burn, &form.from)
}

pub fn usual_day_on(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    date: &str,
) -> Result<Option<InForce<Volitional>>, Failure> {
    let day = day(date)?;
    Ok(usual_day_in_force(
        day,
        &burn.burn_records()?.usual_days,
        &gather.gather_records()?.profiles,
    ))
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelTargetsForm {
    pub from: String,
    pub kept: Kept<FuelTargets>,
}

pub fn set_fuel_targets(
    gather: &impl ForStoringGatherRecords,
    fuel: &impl ForStoringFuelRecords,
    form: &FuelTargetsForm,
) -> Result<Option<InForce<FuelTargets>>, Failure> {
    let from = day(&form.from)?;
    let kept = match form.kept {
        Kept::Own(targets) => Kept::Own(
            fuel_targets(
                targets.protein_g_per_kg,
                targets.fat_g_per_kg,
                targets.fibre_g_per_kg,
            )
            .map_err(Failure::Refused)?,
        ),
        Kept::Recommended => Kept::Recommended,
    };
    fuel.save_targets(from, &kept)?;
    fuel_targets_on(gather, fuel, &form.from)
}

pub fn fuel_targets_on(
    gather: &impl ForStoringGatherRecords,
    fuel: &impl ForStoringFuelRecords,
    date: &str,
) -> Result<Option<InForce<FuelTargets>>, Failure> {
    let day = day(date)?;
    let gathered = gather.gather_records()?;
    Ok(fuel_targets_in_force(
        day,
        &fuel.fuel_records()?.targets,
        &gathered.profiles,
        body_mass_in_force_on(day, &gathered.measurements),
    ))
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
    pub kept: Kept<f64>,
}

pub fn set_burn_target(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    form: &BurnTargetForm,
) -> Result<Option<InForce<f64>>, Failure> {
    let from = day(&form.from)?;
    if form
        .kept
        .own()
        .is_some_and(|kcal| !(kcal.is_finite() && *kcal >= 0.0))
    {
        return Err(Failure::Refused(Refusal::NotAnEnergy));
    }
    burn.save_burn_target(from, form.kept)?;
    burn_target_on(gather, burn, &form.from)
}

pub fn burn_target_on(
    gather: &impl ForStoringGatherRecords,
    burn: &impl ForStoringBurnRecords,
    date: &str,
) -> Result<Option<InForce<f64>>, Failure> {
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

        let set = set_goals(&records, &goals_kept("2026-10-10", 70.0, 10.0));

        assert_eq!(goals_on(&records, "2026-10-12"), set);
        assert_eq!(
            set,
            Ok(Some(InForce {
                values: Goals {
                    body_mass_kg: 70.0,
                    fat_ratio: 0.1
                },
                recommended: false
            }))
        );
    }

    fn goals_kept(from: &str, body_mass_kg: f64, fat_percentage: f64) -> GoalsForm {
        GoalsForm {
            from: from.into(),
            kept: Kept::Own(TypedGoals {
                body_mass_kg,
                fat_percentage,
            }),
        }
    }

    fn the_recommended_goals(birthday: &str, date: &str) -> InForce<Goals> {
        let advised = recommended(
            &Profile {
                birthday: NaiveDate::parse_from_str(birthday, "%Y-%m-%d").expect("a day"),
                gender: Gender::Male,
                height_cm: 180.0,
                neck_cm: 38.5,
                hip_cm: None,
            },
            NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a day"),
            None,
        );
        InForce {
            values: Goals {
                body_mass_kg: advised.goal_mass_kg,
                fat_ratio: advised.goal_fat_ratio,
            },
            recommended: true,
        }
    }

    #[test]
    fn goals_never_kept_follow_the_recommendation_for_the_height_and_age_none_without_a_profile() {
        assert_eq!(
            (
                goals_on(&born("1985-04-12"), "2026-10-10"),
                goals_on(&GatherRecordsInMemory::default(), "2026-10-10")
            ),
            (
                Ok(Some(the_recommended_goals("1985-04-12", "2026-10-10"))),
                Ok(None)
            )
        );
    }

    #[test]
    fn the_goals_recommendation_kept_from_a_day_follows_the_age_own_goals_holding_before_it() {
        let gather = born("1966-10-15");
        set_goals(&gather, &goals_kept("2026-03-01", 70.0, 10.0)).expect("goals keep");
        set_goals(
            &gather,
            &GoalsForm {
                from: "2026-10-10".into(),
                kept: Kept::Recommended,
            },
        )
        .expect("the recommendation keeps");

        let in_force = |date| goals_on(&gather, date).ok().flatten();

        assert_eq!(
            [
                in_force("2026-10-05"),
                in_force("2026-10-11"),
                in_force("2026-10-16")
            ],
            [
                Some(InForce {
                    values: Goals {
                        body_mass_kg: 70.0,
                        fat_ratio: 0.1
                    },
                    recommended: false
                }),
                Some(the_recommended_goals("1966-10-15", "2026-10-11")),
                Some(the_recommended_goals("1966-10-15", "2026-10-16"))
            ]
        );
    }

    #[test]
    fn a_refused_profile_or_refused_goals_leave_what_was_set_as_it_was() {
        let records = GatherRecordsInMemory::default();
        let refused_profile = ProfileForm {
            height_cm: 0.0,
            ..a_mans_form("2026-10-10")
        };
        let refused_goals = goals_kept("2026-10-10", 70.0, 100.0);

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
        let goals_form = goals_kept("2026-10-10", 70.0, 10.0);

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

    fn a_desk_day() -> Volitional {
        Volitional {
            sitting_min: 480.0,
            standing_min: 120.0,
            moving_km: 3.0,
        }
    }

    fn a_desk_day_from(from: &str) -> UsualDayForm {
        UsualDayForm {
            from: from.into(),
            kept: Kept::Own(a_desk_day()),
        }
    }

    fn the_recommended_usual_day(moving_km: f64) -> InForce<Volitional> {
        InForce {
            values: Volitional {
                sitting_min: 480.0,
                standing_min: 120.0,
                moving_km,
            },
            recommended: true,
        }
    }

    fn born(birthday: &str) -> GatherRecordsInMemory {
        let gather = GatherRecordsInMemory::default();
        set_profile(
            &gather,
            &ProfileForm {
                birthday: birthday.into(),
                ..a_mans_form("2026-03-01")
            },
        )
        .expect("a profile sets");
        gather
    }

    #[test]
    fn a_usual_day_set_from_a_date_is_the_one_in_force_from_that_date() {
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            crate::testing::BurnRecordsInMemory::default(),
        );
        set_usual_day(&gather, &burn, &a_desk_day_from("2026-03-01")).expect("a usual day sets");

        assert_eq!(
            usual_day_on(&gather, &burn, "2026-10-10"),
            Ok(Some(InForce {
                values: a_desk_day(),
                recommended: false
            }))
        );
    }

    #[test]
    fn a_usual_day_never_kept_follows_the_recommendation_for_the_age_none_without_a_profile() {
        let burn = crate::testing::BurnRecordsInMemory::default();

        assert_eq!(
            (
                usual_day_on(&born("1985-04-12"), &burn, "2026-10-10"),
                usual_day_on(&GatherRecordsInMemory::default(), &burn, "2026-10-10")
            ),
            (Ok(Some(the_recommended_usual_day(6.0))), Ok(None))
        );
    }

    #[test]
    fn the_usual_days_recommendation_kept_from_a_day_follows_the_age_own_values_holding_before_it()
    {
        let (gather, burn) = (
            born("1966-10-15"),
            crate::testing::BurnRecordsInMemory::default(),
        );
        set_usual_day(&gather, &burn, &a_desk_day_from("2026-03-01")).expect("a usual day sets");
        set_usual_day(
            &gather,
            &burn,
            &UsualDayForm {
                from: "2026-10-10".into(),
                kept: Kept::Recommended,
            },
        )
        .expect("the recommendation keeps");

        let in_force = |date| usual_day_on(&gather, &burn, date).ok().flatten();

        assert_eq!(
            [
                in_force("2026-10-05"),
                in_force("2026-10-11"),
                in_force("2026-10-16")
            ],
            [
                Some(InForce {
                    values: a_desk_day(),
                    recommended: false
                }),
                Some(the_recommended_usual_day(6.0)),
                Some(the_recommended_usual_day(4.5))
            ]
        );
    }

    #[test]
    fn a_usual_day_with_more_minutes_than_a_day_holds_is_refused() {
        let gather = GatherRecordsInMemory::default();
        let refused = set_usual_day(
            &gather,
            &crate::testing::BurnRecordsInMemory::default(),
            &UsualDayForm {
                from: "2026-03-01".into(),
                kept: Kept::Own(Volitional {
                    sitting_min: 1500.0,
                    ..a_desk_day()
                }),
            },
        );

        assert_eq!(refused, Err(Failure::Refused(Refusal::NotADistance)));
        assert_eq!(
            usual_day_on(
                &gather,
                &crate::testing::BurnRecordsInMemory::default(),
                "2026-02-30"
            ),
            Err(Failure::Refused(Refusal::NotADate))
        );
    }

    fn own_targets() -> FuelTargets {
        FuelTargets {
            protein_g_per_kg: 2.025,
            fat_g_per_kg: 0.9,
            fibre_g_per_kg: 0.4,
        }
    }

    fn the_recommended_targets(fibre_g_per_kg: f64) -> InForce<FuelTargets> {
        InForce {
            values: FuelTargets {
                protein_g_per_kg: 1.6,
                fat_g_per_kg: 0.8,
                fibre_g_per_kg,
            },
            recommended: true,
        }
    }

    fn fuel_targets_kept(from: &str, kept: Kept<FuelTargets>) -> FuelTargetsForm {
        FuelTargetsForm {
            from: from.into(),
            kept,
        }
    }

    #[test]
    fn fuel_targets_kept_from_a_date_are_the_ones_in_force_from_that_date() {
        let (gather, fuel) = (
            GatherRecordsInMemory::default(),
            crate::testing::FuelRecordsInMemory::default(),
        );
        set_fuel_targets(
            &gather,
            &fuel,
            &fuel_targets_kept("2026-03-01", Kept::Own(own_targets())),
        )
        .expect("targets set");

        assert_eq!(
            fuel_targets_on(&gather, &fuel, "2026-10-10"),
            Ok(Some(InForce {
                values: own_targets(),
                recommended: false
            }))
        );
    }

    #[test]
    fn fuel_targets_never_kept_follow_the_recommendation_its_fibre_spread_over_the_body_mass_none_without_a_profile()
     {
        let gather = born("1985-04-12");
        crate::for_tracking::log_measurement(&gather, "2026-10-01", 80.0, None)
            .expect("a measurement logs");
        let fuel = crate::testing::FuelRecordsInMemory::default();

        assert_eq!(
            (
                fuel_targets_on(&gather, &fuel, "2026-10-10"),
                fuel_targets_on(&GatherRecordsInMemory::default(), &fuel, "2026-10-10")
            ),
            (Ok(Some(the_recommended_targets(0.31))), Ok(None))
        );
    }

    #[test]
    fn the_fuel_targets_recommendation_kept_from_a_day_follows_the_body_mass_own_values_holding_before_it()
     {
        let gather = born("1985-04-12");
        crate::for_tracking::log_measurement(&gather, "2026-10-01", 80.0, None)
            .expect("a measurement logs");
        crate::for_tracking::log_measurement(&gather, "2026-10-12", 62.5, None)
            .expect("a measurement logs");
        let fuel = crate::testing::FuelRecordsInMemory::default();
        for (from, kept) in [
            ("2026-03-01", Kept::Own(own_targets())),
            ("2026-10-10", Kept::Recommended),
        ] {
            set_fuel_targets(&gather, &fuel, &fuel_targets_kept(from, kept)).expect("targets keep");
        }

        let in_force = |date| fuel_targets_on(&gather, &fuel, date).ok().flatten();

        assert_eq!(
            [
                in_force("2026-10-05"),
                in_force("2026-10-11"),
                in_force("2026-10-12")
            ],
            [
                Some(InForce {
                    values: own_targets(),
                    recommended: false
                }),
                Some(the_recommended_targets(0.31)),
                Some(the_recommended_targets(0.4))
            ]
        );
    }

    #[test]
    fn fuel_targets_below_nothing_are_refused() {
        let refused = set_fuel_targets(
            &GatherRecordsInMemory::default(),
            &crate::testing::FuelRecordsInMemory::default(),
            &fuel_targets_kept(
                "2026-03-01",
                Kept::Own(FuelTargets {
                    protein_g_per_kg: -2.0,
                    ..own_targets()
                }),
            ),
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
    fn a_burn_target_set_from_a_date_is_the_one_in_force_from_that_date_the_recommendation_before_it()
     {
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            crate::testing::BurnRecordsInMemory::default(),
        );
        crate::for_tracking::log_measurement(&gather, "2026-10-01", 72.0, None)
            .expect("a measurement logs");
        set_burn_target(
            &gather,
            &burn,
            &BurnTargetForm {
                from: "2026-10-15".into(),
                kept: Kept::Own(150.0),
            },
        )
        .expect("a target sets");

        assert_eq!(
            (
                burn_target_on(&gather, &burn, "2026-10-10"),
                burn_target_on(&gather, &burn, "2026-10-20")
            ),
            (
                Ok(Some(InForce {
                    values: 3.0 * 72.0 * 150.0 / 7.0 / 60.0,
                    recommended: true
                })),
                Ok(Some(InForce {
                    values: 150.0,
                    recommended: false
                }))
            )
        );
    }

    #[test]
    fn the_burn_targets_recommendation_kept_from_a_day_follows_the_body_as_it_changes_own_values_holding_before_it()
     {
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            crate::testing::BurnRecordsInMemory::default(),
        );
        crate::for_tracking::log_measurement(&gather, "2026-10-01", 72.0, None)
            .expect("a measurement logs");
        crate::for_tracking::log_measurement(&gather, "2026-10-12", 80.0, None)
            .expect("a measurement logs");
        for (from, kept) in [
            ("2026-10-01", Kept::Own(150.0)),
            ("2026-10-10", Kept::Recommended),
        ] {
            set_burn_target(
                &gather,
                &burn,
                &BurnTargetForm {
                    from: from.into(),
                    kept,
                },
            )
            .expect("a target keeps");
        }

        let in_force = |date| burn_target_on(&gather, &burn, date).ok().flatten();

        assert_eq!(
            [
                in_force("2026-10-05"),
                in_force("2026-10-11"),
                in_force("2026-10-12")
            ],
            [
                Some(InForce {
                    values: 150.0,
                    recommended: false
                }),
                Some(InForce {
                    values: 3.0 * 72.0 * 150.0 / 7.0 / 60.0,
                    recommended: true
                }),
                Some(InForce {
                    values: 3.0 * 80.0 * 150.0 / 7.0 / 60.0,
                    recommended: true
                })
            ]
        );
    }

    #[test]
    fn a_burn_target_below_nothing_a_day_the_calendar_lacks_or_broken_records_set_nothing() {
        let target = |from: &str, kcal: f64| BurnTargetForm {
            from: from.into(),
            kept: Kept::Own(kcal),
        };
        let (gather, burn) = (
            GatherRecordsInMemory::default(),
            crate::testing::BurnRecordsInMemory::default(),
        );

        assert_eq!(
            set_burn_target(&gather, &burn, &target("2026-10-15", -10.0)),
            Err(Failure::Refused(Refusal::NotAnEnergy))
        );
        assert_eq!(
            set_burn_target(&gather, &burn, &target("2026-02-30", 150.0)),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            burn_target_on(&GatherRecordsInMemory::default(), &burn, "2026-02-30"),
            Err(Failure::Refused(Refusal::NotADate))
        );
        assert_eq!(
            set_burn_target(
                &gather,
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
