pub fn figure_typed(text: &str) -> Option<f64> {
    let typed = text.trim();
    (!typed.is_empty()).then(|| typed.replace(',', ".").parse().unwrap_or(f64::NAN))
}

pub fn count_typed(text: &str) -> Option<u32> {
    let typed = text.trim();
    (!typed.is_empty()).then(|| typed.parse().unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_typed_is_no_figure() {
        assert_eq!(figure_typed("  "), None);
    }

    #[test]
    fn a_figure_typed_with_a_decimal_comma_reads_as_with_a_point() {
        assert_eq!(figure_typed(" 72,4 "), Some(72.4));
        assert_eq!(figure_typed("72.4"), Some(72.4));
    }

    #[test]
    fn something_typed_that_is_no_number_reaches_the_core_as_not_a_number_for_it_to_refuse() {
        assert!(figure_typed("seventy").is_some_and(f64::is_nan));
    }

    #[test]
    fn a_count_typed_reads_as_a_whole_number_and_nothing_typed_as_none() {
        assert_eq!(count_typed(" 12 "), Some(12));
        assert_eq!(count_typed(""), None);
    }

    #[test]
    fn a_count_that_is_no_whole_number_reaches_the_core_as_zero_for_it_to_refuse() {
        assert_eq!(count_typed("twelve"), Some(0));
        assert_eq!(count_typed("-3"), Some(0));
    }
}
