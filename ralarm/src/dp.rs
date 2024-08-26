#[derive(Debug, PartialEq)]
pub struct Dp<T> {
    pub ts: usize,
    pub val: T,
}

impl<T> From<(usize, T)> for Dp<T> {
    fn from(val: (usize, T)) -> Self {
        return Dp {
            ts: val.0,
            val: val.1,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_works() {
        let actual = Dp::from((1, "hi"));
        let expected = Dp { ts: 1, val: "hi" };

        assert_eq!(actual, expected)
    }
}
