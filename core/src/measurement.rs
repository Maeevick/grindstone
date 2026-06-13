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

fn to_two_decimals(figure: f64) -> f64 {
    (figure * 100.0).round() / 100.0
}

pub fn measurement(
    date: &str,
    body_mass_kg: f64,
    waist_cm: Option<f64>,
) -> Result<Measurement, Refusal> {
    let day = NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| Refusal::NotADate)?;
    Ok(Measurement {
        day,
        body_mass_kg: to_two_decimals(body_mass_kg),
        waist_cm,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(date: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("a real day")
    }

    #[test]
    fn a_measurement_keeps_its_day_and_its_mass_to_two_decimals() {
        assert_eq!(
            measurement("2026-10-10", 72.149, None),
            Ok(Measurement {
                day: day("2026-10-10"),
                body_mass_kg: 72.15,
                waist_cm: None,
            })
        );
    }

    #[test]
    fn a_date_that_does_not_exist_is_refused() {
        assert_eq!(measurement("2026-02-30", 72.1, None), Err(Refusal::NotADate));
        assert_eq!(measurement("yesterday", 72.1, None), Err(Refusal::NotADate));
    }
}
