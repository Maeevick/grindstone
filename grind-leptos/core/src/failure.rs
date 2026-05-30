use serde::{Deserialize, Serialize};

use crate::weigh_in::Refusal;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Failure {
    Refused(Refusal),
    Broken(String),
}
