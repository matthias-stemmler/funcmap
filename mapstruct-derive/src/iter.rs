/// If `into_iter` produces exactly one item, returns that item, otherwise returns `None`
pub fn single<I>(into_iter: I) -> Option<I::Item>
where
    I: IntoIterator,
{
    let mut iter = into_iter.into_iter();

    match iter.next() {
        Some(item) if iter.next().is_none() => Some(item),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let input: [(); 0] = [];

        let result = single(input);

        assert!(result.is_none());
    }

    #[test]
    fn test_single() {
        let input = ["value"];

        let result = single(input);

        assert_eq!(result, Some("value"));
    }

    #[test]
    fn test_multiple() {
        let input = ["value 1", "value 2"];

        let result = single(input);

        assert!(result.is_none());
    }
}
