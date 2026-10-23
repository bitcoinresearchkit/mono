use std::{env, fs, io, net::IpAddr, path::Path, process::exit, str::FromStr};

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
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use toml::from_str;

use crate::{
    RunConfig,
    paths::{default_bitview_dir, expand_home},
};

/// Default size, in bytes, of the shared vector cache (`vecdb::Budgeted::init_global`).
pub const DEFAULT_CACHE_BUDGET: usize = 2 << 30;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
                Long("bitviewdir") => config.bitviewdir = Some(parse_value(&mut parser, &flag)?),
                Long("serverbind") => config.serverbind = Some(parse_value(&mut parser, &flag)?),
                Long("serverport") => config.serverport = Some(parse_value(&mut parser, &flag)?),
                Long("website") => config.website = Some(parse_value(&mut parser, &flag)?),
                Long("cdn") => config.cdn = Some(parse_value(&mut parser, &flag)?),
                Long("maxweight") => config.maxweight = Some(parse_value(&mut parser, &flag)?),
                Long("maxutxos") => config.maxutxos = Some(parse_value(&mut parser, &flag)?),
                Long("cachebudget") => config.cachebudget = Some(parse_value(&mut parser, &flag)?),
                Long("bitcoindir") => config.bitcoindir = Some(parse_value(&mut parser, &flag)?),
                Long("blocksdir") => config.blocksdir = Some(parse_value(&mut parser, &flag)?),
                Long("rpcconnect") => config.rpcconnect = Some(parse_value(&mut parser, &flag)?),
                Long("rpcport") => config.rpcport = Some(parse_value(&mut parser, &flag)?),
                Long("rpccookiefile") => {
                    config.rpccookiefile = Some(parse_value(&mut parser, &flag)?)
                }
                Long("rpcuser") => config.rpcuser = Some(parse_value(&mut parser, &flag)?),
                Long("rpcpassword") => config.rpcpassword = Some(parse_value(&mut parser, &flag)?),
                _ => return Err(parse_error(arg.unexpected())),
            }
        }

        Ok((config, dump))
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
        from_str(&contents).map_err(|e| {
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

fn parse_value<T>(parser: &mut Parser, flag: &str) -> Result<T>
where
    T: FromStr,
    T::Err: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    parser
        .value()
        .map_err(parse_error)?
        .parse()
        .map_err(|error| Error::Parse(format!("{flag}: {error}")))
}

fn parse_error(error: lexopt::Error) -> Error {
    Error::Parse(error.to_string())
}
