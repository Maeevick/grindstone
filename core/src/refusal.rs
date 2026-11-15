use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Refusal {
    NotADate,
    NotAMass,
    NotAWaist,
    NotAHeight,
    NotANeck,
    NotAHip,
    NotARatio,
    /// A goal's fat under the tracker's floor for the gender, in percent.
    GoalFatUnderItsFloor {
        percent: u32,
    },
    NotAName,
    NotAMet,
    NotADuration,
    NotACount,
    NotALoad,
    NotADistance,
    NotAnExercise,
    NotAPortion,
    NotANutrient,
    /// Fuel targets outside the recommendation, per kilo of body mass on the day they are kept.
    FuelTargetsOutsideTheRecommendation,
    NotAQuantity,
    NotAFood,
    NotASingleStep,
    NotACircuit,
    NotAVolume,
    NotADrink,
    NotAProfile,
    NotACookingRatio,
    NotWeighed,
    NotAnEnergy,
    NotACopy,
    NoCopyPushed,
    AnotherSchema {
        copy: i64,
        store: i64,
    },
}
