use std::time::Instant;

use bitview_plugin::UpdateContext;
use brk_error::Result;
use tracing::info;

use crate::{BootstrapAction, ComputePluginSet};

/// Performs and publishes one complete steady-state update.
pub fn update<P>(plugins: &mut P, context: UpdateContext<'_>) -> Result<()>
where
    P: ComputePluginSet,
{
    run(plugins, context, |plugins, context| {
        plugins.compute(context).map(|()| ((), true))
    })
}

pub fn bootstrap_update<P>(plugins: &mut P, context: UpdateContext<'_>) -> Result<BootstrapAction>
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
    let publication = plugins.publication().clone();
    publication.begin_update();

    let start = Instant::now();
    let (output, complete) = compute(plugins, context)?;
    plugins.commit(complete)?;
    publication.finish_update();
    info!("Update completed in {:.2?}", start.elapsed());
    Ok(output)
}
