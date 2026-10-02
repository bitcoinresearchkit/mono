use tempfile::tempdir;

use super::{dir_size as walk_dir_size, *};

#[cfg(unix)]
use std::os::unix::fs::symlink;

fn dir_size(path: &Path) -> Result<u64> {
    walk_dir_size(path, &AtomicBool::new(false))
}

#[test]
fn directory_size_counts_nested_allocated_bytes() -> Result<()> {
    let directory = tempdir()?;
    let nested = directory.path().join("nested");
    fs::create_dir(&nested)?;

    let first = directory.path().join("first");
    let second = nested.join("second");
    fs::write(&first, [0; 1])?;
    fs::write(&second, [0; 8192])?;

    let first_bytes = allocated_bytes(&fs::metadata(&first)?)?;
    let second_bytes = allocated_bytes(&fs::metadata(&second)?)?;
    let mut expected = first_bytes + second_bytes;

    #[cfg(unix)]
    {
        symlink(&second, directory.path().join("second-link"))?;
        expected += second_bytes;
    }

    assert_eq!(dir_size(directory.path())?, expected);
    assert_eq!(dir_size(&first)?, first_bytes);
    Ok(())
}

#[cfg(unix)]
#[test]
fn directory_aliases_are_counted_but_cycles_fail() -> Result<()> {
    let directory = tempdir()?;
    let child = directory.path().join("child");
    fs::create_dir(&child)?;
    fs::write(child.join("data"), [0; 8192])?;
    let bytes = dir_size(&child)?;
    symlink(&child, directory.path().join("alias"))?;
    assert_eq!(dir_size(directory.path())?, 2 * bytes);
    symlink(directory.path(), child.join("back"))?;
    let error = dir_size(directory.path()).unwrap_err();
    assert!(
        error.to_string().contains("directory symlink cycle"),
        "{error}"
    );
    Ok(())
}
