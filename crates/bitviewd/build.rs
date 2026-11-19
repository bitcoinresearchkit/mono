use std::{env, fs, io, path::Path};

use sha2::{Digest, Sha256};

fn main() {
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let package = Path::new(&manifest);
    let (root, sources) = if package.ends_with("crates/bitviewd") {
        (package.ancestors().nth(2).unwrap(), "crates")
    } else {
        // The published daemon's lockfile pins its separately packaged dependencies.
        (package, "src")
    };
    let mut hash = Sha256::new();
    for input in ["Cargo.toml", "Cargo.lock", sources] {
        let path = root.join(input);
        println!("cargo::rerun-if-changed={}", path.display());
        hash_input(&path, &mut hash).unwrap_or_else(|error| {
            panic!("Hashing Bitview build input {}: {error}", path.display());
        });
    }
    let tag: String = hash.finalize()[..8].iter().map(|byte| format!("{byte:02x}")).collect();
    println!("cargo::rustc-env=BITVIEW_SOURCE_TAG={tag}");
}

fn hash_input(path: &Path, hash: &mut Sha256) -> io::Result<()> {
    let kind = path.symlink_metadata()?.file_type();
    // The workspace symlink embeds website assets, which have their own content tags.
    if kind.is_symlink() {
        return Ok(());
    }
    hash.update(if kind.is_dir() { b"d" } else { b"f" });
    let name = path.file_name().unwrap().as_encoded_bytes();
    hash.update((name.len() as u64).to_le_bytes());
    hash.update(name);
    if kind.is_dir() {
        let mut children = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<Vec<_>>>()?;
        children.sort_unstable();
        for child in children {
            hash_input(&child, hash)?;
        }
        hash.update(b"e");
    } else {
        let bytes = fs::read(path)?;
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    Ok(())
}
