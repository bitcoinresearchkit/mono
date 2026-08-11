use std::{fmt, ops::AddAssign};

use brk_traversable::Traversable;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{ColumnId, Formattable, VecValue, Version};

use crate::{
    LOSS_COUNT, Loss, LossId, PROFIT_COUNT, PROFITABILITY_RANGE_COUNT, Profit, ProfitId,
    ProfitabilityRange, ProfitabilityRangeId,
};

pub const PROFITABILITY_COUNT: usize = PROFITABILITY_RANGE_COUNT + PROFIT_COUNT + LOSS_COUNT;

#[derive(Debug, Clone, Traversable, Serialize, JsonSchema)]
pub struct ProfitabilityRow<T> {
    pub range: ProfitabilityRange<T>,
    pub profit: Profit<T>,
    pub loss: Loss<T>,
}

impl<T> ProfitabilityRow<T>
where
    T: AddAssign + Copy + Default,
{
    pub fn from_ranges(range: ProfitabilityRange<T>) -> Self {
        let mut profit = Profit::default();
        let mut profit_ranges = range.iter().take(PROFIT_COUNT + 1);
        let mut total = *profit_ranges.next().expect("profitability profit range");
        for (threshold, &value) in profit.iter_mut().rev().zip(profit_ranges) {
            total += value;
            *threshold = total;
        }

        let mut loss = Loss::default();
        let mut loss_ranges = range.iter().skip(PROFIT_COUNT + 1).rev();
        let mut total = *loss_ranges.next().expect("profitability loss range");
        for (threshold, &value) in loss.iter_mut().rev().zip(loss_ranges) {
            total += value;
            *threshold = total;
        }

        Self {
            range,
            profit,
            loss,
        }
    }
}

impl<T: Formattable> Formattable for ProfitabilityRow<T> {
    fn write_to(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(b"{\"range\":");
        write_array(self.range.iter(), buf);
        buf.extend_from_slice(b",\"profit\":");
        write_array(self.profit.iter(), buf);
        buf.extend_from_slice(b",\"loss\":");
        write_array(self.loss.iter(), buf);
        buf.push(b'}');
    }

    fn fmt_csv(&self, output: &mut String) -> fmt::Result {
        let mut json = Vec::new();
        self.write_to(&mut json);
        let json = std::str::from_utf8(&json).map_err(|_| fmt::Error)?;

        output.push('"');
        for character in json.chars() {
            if character == '"' {
                output.push('"');
            }
            output.push(character);
        }
        output.push('"');
        Ok(())
    }
}

fn write_array<'a, T: Formattable + 'a>(values: impl Iterator<Item = &'a T>, buf: &mut Vec<u8>) {
    buf.push(b'[');
    for (index, value) in values.enumerate() {
        if index > 0 {
            buf.push(b',');
        }
        value.write_to(buf);
    }
    buf.push(b']');
}

/// Every profitability range and aggregate threshold in column storage order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ProfitabilityId {
    RangeOver1000PctInProfit,
    Range500To1000PctInProfit,
    Range300To500PctInProfit,
    Range200To300PctInProfit,
    Range100To200PctInProfit,
    Range90To100PctInProfit,
    Range80To90PctInProfit,
    Range70To80PctInProfit,
    Range60To70PctInProfit,
    Range50To60PctInProfit,
    Range40To50PctInProfit,
    Range30To40PctInProfit,
    Range20To30PctInProfit,
    Range10To20PctInProfit,
    Range0To10PctInProfit,
    Range0To10PctInLoss,
    Range10To20PctInLoss,
    Range20To30PctInLoss,
    Range30To40PctInLoss,
    Range40To50PctInLoss,
    Range50To60PctInLoss,
    Range60To70PctInLoss,
    Range70To80PctInLoss,
    Range80To90PctInLoss,
    Range90To100PctInLoss,
    Profit,
    ProfitOver10Pct,
    ProfitOver20Pct,
    ProfitOver30Pct,
    ProfitOver40Pct,
    ProfitOver50Pct,
    ProfitOver60Pct,
    ProfitOver70Pct,
    ProfitOver80Pct,
    ProfitOver90Pct,
    ProfitOver100Pct,
    ProfitOver200Pct,
    ProfitOver300Pct,
    ProfitOver500Pct,
    Loss,
    LossOver10Pct,
    LossOver20Pct,
    LossOver30Pct,
    LossOver40Pct,
    LossOver50Pct,
    LossOver60Pct,
    LossOver70Pct,
    LossOver80Pct,
}

pub const PROFITABILITY_IDS: [ProfitabilityId; PROFITABILITY_COUNT] = [
    ProfitabilityId::RangeOver1000PctInProfit,
    ProfitabilityId::Range500To1000PctInProfit,
    ProfitabilityId::Range300To500PctInProfit,
    ProfitabilityId::Range200To300PctInProfit,
    ProfitabilityId::Range100To200PctInProfit,
    ProfitabilityId::Range90To100PctInProfit,
    ProfitabilityId::Range80To90PctInProfit,
    ProfitabilityId::Range70To80PctInProfit,
    ProfitabilityId::Range60To70PctInProfit,
    ProfitabilityId::Range50To60PctInProfit,
    ProfitabilityId::Range40To50PctInProfit,
    ProfitabilityId::Range30To40PctInProfit,
    ProfitabilityId::Range20To30PctInProfit,
    ProfitabilityId::Range10To20PctInProfit,
    ProfitabilityId::Range0To10PctInProfit,
    ProfitabilityId::Range0To10PctInLoss,
    ProfitabilityId::Range10To20PctInLoss,
    ProfitabilityId::Range20To30PctInLoss,
    ProfitabilityId::Range30To40PctInLoss,
    ProfitabilityId::Range40To50PctInLoss,
    ProfitabilityId::Range50To60PctInLoss,
    ProfitabilityId::Range60To70PctInLoss,
    ProfitabilityId::Range70To80PctInLoss,
    ProfitabilityId::Range80To90PctInLoss,
    ProfitabilityId::Range90To100PctInLoss,
    ProfitabilityId::Profit,
    ProfitabilityId::ProfitOver10Pct,
    ProfitabilityId::ProfitOver20Pct,
    ProfitabilityId::ProfitOver30Pct,
    ProfitabilityId::ProfitOver40Pct,
    ProfitabilityId::ProfitOver50Pct,
    ProfitabilityId::ProfitOver60Pct,
    ProfitabilityId::ProfitOver70Pct,
    ProfitabilityId::ProfitOver80Pct,
    ProfitabilityId::ProfitOver90Pct,
    ProfitabilityId::ProfitOver100Pct,
    ProfitabilityId::ProfitOver200Pct,
    ProfitabilityId::ProfitOver300Pct,
    ProfitabilityId::ProfitOver500Pct,
    ProfitabilityId::Loss,
    ProfitabilityId::LossOver10Pct,
    ProfitabilityId::LossOver20Pct,
    ProfitabilityId::LossOver30Pct,
    ProfitabilityId::LossOver40Pct,
    ProfitabilityId::LossOver50Pct,
    ProfitabilityId::LossOver60Pct,
    ProfitabilityId::LossOver70Pct,
    ProfitabilityId::LossOver80Pct,
];

impl ProfitabilityId {
    pub fn series<T>(mut create: impl FnMut(Self, &'static str) -> T) -> ProfitabilityRow<T> {
        ProfitabilityRow {
            range: Self::range_series(&mut create),
            profit: Self::profit_series(&mut create),
            loss: Self::loss_series(create),
        }
    }

    pub fn range_ids() -> &'static [Self] {
        &PROFITABILITY_IDS[..PROFITABILITY_RANGE_COUNT]
    }

    pub fn profit_ids() -> &'static [Self] {
        &PROFITABILITY_IDS[PROFITABILITY_RANGE_COUNT..PROFITABILITY_RANGE_COUNT + PROFIT_COUNT]
    }

    pub fn loss_ids() -> &'static [Self] {
        &PROFITABILITY_IDS[PROFITABILITY_RANGE_COUNT + PROFIT_COUNT..]
    }

    pub fn ranges(self) -> &'static [ProfitabilityRangeId] {
        match self.group() {
            ProfitabilityGroupId::Range(id) => &ProfitabilityRangeId::ALL[id.index()..=id.index()],
            ProfitabilityGroupId::Profit(id) => {
                &ProfitabilityRangeId::ALL[..PROFIT_COUNT + 1 - id.index()]
            }
            ProfitabilityGroupId::Loss(id) => {
                &ProfitabilityRangeId::ALL[PROFIT_COUNT + 1 + id.index()..]
            }
        }
    }

    pub fn range_series<T>(
        mut create: impl FnMut(Self, &'static str) -> T,
    ) -> ProfitabilityRange<T> {
        let names = ProfitabilityRange::names();
        ProfitabilityRange {
            over_1000pct_in_profit: create(
                Self::RangeOver1000PctInProfit,
                names.over_1000pct_in_profit.id,
            ),
            _500pct_to_1000pct_in_profit: create(
                Self::Range500To1000PctInProfit,
                names._500pct_to_1000pct_in_profit.id,
            ),
            _300pct_to_500pct_in_profit: create(
                Self::Range300To500PctInProfit,
                names._300pct_to_500pct_in_profit.id,
            ),
            _200pct_to_300pct_in_profit: create(
                Self::Range200To300PctInProfit,
                names._200pct_to_300pct_in_profit.id,
            ),
            _100pct_to_200pct_in_profit: create(
                Self::Range100To200PctInProfit,
                names._100pct_to_200pct_in_profit.id,
            ),
            _90pct_to_100pct_in_profit: create(
                Self::Range90To100PctInProfit,
                names._90pct_to_100pct_in_profit.id,
            ),
            _80pct_to_90pct_in_profit: create(
                Self::Range80To90PctInProfit,
                names._80pct_to_90pct_in_profit.id,
            ),
            _70pct_to_80pct_in_profit: create(
                Self::Range70To80PctInProfit,
                names._70pct_to_80pct_in_profit.id,
            ),
            _60pct_to_70pct_in_profit: create(
                Self::Range60To70PctInProfit,
                names._60pct_to_70pct_in_profit.id,
            ),
            _50pct_to_60pct_in_profit: create(
                Self::Range50To60PctInProfit,
                names._50pct_to_60pct_in_profit.id,
            ),
            _40pct_to_50pct_in_profit: create(
                Self::Range40To50PctInProfit,
                names._40pct_to_50pct_in_profit.id,
            ),
            _30pct_to_40pct_in_profit: create(
                Self::Range30To40PctInProfit,
                names._30pct_to_40pct_in_profit.id,
            ),
            _20pct_to_30pct_in_profit: create(
                Self::Range20To30PctInProfit,
                names._20pct_to_30pct_in_profit.id,
            ),
            _10pct_to_20pct_in_profit: create(
                Self::Range10To20PctInProfit,
                names._10pct_to_20pct_in_profit.id,
            ),
            _0pct_to_10pct_in_profit: create(
                Self::Range0To10PctInProfit,
                names._0pct_to_10pct_in_profit.id,
            ),
            _0pct_to_10pct_in_loss: create(
                Self::Range0To10PctInLoss,
                names._0pct_to_10pct_in_loss.id,
            ),
            _10pct_to_20pct_in_loss: create(
                Self::Range10To20PctInLoss,
                names._10pct_to_20pct_in_loss.id,
            ),
            _20pct_to_30pct_in_loss: create(
                Self::Range20To30PctInLoss,
                names._20pct_to_30pct_in_loss.id,
            ),
            _30pct_to_40pct_in_loss: create(
                Self::Range30To40PctInLoss,
                names._30pct_to_40pct_in_loss.id,
            ),
            _40pct_to_50pct_in_loss: create(
                Self::Range40To50PctInLoss,
                names._40pct_to_50pct_in_loss.id,
            ),
            _50pct_to_60pct_in_loss: create(
                Self::Range50To60PctInLoss,
                names._50pct_to_60pct_in_loss.id,
            ),
            _60pct_to_70pct_in_loss: create(
                Self::Range60To70PctInLoss,
                names._60pct_to_70pct_in_loss.id,
            ),
            _70pct_to_80pct_in_loss: create(
                Self::Range70To80PctInLoss,
                names._70pct_to_80pct_in_loss.id,
            ),
            _80pct_to_90pct_in_loss: create(
                Self::Range80To90PctInLoss,
                names._80pct_to_90pct_in_loss.id,
            ),
            _90pct_to_100pct_in_loss: create(
                Self::Range90To100PctInLoss,
                names._90pct_to_100pct_in_loss.id,
            ),
        }
    }

    pub fn profit_series<T>(mut create: impl FnMut(Self, &'static str) -> T) -> Profit<T> {
        let names = Profit::names();
        Profit {
            all: create(Self::Profit, names.all.id),
            _10pct: create(Self::ProfitOver10Pct, names._10pct.id),
            _20pct: create(Self::ProfitOver20Pct, names._20pct.id),
            _30pct: create(Self::ProfitOver30Pct, names._30pct.id),
            _40pct: create(Self::ProfitOver40Pct, names._40pct.id),
            _50pct: create(Self::ProfitOver50Pct, names._50pct.id),
            _60pct: create(Self::ProfitOver60Pct, names._60pct.id),
            _70pct: create(Self::ProfitOver70Pct, names._70pct.id),
            _80pct: create(Self::ProfitOver80Pct, names._80pct.id),
            _90pct: create(Self::ProfitOver90Pct, names._90pct.id),
            _100pct: create(Self::ProfitOver100Pct, names._100pct.id),
            _200pct: create(Self::ProfitOver200Pct, names._200pct.id),
            _300pct: create(Self::ProfitOver300Pct, names._300pct.id),
            _500pct: create(Self::ProfitOver500Pct, names._500pct.id),
        }
    }

    pub fn loss_series<T>(mut create: impl FnMut(Self, &'static str) -> T) -> Loss<T> {
        let names = Loss::names();
        Loss {
            all: create(Self::Loss, names.all.id),
            _10pct: create(Self::LossOver10Pct, names._10pct.id),
            _20pct: create(Self::LossOver20Pct, names._20pct.id),
            _30pct: create(Self::LossOver30Pct, names._30pct.id),
            _40pct: create(Self::LossOver40Pct, names._40pct.id),
            _50pct: create(Self::LossOver50Pct, names._50pct.id),
            _60pct: create(Self::LossOver60Pct, names._60pct.id),
            _70pct: create(Self::LossOver70Pct, names._70pct.id),
            _80pct: create(Self::LossOver80Pct, names._80pct.id),
        }
    }

    #[inline]
    pub const fn is_profit(self) -> bool {
        let index = self as usize;
        index < PROFIT_COUNT + 1
            || (index >= PROFITABILITY_RANGE_COUNT
                && index < PROFITABILITY_RANGE_COUNT + PROFIT_COUNT)
    }
}

enum ProfitabilityGroupId {
    Range(ProfitabilityRangeId),
    Profit(ProfitId),
    Loss(LossId),
}

impl ProfitabilityId {
    const fn group(self) -> ProfitabilityGroupId {
        use ProfitabilityGroupId::{Loss, Profit, Range};

        match self {
            Self::RangeOver1000PctInProfit => Range(ProfitabilityRangeId::Over1000PctInProfit),
            Self::Range500To1000PctInProfit => {
                Range(ProfitabilityRangeId::From500PctTo1000PctInProfit)
            }
            Self::Range300To500PctInProfit => {
                Range(ProfitabilityRangeId::From300PctTo500PctInProfit)
            }
            Self::Range200To300PctInProfit => {
                Range(ProfitabilityRangeId::From200PctTo300PctInProfit)
            }
            Self::Range100To200PctInProfit => {
                Range(ProfitabilityRangeId::From100PctTo200PctInProfit)
            }
            Self::Range90To100PctInProfit => Range(ProfitabilityRangeId::From90PctTo100PctInProfit),
            Self::Range80To90PctInProfit => Range(ProfitabilityRangeId::From80PctTo90PctInProfit),
            Self::Range70To80PctInProfit => Range(ProfitabilityRangeId::From70PctTo80PctInProfit),
            Self::Range60To70PctInProfit => Range(ProfitabilityRangeId::From60PctTo70PctInProfit),
            Self::Range50To60PctInProfit => Range(ProfitabilityRangeId::From50PctTo60PctInProfit),
            Self::Range40To50PctInProfit => Range(ProfitabilityRangeId::From40PctTo50PctInProfit),
            Self::Range30To40PctInProfit => Range(ProfitabilityRangeId::From30PctTo40PctInProfit),
            Self::Range20To30PctInProfit => Range(ProfitabilityRangeId::From20PctTo30PctInProfit),
            Self::Range10To20PctInProfit => Range(ProfitabilityRangeId::From10PctTo20PctInProfit),
            Self::Range0To10PctInProfit => Range(ProfitabilityRangeId::From0PctTo10PctInProfit),
            Self::Range0To10PctInLoss => Range(ProfitabilityRangeId::From0PctTo10PctInLoss),
            Self::Range10To20PctInLoss => Range(ProfitabilityRangeId::From10PctTo20PctInLoss),
            Self::Range20To30PctInLoss => Range(ProfitabilityRangeId::From20PctTo30PctInLoss),
            Self::Range30To40PctInLoss => Range(ProfitabilityRangeId::From30PctTo40PctInLoss),
            Self::Range40To50PctInLoss => Range(ProfitabilityRangeId::From40PctTo50PctInLoss),
            Self::Range50To60PctInLoss => Range(ProfitabilityRangeId::From50PctTo60PctInLoss),
            Self::Range60To70PctInLoss => Range(ProfitabilityRangeId::From60PctTo70PctInLoss),
            Self::Range70To80PctInLoss => Range(ProfitabilityRangeId::From70PctTo80PctInLoss),
            Self::Range80To90PctInLoss => Range(ProfitabilityRangeId::From80PctTo90PctInLoss),
            Self::Range90To100PctInLoss => Range(ProfitabilityRangeId::From90PctTo100PctInLoss),
            Self::Profit => Profit(ProfitId::All),
            Self::ProfitOver10Pct => Profit(ProfitId::Over10Pct),
            Self::ProfitOver20Pct => Profit(ProfitId::Over20Pct),
            Self::ProfitOver30Pct => Profit(ProfitId::Over30Pct),
            Self::ProfitOver40Pct => Profit(ProfitId::Over40Pct),
            Self::ProfitOver50Pct => Profit(ProfitId::Over50Pct),
            Self::ProfitOver60Pct => Profit(ProfitId::Over60Pct),
            Self::ProfitOver70Pct => Profit(ProfitId::Over70Pct),
            Self::ProfitOver80Pct => Profit(ProfitId::Over80Pct),
            Self::ProfitOver90Pct => Profit(ProfitId::Over90Pct),
            Self::ProfitOver100Pct => Profit(ProfitId::Over100Pct),
            Self::ProfitOver200Pct => Profit(ProfitId::Over200Pct),
            Self::ProfitOver300Pct => Profit(ProfitId::Over300Pct),
            Self::ProfitOver500Pct => Profit(ProfitId::Over500Pct),
            Self::Loss => Loss(LossId::All),
            Self::LossOver10Pct => Loss(LossId::Over10Pct),
            Self::LossOver20Pct => Loss(LossId::Over20Pct),
            Self::LossOver30Pct => Loss(LossId::Over30Pct),
            Self::LossOver40Pct => Loss(LossId::Over40Pct),
            Self::LossOver50Pct => Loss(LossId::Over50Pct),
            Self::LossOver60Pct => Loss(LossId::Over60Pct),
            Self::LossOver70Pct => Loss(LossId::Over70Pct),
            Self::LossOver80Pct => Loss(LossId::Over80Pct),
        }
    }
}

impl ColumnId for ProfitabilityId {
    type Row<T>
        = ProfitabilityRow<T>
    where
        T: VecValue;

    const VERSION: Version = Version::TWO;
    const ALL: &'static [Self] = &PROFITABILITY_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        match self.group() {
            ProfitabilityGroupId::Range(id) => id.get(&row.range),
            ProfitabilityGroupId::Profit(id) => id.get(&row.profit),
            ProfitabilityGroupId::Loss(id) => id.get(&row.loss),
        }
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        match self.group() {
            ProfitabilityGroupId::Range(id) => id.get_mut(&mut row.range),
            ProfitabilityGroupId::Profit(id) => id.get_mut(&mut row.profit),
            ProfitabilityGroupId::Loss(id) => id.get_mut(&mut row.loss),
        }
    }

    #[inline]
    fn from_fn<T, F>(mut f: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        Self::series(|id, _| f(id))
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, mut f: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        ProfitabilityRow {
            range: ProfitabilityRangeId::map(row.range, &mut f),
            profit: ProfitId::map(row.profit, &mut f),
            loss: LossId::map(row.loss, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profitability_ids_cover_the_schema_in_storage_order() {
        assert_eq!(
            ProfitabilityId::range_ids().len(),
            PROFITABILITY_RANGE_COUNT
        );
        assert_eq!(ProfitabilityId::profit_ids().len(), PROFIT_COUNT);
        assert_eq!(ProfitabilityId::loss_ids().len(), LOSS_COUNT);
        for (index, id) in PROFITABILITY_IDS.into_iter().enumerate() {
            assert_eq!(id.index(), index);
        }

        assert!(
            ProfitabilityId::range_series(|id, _| id)
                .iter()
                .copied()
                .eq(ProfitabilityId::range_ids().iter().copied())
        );
        assert!(
            ProfitabilityId::profit_series(|id, _| id)
                .iter()
                .copied()
                .eq(ProfitabilityId::profit_ids().iter().copied())
        );
        assert!(
            ProfitabilityId::loss_series(|id, _| id)
                .iter()
                .copied()
                .eq(ProfitabilityId::loss_ids().iter().copied())
        );
    }

    #[test]
    fn rows_expand_ranges_into_profit_prefixes_and_loss_suffixes() {
        let ranges = ProfitabilityRangeId::from_fn(|id| id.index() + 1);
        let row = ProfitabilityRow::from_ranges(ranges.clone());

        assert_eq!(
            row.profit.all,
            ranges.iter().take(PROFIT_COUNT + 1).copied().sum::<usize>()
        );
        assert_eq!(
            row.profit._500pct,
            ranges.over_1000pct_in_profit + ranges._500pct_to_1000pct_in_profit
        );
        assert_eq!(
            row.loss.all,
            ranges.iter().skip(PROFIT_COUNT + 1).copied().sum::<usize>()
        );
        assert_eq!(
            row.loss._80pct,
            ranges.iter().rev().take(2).copied().sum::<usize>()
        );
    }

    #[test]
    fn aggregate_ids_select_their_exact_ranges() {
        for &id in ProfitabilityId::range_ids() {
            assert_eq!(id.ranges().len(), 1);
        }
        for (threshold, &id) in ProfitabilityId::profit_ids().iter().enumerate() {
            assert_eq!(
                id.ranges(),
                &ProfitabilityRangeId::ALL[..PROFIT_COUNT + 1 - threshold]
            );
        }
        for (threshold, &id) in ProfitabilityId::loss_ids().iter().enumerate() {
            assert_eq!(
                id.ranges(),
                &ProfitabilityRangeId::ALL[PROFIT_COUNT + 1 + threshold..]
            );
        }
    }
}
