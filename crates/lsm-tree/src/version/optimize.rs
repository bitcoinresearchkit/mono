// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use super::run::Ranged;
use crate::version::Run;

fn merge_disjoint<T: Ranged>(run: &mut Run<T>, additions: Vec<T>) {
    if additions.is_empty() {
        return;
    }

    let existing = std::mem::take(run.inner_mut());
    let mut existing = existing.into_iter().peekable();
    let mut additions = additions.into_iter().peekable();
    let mut merged = Vec::with_capacity(existing.len() + additions.len());

    while let (Some(existing_item), Some(addition)) = (existing.peek(), additions.peek()) {
        let take_existing = existing_item.key_range().min() < addition.key_range().min();

        let item = if take_existing {
            existing.next()
        } else {
            additions.next()
        };

        if let Some(item) = item {
            merged.push(item);
        }
    }

    merged.extend(existing);
    merged.extend(additions);
    *run.inner_mut() = merged;
}

pub fn optimize_runs<T: Ranged>(runs: Vec<Run<T>>) -> Vec<Run<T>> {
    if runs.len() <= 1 {
        runs
    } else {
        let mut new_runs: Vec<Run<T>> = Vec::with_capacity(runs.len());

        for run in runs {
            // Tables within one source run are already sorted and disjoint, so none of them can
            // constrain another table from the same run. Determine all destination runs against
            // the previously processed (newer) runs, then merge each sorted batch once.
            let mut additions = (0..=new_runs.len()).map(|_| Vec::new()).collect::<Vec<_>>();

            for table in run.into_inner() {
                // A table must remain behind every newer run that overlaps it;
                // otherwise point reads can find an older value first.
                let last_overlap = new_runs
                    .iter()
                    .rposition(|run| !run.get_overlapping(table.key_range()).is_empty());

                let target = last_overlap.map_or(0, |index| index + 1);
                #[expect(
                    clippy::expect_used,
                    reason = "the destination is at most the one trailing run allocated above"
                )]
                additions
                    .get_mut(target)
                    .expect("destination run should exist")
                    .push(table);
            }

            #[expect(
                clippy::expect_used,
                reason = "one trailing destination is allocated above"
            )]
            let trailing = additions.pop().expect("trailing destination should exist");

            for (target, additions) in new_runs.iter_mut().zip(additions) {
                merge_disjoint(target, additions);
            }

            if let Some(run) = Run::new(trailing) {
                new_runs.push(run);
            }
        }

        new_runs
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::KeyRange;
    use test_log::test;

    fn optimize_runs_reference<T: Clone + Ranged>(runs: Vec<Run<T>>) -> Vec<Run<T>> {
        if runs.len() <= 1 {
            return runs;
        }

        let mut new_runs: Vec<Run<T>> = Vec::with_capacity(runs.len());

        for run in runs {
            for table in run.into_inner() {
                let last_overlap = new_runs
                    .iter()
                    .rposition(|run| !run.get_overlapping(table.key_range()).is_empty());

                let target = match last_overlap {
                    Some(index) => new_runs.get_mut(index + 1),
                    None => new_runs.first_mut(),
                };

                if let Some(target) = target {
                    target.push(table);
                } else {
                    new_runs.push(Run::new(vec![table]).unwrap());
                }
            }
        }

        new_runs
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct FakeTable {
        id: u64,
        key_range: KeyRange,
    }

    impl Ranged for FakeTable {
        fn key_range(&self) -> &KeyRange {
            &self.key_range
        }
    }

    fn s(id: u64, min: &str, max: &str) -> FakeTable {
        FakeTable {
            id,
            key_range: KeyRange::new((min.as_bytes().into(), max.as_bytes().into())),
        }
    }

    fn numbered(id: u64, min: u32, max: u32) -> FakeTable {
        FakeTable {
            id,
            key_range: KeyRange::new((min.to_be_bytes().into(), max.to_be_bytes().into())),
        }
    }

    fn interleaved_runs(
        run_count: usize,
        tables_per_run: usize,
        overlapping: bool,
    ) -> Vec<Run<FakeTable>> {
        (0..run_count)
            .map(|run_index| {
                let tables = (0..tables_per_run)
                    .map(|table_index| {
                        let key = if overlapping {
                            table_index as u32 * 2
                        } else {
                            (table_index * run_count + run_index) as u32 * 2
                        };
                        numbered(((run_index as u64) << 32) | table_index as u64, key, key)
                    })
                    .collect();

                Run::new(tables).unwrap()
            })
            .collect()
    }

    #[test]
    fn optimize_runs_empty() {
        let runs = vec![];
        let runs = optimize_runs::<FakeTable>(runs);

        assert_eq!(Vec::<Run<FakeTable>>::new(), &*runs);
    }

    #[test]
    fn optimize_runs_one() {
        let runs = vec![Run::new(vec![s(0, "a", "b")]).unwrap()];
        let runs = optimize_runs::<FakeTable>(runs);

        assert_eq!(vec![Run::new(vec![s(0, "a", "b")]).unwrap()], &*runs);
    }

    #[test]
    fn optimize_runs_two_overlap() {
        let runs = vec![
            Run::new(vec![s(0, "a", "b")]).unwrap(),
            Run::new(vec![s(1, "a", "b")]).unwrap(),
        ];
        let runs = optimize_runs::<FakeTable>(runs);

        assert_eq!(
            vec![
                Run::new(vec![s(0, "a", "b")]).unwrap(),
                Run::new(vec![s(1, "a", "b")]).unwrap(),
            ],
            &*runs
        );
    }

    #[test]
    fn optimize_runs_two_overlap_2() {
        let runs = vec![
            Run::new(vec![s(0, "a", "z")]).unwrap(),
            Run::new(vec![s(1, "c", "f")]).unwrap(),
        ];
        let runs = optimize_runs::<FakeTable>(runs);

        assert_eq!(
            vec![
                Run::new(vec![s(0, "a", "z")]).unwrap(),
                Run::new(vec![s(1, "c", "f")]).unwrap(),
            ],
            &*runs
        );
    }

    #[test]
    fn optimize_runs_two_overlap_3() {
        let runs = vec![
            Run::new(vec![s(0, "c", "f")]).unwrap(),
            Run::new(vec![s(1, "a", "z")]).unwrap(),
        ];
        let runs = optimize_runs::<FakeTable>(runs);

        assert_eq!(
            vec![
                Run::new(vec![s(0, "c", "f")]).unwrap(),
                Run::new(vec![s(1, "a", "z")]).unwrap()
            ],
            &*runs
        );
    }

    #[test]
    fn optimize_runs_two_disjoint() {
        let runs = vec![
            Run::new(vec![s(0, "a", "c")]).unwrap(),
            Run::new(vec![s(1, "d", "f")]).unwrap(),
        ];
        let runs = optimize_runs::<FakeTable>(runs);

        assert_eq!(
            vec![Run::new(vec![s(0, "a", "c"), s(1, "d", "f")]).unwrap()],
            &*runs,
        );
    }

    #[test]
    fn optimize_runs_two_disjoint_2() {
        let runs = vec![
            Run::new(vec![s(1, "d", "f")]).unwrap(),
            Run::new(vec![s(0, "a", "c")]).unwrap(),
        ];
        let runs = optimize_runs::<FakeTable>(runs);

        assert_eq!(
            vec![Run::new(vec![s(0, "a", "c"), s(1, "d", "f")]).unwrap()],
            &*runs,
        );
    }

    #[test]
    fn optimize_runs_overlap_transitive() {
        let runs = vec![
            Run::new(vec![s(2, "m", "p")]).unwrap(),
            Run::new(vec![s(1, "a", "z")]).unwrap(),
            Run::new(vec![s(0, "a", "c")]).unwrap(),
        ];
        let runs = optimize_runs::<FakeTable>(runs);

        assert_eq!(
            vec![
                Run::new(vec![s(2, "m", "p")]).unwrap(),
                Run::new(vec![s(1, "a", "z")]).unwrap(),
                Run::new(vec![s(0, "a", "c")]).unwrap(),
            ],
            &*runs
        );
    }

    #[test]
    fn optimize_runs_matches_reference_for_generated_layouts() {
        let mut state = 0x4d59_5df4_d0f3_3173_u64;

        for case in 0..256_u64 {
            let run_count = 2 + (case as usize % 7);
            let mut runs = Vec::with_capacity(run_count);

            for run_index in 0..run_count {
                let table_count = 8 + ((case as usize + run_index) % 25);
                let mut tables = Vec::with_capacity(table_count);
                let mut next_min = 0_u32;

                for table_index in 0..table_count {
                    state = state
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1);
                    let gap = ((state >> 32) as u32) % 5;
                    state = state
                        .wrapping_mul(6_364_136_223_846_793_005)
                        .wrapping_add(1);
                    let width = 1 + ((state >> 32) as u32) % 12;
                    let min = next_min + gap;
                    let max = min + width;
                    next_min = max + 1;

                    tables.push(numbered(
                        (case << 32) | ((run_index as u64) << 16) | table_index as u64,
                        min,
                        max,
                    ));
                }

                runs.push(Run::new(tables).unwrap());
            }

            assert_eq!(
                optimize_runs_reference(runs.clone()),
                optimize_runs(runs),
                "generated case {case}",
            );
        }
    }

    #[test]
    #[ignore = "manual optimizer microbenchmark"]
    fn benchmark_optimize_runs() {
        use std::{hint::black_box, time::Instant};

        fn median(samples: &mut [u128]) -> u128 {
            samples.sort_unstable();
            samples[samples.len() / 2]
        }

        fn measure(
            layout: &[Run<FakeTable>],
            optimize: impl Fn(Vec<Run<FakeTable>>) -> Vec<Run<FakeTable>>,
        ) -> u128 {
            let mut samples = Vec::with_capacity(31);

            for _ in 0..31 {
                let input = layout.to_vec();
                let start = Instant::now();
                let output = black_box(optimize(black_box(input)));
                samples.push(start.elapsed().as_nanos());
                black_box(output);
            }

            median(&mut samples)
        }

        for (name, layout) in [
            ("disjoint-interleaved", interleaved_runs(8, 1_024, false)),
            ("overlapping-generations", interleaved_runs(8, 1_024, true)),
        ] {
            let reference = measure(&layout, optimize_runs_reference);
            let batched = measure(&layout, optimize_runs);
            eprintln!(
                "{name}: reference={reference}ns batched={batched}ns speedup={:.2}x",
                reference as f64 / batched as f64,
            );
        }
    }
}
