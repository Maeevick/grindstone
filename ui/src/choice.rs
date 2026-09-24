const LETTERS_TO_FILTER: usize = 3;

/// The text as compared: lower case, accents dropped, trimmed.
fn folded(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .fold(String::new(), |mut plain, letter| {
            match letter {
                'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => plain.push('a'),
                'ç' => plain.push('c'),
                'è' | 'é' | 'ê' | 'ë' => plain.push('e'),
                'ì' | 'í' | 'î' | 'ï' => plain.push('i'),
                'ñ' => plain.push('n'),
                'ò' | 'ó' | 'ô' | 'õ' | 'ö' => plain.push('o'),
                'ù' | 'ú' | 'û' | 'ü' => plain.push('u'),
                'ý' | 'ÿ' => plain.push('y'),
                'œ' => plain.push_str("oe"),
                'æ' => plain.push_str("ae"),
                other => plain.push(other),
            }
            plain
        })
}

/// The options to offer for what is typed: every one below three letters; from three, those
/// starting with it first, then those containing it, ignoring case and accents.
pub fn matching(options: &[String], typed: &str) -> Vec<String> {
    let typed = folded(typed);
    if typed.chars().count() < LETTERS_TO_FILTER {
        return options.to_vec();
    }
    let (starting, containing): (Vec<&String>, Vec<&String>) = options
        .iter()
        .filter(|option| folded(option).contains(&typed))
        .partition(|option| folded(option).starts_with(&typed));
    starting.into_iter().chain(containing).cloned().collect()
}

/// The option typed whole, ignoring case and accents.
pub fn chosen(options: &[String], typed: &str) -> Option<String> {
    let typed = folded(typed);
    options
        .iter()
        .find(|option| folded(option) == typed)
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
