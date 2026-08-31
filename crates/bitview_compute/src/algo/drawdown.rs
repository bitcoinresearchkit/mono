use brk_error::Result;

use brk_exit::Exit;
use brk_types::PartsPerMillionSigned32;
use vecdb::{EagerVec, PcoVec, ReadableVec, VecIndex, VecValue};

pub trait ComputeDrawdown<I: VecIndex> {
    fn compute_drawdown<C, A>(
        &mut self,
        max_from: I,
        current: &impl ReadableVec<I, C>,
        ath: &impl ReadableVec<I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        C: VecValue,
        A: VecValue,
        f64: From<C> + From<A>;
}

impl<I> ComputeDrawdown<I> for EagerVec<PcoVec<I, PartsPerMillionSigned32>>
where
    I: VecIndex,
{
    fn compute_drawdown<C, A>(
        &mut self,
        max_from: I,
        current: &impl ReadableVec<I, C>,
        ath: &impl ReadableVec<I, A>,
        exit: &Exit,
    ) -> Result<()>
    where
        C: VecValue,
        A: VecValue,
        f64: From<C> + From<A>,
    {
        self.compute_transform2(
            max_from,
            current,
            ath,
            |(i, current, ath, _)| {
                let ath_f64 = f64::from(ath);
                let drawdown = if ath_f64 == 0.0 {
                    PartsPerMillionSigned32::default()
                } else {
                    PartsPerMillionSigned32::from((f64::from(current) - ath_f64) / ath_f64)
                };
                (i, drawdown)
            },
            exit,
        )?;
        Ok(())
    }
}
