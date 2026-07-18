pub fn figure_typed(text: &str) -> Option<f64> {
    let typed = text.trim();
    (!typed.is_empty()).then(|| typed.parse().unwrap_or(f64::NAN))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_typed_is_no_figure() {
        assert_eq!(figure_typed("  "), None);
    }
}
