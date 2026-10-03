use bitview::run;
use bitview_custom_plugin_example::composition::Plugins;
use brk_error::Result;

#[global_allocator]
static GLOBAL: brk_alloc::MiMalloc = brk_alloc::MiMalloc;

fn main() -> Result<()> {
    run(Plugins::import)
}
