use std::{env, error::Error};

use bitviewd_bench_visualizer::Visualizer;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let root = args.next();
    if args.next().is_some() {
        return Err("Usage: bitviewd_bench_visualizer [bitviewdir]".into());
    }
    let visualizer = match root {
        Some(root) => Visualizer::new(root),
        None => Visualizer::from_cargo_env()?,
    };
    visualizer.generate()
}
