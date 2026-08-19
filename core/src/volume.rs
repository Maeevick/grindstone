use crate::refusal::Refusal;

pub fn volume(ml: f64) -> Result<f64, Refusal> {
    if ml.is_finite() && ml > 0.0 {
        Ok(ml)
    } else {
        Err(Refusal::NotAVolume)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_volume_above_nothing_keeps_its_millilitres() {
        assert_eq!(volume(250.0), Ok(250.0));
    }

    #[test]
    fn a_volume_of_nothing_or_no_number_at_all_is_refused() {
        for ml in [0.0, -250.0, f64::NAN, f64::INFINITY] {
            assert_eq!(volume(ml), Err(Refusal::NotAVolume), "{ml}");
        }
    }
}
