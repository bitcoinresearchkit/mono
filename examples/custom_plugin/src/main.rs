use bitview_custom_plugin_example::composition::Plugins;
use brk_error::Result;

fn main() -> Result<()> {
    bitviewd::run(Plugins::import)
}
