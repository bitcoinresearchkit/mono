use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use tempfile::tempdir;

fn setup(home: &Path) -> PathBuf {
    let config = home.join(".bitview");
    let data = home.join("data");
    let bitcoin = home.join("bitcoin");
    fs::create_dir_all(&config).unwrap();
    fs::create_dir_all(data.join("logs")).unwrap();
    // Node paths do not exist and RPC credentials are incomplete: loading paths must still work.
    fs::write(config.join("config.toml"), format!(
        "bitviewdir = '{}'\nbitcoindir = '{}'\nrpcpassword = 'never include this in a report'\n",
        data.display(), bitcoin.display())).unwrap();
    data
}

fn run(home: &Path) -> Output {
    let output = Command::new(env!("CARGO_BIN_EXE_bitviewd_latency"))
        .env("HOME", home)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn daemon_config_paths_daily_deduplication_and_atomic_report() {
    let home = tempdir().unwrap();
    let data = setup(home.path());
    let logs = data.join("logs");
    let mut log = String::new();
    for i in 1..=100 {
        log.push_str(&format!(
            "2026-09-04 12:00:00 - info  200 /api/tx/tx{i}?expand=true {i}ms\n"
        ));
    }
    log.push_str("2026-09-04 12:00:01 - info  304 /api/tx/tx1 10µs\n2026-09-04 12:00:02 - error 400 /api/tx/tx1 2ms\n2026-09-04 12:00:03 - info  New chain tip\n2026-09-04 12:00:04 - info  200 /api truncated\n");
    let source = logs.join("2026-09-04.txt");
    fs::write(&source, &log).unwrap();
    fs::write(logs.join("2026-09-04_info.txt"), &log).unwrap();
    fs::write(
        logs.join("2026-09-05_error.txt"),
        "2026-09-05 12:00:00 - error 500 /api/tx/tx2 3s\n",
    )
    .unwrap();
    let output = run(home.path());
    let path = data.join("latency.md");
    assert!(String::from_utf8_lossy(&output.stdout).contains(path.to_str().unwrap()));
    let report = fs::read_to_string(&path).unwrap();
    assert!(report.contains("**103 captured requests**"));
    assert!(report.contains("1 malformed access lines"));
    assert!(report.contains(
        "| /api/tx/{txid} | 100 | 50.000 | 95.000 | 99.000 | — | 50.500 | 100.000 | 5.050 | 1.94% |"
    ));
    assert!(report.contains("tx100?expand=true"));
    assert!(!report.contains("never include"));
    assert!(!home.path().join("bitcoin").exists());
    assert_eq!(report.matches("### Endpoints by P95").count(), 4);
    assert!(!report.contains("### Highest P99"));
    assert!(!report.contains("### Most accumulated request time"));
    let sections: Vec<_> = report.split("\n## HTTP ").skip(1).collect();
    assert_eq!(sections.len(), 4);
    for (section, code) in sections.iter().zip([200, 304, 400, 500]) {
        assert!(section.starts_with(&format!("{code}\n")));
        assert!(section.contains("### Duration histogram"));
        assert!(section.contains("### Slow request examples"));
    }
    // Default output cannot truncate a source log, even if hard-linked to it.
    fs::remove_file(&path).unwrap();
    fs::hard_link(&source, &path).unwrap();
    run(home.path());
    assert_eq!(fs::read_to_string(&source).unwrap(), log);
    assert_eq!(fs::read_to_string(&path).unwrap(), report);
}

#[test]
fn invalid_config_preserves_existing_report() {
    let home = tempdir().unwrap();
    let data = setup(home.path());
    fs::write(data.join("latency.md"), "previous report").unwrap();
    fs::write(
        home.path().join(".bitview/config.toml"),
        "bitviewdir = [broken",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_bitviewd_latency"))
        .env("HOME", home.path())
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid config"));
    assert_eq!(
        fs::read_to_string(data.join("latency.md")).unwrap(),
        "previous report"
    );
}
