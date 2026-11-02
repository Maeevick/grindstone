use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Heading {
    Toward,
    Away,
    Steady,
    Unaimed,
    /// Falling, yet still above its floor.
    AboveItsFloor,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub by: f64,
    pub heading: Heading,
}
