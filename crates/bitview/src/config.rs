use std::{
    collections::HashSet, env, fmt::Display, fs, io, net::IpAddr, path::Path, process::exit,
    result::Result as StdResult, str::FromStr,
};

use bitview_server::{
    CdnCacheMode, DEFAULT_BIND, DEFAULT_MAX_UTXOS, DEFAULT_MAX_WEIGHT, Port, ServerConfig, Website,
};
use brk_error::{Error, Result};
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

/// Default size, in bytes, of the shared vector cache (`vecdb::Budgeted::init_global`).
pub const DEFAULT_CACHE_BUDGET: usize = 2 << 30;

#[derive(Debug, Default, Serialize)]
pub struct Config {
    bitviewdir: Option<String>,
    serverbind: Option<IpAddr>,
    serverport: Option<Port>,
    website: Option<Website>,
    cdn: Option<bool>,
    maxweight: Option<usize>,
    maxutxos: Option<usize>,
    cachebudget: Option<usize>,
    bitcoindir: Option<String>,
    blocksdir: Option<String>,
    rpcconnect: Option<String>,
    rpcport: Option<u16>,
    rpccookiefile: Option<String>,
    rpcuser: Option<String>,
    rpcpassword: Option<String>,
}

impl Config {
    /// The settings `config.toml` and the flags name, in the order they are written.
    const KEYS: [&'static str; 15] = [
        "bitviewdir",
        "serverbind",
        "serverport",
        "website",
        "cdn",
        "maxweight",
        "maxutxos",
        "cachebudget",
        "bitcoindir",
        "blocksdir",
        "rpcconnect",
        "rpcport",
        "rpccookiefile",
        "rpcuser",
        "rpcpassword",
    ];

    pub fn import() -> Result<RunConfig> {
        let (config_args, dump) = Self::parse_args()?;

        let config =
            Self::read(&default_bitview_dir().join("config.toml"))?.with_overrides(config_args);
        if dump {
            config.dump()?;
            exit(0);
        }
        let runner = config.runner()?;
        fs::create_dir_all(&runner.server.data_path)?;

        Ok(runner)
    }

    fn with_overrides(self, overrides: Self) -> Self {
        Self {
            bitviewdir: overrides.bitviewdir.or(self.bitviewdir),
            serverbind: overrides.serverbind.or(self.serverbind),
            serverport: overrides.serverport.or(self.serverport),
            website: overrides.website.or(self.website),
            cdn: overrides.cdn.or(self.cdn),
            maxweight: overrides.maxweight.or(self.maxweight),
            maxutxos: overrides.maxutxos.or(self.maxutxos),
            cachebudget: overrides.cachebudget.or(self.cachebudget),
            bitcoindir: overrides.bitcoindir.or(self.bitcoindir),
            blocksdir: overrides.blocksdir.or(self.blocksdir),
            rpcconnect: overrides.rpcconnect.or(self.rpcconnect),
            rpcport: overrides.rpcport.or(self.rpcport),
            rpccookiefile: overrides.rpccookiefile.or(self.rpccookiefile),
            rpcuser: overrides.rpcuser.or(self.rpcuser),
            rpcpassword: overrides.rpcpassword.or(self.rpcpassword),
        }
    }

    /// The flags' settings, and whether `--dumpconfig` asks for the settings to be printed.
    fn parse_args() -> Result<(Self, bool)> {
        let mut config = Self::default();
        let mut dump = false;
        let mut parser = Parser::from_env();
        let command = Self::command_name();

        while let Some(arg) = parser.next().map_err(parse_error)? {
            let flag = match &arg {
                Long(name) => format!("--{name}"),
                _ => String::new(),
            };
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
                Long(name) if Self::KEYS.contains(&name) => {
                    let key = name.to_owned();
                    let value = parser.value().map_err(parse_error)?;
                    let value = value
                        .string()
                        .map_err(|error| Error::Parse(format!("{flag}: {error}")))?;
                    // Worded as lexopt words a value its parsing rejects.
                    config.set(&key, &value).map_err(|error| {
                        Error::Parse(format!("{flag}: cannot parse argument {value:?}: {error}"))
                    })?;
                }
                _ => return Err(parse_error(arg.unexpected())),
            }
        }

        Ok((config, dump))
    }

    /// Set KEY, one of KEYS, from its text, as its flag takes it.
    fn set(&mut self, key: &str, value: &str) -> StdResult<(), String> {
        fn parse<T: FromStr>(value: &str) -> StdResult<Option<T>, String>
        where
            T::Err: Display,
        {
            value
                .parse()
                .map(Some)
                .map_err(|error: T::Err| error.to_string())
        }
        match key {
            "bitviewdir" => self.bitviewdir = parse(value)?,
            "serverbind" => self.serverbind = parse(value)?,
            "serverport" => self.serverport = parse(value)?,
            "website" => self.website = parse(value)?,
            "cdn" => self.cdn = parse(value)?,
            "maxweight" => self.maxweight = parse(value)?,
            "maxutxos" => self.maxutxos = parse(value)?,
            "cachebudget" => self.cachebudget = parse(value)?,
            "bitcoindir" => self.bitcoindir = parse(value)?,
            "blocksdir" => self.blocksdir = parse(value)?,
            "rpcconnect" => self.rpcconnect = parse(value)?,
            "rpcport" => self.rpcport = parse(value)?,
            "rpccookiefile" => self.rpccookiefile = parse(value)?,
            "rpcuser" => self.rpcuser = parse(value)?,
            "rpcpassword" => self.rpcpassword = parse(value)?,
            _ => unreachable!("{key} is not one of Config::KEYS"),
        }
        Ok(())
    }

    /// Read `config.toml`: a `key = value` line per setting, its value a TOML string,
    /// integer or boolean, read then as its flag would be, with blank lines and `#` comments
    /// around them.
    fn parse_toml(contents: &str) -> StdResult<Self, String> {
        let mut config = Self::default();
        let mut seen = HashSet::new();
        let contents = contents.strip_prefix('\u{feff}').unwrap_or(contents);
        for (index, line) in contents.lines().enumerate() {
            let at = |problem: String| format!("line {}: {problem}", index + 1);
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| at(format!("expected `key = value`, found `{line}`")))?;
            let key = key.trim();
            let key = key
                .strip_prefix('"')
                .and_then(|key| key.strip_suffix('"'))
                .or_else(|| {
                    key.strip_prefix('\'')
                        .and_then(|key| key.strip_suffix('\''))
                })
                .unwrap_or(key);
            if !Self::KEYS.contains(&key) {
                return Err(at(format!(
                    "unknown setting `{key}`, expected one of {}",
                    Self::KEYS.join(", ")
                )));
            }
            if !seen.insert(key) {
                return Err(at(format!("`{key}` is set twice")));
            }
            let text =
                toml_scalar(value.trim()).map_err(|problem| at(format!("{key}: {problem}")))?;
            config
                .set(key, &text)
                .map_err(|error| at(format!("{key}: cannot parse {text:?}: {error}")))?;
        }
        Ok(config)
    }

    /// Print the settings given, in `config.toml` and as flags, the flags first, as
    /// `config.toml` lines, defaults left out: each is a string, a number or a boolean, which
    /// JSON writes as TOML does.
    fn dump(&self) -> Result<()> {
        let Value::Object(settings) = to_value(self).map_err(|e| Error::Parse(e.to_string()))?
        else {
            unreachable!("a struct serializes to an object");
        };
        for (key, value) in settings {
            if !value.is_null() {
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
        println!();
        println!(
            "    --bitviewdir {}       Output directory {}",
            "<PATH>".bright_black(),
            "[~/.bitview]".bright_black()
        );
        println!(
            "    --serverbind {}       Server bind address {}",
            "<IP>".bright_black(),
            format!("[{DEFAULT_BIND}]").bright_black()
        );
        println!(
            "    --serverport {}       Server port {}",
            "<PORT>".bright_black(),
            format!("[{}]", Port::DEFAULT).bright_black()
        );
        println!(
            "    --website {}     Serve the showcases (true), nothing (false) or a directory {}",
            "<BOOL|PATH>".bright_black(),
            "[true]".bright_black()
        );
        println!(
            "    --cdn {}              Aggressive CDN cache, requires purge on deploy {}",
            "<BOOL>".bright_black(),
            "[false]".bright_black()
        );
        println!(
            "    --maxweight {}       Server cap on series response weight in bytes; rejects /api/series/... over the limit {}",
            "<BYTES>".bright_black(),
            format!("[{}]", DEFAULT_MAX_WEIGHT).bright_black()
        );
        println!(
            "    --maxutxos {}        Server cap on UTXOs per address; /api/address/{{addr}}/utxo errors past the limit {}",
            "<COUNT>".bright_black(),
            format!("[{}]", DEFAULT_MAX_UTXOS).bright_black()
        );
        println!(
            "    --cachebudget {}     Shared vector cache size in bytes {}",
            "<BYTES>".bright_black(),
            format!("[{DEFAULT_CACHE_BUDGET}]").bright_black()
        );
        println!();
        println!(
            "    --bitcoindir {}       Bitcoin directory {}",
            "<PATH>".bright_black(),
            "[OS default]".bright_black()
        );
        println!(
            "    --blocksdir {}        Blocks directory {}",
            "<PATH>".bright_black(),
            "[<bitcoindir>/blocks]".bright_black()
        );
        println!();
        println!(
            "    --rpcconnect {}       RPC host {}",
            "<HOST>".bright_black(),
            "[localhost]".bright_black()
        );
        println!(
            "    --rpcport {}          RPC port {}",
            "<PORT>".bright_black(),
            "[8332]".bright_black()
        );
        println!(
            "    --rpccookiefile {}    RPC cookie file; relative paths resolve inside <bitcoindir> {}",
            "<PATH>".bright_black(),
            "[<bitcoindir>/.cookie]".bright_black()
        );
        println!(
            "    --rpcuser {}      RPC username",
            "<USERNAME>".bright_black()
        );
        println!(
            "    --rpcpassword {}  RPC password",
            "<PASSWORD>".bright_black()
        );
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
        let blocks_path = connect_args.blocks_dir();
        if !blocks_path.is_dir() {
            return Err(Error::NotFound(format!(
                "{} isn't a valid blocks directory; set --blocksdir or --bitcoindir (run with -h for help)",
                blocks_path.display()
            )));
        }
        Ok(RunConfig {
            client: connect_args.client()?,
            blocks_path,
            server: ServerConfig {
                bind: self.serverbind.unwrap_or(DEFAULT_BIND),
                port: self.serverport.unwrap_or_default(),
                data_path: self
                    .bitviewdir
                    .as_deref()
                    .map_or_else(default_bitview_dir, expand_home),
                website: self.website.clone().unwrap_or_default(),
                cdn_cache_mode: if self.cdn.unwrap_or(false) {
                    CdnCacheMode::Aggressive
                } else {
                    CdnCacheMode::Live
                },
                max_weight: self.maxweight.unwrap_or(DEFAULT_MAX_WEIGHT),
                max_utxos: self.maxutxos.unwrap_or(DEFAULT_MAX_UTXOS),
            },
            cache_budget: self.cachebudget.unwrap_or(DEFAULT_CACHE_BUDGET),
        })
    }
}

/// The text of a TOML value, as its setting's parsing reads it: a string's contents, basic
/// or literal, triple-quoted too when it closes on its line, or a boolean or integer as
/// written, an integer without the `+` sign, `_` separators and `0x`, `0o` or `0b` prefixes
/// TOML allows. Only a comment may follow it.
fn toml_scalar(text: &str) -> StdResult<String, String> {
    let (value, rest) = if let Some(text) = text.strip_prefix("\"\"\"") {
        let end = text.find("\"\"\"").ok_or(SPANNING)?;
        (unescape(&text[..end])?, &text[end + 3..])
    } else if let Some(text) = text.strip_prefix("'''") {
        let end = text.find("'''").ok_or(SPANNING)?;
        (text[..end].to_owned(), &text[end + 3..])
    } else if let Some(text) = text.strip_prefix('"') {
        // The closing quote is the first one no backslash escapes.
        let mut escaped = false;
        let end = text
            .char_indices()
            .find(|&(_, c)| {
                let closes = c == '"' && !escaped;
                escaped = c == '\\' && !escaped;
                closes
            })
            .ok_or("unterminated string")?
            .0;
        (unescape(&text[..end])?, &text[end + 1..])
    } else if let Some(text) = text.strip_prefix('\'') {
        let end = text.find('\'').ok_or("unterminated string")?;
        (text[..end].to_owned(), &text[end + 1..])
    } else {
        let end = text
            .find(|c: char| c.is_whitespace() || c == '#')
            .unwrap_or(text.len());
        if end == 0 {
            return Err("missing value".to_owned());
        }
        (bare_value(&text[..end])?, &text[end..])
    };
    let rest = rest.trim_start();
    if !rest.is_empty() && !rest.starts_with('#') {
        return Err(format!("unexpected `{rest}` after the value"));
    }
    Ok(value)
}

const SPANNING: &str = "strings spanning lines are not supported";

/// A basic string's contents, its escapes resolved.
fn unescape(text: &str) -> StdResult<String, String> {
    let mut value = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            value.push(c);
            continue;
        }
        let escape = chars.next().ok_or("unterminated string")?;
        match escape {
            'b' => value.push('\u{8}'),
            't' => value.push('\t'),
            'n' => value.push('\n'),
            'f' => value.push('\u{c}'),
            'r' => value.push('\r'),
            'e' => value.push('\u{1b}'),
            '"' => value.push('"'),
            '\\' => value.push('\\'),
            'x' | 'u' | 'U' => {
                let digits = match escape {
                    'x' => 2,
                    'u' => 4,
                    _ => 8,
                };
                let hex: String = chars.by_ref().take(digits).collect();
                let c = u32::from_str_radix(&hex, 16)
                    .ok()
                    .filter(|_| hex.len() == digits)
                    .and_then(char::from_u32)
                    .ok_or_else(|| format!("invalid escape `\\{escape}{hex}`"))?;
                value.push(c);
            }
            other => return Err(format!("invalid escape `\\{other}`")),
        }
    }
    Ok(value)
}

/// A boolean as written; an integer in decimal digits, as its setting's parsing reads it.
fn bare_value(token: &str) -> StdResult<String, String> {
    if !token.starts_with(|c: char| c.is_ascii_digit() || c == '+' || c == '-') {
        return Ok(token.to_owned());
    }
    let number = token.strip_prefix('+').unwrap_or(token).replace('_', "");
    for (prefix, radix) in [("0x", 16), ("0o", 8), ("0b", 2)] {
        if let Some(digits) = number.strip_prefix(prefix) {
            return u64::from_str_radix(digits, radix)
                .map(|n| n.to_string())
                .map_err(|error| format!("invalid integer `{token}`: {error}"));
        }
    }
    Ok(number)
}

fn parse_error(error: lexopt::Error) -> Error {
    Error::Parse(error.to_string())
}
