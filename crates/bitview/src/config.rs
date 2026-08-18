use std::{
    fs, io,
    path::{Path, PathBuf},
};

use brk_error::{Error, Result};
use brk_rpc::{Auth, Client};
use brk_server::{CdnCacheMode, DEFAULT_MAX_UTXOS, DEFAULT_MAX_WEIGHT, Website};
use brk_types::Port;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::paths::{default_brk_path, dot_brk_path, fix_user_path};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    #[serde(default)]
    brkdir: Option<String>,

    #[serde(default)]
    brkport: Option<Port>,

    #[serde(default)]
    website: Option<Website>,

    #[serde(default)]
    cdn: Option<bool>,

    #[serde(default)]
    maxweight: Option<usize>,

    #[serde(default)]
    maxutxos: Option<usize>,

    #[serde(default)]
    bitcoindir: Option<String>,

    #[serde(default)]
    blocksdir: Option<String>,

    #[serde(default)]
    rpcconnect: Option<String>,

    #[serde(default)]
    rpcport: Option<u16>,

    #[serde(default)]
    rpccookiefile: Option<String>,

    #[serde(default)]
    rpcuser: Option<String>,

    #[serde(default)]
    rpcpassword: Option<String>,
}

impl Config {
    pub fn import() -> Result<Self> {
        let config_args = Self::parse_args();

        let path = dot_brk_path();

        let _ = fs::create_dir_all(&path);

        let path = path.join("config.toml");

        let mut config = Self::read(&path);

        if let Some(v) = config_args.brkdir {
            config.brkdir = Some(v);
        }
        if let Some(v) = config_args.brkport {
            config.brkport = Some(v);
        }
        if let Some(v) = config_args.website {
            config.website = Some(v);
        }
        if let Some(v) = config_args.cdn {
            config.cdn = Some(v);
        }
        if let Some(v) = config_args.maxweight {
            config.maxweight = Some(v);
        }
        if let Some(v) = config_args.maxutxos {
            config.maxutxos = Some(v);
        }
        if let Some(v) = config_args.bitcoindir {
            config.bitcoindir = Some(v);
        }
        if let Some(v) = config_args.blocksdir {
            config.blocksdir = Some(v);
        }
        if let Some(v) = config_args.rpcconnect {
            config.rpcconnect = Some(v);
        }
        if let Some(v) = config_args.rpcport {
            config.rpcport = Some(v);
        }
        if let Some(v) = config_args.rpccookiefile {
            config.rpccookiefile = Some(v);
        }
        if let Some(v) = config_args.rpcuser {
            config.rpcuser = Some(v);
        }
        if let Some(v) = config_args.rpcpassword {
            config.rpcpassword = Some(v);
        }

        config.check();

        Ok(config)
    }

    fn parse_args() -> Self {
        use lexopt::prelude::*;

        let mut config = Self::default();
        let mut parser = lexopt::Parser::from_env();

        while let Some(arg) = parser.next().unwrap() {
            match arg {
                Short('h') | Long("help") => {
                    Self::print_help();
                    std::process::exit(0);
                }
                Short('V') | Long("version") => {
                    println!("brk {}", env!("CARGO_PKG_VERSION"));
                    std::process::exit(0);
                }
                Long("brkdir") => config.brkdir = Some(parser.value().unwrap().parse().unwrap()),
                Long("brkport") => config.brkport = Some(parser.value().unwrap().parse().unwrap()),
                Long("website") => config.website = Some(parser.value().unwrap().parse().unwrap()),
                Long("cdn") => config.cdn = Some(parser.value().unwrap().parse().unwrap()),
                Long("maxweight") => {
                    config.maxweight = Some(parser.value().unwrap().parse().unwrap())
                }
                Long("maxutxos") => {
                    config.maxutxos = Some(parser.value().unwrap().parse().unwrap())
                }
                Long("bitcoindir") => {
                    config.bitcoindir = Some(parser.value().unwrap().parse().unwrap())
                }
                Long("blocksdir") => {
                    config.blocksdir = Some(parser.value().unwrap().parse().unwrap())
                }
                Long("rpcconnect") => {
                    config.rpcconnect = Some(parser.value().unwrap().parse().unwrap())
                }
                Long("rpcport") => config.rpcport = Some(parser.value().unwrap().parse().unwrap()),
                Long("rpccookiefile") => {
                    config.rpccookiefile = Some(parser.value().unwrap().parse().unwrap())
                }
                Long("rpcuser") => config.rpcuser = Some(parser.value().unwrap().parse().unwrap()),
                Long("rpcpassword") => {
                    config.rpcpassword = Some(parser.value().unwrap().parse().unwrap())
                }
                _ => {
                    eprintln!("{}", arg.unexpected());
                    std::process::exit(1);
                }
            }
        }

        config
    }

    fn print_help() {
        let v = env!("CARGO_PKG_VERSION");

        println!("{} {}", "brk".bold(), v.bright_black());
        println!("Bitcoin Research Kit");
        println!();
        println!("{}", "USAGE:".bold());
        println!(
            "    {} brk {}",
            "[ENV]".bright_black(),
            "[OPTIONS]".bright_black()
        );
        println!();
        println!("{}", "OPTIONS:".bold());
        println!("    -h, --help                Print help");
        println!("    -V, --version             Print version");
        println!();
        println!(
            "    --brkdir {}           Output directory {}",
            "<PATH>".bright_black(),
            "[~/.brk]".bright_black()
        );
        println!(
            "    --brkport {}          Server port {}",
            "<PORT>".bright_black(),
            "[3110]".bright_black()
        );
        println!(
            "    --website {}     Website {}",
            "<BOOL|PATH>".bright_black(),
            "[true]".bright_black()
        );
        println!(
            "    --cdn {}              Aggressive CDN cache, requires purge on deploy {}",
            "<BOOL>".bright_black(),
            "[false]".bright_black()
        );
        println!(
            "    --maxweight {}       Server cap on series response weight in bytes; rejects /api/{{series,metric}}/... over the limit {}",
            "<BYTES>".bright_black(),
            format!("[{}]", DEFAULT_MAX_WEIGHT).bright_black()
        );
        println!(
            "    --maxutxos {}        Server cap on UTXOs per address; /api/address/{{addr}}/utxo errors past the limit {}",
            "<COUNT>".bright_black(),
            format!("[{}]", DEFAULT_MAX_UTXOS).bright_black()
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
            "    --rpcconnect {}         RPC host {}",
            "<IP>".bright_black(),
            "[localhost]".bright_black()
        );
        println!(
            "    --rpcport {}          RPC port {}",
            "<PORT>".bright_black(),
            "[8332]".bright_black()
        );
        println!(
            "    --rpccookiefile {}    RPC cookie file {}",
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
            "~/.brk/config.toml".bright_black()
        );
        println!("    {}", "brkdir = \"/path/to/data\"".bright_black());
        println!(
            "    {}",
            "bitcoindir = \"/path/to/.bitcoin\"".bright_black()
        );
    }

    fn check(&self) {
        if !self.bitcoindir().is_dir() {
            println!("{:?} isn't a valid directory", self.bitcoindir());
            println!("Please use the --bitcoindir parameter to set a valid path.");
            println!("Run the program with '-h' for help.");
            std::process::exit(1);
        }

        if !self.blocksdir().is_dir() {
            println!("{:?} isn't a valid directory", self.blocksdir());
            println!("Please use the --blocksdir parameter to set a valid path.");
            println!("Run the program with '-h' for help.");
            std::process::exit(1);
        }

        if !self.brkdir().is_dir() {
            println!("{:?} isn't a valid directory", self.brkdir());
            println!("Please use the --brkdir parameter to set a valid path.");
            println!("Run the program with '-h' for help.");
            std::process::exit(1);
        }

        if self.rpc_auth().is_err() {
            println!(
                "Unsuccessful authentication with the RPC client.
First make sure that `bitcoind` is running. If it is then please either set --rpccookiefile or --rpcuser and --rpcpassword as the default values seemed to have failed.
Finally, you can run the program with '-h' for help."
            );
            std::process::exit(1);
        }
    }

    fn read(path: &Path) -> Self {
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Config::default(),
            Err(e) => {
                eprintln!("Cannot read {}: {e}", path.display());
                std::process::exit(1);
            }
        };
        toml::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Invalid {}:\n{e}", path.display());
            std::process::exit(1);
        })
    }

    pub fn rpc(&self) -> Result<Client> {
        Client::new(
            &format!(
                "http://{}:{}",
                self.rpcconnect().unwrap_or(&"localhost".to_string()),
                self.rpcport().unwrap_or(8332)
            ),
            self.rpc_auth()?,
        )
    }

    fn rpc_auth(&self) -> Result<Auth> {
        let cookie = self.path_cookiefile();

        if cookie.is_file() {
            Ok(Auth::CookieFile(cookie))
        } else if self.rpcuser.is_some() && self.rpcpassword.is_some() {
            Ok(Auth::UserPass(
                self.rpcuser.clone().unwrap(),
                self.rpcpassword.clone().unwrap(),
            ))
        } else {
            Err(Error::AuthFailed)
        }
    }

    fn rpcconnect(&self) -> Option<&String> {
        self.rpcconnect.as_ref()
    }

    fn rpcport(&self) -> Option<u16> {
        self.rpcport
    }

    pub fn bitcoindir(&self) -> PathBuf {
        self.bitcoindir
            .as_ref()
            .map_or_else(Client::default_bitcoin_path, |s| fix_user_path(s.as_ref()))
    }

    pub fn blocksdir(&self) -> PathBuf {
        self.blocksdir.as_ref().map_or_else(
            || self.bitcoindir().join("blocks"),
            |blocksdir| fix_user_path(blocksdir.as_str()),
        )
    }

    pub fn brkdir(&self) -> PathBuf {
        self.brkdir
            .as_ref()
            .map_or_else(default_brk_path, |s| fix_user_path(s.as_ref()))
    }

    fn path_cookiefile(&self) -> PathBuf {
        self.rpccookiefile.as_ref().map_or_else(
            || self.bitcoindir().join(".cookie"),
            |p| fix_user_path(p.as_str()),
        )
    }

    pub fn website(&self) -> Website {
        self.website.clone().unwrap_or_default()
    }

    pub fn cdn_cache_mode(&self) -> CdnCacheMode {
        if self.cdn.unwrap_or(false) {
            CdnCacheMode::Aggressive
        } else {
            CdnCacheMode::Live
        }
    }

    pub fn max_weight(&self) -> usize {
        self.maxweight.unwrap_or(DEFAULT_MAX_WEIGHT)
    }

    pub fn max_utxos(&self) -> usize {
        self.maxutxos.unwrap_or(DEFAULT_MAX_UTXOS)
    }

    pub fn brkport(&self) -> Option<Port> {
        self.brkport
    }
}
