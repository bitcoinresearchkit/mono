#![cfg(not(any(
    feature = "bedrock",
    feature = "blocks",
    feature = "coinflow",
    feature = "cointime",
    feature = "distribution_age",
    feature = "distribution_addresses",
    feature = "inputs",
    feature = "mappings",
    feature = "mining",
    feature = "outputs",
    feature = "pools",
    feature = "price",
    feature = "transactions",
)))]

use std::{
    env, fs,
    net::Ipv4Addr,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};

use bitcoin::{
    Block, Network, ScriptBuf, blockdata::constants::genesis_block, consensus::serialize,
};
use bitview::{
    ComputePluginSet, Config, DEFAULT_CACHE_BUDGET, ImportContext, PluginSet, UpdateContext, run,
};
use bitview_plugin::{ComputePlugin, Publication};
use bitview_plugin_indexer::{HasIndexer, Indexer};
use bitview_server::ServerConfig;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_rpc::{Auth, Client};
use serde_json::{Value, from_slice, json};
use tempfile::tempdir;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    runtime::Builder,
    time as TokioTime,
};
use vecdb::{Rw, StorageMode};

static ACTIVE: AtomicUsize = AtomicUsize::new(1);
static COMMITS: AtomicUsize = AtomicUsize::new(0);
const CHILD_DIRECTORY: &str = "BITVIEW_REORG_TEST_DIRECTORY";

#[derive(PluginSet, Traversable)]
struct Plugins<M: StorageMode = Rw> {
    indexer: Indexer<M>,
}

impl<M: StorageMode> HasIndexer<M> for Plugins<M> {
    fn indexer(&self) -> &Indexer<M> {
        &self.indexer
    }
}

impl ComputePluginSet for Plugins {
    fn publication(&self) -> &Publication {
        self.indexer.publication()
    }

    fn compute(&mut self, context: UpdateContext<'_>) -> Result<()> {
        self.indexer.compute((), context)
    }

    fn commit(&mut self) -> Result<()> {
        self.indexer.commit()?;
        let expected = chain()[ACTIVE.load(Ordering::SeqCst)].block_hash();
        assert_eq!(
            self.indexer.tip_blockhash().to_string(),
            expected.to_string()
        );
        if COMMITS.fetch_add(1, Ordering::SeqCst) == 1 {
            // Change the node again during publication. The next poll must
            // compare with the committed second branch, not acknowledge this
            // third branch before it has actually been indexed.
            ACTIVE.store(3, Ordering::SeqCst);
        }
        Ok(())
    }
}

fn chain() -> [Block; 4] {
    let genesis = genesis_block(Network::Bitcoin);
    let fork = |branch: u8| {
        let mut block = genesis.clone();
        block.header.prev_blockhash = genesis.block_hash();
        block.header.time += 600 * u32::from(branch);
        block.txdata[0].input[0].script_sig = ScriptBuf::from_bytes(vec![1, branch]);
        block.header.merkle_root = block.compute_merkle_root().unwrap();
        block
    };
    // Synthetic forks: the reader trusts the RPC node's chain selection.
    [genesis.clone(), fork(1), fork(2), fork(3)]
}

fn reply(request: &Value, blocks: &[Block; 4]) -> Value {
    if let Some(batch) = request.as_array() {
        return batch.iter().map(|request| reply(request, blocks)).collect();
    }
    let active = ACTIVE.load(Ordering::SeqCst);
    let method = request["method"].as_str().unwrap();
    let result = match method {
        "getbestblockhash" if COMMITS.load(Ordering::SeqCst) >= 3 => {
            return json!({"id": request["id"], "result": null,
                "error": {"code": -1, "message": "reorg fixture complete"}});
        }
        "getbestblockhash" => json!(blocks[active].block_hash().to_string()),
        "getblockcount" => json!(1),
        "getblockhash" => json!(
            blocks[if request["params"][0] == 0 { 0 } else { active }]
                .block_hash()
                .to_string()
        ),
        "getblockchaininfo" => json!({"chain": "main", "blocks": 1, "headers": 1,
            "bestblockhash": blocks[active].block_hash().to_string(), "difficulty": 1,
            "time": blocks[active].header.time, "mediantime": blocks[active].header.time,
            "verificationprogress": 1, "initialblockdownload": false,
            "chainwork": "01", "size_on_disk": 1000, "pruned": false, "warnings": []}),
        "getblockheader" | "getblock" => {
            let position = blocks
                .iter()
                .position(|block| request["params"][0] == block.block_hash().to_string())
                .unwrap();
            let block = &blocks[position];
            json!({"hash": block.block_hash().to_string(),
                "confirmations": if position == 0 || position == active { 1 } else { -1 },
                "height": usize::from(position != 0),
                "previousblockhash": (position != 0).then(|| block.header.prev_blockhash.to_string()),
                "version": 1, "versionHex": "00000001", "merkleroot": block.header.merkle_root.to_string(),
                "time": block.header.time, "mediantime": block.header.time,
                "nonce": block.header.nonce, "bits": "1d00ffff", "difficulty": 1,
                "chainwork": "0000000000000000000000000000000000000000000000000000000100010001",
                "nTx": block.txdata.len(), "size": serialize(block).len(), "weight": block.weight().to_wu(),
                "tx": block.txdata.iter().map(|tx| tx.compute_txid().to_string()).collect::<Vec<_>>()})
        }
        // Mempool indexing is unrelated to this chain-polling scenario.
        _ => {
            return json!({"id": request["id"], "result": null,
            "error": {"code": -32601, "message": "method not provided by chain fixture"}});
        }
    };
    json!({"id": request["id"], "result": result, "error": null})
}

#[test]
fn runner_publishes_same_height_reorgs() {
    let Ok(directory) = env::var(CHILD_DIRECTORY) else {
        // The production runner owns a process-lifetime mempool thread. Keep
        // it isolated, and bound failure time if height-only polling returns.
        let directory = tempdir().unwrap();
        let mut child = Command::new(env::current_exe().unwrap())
            .args([
                "--exact",
                "runner_publishes_same_height_reorgs",
                "--nocapture",
            ])
            .env(CHILD_DIRECTORY, directory.path())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "runner regression child failed: {status}");
                return;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("runner did not publish the same-height replacements within 30s");
            }
            thread::sleep(Duration::from_millis(100));
        }
    };

    let directory = PathBuf::from(directory);
    let blocks_path = directory.join("blocks");
    fs::create_dir(&blocks_path).unwrap();
    let blocks = chain();
    let mut record = Vec::new();
    for block in &blocks {
        let bytes = serialize(block);
        record.extend_from_slice(&[0xf9, 0xbe, 0xb4, 0xd9]);
        record.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        record.extend_from_slice(&bytes);
    }
    fs::write(blocks_path.join("blk00000.dat"), record).unwrap();

    let runtime = Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let listener = runtime
        .block_on(TcpListener::bind((Ipv4Addr::LOCALHOST, 0)))
        .unwrap();
    let client = Client::new_with(
        &format!("http://{}", listener.local_addr().unwrap()),
        Auth::None,
        0,
        Duration::ZERO,
    )
    .unwrap();
    runtime.spawn(async move {
        loop {
            let mut socket = BufReader::new(listener.accept().await.unwrap().0);
            let mut length = 0;
            loop {
                let mut line = String::new();
                assert_ne!(socket.read_line(&mut line).await.unwrap(), 0);
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length: ") {
                    length = value.trim().parse::<usize>().unwrap();
                }
            }
            assert!(length < 4096);
            let mut body = vec![0; length];
            socket.read_exact(&mut body).await.unwrap();
            let body = reply(&from_slice(&body).unwrap(), &blocks).to_string();
            socket
                .get_mut()
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
        }
    });
    runtime.spawn(async {
        while COMMITS.load(Ordering::SeqCst) == 0 {
            TokioTime::sleep(Duration::from_millis(10)).await;
        }
        // Allow the real idle loop to observe an unchanged tip first.
        TokioTime::sleep(Duration::from_secs(2)).await;
        assert_eq!(COMMITS.load(Ordering::SeqCst), 1);
        ACTIVE.store(2, Ordering::SeqCst);
    });

    let result = run(
        Config {
            client,
            blocks_path,
            server: ServerConfig {
                bind: Ipv4Addr::LOCALHOST.into(),
                port: 0.into(),
                data_path: directory,
                ..ServerConfig::default()
            },
            cache_budget: DEFAULT_CACHE_BUDGET,
        },
        Exit::default(),
        |context: ImportContext<'_>, reader| {
            Ok(Plugins {
                indexer: Indexer::import(context, reader)?,
            })
        },
    );
    let error = result.unwrap_err().to_string();
    assert!(
        error.contains("reorg fixture complete"),
        "unexpected runner error: {error}"
    );
    assert_eq!(COMMITS.load(Ordering::SeqCst), 3);
}
