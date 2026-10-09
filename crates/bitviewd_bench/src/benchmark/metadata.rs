use std::{
    env::consts,
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
    process::Command,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use brk_types::Height;

pub fn write(
    run_path: &Path,
    workspace: &Path,
    data_path: &Path,
    blocks_path: &Path,
    chain_height: Height,
    cache_budget: usize,
) -> io::Result<()> {
    let mut writer = BufWriter::new(File::create(run_path.join("metadata.txt"))?);
    let revision = git(workspace, &["rev-parse", "HEAD"]);
    let dirty = !git(workspace, &["status", "--porcelain"]).is_empty();
    let started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    writeln!(writer, "version={}", env!("CARGO_PKG_VERSION"))?;
    writeln!(
        writer,
        "profile={}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    )?;
    writeln!(writer, "hostname={}", command("uname", &["-n"]))?;
    writeln!(writer, "cpu={}", cpu())?;
    writeln!(writer, "ram_bytes={}", ram_bytes())?;
    writeln!(writer, "target_os={}", consts::OS)?;
    writeln!(writer, "target_arch={}", consts::ARCH)?;
    writeln!(
        writer,
        "parallelism={}",
        thread::available_parallelism().map_or(1, usize::from)
    )?;
    writeln!(writer, "started_at_unix={started_at}")?;
    writeln!(writer, "chain_height={chain_height}")?;
    writeln!(writer, "revision={revision}")?;
    writeln!(writer, "dirty={dirty}")?;
    writeln!(writer, "blocks_path={}", blocks_path.display())?;
    writeln!(writer, "data_path={}", data_path.display())?;
    writeln!(writer, "cache_budget={cache_budget}")?;
    writer.flush()
}

fn git(workspace: &Path, args: &[&str]) -> String {
    Command::new("git")
        .args(args)
        .current_dir(workspace)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_owned())
        .unwrap_or_default()
}

fn command(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|output| output.trim().to_owned())
        .unwrap_or_default()
}

#[cfg(target_os = "macos")]
fn cpu() -> String {
    command("sysctl", &["-n", "machdep.cpu.brand_string"])
}

#[cfg(target_os = "macos")]
fn ram_bytes() -> String {
    command("sysctl", &["-n", "hw.memsize"])
}

#[cfg(target_os = "linux")]
fn cpu() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|info| {
            info.lines()
                .find_map(|line| line.strip_prefix("model name"))
                .and_then(|value| value.split_once(':'))
                .map(|(_, name)| name.trim().to_owned())
        })
        .unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn ram_bytes() -> String {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|info| {
            let kib = info
                .lines()
                .find_map(|line| line.strip_prefix("MemTotal:"))?;
            let kib: u64 = kib.split_whitespace().next()?.parse().ok()?;
            Some((kib * 1024).to_string())
        })
        .unwrap_or_default()
}
