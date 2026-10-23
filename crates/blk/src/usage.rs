use std::{
    env,
    io::{IsTerminal, stdout},
    sync::OnceLock,
};

use owo_colors::OwoColorize;

use crate::fields::{self, Fields};

const SEL_W: usize = 5; // longest selector token: "tip-N"
const LABEL_W: usize = 28; // longest label across OUTPUT/OPTIONS/EXAMPLES (= example cmd "blk 800000 tx.0.vout.0.value")
const FLAG_W: usize = 15; // longest flag: "--rpccookiefile"
const PH_W: usize = LABEL_W - FLAG_W - 1; // placeholder column width so flag+ph total = LABEL_W
const NAMES_W: usize = 60;

pub fn print() {
    println!("{} - inspect a Bitcoin Core block", bold("blk"));
    println!();

    section("USAGE");
    println!(
        "    blk {} [{} ...] [OPTIONS]",
        dim("<selector>"),
        dim("<field>")
    );
    println!(
        "    {}",
        dim("no fields = full block as JSON (analog of `bitcoin-cli getblock <hash> 2`)")
    );
    println!();

    section("SELECTOR");
    sel("<n>", "single height (e.g. 800000)");
    sel("tip", "current chain tip");
    sel("tip-N", "tip minus N");
    sel("a..b", "inclusive range, endpoints can be height/tip/tip-N");
    println!();

    section("FIELDS");
    println!(
        "    {}",
        dim("dotted paths drill into nested data, omit an index for arrays")
    );
    println!();
    group("block");
    names(fields::BLOCK);
    println!();
    group_note("tx.i", "omit i for all txs");
    names(fields::TX);
    println!();
    group_note("tx.i.vin.j", "omit j for all inputs");
    names(fields::VIN);
    println!();
    group_note("tx.i.vout.j", "omit j for all outputs");
    names(fields::VOUT);
    println!();
    println!(
        "    {}",
        dim(
            "A path ending at tx, tx.i, tx.i.vin or tx.i.vout returns the whole sub-object as JSON."
        )
    );
    println!();

    section("OUTPUT");
    out("no fields", "full block JSON object, one per line (NDJSON)");
    out("1 field", "bare value, one per line");
    out("2+ fields", "JSON object, one per line (NDJSON)");
    out(
        "-p, --pretty",
        "pretty-print JSON values; strings stay bare",
    );
    out(
        "-c, --compact",
        "tab-separated values, no field names (TSV)",
    );
    println!();

    section("OPTIONS");
    opt(
        "--bitcoindir",
        "<PATH>",
        "Bitcoin directory",
        Some("[OS default]"),
    );
    opt(
        "--blocksdir",
        "<PATH>",
        "Blocks directory",
        Some("[<bitcoindir>/blocks]"),
    );
    opt("--rpcconnect", "<HOST>", "RPC host", Some("[localhost]"));
    opt("--rpcport", "<PORT>", "RPC port", Some("[8332]"));
    opt(
        "--rpccookiefile",
        "<PATH>",
        "RPC cookie file; relative paths resolve inside <bitcoindir>",
        Some("[<bitcoindir>/.cookie]"),
    );
    opt("--rpcuser", "<USERNAME>", "RPC username", None);
    opt("--rpcpassword", "<PASSWORD>", "RPC password", None);
    println!();

    section("EXAMPLES");
    ex("blk 800000", "full block as JSON");
    ex("blk 800000 hash", "bare hash");
    ex("blk 800000 height hash time", "one compact JSON line");
    ex("blk 800000 tx.0.txid", "coinbase txid");
    ex("blk 800000 tx.txid", "all txids in block (array)");
    ex("blk 800000 tx.0.vout.0.value", "coinbase output 0 sats");
    ex("blk 800000 tx.0.vout.value", "all output sats for tx 0");
    ex("blk 800000 tx.vout.value", "array of arrays (per tx)");
    ex("blk 0..2 hash tx.0.txid", "3 NDJSON lines");
    ex("blk tip tx.0", "whole coinbase tx as JSON");
}

fn section(name: &str) {
    println!("{}", bold(&format!("{name}:")));
}

fn group(name: &str) {
    println!("  {}", bold(&format!("{name}:")));
}

fn group_note(name: &str, note: &str) {
    println!(
        "  {}  {}",
        bold(&format!("{name}:")),
        dim(&format!("({note})"))
    );
}

/// The field names, comma-separated and wrapped at `NAMES_W`.
fn names<F>(fields: Fields<F>) {
    let mut line = String::new();
    for (i, (name, _)) in fields.iter().enumerate() {
        let comma = if i + 1 < fields.len() { "," } else { "" };
        if !line.is_empty() && line.len() + 1 + name.len() + comma.len() > NAMES_W {
            println!("    {line}");
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(name);
        line.push_str(comma);
    }
    println!("    {line}");
}

// Plain text is padded before coloring, so escape codes never count toward a column width.
fn sel(token: &str, desc: &str) {
    println!("    {}    {desc}", dim(&format!("{token:SEL_W$}")));
}

fn out(label: &str, desc: &str) {
    println!("    {label:LABEL_W$}    {desc}");
}

fn opt(flag: &str, ph: &str, desc: &str, default: Option<&str>) {
    let head = format!("    {flag:FLAG_W$} {}    ", dim(&format!("{ph:PH_W$}")));
    match default {
        Some(d) => println!("{head}{desc} {}", dim(d)),
        None => println!("{head}{desc}"),
    }
}

fn ex(cmd: &str, note: &str) {
    println!("    {cmd:LABEL_W$}    {}", dim(&format!("# {note}")));
}

fn bold(s: &str) -> String {
    if colors() { s.bold().to_string() } else { s.to_owned() }
}

fn dim(s: &str) -> String {
    if colors() { s.bright_black().to_string() } else { s.to_owned() }
}

/// Whether stdout takes colors, as supports-color decides: FORCE_COLOR, or else CLICOLOR_FORCE,
/// turns them on; otherwise a terminal has them, but for NO_COLOR or a dumb TERM.
fn colors() -> bool {
    static COLORS: OnceLock<bool> = OnceLock::new();
    *COLORS.get_or_init(|| {
        let var = |name| env::var(name).ok();
        let forced = match var("FORCE_COLOR").as_deref() {
            Some(value) => !matches!(value, "false" | "0"),
            None => var("CLICOLOR_FORCE").is_some_and(|value| value != "0"),
        };
        forced
            || (var("NO_COLOR").is_none_or(|value| value == "0")
                && var("TERM").as_deref() != Some("dumb")
                && stdout().is_terminal())
    })
}
