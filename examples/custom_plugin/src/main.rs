use bitview::{Result, run};
use bitview_custom_plugin_example::composition::Plugins;

#[global_allocator]
static GLOBAL: brk_alloc::MiMalloc = brk_alloc::MiMalloc;

fn main() -> Result<()> {
    bitview_default::build_thread_pool();
    run(Plugins::import)
}
