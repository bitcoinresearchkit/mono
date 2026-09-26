use std::{fs, os::unix::fs::MetadataExt};

use brk_error::Result;
use brk_types::Height;
use tempfile::tempdir;

use super::Benchmark;

#[test]
fn reports_follow_data_directory_without_affecting_disk_usage() -> Result<()> {
    let directory = tempdir()?;
    let data_path = directory.path().join("data");
    let runs = data_path.join("benches/bitviewd");
    let previous = runs.join("run-previous");
    fs::create_dir_all(&previous)?;
    fs::write(previous.join("progress.csv"), [b'x'; 8192])?;

    // A similarly named directory inside plugin data must still be counted.
    let plugin_path = data_path.join("plugins/example/benches/bitviewd");
    fs::create_dir_all(&plugin_path)?;
    let payload = plugin_path.join("data.bin");
    fs::write(&payload, [b'x'; 4096])?;
    let expected_bytes = fs::metadata(payload)?.blocks() * 512;

    let benchmark = Benchmark::new(&data_path, &directory.path().join("blocks"), Height::new(0))?;
    assert_eq!(benchmark.path().parent(), Some(runs.as_path()));
    benchmark.measure(|| Ok(()))?;

    for name in [
        "metadata.txt",
        "disk.csv",
        "memory.csv",
        "io.csv",
        "progress.csv",
        "run.csv",
        "timings.csv",
    ] {
        assert!(benchmark.path().join(name).is_file(), "missing {name}");
    }
    let disk = fs::read_to_string(benchmark.path().join("disk.csv"))?;
    let samples: Vec<_> = disk.lines().skip(1).collect();
    assert_eq!(samples.len(), 2);
    for sample in samples {
        let (_, bytes) = sample.split_once(',').unwrap();
        assert_eq!(bytes, expected_bytes.to_string());
    }
    assert!(fs::read_to_string(benchmark.path().join("run.csv"))?.contains(",complete"));
    assert!(previous.join("progress.csv").is_file());
    Ok(())
}
