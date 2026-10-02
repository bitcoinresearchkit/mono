use toml::{Table, from_str};

use super::*;

#[test]
fn only_present_command_line_fields_override_persisted_settings() {
    let source = r#"
        bitviewdir = "/fixture/bitview"
        serverbind = "127.0.0.1"
        serverport = 3111
        website = true
        cdn = true
        maxweight = 123
        maxutxos = 456
        bitcoindir = "/fixture/bitcoin"
        blocksdir = "/fixture/blocks"
        rpcconnect = "localhost"
        rpcport = 8333
        rpccookiefile = "/fixture/cookie"
        rpcuser = "fixture-user"
        rpcpassword = "fixture-password"
    "#;
    let expected: Config = from_str(source).unwrap();
    let from_default = Config::default().with_overrides(from_str(source).unwrap());
    assert_eq!(from_default, expected);
    assert_eq!(from_default.with_overrides(Config::default()), expected);
    let mut values: Table = from_str(source).unwrap();
    values.insert("website".into(), false.into());
    values.insert("cdn".into(), false.into());
    values.insert("maxweight".into(), 0.into());
    values.insert("maxutxos".into(), 0.into());
    let merged = expected.with_overrides(
        from_str("website = false\ncdn = false\nmaxweight = 0\nmaxutxos = 0").unwrap(),
    );
    assert_eq!(merged, values.try_into::<Config>().unwrap());
    let resolved = merged.server_config();
    assert_eq!(resolved.website, Website::Disabled);
    assert_eq!(resolved.cdn_cache_mode, CdnCacheMode::Live);
    assert_eq!(resolved.max_weight, 0);
    assert_eq!(resolved.max_utxos, 0);
}
