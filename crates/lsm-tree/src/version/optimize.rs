// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::mem;

use super::run::Ranged;
use crate::version::Run;

fn merge_disjoint<T: Ranged>(run: &mut Run<T>, additions: Vec<T>) {
    if additions.is_empty() {
        return;
    }

    let existing = mem::take(run.inner_mut());
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
