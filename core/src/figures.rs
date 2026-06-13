#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Figure {
    Mass(f64),
    Length(f64),
    Ratio(f64),
}

fn to_the_tenth(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

pub fn shown(figure: Figure) -> String {
    let (value, unit) = match figure {
        Figure::Mass(kg) => (kg, " kg"),
        Figure::Length(cm) => (cm, " cm"),
        Figure::Ratio(ratio) => (ratio * 100.0, "%"),
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
    fn a_figure_halfway_between_two_tenths_shows_the_upper_one() {
        assert_eq!(shown(Figure::Mass(72.25)), "72.3 kg");
    }
}
