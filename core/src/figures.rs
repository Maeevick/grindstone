use crate::food::Unit;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Figure {
    Mass(f64),
    /// A mass moved in a week, where the hundredth tells a pace from its limit.
    WeeklyMass(f64),
    Length(f64),
    Ratio(f64),
    Energy(f64),
    Duration(f64),
    Distance(f64),
    Grams(f64),
    Amount(f64, Unit),
    Volume(f64),
    Count(f64),
    Times(f64),
}

fn to_the_tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn to_the_hundredth(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub fn typed_back(value: f64) -> String {
    format!("{}", to_the_hundredth(value) + 0.0)
}

fn parts(figure: Figure) -> (String, &'static str) {
    match figure {
        Figure::Mass(kg) => (format!("{:.1}", to_the_tenth(kg) + 0.0), " kg"),
        Figure::WeeklyMass(kg) => (format!("{:.2}", to_the_hundredth(kg) + 0.0), " kg"),
        Figure::Length(cm) => (format!("{:.1}", to_the_tenth(cm) + 0.0), " cm"),
        Figure::Ratio(ratio) => (format!("{:.1}", to_the_tenth(ratio * 100.0) + 0.0), "%"),
        Figure::Energy(kcal) => (format!("{}", kcal.round() + 0.0), " kcal"),
        Figure::Duration(minutes) => (format!("{}", to_the_hundredth(minutes) + 0.0), " min"),
        Figure::Distance(km) => (format!("{}", to_the_hundredth(km) + 0.0), " km"),
        Figure::Grams(grams) => (format!("{}", grams.round() + 0.0), " g"),
        Figure::Count(count) => (format!("{}", to_the_tenth(count) + 0.0), ""),
        Figure::Amount(amount, unit) => {
            let amount = to_the_tenth(amount) + 0.0;
            let unit = match unit {
                Unit::Gram => " g",
                Unit::Millilitre => " mL",
                Unit::Piece if amount == 1.0 => " piece",
                Unit::Piece => " pieces",
            };
            (format!("{amount}"), unit)
        }
        Figure::Volume(ml) => (format!("{}", ml.round() + 0.0), " mL"),
        Figure::Times(ratio) => (format!("{}", to_the_hundredth(ratio) + 0.0), ""),
    }
}

pub fn shown(figure: Figure) -> String {
    let (number, unit) = parts(figure);
    match figure {
        Figure::Times(_) => format!("× {number}"),
        _ => format!("{number}{unit}"),
    }
}

/// The number alone, for a table whose header holds the unit.
pub fn bare(figure: Figure) -> String {
    parts(figure).0
}

/// A change, showing which way it went; one that rounds to nothing shows no sign.
pub fn signed(figure: Figure) -> String {
    let number = bare(figure);
    if number.starts_with('-') || number.trim_start_matches(['0', '.']).is_empty() {
        number
    } else {
        format!("+{number}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mass_moved_in_a_week_shows_to_the_hundredth() {
        assert_eq!(
            [
                shown(Figure::WeeklyMass(-0.623)),
                shown(Figure::WeeklyMass(0.5)),
                shown(Figure::WeeklyMass(-0.2)),
            ],
            ["-0.62 kg", "0.50 kg", "-0.20 kg"]
        );
    }
    #[test]
    fn a_mass_shows_in_kilograms_to_the_tenth() {
        assert_eq!(shown(Figure::Mass(72.43)), "72.4 kg");
    }

    #[test]
    fn a_length_shows_in_centimetres_to_the_tenth() {
        assert_eq!(shown(Figure::Length(84.46)), "84.5 cm");
    }

    #[test]
    fn a_ratio_shows_as_a_percentage_to_the_tenth() {
        assert_eq!(shown(Figure::Ratio(0.1521)), "15.2%");
    }

    #[test]
    fn an_energy_shows_in_whole_kilocalories() {
        assert_eq!(shown(Figure::Energy(251.6)), "252 kcal");
        assert_eq!(shown(Figure::Energy(-0.4)), "0 kcal");
    }

    #[test]
    fn a_duration_shows_in_minutes_to_the_hundredth_without_trailing_zeros() {
        assert_eq!(
            [
                shown(Figure::Duration(30.0)),
                shown(Figure::Duration(0.75)),
                shown(Figure::Duration(1.5)),
                shown(Figure::Duration(1.0 / 3.0)),
                bare(Figure::Duration(0.75)),
            ],
            ["30 min", "0.75 min", "1.5 min", "0.33 min", "0.75"]
        );
    }

    #[test]
    fn a_distance_shows_in_kilometres_to_the_hundredth() {
        assert_eq!(shown(Figure::Distance(4.234)), "4.23 km");
    }

    #[test]
    fn grams_of_a_macro_show_whole() {
        assert_eq!(shown(Figure::Grams(19.4)), "19 g");
    }

    #[test]
    fn an_amount_shows_in_its_unit_to_the_tenth_and_pieces_count_as_words() {
        assert_eq!(shown(Figure::Amount(200.0, Unit::Gram)), "200 g");
        assert_eq!(shown(Figure::Amount(330.0, Unit::Millilitre)), "330 mL");
        assert_eq!(shown(Figure::Amount(1.0, Unit::Piece)), "1 piece");
        assert_eq!(shown(Figure::Amount(2.5, Unit::Piece)), "2.5 pieces");
    }

    #[test]
    fn a_count_shows_to_the_tenth_without_a_trailing_zero() {
        assert_eq!(shown(Figure::Count(36.0)), "36");
        assert_eq!(shown(Figure::Count(1.1875)), "1.2");
    }

    #[test]
    fn a_figure_typed_back_into_a_field_reads_to_the_hundredth_without_trailing_zeros() {
        assert_eq!(typed_back(70.308), "70.31");
        assert_eq!(typed_back(0.83), "0.83");
        assert_eq!(typed_back(2000.0), "2000");
        assert_eq!(typed_back(6.6), "6.6");
    }

    #[test]
    fn a_figure_halfway_between_two_tenths_shows_the_upper_one() {
        assert_eq!(shown(Figure::Mass(72.25)), "72.3 kg");
    }

    #[test]
    fn a_volume_shows_in_whole_millilitres_a_half_going_up() {
        assert_eq!(shown(Figure::Volume(1234.5)), "1235 mL");
    }

    #[test]
    fn a_bare_figure_shows_its_number_without_its_unit_for_a_header_to_hold_it() {
        assert_eq!(
            [
                bare(Figure::Mass(72.43)),
                bare(Figure::Length(84.46)),
                bare(Figure::Ratio(0.1521)),
                bare(Figure::Energy(251.6)),
                bare(Figure::Grams(19.4)),
                bare(Figure::Volume(1234.5)),
                bare(Figure::Duration(30.0)),
                bare(Figure::Amount(2.5, Unit::Piece)),
            ],
            ["72.4", "84.5", "15.2", "252", "19", "1235", "30", "2.5"]
        );
    }

    #[test]
    fn a_signed_figure_shows_which_way_it_went_and_nothing_when_it_rounds_to_nothing() {
        assert_eq!(
            [
                signed(Figure::Mass(0.34)),
                signed(Figure::Mass(-1.0)),
                signed(Figure::Mass(-0.04)),
                signed(Figure::Ratio(-0.015)),
                signed(Figure::Grams(12.4)),
                signed(Figure::Grams(-0.3)),
            ],
            ["+0.3", "-1.0", "0.0", "-1.5", "+12", "0"]
        );
    }

    #[test]
    fn a_figure_that_rounds_to_nothing_shows_no_minus() {
        assert_eq!(
            [
                shown(Figure::Mass(-0.04)),
                shown(Figure::Length(-0.04)),
                shown(Figure::Ratio(-0.0004)),
                shown(Figure::Energy(-0.4)),
                shown(Figure::Duration(-0.004)),
                shown(Figure::Distance(-0.004)),
                shown(Figure::Grams(-0.4)),
                shown(Figure::Count(-0.04)),
                shown(Figure::Amount(-0.04, Unit::Gram)),
                shown(Figure::Volume(-0.4)),
            ],
            [
                "0.0 kg", "0.0 cm", "0.0%", "0 kcal", "0 min", "0 km", "0 g", "0", "0 g", "0 mL"
            ]
        );
    }

    #[test]
    fn a_ratio_of_masses_shows_as_times_to_the_hundredth_without_trailing_zeros() {
        assert_eq!(
            [
                shown(Figure::Times(0.66)),
                shown(Figure::Times(2.5)),
                shown(Figure::Times(1.0)),
                bare(Figure::Times(0.666)),
            ],
            ["× 0.66", "× 2.5", "× 1", "0.67"]
        );
    }
}
