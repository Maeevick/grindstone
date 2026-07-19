pub fn figure_typed(text: &str) -> Option<f64> {
    let typed = text.trim();
    (!typed.is_empty()).then(|| typed.replace(',', ".").parse().unwrap_or(f64::NAN))
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
}
