use std::{
    collections::HashSet, env, fmt::Display, fs, io, net::IpAddr, path::Path, process::exit,
    result::Result as StdResult, str::FromStr,
};

use bitview_server::{
    CdnCacheMode, DEFAULT_BIND, DEFAULT_MAX_UTXOS, DEFAULT_MAX_WEIGHT, Port, PublicUrl,
    ServerConfig, SiteUrl,
};
use brk_error::{Error, Result};
use brk_reader::Reader;
use brk_rpc::ConnectArgs;
use lexopt::{
    Arg::{Long, Short},
    Parser, ValueExt,
};
use owo_colors::OwoColorize;
use serde::Serialize;
use serde_json::{Value, to_value};

use crate::{
    RunConfig,
    paths::{default_bitview_dir, expand_home},
};

/// Default size, in bytes, of the shared vector cache (`vecdb::Budgeted::init_global`). The page cache
/// holds the same files: on a 16 GB machine 1 GiB synced ~80 s faster than 2 GiB, 512 MiB no faster.
pub const DEFAULT_CACHE_BUDGET: usize = 1 << 30;

/// One setting: `key = value` in `config.toml`, `--key <VALUE>` as a flag.
struct Setting {
    key: &'static str,
    /// Whether the setting's value is omitted from dumps and diagnostics.
    secret: bool,
    /// Parses its text into its field.
    set: fn(&mut Config, &str) -> StdResult<(), String>,
}

impl Setting {
    /// Some parser errors include the input that failed.
    fn diagnostic(&self, detail: impl FnOnce() -> String) -> String {
        if self.secret {
            "malformed value (a secret, not shown)".to_owned()
        } else {
            detail()
        }
    }
}

/// Declares every setting once, as `[#[secret]] key: type, value, help[, default];` (rustc's `options!`
/// is the model): `Config` holds each as an `Option` field, `SETTINGS` is what the flags and
/// `config.toml` set them through, and `print_settings` writes their lines of `--help`, a blank line
/// before each group.
macro_rules! settings {
    ($([$($(#[$secret:ident])? $key:ident: $type:ty, $value:literal, $help:literal $(, $default:expr)?;)*])*) => {
        // A derived `Debug` would include secret fields.
        #[derive(Default, Serialize)]
        pub struct Config {
            $($($key: Option<$type>,)*)*
        }

        static SETTINGS: &[Setting] = &[$($(Setting {
            key: stringify!($key),
            secret: secret!($($secret)?),
            set: |config, text| store(&mut config.$key, text),
        },)*)*];

        fn print_settings() {
            $(
                println!();
                $(
                    // Every description starts in the same column.
                    let pad = 23usize.saturating_sub(stringify!($key).len() + $value.len());
                    print!("    --{} {}{:pad$}{}", stringify!($key), $value.bright_black(), "", $help);
                    $(print!(" {}", format!("[{}]", $default).bright_black());)?
                    println!();
                )*
            )*
        }
    };
}

/// Whether a setting's declaration starts with `#[secret]`; any other attribute doesn't compile.
macro_rules! secret {
    () => {
        false
    };
    (secret) => {
        true
    };
}

fn store<T: FromStr<Err: Display>>(field: &mut Option<T>, text: &str) -> StdResult<(), String> {
    *field = Some(text.parse().map_err(|error: T::Err| error.to_string())?);
    Ok(())
}

settings! {
    [
        bitviewdir: String, "<PATH>", "Output directory", "~/.bitview";
        serverbind: IpAddr, "<IP>", "Server bind address", DEFAULT_BIND;
        serverport: Port, "<PORT>", "Server port", Port::DEFAULT;
        cdn: bool, "<BOOL>", "Aggressive CDN cache, requires purge on deploy", false;
        maxweight: usize, "<BYTES>",
            "Server cap on series response weight in bytes; rejects /api/series/... over the limit",
            DEFAULT_MAX_WEIGHT;
        maxutxos: usize, "<COUNT>",
            "Server cap on UTXOs per address; /api/address/{addr}/utxo errors past the limit",
            DEFAULT_MAX_UTXOS;
        publicurl: PublicUrl, "<URL>",
            "The URL reaching this server alone, sent to clients so they stay on it; only for several servers behind one URL without affinity";
        siteurl: SiteUrl, "<URL>", "The site's public URL, which the MCP server at <URL>/mcp names itself by",
            "http://localhost:<serverport>";
        cachebudget: usize, "<BYTES>", "Shared vector cache size in bytes", DEFAULT_CACHE_BUDGET;
    ]
    [
        bitcoindir: String, "<PATH>", "Bitcoin directory", "OS default";
        blocksdir: String, "<PATH>", "Blocks directory", "<chain data dir>/blocks";
    ]
    [
        rpcconnect: String, "<HOST>", "RPC host", "localhost";
        rpcport: u16, "<PORT>", "RPC port", 8332;
        rpccookiefile: String, "<PATH>", "RPC cookie file; relative paths resolve inside <bitcoindir>",
            "<chain data dir>/.cookie";
        rpcuser: String, "<USERNAME>", "RPC username";
        #[secret] rpcpassword: String, "<PASSWORD>", "RPC password";
    ]
}

/// What an old config or command line passing `website` is told.
const WEBSITE_GONE: &str = "the setting is gone: the site is always served, remove it";

fn setting(key: &str) -> Option<&'static Setting> {
    SETTINGS.iter().find(|setting| setting.key == key)
}

impl Config {
    pub fn import() -> Result<RunConfig> {
        let (flags, dump) = Self::parse_args()?;

        // Without a home directory there's no config.toml: the flags alone.
        let mut config = match default_bitview_dir() {
            Some(dir) => Self::read(&dir.join("config.toml"))?,
            None => Self::default(),
        };
        // Flags go over the file's settings, a flag given again over its earlier value.
        for (setting, value) in flags {
            // Worded as lexopt words a value its parsing rejects.
            (setting.set)(&mut config, &value).map_err(|error| {
                let problem =
                    setting.diagnostic(|| format!("cannot parse argument {value:?}: {error}"));
                Error::Parse(format!("--{}: {problem}", setting.key))
            })?;
        }
        if dump {
            config.dump()?;
            exit(0);
        }
        let runner = config.runner()?;
        fs::create_dir_all(&runner.server.data_path)?;

        Ok(runner)
    }

    /// The flags' settings as given, in order, and whether `--dumpconfig` asks for the settings to be
    /// printed.
    fn parse_args() -> Result<(Vec<(&'static Setting, String)>, bool)> {
        let mut flags = Vec::new();
        let mut dump = false;
        let mut parser = Parser::from_env();
        let command = Self::command_name();

        while let Some(arg) = parser.next().map_err(parse_error)? {
            match arg {
                Short('h') | Long("help") => {
                    Self::print_help(&command);
                    exit(0);
                }
                Short('V') | Long("version") => {
                    println!("{command} {}", env!("CARGO_PKG_VERSION"));
                    exit(0);
                }
                Long("dumpconfig") => dump = true,
                // fleet checks a copy of another server's audit store before it stops this server to
                // merge it (AuditStore::open): a torn copy fails here, while the server still serves.
                #[cfg(feature = "full-api")]
                Long("checkaudits") => {
                    let dir = parser.value().map_err(parse_error)?;
                    let records = brk_mempool::audit::check_copy(Path::new(&dir))?;
                    println!("{} audits in {}", records.len() / 2, dir.display());
                    exit(0);
                }
                Long("website") => return Err(Error::Parse(format!("--website: {WEBSITE_GONE}"))),
                Long(name) => {
                    let Some(setting) = setting(name) else {
                        return Err(parse_error(arg.unexpected()));
                    };
                    let value = parser.value().map_err(parse_error)?;
                    let value = value.string().map_err(|error| {
                        let problem = setting.diagnostic(|| error.to_string());
                        Error::Parse(format!("--{}: {problem}", setting.key))
                    })?;
                    flags.push((setting, value));
                }
                _ => return Err(parse_error(arg.unexpected())),
            }
        }

        Ok((flags, dump))
    }

    /// Read `config.toml`: a `key = value` line per setting, its value read as its flag would be,
    /// with blank lines and `#` comments around them.
    fn parse_toml(contents: &str) -> StdResult<Self, String> {
        let mut config = Self::default();
        let mut seen = HashSet::new();
        for (index, line) in contents.trim_prefix('\u{feff}').lines().enumerate() {
            let at = |problem: String| format!("line {}: {problem}", index + 1);
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // A malformed assignment can contain a secret before the first `=`.
            let malformed = || at("expected `key = value`".to_owned());
            let (key, value) = line.split_once('=').ok_or_else(malformed)?;
            let key = key.trim();
            let key = ['"', '\'']
                .into_iter()
                .find_map(|quote| key.strip_prefix(quote)?.strip_suffix(quote))
                .unwrap_or(key);
            let Some(setting) = setting(key) else {
                if key == "website" {
                    return Err(at(format!("website: {WEBSITE_GONE}")));
                }
                if key.is_empty()
                    || !key
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
                {
                    return Err(malformed());
                }
                let keys: Vec<_> = SETTINGS.iter().map(|setting| setting.key).collect();
                return Err(at(format!(
                    "unknown setting `{key}`, expected one of {}",
                    keys.join(", ")
                )));
            };
            if !seen.insert(key) {
                return Err(at(format!("`{key}` is set twice")));
            }
            let text = toml_value(value.trim()).map_err(|error| {
                let problem = setting.diagnostic(|| error);
                at(format!("{key}: {problem}"))
            })?;
            (setting.set)(&mut config, &text).map_err(|error| {
                let problem = setting.diagnostic(|| format!("cannot parse {text:?}: {error}"));
                at(format!("{key}: {problem}"))
            })?;
        }
        Ok(config)
    }

    /// Print the settings given, in `config.toml` and as flags, the flags first, as
    /// `config.toml` lines, defaults left out: each is a string, a number or a boolean, which
    /// JSON writes as TOML does. A secret gets a comment line saying it is set.
    fn dump(&self) -> Result<()> {
        let Value::Object(settings) = to_value(self).map_err(|e| Error::Parse(e.to_string()))?
        else {
            unreachable!("a struct serializes to an object");
        };
        for (key, value) in settings {
            if value.is_null() {
                continue;
            }
            if setting(&key)
                .expect("serialized settings are declared")
                .secret
            {
                println!("# {key} is set (a secret, not shown)");
            } else {
                println!("{key} = {value}");
            }
        }
        Ok(())
    }

    fn command_name() -> String {
        env::args_os()
            .next()
            .and_then(|path| {
                Path::new(&path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "bitviewd".to_owned())
    }

    fn print_help(command: &str) {
        let v = env!("CARGO_PKG_VERSION");

        println!("{} {}", command.bold(), v.bright_black());
        println!("Self-hosted Bitcoin data platform built on BRK");
        println!();
        println!("{}", "USAGE:".bold());
        println!(
            "    {} {command} {}",
            "[ENV]".bright_black(),
            "[OPTIONS]".bright_black()
        );
        println!();
        println!("{}", "OPTIONS:".bold());
        println!("    -h, --help                Print help");
        println!("    -V, --version             Print version");
        println!(
            "    --dumpconfig              Print the settings given, file and flags, as {} lines",
            "config.toml".bright_black()
        );
        #[cfg(feature = "full-api")]
        println!("    --checkaudits <DIR>       Check an audit store copy before it goes into <bitviewdir>/mempool/audits/import");
        print_settings();
        println!();
        println!("{}", "ENVIRONMENT:".bold());
        println!(
            "    LOG={}               Log level {}",
            "<LEVEL>".bright_black(),
            "[info]".bright_black()
        );
        println!(
            "    RUST_LOG={}          Full log filter",
            "<RULES>".bright_black()
        );
        println!();
        println!("{}", "CONFIG:".bold());
        println!(
            "    Edit {} to persist settings:",
            "~/.bitview/config.toml".bright_black()
        );
        println!("    {}", "bitviewdir = \"/path/to/data\"".bright_black());
        println!(
            "    {}",
            "bitcoindir = \"/path/to/.bitcoin\"".bright_black()
        );
    }

    fn read(path: &Path) -> io::Result<Self> {
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Config::default()),
            Err(e) => {
                return Err(io::Error::new(
                    e.kind(),
                    format!("Cannot read {}: {e}", path.display()),
                ));
            }
        };
        Self::parse_toml(&contents).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid config {}: {e}", path.display()),
            )
        })
    }

    /// Node connection settings, with `~` expanded.
    fn connect_args(&self) -> ConnectArgs {
        let path = |value: &Option<String>| value.as_deref().map(expand_home);
        ConnectArgs {
            bitcoindir: path(&self.bitcoindir),
            blocksdir: path(&self.blocksdir),
            rpcconnect: self.rpcconnect.clone(),
            rpcport: self.rpcport,
            rpccookiefile: path(&self.rpccookiefile),
            rpcuser: self.rpcuser.clone(),
            rpcpassword: self.rpcpassword.clone(),
        }
    }

    /// Resolve everything a runner needs: the blocks directory must exist and
    /// [`ConnectArgs::client`] must find RPC credentials.
    fn runner(&self) -> Result<RunConfig> {
        let connect_args = self.connect_args();
        let client = connect_args.client()?;
        let reader = Reader::connect(&connect_args, &client)?;
        let data_path = match self.bitviewdir.as_deref() {
            // Absolute, once: every reader of the path then spells it the same way (the stores
            // keep theirs absolute, and the disk usage leaves their directory out by path).
            Some(dir) => std::path::absolute(expand_home(dir))?,
            None => default_bitview_dir()
                .map(|dir| dir.join(reader.chain().subdir))
                .ok_or_else(|| {
                    Error::NotFound(
                        "no home directory to put ~/.bitview in: set --bitviewdir".into(),
                    )
                })?,
        };
        Ok(RunConfig {
            reader,
            server: ServerConfig {
                bind: self.serverbind.unwrap_or(DEFAULT_BIND),
                port: self.serverport.unwrap_or_default(),
                data_path,
                cdn_cache_mode: if self.cdn.unwrap_or(false) {
                    CdnCacheMode::Aggressive
                } else {
                    CdnCacheMode::Live
                },
                max_weight: self.maxweight.unwrap_or(DEFAULT_MAX_WEIGHT),
                max_utxos: self.maxutxos.unwrap_or(DEFAULT_MAX_UTXOS),
                public_url: self.publicurl.clone(),
                site_url: self.siteurl.clone(),
            },
            cache_budget: self.cachebudget.unwrap_or(DEFAULT_CACHE_BUDGET),
        })
    }
}

/// The text of a `config.toml` value, as its setting's parsing reads it: a bare word, an integer
/// through `toml_integer`, or a string's contents, in `'…'` as written, in `"…"` with the escapes
/// `--dumpconfig` writes. Only a comment may follow it.
fn toml_value(text: &str) -> StdResult<String, String> {
    let (value, rest) = if let Some(text) = text.strip_prefix('"') {
        let mut value = String::new();
        let mut chars = text.char_indices();
        loop {
            match chars.next().ok_or("unterminated string")? {
                (end, '"') => break (value, &text[end + 1..]),
                (_, '\\') => value.push(match chars.next().ok_or("unterminated string")?.1 {
                    c @ ('"' | '\\') => c,
                    'b' => '\u{8}',
                    'f' => '\u{c}',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    other => return Err(format!("invalid escape `\\{other}`")),
                }),
                (_, c) => value.push(c),
            }
        }
    } else if let Some(text) = text.strip_prefix('\'') {
        let (value, rest) = text.split_once('\'').ok_or("unterminated string")?;
        (value.to_owned(), rest)
    } else {
        let end = text
            .find(|c: char| c.is_whitespace() || c == '#')
            .unwrap_or(text.len());
        if end == 0 {
            return Err("missing value".to_owned());
        }
        (toml_integer(&text[..end])?, &text[end..])
    };
    let rest = rest.trim_start();
    if !rest.is_empty() && !rest.starts_with('#') {
        return Err(format!("unexpected `{rest}` after the value"));
    }
    Ok(value)
}

/// A bare integer in TOML's other spellings, written in plain decimal: `_` between digits, and `0x`,
/// `0o` or `0b` ahead of them, which files written for earlier versions may hold. Any other token
/// is returned as written, for its setting to read.
fn toml_integer(token: &str) -> StdResult<String, String> {
    let (radix, digits) = [("0x", 16), ("0o", 8), ("0b", 2)]
        .into_iter()
        .find_map(|(prefix, radix)| Some((radix, token.strip_prefix(prefix)?)))
        .unwrap_or((10, token.strip_prefix(['+', '-']).unwrap_or(token)));
    // Plain decimals already parse as written. Separators cannot follow a decimal leading zero.
    if radix == 10 && (!token.contains('_') || digits.starts_with('0')) {
        return Ok(token.to_owned());
    }
    if !digits
        .split('_')
        .all(|group| !group.is_empty() && group.chars().all(|c| c.is_digit(radix)))
    {
        return Ok(token.to_owned());
    }
    if radix == 10 {
        return Ok(token.replace('_', ""));
    }
    u64::from_str_radix(&digits.replace('_', ""), radix)
        .map(|n| n.to_string())
        .map_err(|error| format!("invalid integer `{token}`: {error}"))
}

fn parse_error(error: lexopt::Error) -> Error {
    Error::Parse(error.to_string())
}
