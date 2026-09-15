/// The options to offer for what is typed: every one below three letters; from three, those
/// starting with it first, then those containing it, ignoring case.
pub fn matching(_options: &[String], _typed: &str) -> Vec<String> {
    vec![]
}

/// The option typed whole, ignoring case.
pub fn chosen(_options: &[String], _typed: &str) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn the_catalog() -> Vec<String> {
        ["Ail", "Pain complet", "Épinards", "Skyr", "Tapenade"]
            .map(String::from)
            .to_vec()
    }

    #[test]
    fn below_three_letters_every_option_is_offered() {
        assert_eq!(matching(&the_catalog(), "pa"), the_catalog());
    }

    #[test]
    fn from_three_letters_the_options_starting_with_them_come_first_then_those_containing_them() {
        assert_eq!(
            matching(&the_catalog(), " PAI "),
            vec!["Pain complet".to_string()]
        );
        assert_eq!(
            matching(&the_catalog(), "nad"),
            vec!["Tapenade".to_string()]
        );
        assert_eq!(
            matching(&["Tapenade".to_string(), "Penne".to_string()], "pen"),
            vec!["Penne".to_string(), "Tapenade".to_string()]
        );
    }

    #[test]
    fn the_option_chosen_is_the_one_typed_whole_ignoring_case() {
        assert_eq!(chosen(&the_catalog(), " skyr"), Some("Skyr".to_string()));
        assert_eq!(
            chosen(&the_catalog(), "épinards"),
            Some("Épinards".to_string())
        );
        assert_eq!(chosen(&the_catalog(), "sky"), None);
    }
}
