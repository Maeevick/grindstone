#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Figure {
    Mass(f64),
    Length(f64),
    Ratio(f64),
}

pub fn shown(figure: Figure) -> String {
    match figure {
        Figure::Mass(kg) => format!("{kg:.1} kg"),
        Figure::Length(cm) => format!("{cm:.1} cm"),
        Figure::Ratio(_) => String::new(),
    }
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
}
