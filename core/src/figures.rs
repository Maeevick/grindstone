use crate::food::Unit;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Figure {
    Mass(f64),
    Length(f64),
    Ratio(f64),
    Energy(f64),
    Duration(f64),
    Distance(f64),
    Grams(f64),
    Amount(f64, Unit),
    Volume(f64),
    Count(f64),
}

fn to_the_tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn to_the_hundredth(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub fn shown(figure: Figure) -> String {
    let (value, unit) = match figure {
        Figure::Mass(kg) => (kg, " kg"),
        Figure::Length(cm) => (cm, " cm"),
        Figure::Ratio(ratio) => (ratio * 100.0, "%"),
        Figure::Energy(kcal) => return format!("{} kcal", kcal.round() + 0.0),
        Figure::Duration(minutes) => return format!("{} min", to_the_tenth(minutes)),
        Figure::Distance(km) => return format!("{} km", to_the_hundredth(km)),
        Figure::Grams(grams) => return format!("{} g", grams.round() + 0.0),
        Figure::Count(count) => return format!("{}", to_the_tenth(count) + 0.0),
        Figure::Amount(amount, unit) => {
            let amount = to_the_tenth(amount);
            let unit = match unit {
                Unit::Gram => "g",
                Unit::Millilitre => "mL",
                Unit::Piece if amount == 1.0 => "piece",
                Unit::Piece => "pieces",
            };
            return format!("{amount} {unit}");
        }
        Figure::Volume(ml) => return format!("{} mL", ml.round() + 0.0),
    };
    format!("{:.1}{unit}", to_the_tenth(value))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn a_duration_shows_in_minutes_to_the_tenth_without_a_trailing_zero() {
        assert_eq!(shown(Figure::Duration(30.0)), "30 min");
        assert_eq!(shown(Figure::Duration(0.25)), "0.3 min");
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
    fn a_figure_halfway_between_two_tenths_shows_the_upper_one() {
        assert_eq!(shown(Figure::Mass(72.25)), "72.3 kg");
    }

    #[test]
    fn a_volume_shows_in_whole_millilitres_a_half_going_up() {
        assert_eq!(shown(Figure::Volume(1234.5)), "1235 mL");
    }
}
