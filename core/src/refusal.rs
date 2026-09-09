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
    NotAName,
    NotAMet,
    NotADuration,
    NotACount,
    NotALoad,
    NotADistance,
    NotAnExercise,
    NotAPortion,
    NotANutrient,
    NotAQuantity,
    NotAFood,
    NotASingleStep,
    NotACircuit,
    NotAVolume,
    NotADrink,
    NotAProfile,
}
