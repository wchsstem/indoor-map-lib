use std::collections::HashSet;
use std::hash::Hash;

pub fn shoelace_area(points: &[(f32, f32)]) -> f32 {
    let this = points.iter();
    let next = points.iter().cycle().skip(1);
    let double_area: f32 = this
        .zip(next)
        .map(|((this_x, this_y), (next_x, next_y))| this_x * next_y - (next_x * this_y))
        .sum();
    0.5 * double_area
}

pub fn centroid(points: &[(f32, f32)]) -> (f32, f32) {
    let this = points.iter();
    let next = points.iter().cycle().skip(1);
    let (center_x, center_y) = this
        .zip(next)
        .map(|((this_x, this_y), (next_x, next_y))| {
            let diff = (this_x * next_y) - (next_x * this_y);
            let x = (this_x + next_x) * diff;
            let y = (this_y + next_y) * diff;
            (x, y)
        })
        .fold((0.0, 0.0), |(acc_x, acc_y), (x, y)| (acc_x + x, acc_y + y));

    let coefficient = 1.0 / (6.0 * shoelace_area(points));

    (coefficient * center_x, coefficient * center_y)
}

pub fn max_f64(iter: impl Iterator<Item = f64>) -> Option<f64> {
    iter.reduce(|a, b| if a > b { a } else { b })
}

/// Determines if all `items` are unique
///
/// # Returns
/// If all items are unique, `Ok(<HashSet of unique elements>)`. If there is a repeated element,
/// `Err(<first repeat element>)`.
pub fn unique<T: Eq + Hash>(items: impl Iterator<Item = T>) -> Result<HashSet<T>, T> {
    let mut map = HashSet::new();
    for item in items {
        if !map.contains(&item) {
            map.insert(item);
        } else {
            return Err(item);
        }
    }
    Ok(map)
}

/// Determines if all `items` are present in `defined`.
///
/// # Returns
/// If all items are defined, `Ok(())`. If an item is not defined, `Err(<first undefined element>)`.
pub fn undefined<'a, 'b, T: Eq + Hash + ?Sized>(
    mut items: impl Iterator<Item = &'a T>,
    defined: &'b HashSet<&'b T>,
) -> Result<(), &'a T> {
    if let Some(undefined_item) = items.find(|item| !defined.contains(item)) {
        Err(undefined_item)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use common_macros::hash_set;

    use super::*;

    #[test]
    fn unique_items() {
        let expected = hash_set!["hello", "world", "testing", "643tbu346y", "u34i6"];
        let actual: HashSet<&str> = unique(expected.clone().into_iter()).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn repeated_items() {
        let items = vec!["hello", "world", "testing", "643tbu346y", "world", "hello"];
        let actual: &str = unique(items.iter()).unwrap_err();
        assert_eq!("world", actual);
    }

    #[test]
    fn no_undefined_items() {
        let defined = hash_set!["ab", "bc", "cd"];
        let items = vec!["bc", "cd"].into_iter();
        undefined(items, &defined).unwrap();
    }

    #[test]
    fn undefined_items() {
        let defined = hash_set!["ab", "bc", "cd"];
        let items = vec!["cd", "xy", "ab", "zz"].into_iter();
        let actual = undefined(items, &defined).unwrap_err();
        assert_eq!("xy", actual);
    }
}
