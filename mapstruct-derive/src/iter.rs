/// Replaces the item with index `idx` produced by `into_iter` with the items produced by `replacement_into_iter`
/// If `idx` equals the count of `into_iter`, appends the items produced by `replacement_into_iter` at the end
pub fn replace_at<I, J>(into_iter: I, idx: usize, replacement: J) -> impl Iterator<Item = I::Item>
where
    I: IntoIterator,
    J: IntoIterator<Item = I::Item>,
{
    ReplaceAt::new(into_iter.into_iter(), idx, replacement.into_iter())
}

#[derive(Debug)]
struct ReplaceAt<I, J> {
    iter: I,
    replacement: Option<(usize, J)>,
}

impl<I, J> ReplaceAt<I, J> {
    fn new(iter: I, idx: usize, replacement_iter: J) -> Self {
        Self {
            iter,
            replacement: Some((idx, replacement_iter)),
        }
    }
}

impl<I, J> Iterator for ReplaceAt<I, J>
where
    I: Iterator,
    J: Iterator<Item = I::Item>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.replacement {
            Some((0, replacement_iter)) => match replacement_iter.next() {
                Some(item) => return Some(item),
                None => {
                    self.replacement = None;
                    self.iter.next();
                }
            },
            Some((count, _)) => *count -= 1,
            _ => (),
        };

        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_at_iter_empty() {
        let iter: [&str; 0] = [];
        let replacement = ["A", "B"];

        let result: Vec<_> = replace_at(iter, 0, replacement).collect();

        assert_eq!(result, replacement);
    }

    #[test]
    fn replace_at_replacement_empty() {
        let iter = ["a", "b"];
        let replacement: [&str; 0] = [];

        let result: Vec<_> = replace_at(iter, 1, replacement).collect();

        assert_eq!(result, vec!["a"]);
    }

    #[test]
    fn replace_at_idx_out_of_range() {
        let iter = ["a", "b"];
        let replacement = ["A", "B"];

        let result: Vec<_> = replace_at(iter, 3, replacement).collect();

        assert_eq!(result, iter);
    }

    #[test]
    fn replace_at_beginning() {
        let iter = ["a", "b"];
        let replacement = ["A", "B"];

        let result: Vec<_> = replace_at(iter, 0, replacement).collect();

        assert_eq!(result, vec!["A", "B", "b"]);
    }

    #[test]
    fn replace_at_middle() {
        let iter = ["a", "b"];
        let replacement = ["A", "B"];

        let result: Vec<_> = replace_at(iter, 1, replacement).collect();

        assert_eq!(result, vec!["a", "A", "B"]);
    }

    #[test]
    fn replace_at_end() {
        let iter = ["a", "b"];
        let replacement = ["A", "B"];

        let result: Vec<_> = replace_at(iter, 2, replacement).collect();

        assert_eq!(result, vec!["a", "b", "A", "B"]);
    }
}
