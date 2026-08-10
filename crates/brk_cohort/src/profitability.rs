use std::{fmt, ops::AddAssign};

use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{ColumnId, Formattable, VecValue, Version};

use crate::{
    LOSS_COUNT, Loss, PROFIT_COUNT, PROFITABILITY_RANGE_COUNT, Profit, ProfitabilityRange,
};

pub const PROFITABILITY_COUNT: usize = PROFITABILITY_RANGE_COUNT + PROFIT_COUNT + LOSS_COUNT;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProfitabilityRow<T> {
    pub range: [T; PROFITABILITY_RANGE_COUNT],
    pub profit: [T; PROFIT_COUNT],
    pub loss: [T; LOSS_COUNT],
}

impl<T> ProfitabilityRow<T>
where
    T: AddAssign + Copy + Default,
{
    pub fn from_ranges(range: [T; PROFITABILITY_RANGE_COUNT]) -> Self {
        let (profit_ranges, loss_ranges) = range.split_at(PROFIT_COUNT + 1);

        let mut profit = [T::default(); PROFIT_COUNT];
        let mut total = profit_ranges[0];
        for (threshold, &value) in profit.iter_mut().rev().zip(&profit_ranges[1..]) {
            total += value;
            *threshold = total;
        }

        let mut loss = [T::default(); LOSS_COUNT];
        let mut total = loss_ranges[loss_ranges.len() - 1];
        for (threshold, &value) in loss
            .iter_mut()
            .rev()
            .zip(loss_ranges[..loss_ranges.len() - 1].iter().rev())
        {
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
        self.range.write_to(buf);
        buf.extend_from_slice(b",\"profit\":");
        self.profit.write_to(buf);
        buf.extend_from_slice(b",\"loss\":");
        self.loss.write_to(buf);
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
    pub fn range_ids() -> &'static [Self] {
        &PROFITABILITY_IDS[..PROFITABILITY_RANGE_COUNT]
    }

    pub fn profit_ids() -> &'static [Self] {
        &PROFITABILITY_IDS[PROFITABILITY_RANGE_COUNT..PROFITABILITY_RANGE_COUNT + PROFIT_COUNT]
    }

    pub fn loss_ids() -> &'static [Self] {
        &PROFITABILITY_IDS[PROFITABILITY_RANGE_COUNT + PROFIT_COUNT..]
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

impl ColumnId for ProfitabilityId {
    type Row<T>
        = ProfitabilityRow<T>
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &PROFITABILITY_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        let index = self as usize;
        if index < PROFITABILITY_RANGE_COUNT {
            &row.range[index]
        } else if index < PROFITABILITY_RANGE_COUNT + PROFIT_COUNT {
            &row.profit[index - PROFITABILITY_RANGE_COUNT]
        } else {
            &row.loss[index - PROFITABILITY_RANGE_COUNT - PROFIT_COUNT]
        }
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        let index = self as usize;
        if index < PROFITABILITY_RANGE_COUNT {
            &mut row.range[index]
        } else if index < PROFITABILITY_RANGE_COUNT + PROFIT_COUNT {
            &mut row.profit[index - PROFITABILITY_RANGE_COUNT]
        } else {
            &mut row.loss[index - PROFITABILITY_RANGE_COUNT - PROFIT_COUNT]
        }
    }

    #[inline]
    fn from_fn<T, F>(mut f: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        ProfitabilityRow {
            range: std::array::from_fn(|index| f(PROFITABILITY_IDS[index])),
            profit: std::array::from_fn(|index| {
                f(PROFITABILITY_IDS[PROFITABILITY_RANGE_COUNT + index])
            }),
            loss: std::array::from_fn(|index| {
                f(PROFITABILITY_IDS[PROFITABILITY_RANGE_COUNT + PROFIT_COUNT + index])
            }),
        }
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, mut f: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        ProfitabilityRow {
            range: row.range.map(&mut f),
            profit: row.profit.map(&mut f),
            loss: row.loss.map(f),
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
        let ranges = std::array::from_fn(|index| index + 1);
        let row = ProfitabilityRow::from_ranges(ranges);

        assert_eq!(
            row.profit[0],
            ranges[..PROFIT_COUNT + 1].iter().copied().sum::<usize>()
        );
        assert_eq!(row.profit[PROFIT_COUNT - 1], ranges[0] + ranges[1]);
        assert_eq!(
            row.loss[0],
            ranges[PROFIT_COUNT + 1..].iter().copied().sum::<usize>()
        );
        assert_eq!(
            row.loss[LOSS_COUNT - 1],
            ranges[PROFITABILITY_RANGE_COUNT - 2..]
                .iter()
                .copied()
                .sum::<usize>()
        );
    }
}
