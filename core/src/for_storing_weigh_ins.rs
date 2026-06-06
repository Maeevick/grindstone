use crate::failure::Broken;
use crate::weigh_in::WeighIn;

pub trait ForStoringWeighIns {
    fn save(&self, weigh_in: &WeighIn) -> Result<(), Broken>;
    fn recent(&self, count: u32) -> Result<Vec<WeighIn>, Broken>;
}
