mod composition;
mod near_full_blocks;

use brk_error::Result;

use composition::Plugins;

fn main() -> Result<()> {
    bitview::run_with(Plugins::forced_import)
}
