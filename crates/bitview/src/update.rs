use std::time::Instant;

use bitview_plugin::{BootstrapAction, ComputePluginSet, UpdateContext};
use brk_error::Result;
use tracing::info;

/// Performs and publishes one complete steady-state update.
pub(crate) fn update<P>(plugins: &mut P, context: UpdateContext<'_>) -> Result<()>
where
    P: ComputePluginSet,
{
    run(plugins, context, |plugins, context| {
        plugins.compute(context).map(|()| ((), true))
    })
}

pub(crate) fn bootstrap_update<P>(
    plugins: &mut P,
    context: UpdateContext<'_>,
) -> Result<BootstrapAction>
where
    P: ComputePluginSet,
{
    run(plugins, context, |plugins, context| {
        let action = plugins.bootstrap_compute(context)?;
        Ok((action, action == BootstrapAction::Ready))
    })
}

fn run<P, T>(
    plugins: &mut P,
    context: UpdateContext<'_>,
    compute: impl FnOnce(&mut P, UpdateContext<'_>) -> Result<(T, bool)>,
) -> Result<T>
where
    P: ComputePluginSet,
{
    // A pending shutdown must not wait behind back-to-back updates.
    context.exit().park_if_shutting_down();
    // Readers keep the last publication (lowered to the fork once a reorg rolls back) until `commit`
    // swaps in this one; a failed compute leaves it. What the commit published from memory is
    // written before the next update.
    let start = Instant::now();
    let (output, complete) = compute(plugins, context)?;
    let _lock = context.exit().lock();
    plugins.commit(complete)?;
    info!("Update completed in {:.2?}", start.elapsed());
    plugins.write_published()?;
    Ok(output)
}
