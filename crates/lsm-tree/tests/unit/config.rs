use super::*;

#[test]
fn policies_repeat_the_final_value_without_changing_slice_lookup() {
    macro_rules! check {
        ($policy:ident, $first:expr, $last:expr) => {{
            let policy = $policy::new(vec![$first, $last]);
            assert_eq!(policy.at_level(0), $first);
            assert_eq!(policy.at_level(1), $last);
            assert_eq!(policy.at_level(2), $last);
            assert_eq!(policy.at_level(usize::MAX), $last);
            assert_eq!(policy.get(2), None);
            let uniform = $policy::all($first);
            assert_eq!(uniform.at_level(usize::MAX), $first);
        }};
    }
    check!(BlockSizePolicy, 4096, 8192);
    check!(
        CompressionPolicy,
        CompressionType::None,
        CompressionType::Lz4
    );
    check!(
        FilterPolicy,
        FilterPolicyEntry::None,
        FilterPolicyEntry::Bloom(BloomConstructionPolicy::BitsPerKey(10.0))
    );
    check!(PinningPolicy, false, true);
    check!(RestartIntervalPolicy, 8, 16);
}
