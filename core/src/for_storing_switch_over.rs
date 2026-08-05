use chrono::NaiveDate;

use crate::failure::Broken;

pub trait ForStoringSwitchOver {
    fn save_switch_over(&self, day: NaiveDate) -> Result<(), Broken>;
    fn switch_over(&self) -> Result<Option<NaiveDate>, Broken>;
}
