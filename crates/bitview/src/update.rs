use std::time::Instant;

use bitview_plugin::{BootstrapAction, ComputePluginSet, UpdateContext};
use brk_error::Result;
use tracing::info;

/// Performs one complete steady-state update and publishes it, unless its chain did not move:
/// returns whether it published. `on_publish` runs as readers first see the publication.
pub(crate) fn update<P>(
    plugins: &mut P,
    context: UpdateContext<'_>,
    on_publish: impl FnOnce(),
) -> Result<bool>
where
    P: ComputePluginSet,
{
    run(
        plugins,
        context,
        |plugins, context| {
            let moved = plugins.compute(context)?;
            Ok((moved, moved.then_some(true)))
        },
        on_publish,
    )
}

pub(crate) fn bootstrap_update<P>(
    plugins: &mut P,
    context: UpdateContext<'_>,
) -> Result<BootstrapAction>
where
    P: ComputePluginSet,
{
    run(
        plugins,
        context,
        |plugins, context| {
            let action = plugins.bootstrap_compute(context)?;
            Ok((action, Some(action == BootstrapAction::Ready)))
        },
        || (),
    )
}

/// `compute` returns its output and how to commit: `Some(complete)`, or `None` to leave the last
/// publication as it is.
fn run<P, T>(
    plugins: &mut P,
    context: UpdateContext<'_>,
    compute: impl FnOnce(&mut P, UpdateContext<'_>) -> Result<(T, Option<bool>)>,
    on_publish: impl FnOnce(),
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
    let (output, commit) = compute(plugins, context)?;
    let Some(complete) = commit else {
        return Ok(output);
    };
    let _lock = context.exit().lock();
    plugins
        .commit(complete)
        .map_err(|error| error.context("commit"))?;
    info!("Update published in {:.2?}", start.elapsed());
    on_publish();
    plugins
        .write_published()
        .map_err(|error| error.context("write published state"))?;
    Ok(output)
}
