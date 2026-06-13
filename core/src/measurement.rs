use chrono::NaiveDate;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurement {
    pub day: NaiveDate,
    pub body_mass_kg: f64,
    pub waist_cm: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    NotADate,
}

pub fn measurement(
    date: &str,
    body_mass_kg: f64,
    waist_cm: Option<f64>,
) -> Result<Measurement, Refusal> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Refusal::NotADate)?;
    Ok(Measurement {
        day,
        body_mass_kg,
        waist_cm,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_date_that_does_not_exist_is_refused() {
        assert_eq!(measurement("2026-02-30", 72.1, None), Err(Refusal::NotADate));
        assert_eq!(measurement("yesterday", 72.1, None), Err(Refusal::NotADate));
    }
}
