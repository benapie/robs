#[derive(Debug)]
pub struct TSData {
    tss: Vec<usize>,
    vals: Vec<Option<f64>>,
}

impl FromIterator<(usize, Option<f64>)> for TSData {
    fn from_iter<T: IntoIterator<Item = (usize, Option<f64>)>>(iter: T) -> Self {
        let mut items: Vec<(usize, Option<f64>)> = iter.into_iter().collect();

        items.sort_by_key(|&a| (a).0);

        let tss = items.iter().map(|&a| a.0).collect();
        let vals = items.iter().map(|&a| a.1).collect();

        return TSData { tss, vals };
    }
}

impl TSData {
    fn from_iters<T: IntoIterator<Item = usize>, S: IntoIterator<Item = Option<f64>>>(
        ts_iter: T,
        val_iter: S,
    ) -> Result<Self, &'static str> {
        let tss: Vec<usize> = ts_iter.into_iter().collect();
        let vals: Vec<Option<f64>> = val_iter.into_iter().collect();

        if tss.len() != vals.len() {
            return Err("ts and val counts don't match");
        }

        return Ok(TSData { tss, vals });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal_from_iter_should_work() {
        let data = [
            (1, Some(12.0)),
            (6, Some(10.0)),
            (3, Some(2.1)),
            (2, Some(19.2)),
        ];
        let ts_data = TSData::from_iter(data);

        assert_eq!(ts_data.tss, [1, 2, 3, 6]);
        assert_eq!(
            ts_data.vals,
            [Some(12.0), Some(19.2), Some(2.1), Some(10.0)]
        );
    }
}
