use std::{env, path::Path, process};

use importmap::ImportMap;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("Usage: importmap [dir]");
        eprintln!();
        eprintln!("  dir  Directory with index.html (default: .)");
        eprintln!();
        eprintln!("Updates content between <!-- IMPORTMAP --> and <!-- /IMPORTMAP --> markers.");
        process::exit(0);
    }

    let dir = args.first().map(|s| s.as_str()).unwrap_or(".");

    if let Err(e) = run(Path::new(dir)) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn run(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let html_path = dir.join("index.html");
    if !html_path.exists() {
        return Err(format!("{} not found", html_path.display()).into());
    }

    let map = ImportMap::scan(dir, "")?;
    if map.update_html_file(&html_path)? {
        eprintln!("Updated {}", html_path.display());
    }

    Ok(())
}
