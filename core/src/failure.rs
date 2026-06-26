use serde::{Deserialize, Serialize};

use crate::refusal::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Failure {
    Refused(Refusal),
    Broken(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Broken(pub String);

impl From<Broken> for Failure {
    fn from(Broken(reason): Broken) -> Self {
        Failure::Broken(reason)
    }
}
