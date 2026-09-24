const LETTERS_TO_FILTER: usize = 3;

/// The options to offer for what is typed: every one below three letters; from three, those
/// starting with it first, then those containing it, ignoring case.
pub fn matching(options: &[String], typed: &str) -> Vec<String> {
    let typed = typed.trim().to_lowercase();
    if typed.chars().count() < LETTERS_TO_FILTER {
        return options.to_vec();
    }
    let (starting, containing): (Vec<&String>, Vec<&String>) = options
        .iter()
        .filter(|option| option.to_lowercase().contains(&typed))
        .partition(|option| option.to_lowercase().starts_with(&typed));
    starting.into_iter().chain(containing).cloned().collect()
}

/// The option typed whole, ignoring case.
pub fn chosen(options: &[String], typed: &str) -> Option<String> {
    let typed = typed.trim().to_lowercase();
    options
        .iter()
        .find(|option| option.to_lowercase() == typed)
        .cloned()
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

    #[test]
    fn accents_are_ignored_whichever_side_carries_them() {
        let catalog = ["Épinards", "Crème fraîche", "Œuf", "Pâtes"]
            .map(String::from)
            .to_vec();

        assert_eq!(matching(&catalog, "epi"), vec!["Épinards".to_string()]);
        assert_eq!(
            matching(&catalog, "CREME"),
            vec!["Crème fraîche".to_string()]
        );
        assert_eq!(matching(&catalog, "oeu"), vec!["Œuf".to_string()]);
        assert_eq!(matching(&catalog, "pât"), vec!["Pâtes".to_string()]);
        assert_eq!(
            chosen(&catalog, "creme fraiche"),
            Some("Crème fraîche".to_string())
        );
    }
}
