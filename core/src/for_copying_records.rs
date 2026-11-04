use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::failure::Broken;

/// A copy of every record, as the store writes it, with how many tables and records it holds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordsCopy {
    pub text: String,
    pub tables: usize,
    pub records: usize,
}

pub trait ForCopyingRecords {
    fn copied(&self, on: NaiveDate) -> Result<RecordsCopy, Broken>;
}
