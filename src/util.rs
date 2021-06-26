pub fn max_f64(iter: impl Iterator<Item = f64>) -> Option<f64> {
    iter.reduce(|a, b| if a > b { a } else { b })
}
