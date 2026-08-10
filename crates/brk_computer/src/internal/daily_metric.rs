use std::{marker::PhantomData, sync::Arc};

use brk_error::Result;
use brk_traversable::{Index, SeriesLeaf, SeriesLeafWithSchema, Traversable, TreeNode};
use brk_types::{
    Date, Day1, Day3, Epoch, Halving, Height, Hour1, Hour4, Hour12, Minute10, Minute30, Month1,
    Month3, Month6, Timestamp, Version, Week1, Year1, Year10,
};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyStoredVec, AnyVec, ColumnId, ColumnarVec, Database, EagerVec, Formattable,
    ImportableVec, LazyColumnVec, LazyVec, PcoVec, PcoVecValue, ReadOnlyClone, ReadOnlyColumnarVec,
    ReadableBoxedVec, ReadableCloneableVec, ReadableColumnarVec, ReadableVec, Rw, StorageMode,
    TypedVec, UnaryTransform, VecIndex, VecValue, WritableVec, short_type_name,
};

use crate::indexes;

type StoredDay<T, M> = <M as StorageMode>::Stored<EagerVec<PcoVec<Day1, T>>>;
type DayMapping<I, T> = LazyVec<I, Day1, I, T>;
type Repeated<I, T> = DailyView<I, T, RepeatDay>;
type Last<I, T> = DailyView<I, T, LastDay>;

pub trait DailyValue: VecValue + Formattable + JsonSchema + Serialize {}

impl<T> DailyValue for T where T: VecValue + Formattable + JsonSchema + Serialize {}

pub struct RepeatDay;
pub struct LastDay;

#[derive(Clone)]
pub(crate) struct DailyMappings {
    height: DayMapping<Height, Day1>,
    minute10: DayMapping<Minute10, Timestamp>,
    minute30: DayMapping<Minute30, Timestamp>,
    hour1: DayMapping<Hour1, Timestamp>,
    hour4: DayMapping<Hour4, Timestamp>,
    hour12: DayMapping<Hour12, Timestamp>,
    day3: DayMapping<Day3, Date>,
    week1: DayMapping<Week1, Date>,
    month1: DayMapping<Month1, Date>,
    month3: DayMapping<Month3, Date>,
    month6: DayMapping<Month6, Date>,
    year1: DayMapping<Year1, Date>,
    year10: DayMapping<Year10, Date>,
    halving: DayMapping<Halving, Timestamp>,
    epoch: DayMapping<Epoch, Timestamp>,
}

impl DailyMappings {
    pub(crate) fn new(indexes: &indexes::Vecs) -> Self {
        let height = LazyVec::init(
            "day1",
            Version::ZERO,
            indexes.height.day1_read_only_boxed_clone(),
            |_, day| day,
        );

        Self {
            height,
            minute10: timestamp_mapping(indexes.timestamp.minute10.read_only_boxed_clone()),
            minute30: timestamp_mapping(indexes.timestamp.minute30.read_only_boxed_clone()),
            hour1: timestamp_mapping(indexes.timestamp.hour1.read_only_boxed_clone()),
            hour4: timestamp_mapping(indexes.timestamp.hour4.read_only_boxed_clone()),
            hour12: timestamp_mapping(indexes.timestamp.hour12.read_only_boxed_clone()),
            day3: date_mapping(indexes.day3.date.read_only_boxed_clone()),
            week1: date_mapping(indexes.week1.date.read_only_boxed_clone()),
            month1: date_mapping(indexes.month1.date.read_only_boxed_clone()),
            month3: date_mapping(indexes.month3.date.read_only_boxed_clone()),
            month6: date_mapping(indexes.month6.date.read_only_boxed_clone()),
            year1: date_mapping(indexes.year1.date.read_only_boxed_clone()),
            year10: date_mapping(indexes.year10.date.read_only_boxed_clone()),
            halving: timestamp_mapping(indexes.timestamp.halving.read_only_boxed_clone()),
            epoch: timestamp_mapping(indexes.timestamp.epoch.read_only_boxed_clone()),
        }
    }
}

fn timestamp_mapping<I: VecIndex>(
    source: ReadableBoxedVec<I, Timestamp>,
) -> DayMapping<I, Timestamp> {
    LazyVec::init("day1", Version::ZERO, source, |_, timestamp| {
        Day1::try_from(Date::from(timestamp)).unwrap_or_default()
    })
}

fn date_mapping<I: VecIndex>(source: ReadableBoxedVec<I, Date>) -> DayMapping<I, Date> {
    LazyVec::init("day1", Version::ZERO, source, |_, date| {
        Day1::try_from(date).unwrap_or_default()
    })
}

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct DailyViews<T>
where
    T: DailyValue,
{
    pub height: Repeated<Height, T>,
    pub minute10: Repeated<Minute10, T>,
    pub minute30: Repeated<Minute30, T>,
    pub hour1: Repeated<Hour1, T>,
    pub hour4: Repeated<Hour4, T>,
    pub hour12: Repeated<Hour12, T>,
    pub day3: Last<Day3, T>,
    pub week1: Last<Week1, T>,
    pub month1: Last<Month1, T>,
    pub month3: Last<Month3, T>,
    pub month6: Last<Month6, T>,
    pub year1: Last<Year1, T>,
    pub year10: Last<Year10, T>,
    pub halving: Last<Halving, T>,
    pub epoch: Last<Epoch, T>,
}

impl<T> DailyViews<T>
where
    T: DailyValue,
{
    pub(crate) fn new(
        name: &str,
        source: ReadableBoxedVec<Day1, T>,
        version: Version,
        mappings: &DailyMappings,
    ) -> Self {
        Self {
            height: repeated(name, source.clone(), version, &mappings.height),
            minute10: repeated(name, source.clone(), version, &mappings.minute10),
            minute30: repeated(name, source.clone(), version, &mappings.minute30),
            hour1: repeated(name, source.clone(), version, &mappings.hour1),
            hour4: repeated(name, source.clone(), version, &mappings.hour4),
            hour12: repeated(name, source.clone(), version, &mappings.hour12),
            day3: last(name, source.clone(), version, &mappings.day3),
            week1: last(name, source.clone(), version, &mappings.week1),
            month1: last(name, source.clone(), version, &mappings.month1),
            month3: last(name, source.clone(), version, &mappings.month3),
            month6: last(name, source.clone(), version, &mappings.month6),
            year1: last(name, source.clone(), version, &mappings.year1),
            year10: last(name, source.clone(), version, &mappings.year10),
            halving: last(name, source.clone(), version, &mappings.halving),
            epoch: last(name, source, version, &mappings.epoch),
        }
    }
}

#[derive(Traversable)]
#[traversable(merge)]
pub struct DailyMetric<T, M: StorageMode = Rw>
where
    T: DailyValue + PcoVecValue,
{
    pub day1: StoredDay<T, M>,
    #[traversable(flatten)]
    pub views: Box<DailyViews<T>>,
}

impl<T> DailyMetric<T>
where
    T: DailyValue + PcoVecValue,
{
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &DailyMappings,
    ) -> Result<Self> {
        let day1 = EagerVec::forced_import(db, name, version)?;
        let source = day1.read_only_boxed_clone();
        let views = Box::new(DailyViews::new(name, source, version, mappings));

        Ok(Self { day1, views })
    }
}

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyColumnDailyMetric<T, C>
where
    T: DailyValue + PcoVecValue,
    C: ColumnId,
{
    pub day1: LazyColumnVec<ReadOnlyColumnarVec<PcoVec<Day1, T>, C>, C>,
    #[traversable(flatten)]
    pub views: Box<DailyViews<T>>,
}

impl<T, C> LazyColumnDailyMetric<T, C>
where
    T: DailyValue + PcoVecValue,
    C: ColumnId,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Day1, T>, C>,
        column: C,
        mappings: &DailyMappings,
    ) -> Self {
        let day1 = source.column(name, version, column);
        let views = Box::new(DailyViews::new(
            name,
            day1.read_only_boxed_clone(),
            version,
            mappings,
        ));

        Self { day1, views }
    }
}

#[derive(Deref, DerefMut, Traversable)]
pub struct ColumnarDailyMetric<T, C, S: Clone, M: StorageMode = Rw>
where
    T: DailyValue + PcoVecValue,
    C: ColumnId,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub series: S,
    #[traversable(hidden)]
    pub day1: M::Stored<EagerVec<ColumnarVec<PcoVec<Day1, T>, C>>>,
}

impl<T, C, S: Clone> ColumnarDailyMetric<T, C, S>
where
    T: DailyValue + PcoVecValue,
    C: ColumnId,
{
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        build_series: impl FnOnce(&ReadOnlyColumnarVec<PcoVec<Day1, T>, C>) -> S,
    ) -> Result<Self> {
        let day1 = EagerVec::forced_import(db, name, version)?;
        let series = build_series(&day1.read_only_clone());

        Ok(Self { series, day1 })
    }

    #[inline(always)]
    pub(crate) fn push(&mut self, row: C::Row<T>) {
        self.day1.push(row);
    }

    pub(crate) fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.day1
    }
}

type LazyDay<T, S> = LazyVec<Day1, T, Day1, S>;

#[derive(Clone, Traversable)]
#[traversable(merge)]
pub struct LazyDailyMetric<T, S>
where
    T: DailyValue,
    S: VecValue,
{
    pub day1: LazyDay<T, S>,
    #[traversable(flatten)]
    pub views: Box<DailyViews<T>>,
}

impl<T, S> LazyDailyMetric<T, S>
where
    T: DailyValue,
    S: VecValue,
{
    pub(crate) fn from_source<F>(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Day1, S>,
        mappings: &DailyMappings,
    ) -> Self
    where
        F: UnaryTransform<S, T>,
    {
        let day1 = LazyVec::transformed::<F>(name, version, source);
        let views = Box::new(DailyViews::new(
            name,
            day1.read_only_boxed_clone(),
            version,
            mappings,
        ));

        Self { day1, views }
    }
}

fn repeated<I, T, V>(
    name: &str,
    source: ReadableBoxedVec<Day1, T>,
    version: Version,
    mapping: &V,
) -> Repeated<I, T>
where
    I: VecIndex,
    T: VecValue,
    V: ReadableCloneableVec<I, Day1> + ?Sized,
{
    DailyView::new(name, version, source, mapping.read_only_boxed_clone())
}

fn last<I, T, V>(
    name: &str,
    source: ReadableBoxedVec<Day1, T>,
    version: Version,
    mapping: &V,
) -> Last<I, T>
where
    I: VecIndex,
    T: VecValue,
    V: ReadableCloneableVec<I, Day1> + ?Sized,
{
    DailyView::new(name, version, source, mapping.read_only_boxed_clone())
}

pub trait DayStrategy: Send + Sync + 'static {
    fn mapping_end(to: usize, mapping_len: usize) -> usize;
    fn source_index(mapping: &[Day1], index: usize, source_len: usize) -> Option<usize>;
}

impl DayStrategy for RepeatDay {
    fn mapping_end(to: usize, _mapping_len: usize) -> usize {
        to
    }

    fn source_index(mapping: &[Day1], index: usize, source_len: usize) -> Option<usize> {
        repeated_source_index(mapping, index, source_len)
    }
}

impl DayStrategy for LastDay {
    fn mapping_end(to: usize, mapping_len: usize) -> usize {
        to.saturating_add(1).min(mapping_len)
    }

    fn source_index(mapping: &[Day1], index: usize, source_len: usize) -> Option<usize> {
        last_source_index(mapping, index, source_len)
    }
}

pub struct DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
{
    name: Arc<str>,
    version: Version,
    source: ReadableBoxedVec<Day1, T>,
    mapping: ReadableBoxedVec<I, Day1>,
    _phantom: PhantomData<fn() -> S>,
}

impl<I, T, S> Clone for DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
{
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            version: self.version,
            source: self.source.clone(),
            mapping: self.mapping.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<I, T, S> DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: DayStrategy,
{
    fn new(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Day1, T>,
        mapping: ReadableBoxedVec<I, Day1>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            version,
            source,
            mapping,
            _phantom: PhantomData,
        }
    }

    fn try_fold_values<B, E, F>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E>
    where
        F: FnMut(B, Option<T>) -> std::result::Result<B, E>,
    {
        let mapping_len = self.mapping.len();
        let to = to.min(mapping_len);
        if from >= to {
            return Ok(init);
        }

        let mapping = self
            .mapping
            .collect_range_dyn(from, S::mapping_end(to, mapping_len));
        let source_len = self.source.len();
        try_fold_mapped(
            &*self.source,
            0,
            to - from,
            |index| S::source_index(&mapping, index, source_len),
            init,
            f,
        )
    }

    fn fold_values<B, F>(&self, from: usize, to: usize, init: B, mut f: F) -> B
    where
        F: FnMut(B, Option<T>) -> B,
    {
        match self.try_fold_values(from, to, init, |acc, value| {
            Ok::<_, std::convert::Infallible>(f(acc, value))
        }) {
            Ok(result) => result,
            Err(error) => match error {},
        }
    }
}

impl<I, T, S> AnyVec for DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: DayStrategy,
{
    fn version(&self) -> Version {
        self.version + self.source.version() + self.mapping.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.mapping.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        vec![]
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<Option<T>>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<Option<T>>()
    }
}

impl<I, T, S> TypedVec for DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: DayStrategy,
{
    type I = I;
    type T = Option<T>;
}

impl<I, T, S> ReadableVec<I, Option<T>> for DailyView<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: DayStrategy,
{
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Option<T>>) {
        self.fold_values(from, to, (), |(), value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(Option<T>)) {
        self.fold_values(from, to, (), |(), value| f(value));
    }

    fn fold_range_at<B, F: FnMut(B, Option<T>) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        self.fold_values(from, to, init, f)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Option<T>) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E> {
        self.try_fold_values(from, to, init, f)
    }

    fn collect_one_at(&self, index: usize) -> Option<Option<T>> {
        let mapping_len = self.mapping.len();
        if index >= mapping_len {
            return None;
        }

        let mapping = self
            .mapping
            .collect_range_dyn(index, S::mapping_end(index.saturating_add(1), mapping_len));
        Some(
            S::source_index(&mapping, 0, self.source.len())
                .and_then(|day| self.source.collect_one_at(day)),
        )
    }
}

impl<I, T, S> Traversable for DailyView<I, T, S>
where
    I: VecIndex,
    T: DailyValue,
    S: DayStrategy,
{
    fn to_tree_node(&self) -> TreeNode {
        let indexes = Index::try_from(I::to_string()).ok().into_iter().collect();
        let leaf = SeriesLeaf::new(
            self.name().to_string(),
            self.value_type_to_string().to_string(),
            indexes,
        );
        let schema = schemars::SchemaGenerator::default().into_root_schema_for::<Option<T>>();
        let schema_json = serde_json::to_value(schema).unwrap_or_default();

        TreeNode::Leaf(SeriesLeafWithSchema::new(leaf, schema_json))
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }
}

fn try_fold_mapped<T, S, B, E, F, G>(
    source: &S,
    from: usize,
    to: usize,
    mut source_index: G,
    init: B,
    mut f: F,
) -> std::result::Result<B, E>
where
    T: VecValue,
    S: ReadableVec<Day1, T> + ?Sized,
    F: FnMut(B, Option<T>) -> std::result::Result<B, E>,
    G: FnMut(usize) -> Option<usize>,
{
    let mut indices = Vec::with_capacity(to - from);
    let mut slots: Vec<Option<u32>> = Vec::with_capacity(to - from);

    for output_index in from..to {
        let Some(source_index) = source_index(output_index) else {
            slots.push(None);
            continue;
        };

        let slot = match indices.last() {
            Some(&last) if last == source_index => indices.len() - 1,
            Some(&last) => {
                debug_assert!(last < source_index);
                indices.push(source_index);
                indices.len() - 1
            }
            None => {
                indices.push(source_index);
                0
            }
        };
        debug_assert!(u32::try_from(slot).is_ok());
        slots.push(Some(slot as u32));
    }

    let values = source.read_sorted_at(&indices);
    slots.into_iter().try_fold(init, |acc, slot| match slot {
        Some(slot) => f(acc, Some(values[slot as usize].clone())),
        None => f(acc, None),
    })
}

fn repeated_source_index(mapping: &[Day1], index: usize, source_len: usize) -> Option<usize> {
    let day = mapping[index].to_usize();
    (day < source_len).then_some(day)
}

fn last_source_index(mapping: &[Day1], index: usize, source_len: usize) -> Option<usize> {
    let first = mapping[index].to_usize();
    let next_first = mapping
        .get(index + 1)
        .map(|day| day.to_usize())
        .unwrap_or(source_len)
        .min(source_len);
    (first < next_first).then_some(next_first - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use brk_types::{StoredBool, StoredF64};
    use vecdb::{AnyStoredVec, WritableVec};

    #[test]
    fn repeat_uses_the_same_daily_value_throughout_the_day() {
        let mapping = [Day1::from(0), Day1::from(0), Day1::from(1)];

        assert_eq!(repeated_source_index(&mapping, 0, 2), Some(0));
        assert_eq!(repeated_source_index(&mapping, 1, 2), Some(0));
        assert_eq!(repeated_source_index(&mapping, 2, 2), Some(1));
        assert_eq!(repeated_source_index(&mapping, 2, 1), None);
    }

    #[test]
    fn coarser_period_uses_its_last_available_day() {
        let mapping = [Day1::from(0), Day1::from(3), Day1::from(6)];

        assert_eq!(last_source_index(&mapping, 0, 8), Some(2));
        assert_eq!(last_source_index(&mapping, 1, 8), Some(5));
        assert_eq!(last_source_index(&mapping, 2, 8), Some(7));
        assert_eq!(last_source_index(&mapping, 1, 5), Some(4));
        assert_eq!(last_source_index(&mapping, 2, 5), None);
    }

    #[test]
    fn repeated_view_maps_ranges_and_preserves_missing_days() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("brk-daily-view-{}-{suffix}", std::process::id()));
        let db = Database::open(&path).unwrap();

        let mut source: EagerVec<PcoVec<Day1, StoredF64>> =
            EagerVec::forced_import(&db, "source", Version::ONE).unwrap();
        let mut mapping: EagerVec<PcoVec<Height, Day1>> =
            EagerVec::forced_import(&db, "mapping", Version::ONE).unwrap();
        for value in [10.0, 20.0, 30.0] {
            source.push(StoredF64::from(value));
        }
        for day in [0, 0, 1, 2, 3] {
            mapping.push(Day1::from(day));
        }
        source.write().unwrap();
        mapping.write().unwrap();

        let view = DailyView::<Height, StoredF64, RepeatDay>::new(
            "test",
            Version::ONE,
            source.read_only_boxed_clone(),
            mapping.read_only_boxed_clone(),
        );

        assert_eq!(
            view.collect_range_at(0, 5),
            vec![
                Some(StoredF64::from(10.0)),
                Some(StoredF64::from(10.0)),
                Some(StoredF64::from(20.0)),
                Some(StoredF64::from(30.0)),
                None,
            ]
        );

        drop(view);
        drop(mapping);
        drop(source);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn repeated_view_supports_stored_booleans() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("brk-daily-bool-{}-{suffix}", std::process::id()));
        let db = Database::open(&path).unwrap();

        let mut source: EagerVec<PcoVec<Day1, StoredBool>> =
            EagerVec::forced_import(&db, "source", Version::ONE).unwrap();
        let mut mapping: EagerVec<PcoVec<Height, Day1>> =
            EagerVec::forced_import(&db, "mapping", Version::ONE).unwrap();
        source.push(StoredBool::FALSE);
        source.push(StoredBool::TRUE);
        for day in [0, 0, 1, 1] {
            mapping.push(Day1::from(day));
        }
        source.write().unwrap();
        mapping.write().unwrap();

        let view = DailyView::<Height, StoredBool, RepeatDay>::new(
            "test",
            Version::ONE,
            source.read_only_boxed_clone(),
            mapping.read_only_boxed_clone(),
        );

        assert_eq!(
            view.collect_range_at(0, 4),
            vec![
                Some(StoredBool::FALSE),
                Some(StoredBool::FALSE),
                Some(StoredBool::TRUE),
                Some(StoredBool::TRUE),
            ]
        );

        drop(view);
        drop(mapping);
        drop(source);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
