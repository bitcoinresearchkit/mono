use crate::VecValue;

use super::{Cache, Request, Table, Value};

const MERGE_BYTES: usize = 64 * 1024;

impl<T: VecValue> Cache<T> {
    // Fills contain only missing ranges, so existing entries never overlap.
    pub(super) fn insert(&self, offers: Vec<(usize, Value<T>)>) {
        let mut table = self.table.write();
        let mut admitted = Vec::with_capacity(offers.len());
        let limit = MERGE_BYTES / size_of::<T>().max(1);
        for (start, mut value) in offers {
            if value.allocation_bytes() + Table::<T>::charge_for(1) > self.account.budget.limit() {
                continue;
            }
            let last = admitted.last_mut().or_else(|| table.entries.last_mut());
            if let Some((at, previous)) = last {
                debug_assert!(*at != start);
                if *at + previous.len() == start
                    && previous.try_append(value.slice(), limit, &self.account)
                {
                    continue;
                }
            }
            let end = start + value.len();
            let next = table.entries.partition_point(|(at, _)| *at < end);
            if let Some((at, next)) = table.entries.get_mut(next)
                && *at == end
                && next.try_prepend(value.slice(), limit, &self.account)
            {
                *at = start;
                continue;
            }
            if value.admit(&self.account) {
                admitted.push((start, value));
            }
        }
        table.insert(admitted, &self.account);
    }

    pub(super) fn invalidate_from(&self, from: usize) {
        let mut table = self.table.write();
        let keep = table.entries.partition_point(|(at, _)| *at < from);
        table.entries.truncate(keep);
        if let Some((at, value)) = table.entries.last_mut()
            && *at + value.len() > from
        {
            value.truncate(from - *at);
        }
        if table.is_empty() {
            *table = Table::new();
        }
    }

    /// Called inside publication, after persistence succeeds. Never warms a cold source.
    pub(crate) fn extend_tail(&self, from: usize, values: &[T]) {
        if values.is_empty() {
            return;
        }
        let mut table = self.table.write();
        if let Some((at, tail)) = table.entries.last_mut()
            && *at + tail.len() == from
        {
            tail.try_append(values, usize::MAX, &self.account);
        }
    }

    pub(super) fn borrowed(&self, request: Request<'_>) -> Vec<(usize, Value<T>)> {
        let table = self.table.read();
        match request {
            Request::Range(from, to) => table[table.first(from)..]
                .iter()
                .take_while(|(at, _)| *at < to)
                .map(|(at, value)| (*at, value.clone()))
                .collect(),
            Request::Sorted(indices) => {
                let mut values = Vec::new();
                for &index in indices {
                    if values
                        .last()
                        .is_some_and(|(at, value): &(usize, Value<T>)| index < *at + value.len())
                    {
                        continue;
                    }
                    if let Some((at, value)) = table.at(index) {
                        values.push((at, value.clone()));
                    }
                }
                values
            }
        }
    }

    /// An all-or-nothing probe: a miss does not clone values or change output.
    pub(super) fn copy_range(&self, from: usize, to: usize, out: &mut Vec<T>) -> bool {
        if from >= to {
            return true;
        }
        let Some(table) = self.table.try_read() else {
            return false;
        };
        let ranges = &table[table.first(from)..];
        let mut at = from;
        for (start, values) in ranges {
            if *start > at {
                return false;
            }
            at = at.max(start + values.len());
            if at >= to {
                break;
            }
        }
        if at < to {
            return false;
        }
        out.reserve(to - from);
        for (start, values) in ranges.iter().take_while(|(start, _)| *start < to) {
            let end = (to - start).min(values.len());
            out.extend_from_slice(&values.slice()[from.saturating_sub(*start)..end]);
        }
        true
    }

    /// Copies the retained prefix once and returns only the unfulfilled request.
    pub(super) fn copy_prefix<'a>(
        &self,
        request: Request<'a>,
        out: &mut Vec<T>,
    ) -> Option<Request<'a>> {
        let table = self.table.read();
        match request {
            Request::Range(from, to) => {
                let mut at = from;
                for (start, values) in &table[table.first(from)..] {
                    if *start > at || at >= to {
                        break;
                    }
                    let end = (start + values.len()).min(to);
                    out.extend_from_slice(&values.slice()[at - start..end - start]);
                    at = end;
                }
                (at < to).then_some(Request::Range(at, to))
            }
            Request::Sorted(indices) => {
                let &first = indices.first()?;
                if let Some((at, value)) = table.at(first)
                    && indices[indices.len() - 1] - at < value.len()
                {
                    let values = value.slice();
                    out.extend(indices.iter().map(|&index| values[index - at].clone()));
                    return None;
                }
                let mut at = table.first(first);
                for (offset, &index) in indices.iter().enumerate() {
                    if table
                        .get(at)
                        .is_some_and(|(start, value)| *start + value.len() <= index)
                    {
                        at += 1;
                        if table
                            .get(at)
                            .is_some_and(|(start, value)| *start + value.len() <= index)
                        {
                            at = table.first(index);
                        }
                    }
                    let Some(value) = table.get(at).and_then(|(start, value)| {
                        index.checked_sub(*start).and_then(|index| value.get(index))
                    }) else {
                        return Some(Request::Sorted(&indices[offset..]));
                    };
                    out.push(value);
                }
                None
            }
        }
    }
}
