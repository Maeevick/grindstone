#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Figure {
    Mass(f64),
}

pub fn shown(figure: Figure) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mass_shows_in_kilograms_to_the_tenth() {
        assert_eq!(shown(Figure::Mass(72.43)), "72.4 kg");
    }
}
