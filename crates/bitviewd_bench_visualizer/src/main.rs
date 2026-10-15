use std::{env, error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let root = args.next();
    if args.next().is_some() {
        return Err("Usage: bitviewd_bench_visualizer [bitviewdir]".into());
    }
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().nth(2);
    let root = match &root {
        Some(root) => Path::new(root),
        None => workspace.ok_or("Failed to find workspace root")?,
    };
    bitviewd_bench_visualizer::generate(root)
}
