use brk_types::{Bitcoin, Day1, Dollars, Height, Sats, Version};
use vecdb::{
    AnyVec, CachedBoxedVec, CachedVec, PrintableIndex, ReadOnlyClone, ReadableBoxedVec,
    ReadableVec, TypedVec, VecIndex, short_type_name,
};

use super::DCA_AMOUNT;

/// Small pinned cumulative DCA cache indexed by day.
#[derive(Clone)]
pub struct CachedDcaSats {
    daily: CachedVec<DcaSatsByDay>,
    days: CachedBoxedVec<Height, Day1>,
}

impl CachedDcaSats {
    pub fn new(
        daily_close: ReadableBoxedVec<Day1, Option<Dollars>>,
        days: CachedBoxedVec<Height, Day1>,
    ) -> Self {
        Self {
            daily: CachedVec::wrap(DcaSatsByDay { daily_close }),
            days,
        }
    }

    /// Daily closes can be rewritten without changing their length.
    pub fn invalidate(&self) {
        self.daily.invalidate();
    }

    fn try_for_each_value<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(Sats) -> Result<(), E>,
    ) -> Result<(), E> {
        let days = self.days.snapshot();
        let daily = self.daily.snapshot();
        let to = to.min(days.len());
        if from >= to {
            return Ok(());
        }

        let last = daily.last().copied().unwrap_or_default();
        for day in &days[from..to] {
            each(daily.get(day.to_usize()).copied().unwrap_or(last))?;
        }
        Ok(())
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(Sats)) {
        let result = self.try_for_each_value(from, to, |value| {
            each(value);
            Ok::<_, std::convert::Infallible>(())
        });
        match result {
            Ok(()) => {}
            Err(error) => match error {},
        }
    }
}

impl AnyVec for CachedDcaSats {
    fn version(&self) -> Version {
        self.days.version() + self.daily.version()
    }

    fn name(&self) -> &str {
        "dca_sats_cumulative"
    }

    fn len(&self) -> usize {
        self.days.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        <Height as PrintableIndex>::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<Sats>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<Sats>()
    }
}

impl TypedVec for CachedDcaSats {
    type I = Height;
    type T = Sats;
}

impl ReadableVec<Height, Sats> for CachedDcaSats {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Sats>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Sats)) {
        self.for_each_value(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, Sats) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        let mut acc = Some(init);
        self.for_each_value(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value));
        });
        acc.unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Sats) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> Result<B, E> {
        let mut acc = Some(init);
        self.try_for_each_value(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value)?);
            Ok(())
        })?;
        Ok(acc.unwrap())
    }

    fn collect_one_at(&self, index: usize) -> Option<Sats> {
        let days = self.days.snapshot();
        let daily = self.daily.snapshot();
        let day = days.get(index)?;
        let last = daily.last().copied().unwrap_or_default();
        Some(daily.get(day.to_usize()).copied().unwrap_or(last))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Sats>) {
        let days = self.days.snapshot();
        let daily = self.daily.snapshot();
        let last = daily.last().copied().unwrap_or_default();

        out.reserve(indices.len());
        indices
            .iter()
            .take_while(|&&index| index < days.len())
            .for_each(|&index| {
                out.push(daily.get(days[index].to_usize()).copied().unwrap_or(last));
            });
    }
}

impl ReadOnlyClone for CachedDcaSats {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

#[derive(Clone)]
struct DcaSatsByDay {
    daily_close: ReadableBoxedVec<Day1, Option<Dollars>>,
}

impl DcaSatsByDay {
    fn try_for_each_cumulative<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(Sats) -> Result<(), E>,
    ) -> Result<(), E> {
        let to = to.min(self.len());
        if from >= to {
            return Ok(());
        }

        let mut cumulative = Sats::ZERO;
        for (day, price) in self
            .daily_close
            .collect_range_dyn(0, to)
            .into_iter()
            .enumerate()
        {
            cumulative += sats_from_dca(price.unwrap_or_default());
            if day >= from {
                each(cumulative)?;
            }
        }
        Ok(())
    }

    fn for_each_cumulative(&self, from: usize, to: usize, mut each: impl FnMut(Sats)) {
        let result = self.try_for_each_cumulative(from, to, |value| {
            each(value);
            Ok::<_, std::convert::Infallible>(())
        });
        match result {
            Ok(()) => {}
            Err(error) => match error {},
        }
    }
}

impl AnyVec for DcaSatsByDay {
    fn version(&self) -> Version {
        self.daily_close.version()
    }

    fn name(&self) -> &str {
        "dca_sats_cumulative"
    }

    fn len(&self) -> usize {
        self.daily_close.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        <Day1 as PrintableIndex>::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<Sats>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<Sats>()
    }
}

impl TypedVec for DcaSatsByDay {
    type I = Day1;
    type T = Sats;
}

impl ReadableVec<Day1, Sats> for DcaSatsByDay {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Sats>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_cumulative(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Sats)) {
        self.for_each_cumulative(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, Sats) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        let mut acc = Some(init);
        self.for_each_cumulative(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value));
        });
        acc.unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Sats) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> Result<B, E> {
        let mut acc = Some(init);
        self.try_for_each_cumulative(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value)?);
            Ok(())
        })?;
        Ok(acc.unwrap())
    }
}

fn sats_from_dca(price: Dollars) -> Sats {
    if price == Dollars::ZERO {
        Sats::ZERO
    } else {
        Sats::from(Bitcoin::from(DCA_AMOUNT / price))
    }
}

#[cfg(test)]
mod tests {
    use std::{marker::PhantomData, sync::Arc};

    use parking_lot::RwLock;
    use vecdb::{CachedReadableVec, VecValue, short_type_name};

    use super::*;

    #[derive(Clone)]
    struct MemoryVec<I, T> {
        values: Arc<RwLock<Vec<T>>>,
        index: PhantomData<fn() -> I>,
    }

    impl<I, T> MemoryVec<I, T> {
        fn new(values: impl IntoIterator<Item = T>) -> Self {
            Self {
                values: Arc::new(RwLock::new(values.into_iter().collect())),
                index: PhantomData,
            }
        }

        fn replace(&self, index: usize, value: T) {
            self.values.write()[index] = value;
        }
    }

    impl<I: VecIndex, T: VecValue> AnyVec for MemoryVec<I, T> {
        fn version(&self) -> Version {
            Version::ONE
        }

        fn name(&self) -> &str {
            "memory"
        }

        fn len(&self) -> usize {
            self.values.read().len()
        }

        fn index_type_to_string(&self) -> &'static str {
            I::to_string()
        }

        fn region_names(&self) -> Vec<String> {
            Vec::new()
        }

        fn value_type_to_size_of(&self) -> usize {
            size_of::<T>()
        }

        fn value_type_to_string(&self) -> &'static str {
            short_type_name::<T>()
        }
    }

    impl<I: VecIndex, T: VecValue> TypedVec for MemoryVec<I, T> {
        type I = I;
        type T = T;
    }

    impl<I: VecIndex, T: VecValue> ReadableVec<I, T> for MemoryVec<I, T> {
        fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<T>) {
            let values = self.values.read();
            let to = to.min(values.len());
            if from < to {
                buf.extend_from_slice(&values[from..to]);
            }
        }

        fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(T)) {
            let values = self.values.read();
            let to = to.min(values.len());
            for value in &values[from.min(to)..to] {
                each(value.clone());
            }
        }

        fn fold_range_at<B, F: FnMut(B, T) -> B>(
            &self,
            from: usize,
            to: usize,
            init: B,
            mut fold: F,
        ) -> B {
            let values = self.values.read();
            let to = to.min(values.len());
            values[from.min(to)..to]
                .iter()
                .cloned()
                .fold(init, &mut fold)
        }

        fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
            &self,
            from: usize,
            to: usize,
            init: B,
            mut fold: F,
        ) -> Result<B, E> {
            let values = self.values.read();
            let to = to.min(values.len());
            values[from.min(to)..to]
                .iter()
                .cloned()
                .try_fold(init, &mut fold)
        }
    }

    #[test]
    fn daily_cache_is_cumulative_and_refreshes_same_length_rewrites() {
        let prices = MemoryVec::<Day1, Option<Dollars>>::new([
            Some(Dollars::mint(100.0)),
            None,
            Some(Dollars::mint(200.0)),
        ]);
        let days = CachedVec::wrap(MemoryVec::<Height, Day1>::new([
            Day1::from(0),
            Day1::from(1),
            Day1::from(2),
        ]));
        let cached = CachedDcaSats::new(
            ReadableBoxedVec::new(prices.clone()),
            days.cached_boxed_clone(),
        );

        let first = sats_from_dca(Dollars::mint(100.0));
        let third = first + sats_from_dca(Dollars::mint(200.0));
        assert_eq!(cached.daily.snapshot().as_slice(), &[first, first, third]);

        prices.replace(0, Some(Dollars::mint(50.0)));
        assert_eq!(cached.daily.snapshot().as_slice(), &[first, first, third]);

        cached.invalidate();
        let replaced = sats_from_dca(Dollars::mint(50.0));
        let third = replaced + sats_from_dca(Dollars::mint(200.0));
        assert_eq!(
            cached.daily.snapshot().as_slice(),
            &[replaced, replaced, third]
        );
    }

    #[test]
    fn height_source_maps_days_and_carries_the_latest_available_total() {
        let prices = MemoryVec::<Day1, Option<Dollars>>::new([
            Some(Dollars::mint(100.0)),
            None,
            Some(Dollars::mint(200.0)),
        ]);
        let days = CachedVec::wrap(MemoryVec::<Height, Day1>::new([
            Day1::from(0),
            Day1::from(0),
            Day1::from(1),
            Day1::from(2),
            Day1::from(3),
        ]));
        let cached = CachedDcaSats::new(ReadableBoxedVec::new(prices), days.cached_boxed_clone());

        let first = sats_from_dca(Dollars::mint(100.0));
        let third = first + sats_from_dca(Dollars::mint(200.0));
        assert_eq!(cached.collect(), [first, first, first, third, third]);
        assert_eq!(cached.collect_one_at(3), Some(third));
        assert_eq!(cached.collect_one_at(5), None);
        assert_eq!(
            cached.read_sorted_at(&[0, 2, 2, 4, 5]),
            [first, first, first, third],
        );
    }
}
