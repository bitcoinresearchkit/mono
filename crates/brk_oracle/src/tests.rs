use crate::{Config, HistogramRaw, Oracle, START_HEIGHT_SLOW, cents_to_bin};

/// A candidate needs mass on two arms: one arm's lone hit 8 bins off the reference holds it, a
/// second arm's mass behind the same candidate moves it there.
#[test]
fn a_candidate_needs_two_arms() {
    let reference = cents_to_bin(5_000_000.0).round();
    let candidate = reference as usize + 8;
    let pick = |bins: &[usize]| {
        let mut block = HistogramRaw::default();
        bins.iter().for_each(|&bin| block.increment(bin));
        let mut oracle =
            Oracle::from_checkpoint(reference, Config::default(), |_| Ok::<_, ()>(())).unwrap();
        oracle.process_histogram(&block)
    };
    // The $100 arm alone sees it: every other arm is 35 bins or more away, past the window's 12.
    assert_eq!(pick(&[candidate]), reference, "one arm holds");
    // The $150 arm (35 bins up) sees the second one from the same candidate.
    assert!((pick(&[candidate, candidate + 35]) - candidate as f64).abs() < 0.5, "two arms move it");
}

/// The block's pick from `reference`, with `counts` payments at each bin, under `config`.
fn pick(config: Config, reference: f64, counts: &[(usize, u32)]) -> f64 {
    let mut block = HistogramRaw::default();
    for &(bin, count) in counts {
        (0..count).for_each(|_| block.increment(bin));
    }
    let mut oracle = Oracle::from_checkpoint(reference, config, |_| Ok::<_, ()>(())).unwrap();
    oracle.process_histogram(&block)
}

/// The best supported candidate beside a higher unsupported one keeps its own bin: a parabola
/// through them would push it half a bin onto the empty neighbour.
#[test]
fn a_supported_pick_beside_a_higher_one_keeps_its_bin() {
    // 1008 alone has two arms (1008 and its $150 at 1043); 1007 and 1011 (whose $150 is 1046) one.
    let counts = [(1008, 1), (1043, 1), (1007, 100), (1046, 100)];
    assert_eq!(pick(Config::default(), 1000.0, &counts), 1008.0);
}

/// The slow regime scores an empty candidate's shape 0, above a supported candidate's poor one
/// (two arms against a flat profile): the supported candidate still wins.
#[test]
fn the_slow_regime_never_picks_an_empty_candidate() {
    let reference = cents_to_bin(30_000.0).round();
    let candidate = reference as usize + 5;
    let config = Config::for_height(START_HEIGHT_SLOW);
    let got = pick(config, reference, &[(candidate, 1), (candidate + 35, 1)]);
    assert!((got - candidate as f64).abs() < 0.5, "{got} is not {candidate}");
}
