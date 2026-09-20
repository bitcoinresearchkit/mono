use super::*;

#[test]
fn date_intersection_stays_sorted() {
    let a = Date::new(2026, 8, 1);
    let b = Date::new(2026, 8, 2);
    let c = Date::new(2026, 8, 3);

    for left in [vec![], vec![a], vec![a, b, c]] {
        for right in [vec![], vec![a], vec![b], vec![c], vec![a, c], vec![a, b, c]] {
            let expected: Vec<_> = left
                .iter()
                .copied()
                .filter(|date| right.contains(date))
                .collect();
            let mut actual = Vec::new();
            visit_intersection(left.clone(), right, |date| actual.push(date));
            assert_eq!(actual, expected);
        }
    }
}
