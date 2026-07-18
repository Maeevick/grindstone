pub fn figure_typed(text: &str) -> Option<f64> {
    Some(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_typed_is_no_figure() {
        assert_eq!(figure_typed("  "), None);
    }
}
