use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::failure::Broken;

/// A copy of every record, as the store writes it, with how many tables and records it holds.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordsCopy {
    pub text: String,
    pub counted: Counted,
}

/// How many tables and records a copy holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Counted {
    pub tables: usize,
    pub records: usize,
}

/// Why a copy could not be restored: not a copy at all, a copy of another schema, or a broken store.
#[derive(Clone, Debug, PartialEq)]
pub enum Unrestorable {
    NotACopy,
    AnotherSchema { copy: i64, store: i64 },
    Broken(Broken),
}

pub trait ForCopyingRecords {
    fn copied(&self, on: NaiveDate) -> Result<RecordsCopy, Broken>;
    /// Every record replaced by the copy's, all of them or none.
    fn restored(&self, copy: &str) -> Result<Counted, Unrestorable>;
}
