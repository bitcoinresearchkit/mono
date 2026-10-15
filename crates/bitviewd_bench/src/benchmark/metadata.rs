use std::{
    ffi::CString,
    fmt,
    fs::File,
    io::{self, BufWriter, Write},
    mem::MaybeUninit,
    os::unix::ffi::OsStrExt,
    path::Path,
    process::Command,
    thread,
};

use brk_types::Height;

/// The hardware a run measures, from its CPU, RAM and the data directory's storage
/// (`apple-m6-16gb-external-tb`): runs of one machine compare.
pub fn machine(data_path: &Path) -> String {
    let mut slug = String::new();
    for word in cpu().split(|c: char| !c.is_ascii_alphanumeric()) {
        if !word.is_empty() {
            slug.push_str(&word.to_ascii_lowercase());
            slug.push('-');
        }
    }
    // Rounded: the reported total can sit below the physical size.
    let gib = (ram_bytes().parse::<u64>().unwrap_or_default() + (1 << 29)) >> 30;
    format!("{slug}{gib}gb-{}", storage(data_path).class)
}

/// A path's drive: `internal`, `external-tb` (Thunderbolt or USB4, PCIe; macOS), `external-usb`,
/// `external` or `unknown`, and its model.
struct Storage {
    class: &'static str,
    model: String,
}

impl fmt::Display for Storage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.class)?;
        if !self.model.is_empty() {
            write!(f, " {}", self.model)?;
        }
        Ok(())
    }
}

pub fn write(
    run_path: &Path,
    data_path: &Path,
    blocks_path: &Path,
    started_at: u32,
    chain_height: Height,
    cache_budget: usize,
) -> io::Result<()> {
    let mut writer = BufWriter::new(File::create(run_path.join("metadata.txt"))?);

    // The build, from compile time (`build.rs`).
    writeln!(writer, "version={}", env!("CARGO_PKG_VERSION"))?;
    writeln!(writer, "revision={}", env!("BENCH_COMMIT"))?;
    writeln!(writer, "dirty={}", env!("BENCH_DIRTY"))?;
    writeln!(writer, "profile={}", env!("BENCH_PROFILE"))?;
    writeln!(writer, "rustc={}", env!("BENCH_RUSTC"))?;
    writeln!(writer, "rustflags={}", env!("BENCH_RUSTFLAGS"))?;
    writeln!(writer, "target={}", env!("BENCH_TARGET"))?;
    // The host and the input data.
    writeln!(writer, "hostname={}", command("uname", &["-n"]))?;
    writeln!(writer, "cpu={}", cpu())?;
    writeln!(writer, "ram_bytes={}", ram_bytes())?;
    writeln!(writer, "storage={}", storage(data_path))?;
    writeln!(writer, "blocks_storage={}", storage(blocks_path))?;
    // SSDs slow down as they fill: runs compare only at equal used space.
    writeln!(writer, "data_volume={}", volume(data_path))?;
    writeln!(writer, "blocks_volume={}", volume(blocks_path))?;
    writeln!(
        writer,
        "parallelism={}",
        thread::available_parallelism().map_or(1, usize::from)
    )?;
    writeln!(writer, "started_at_unix={started_at}")?;
    writeln!(writer, "chain_height={chain_height}")?;
    writeln!(writer, "blocks_path={}", blocks_path.display())?;
    writeln!(writer, "data_path={}", data_path.display())?;
    writeln!(writer, "cache_budget={cache_budget}")?;
    writer.flush()
}

/// `used_bytes/total_bytes` of the volume holding `path`, empty when unavailable.
fn volume(path: &Path) -> String {
    let Ok(path) = CString::new(path.as_os_str().as_bytes()) else {
        return String::new();
    };
    let mut stat = MaybeUninit::<libc::statvfs>::zeroed();
    // SAFETY: `path` is NUL-terminated and `stat` is a writable `statvfs`.
    if unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) } != 0 {
        return String::new();
    }
    // SAFETY: `statvfs` succeeded, so it initialized `stat`.
    let stat = unsafe { stat.assume_init() };
    let block = stat.f_frsize;
    let total = u64::from(stat.f_blocks) * block;
    let used = total - u64::from(stat.f_bfree) * block;
    format!("{used}/{total}")
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

/// From `diskutil`: the volume's location and protocol, the model of its drive.
#[cfg(target_os = "macos")]
fn storage(path: &Path) -> Storage {
    let field = |info: &str, key: &str| {
        info.lines()
            .find_map(|line| line.trim().strip_prefix(key)?.strip_prefix(':'))
            .map(|value| value.trim().to_owned())
            .unwrap_or_default()
    };
    let df = command("df", &["-P", &path.to_string_lossy()]);
    let device = df
        .lines()
        .nth(1)
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or_default();
    let volume = command("diskutil", &["info", device]);
    let class = match (
        field(&volume, "Device Location").as_str(),
        field(&volume, "Protocol").as_str(),
    ) {
        ("Internal", _) => "internal",
        ("External", "PCI-Express" | "Thunderbolt") => "external-tb",
        ("External", "USB") => "external-usb",
        ("External", _) => "external",
        _ => "unknown",
    };
    // An APFS volume's drive is its physical store's whole disk.
    let store = field(&volume, "APFS Physical Store");
    let partition = if store.is_empty() {
        volume
    } else {
        command("diskutil", &["info", &store])
    };
    let drive = command("diskutil", &["info", &field(&partition, "Part of Whole")]);
    Storage {
        class,
        model: field(&drive, "Device / Media Name"),
    }
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

/// From `lsblk`: the transport, hotplug flag and model of the disk under the path's mount. A
/// hotplug disk is `external`: Linux does not tell Thunderbolt from other hot-swap buses here.
#[cfg(target_os = "linux")]
fn storage(path: &Path) -> Storage {
    let source = command(
        "findmnt",
        &["-nvo", "SOURCE", "--target", &path.to_string_lossy()],
    );
    let devices = command("lsblk", &["-nsPo", "TYPE,TRAN,HOTPLUG,MODEL", &source]);
    let value = |line: &str, key: &str| {
        line.split(&format!("{key}=\""))
            .nth(1)
            .and_then(|rest| rest.split('"').next())
            .unwrap_or_default()
            .to_owned()
    };
    let drive = devices
        .lines()
        .find(|line| value(line, "TYPE") == "disk")
        .unwrap_or_default();
    let class = match (
        value(drive, "TRAN").as_str(),
        value(drive, "HOTPLUG").as_str(),
    ) {
        ("", _) => "unknown",
        ("usb", _) => "external-usb",
        (_, "1") => "external",
        _ => "internal",
    };
    Storage {
        class,
        model: value(drive, "MODEL"),
    }
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
