use crate::test_cache::init_cache;
use std::{
    fs,
    future::Future,
    net::{Ipv4Addr, SocketAddr},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use bitcoin::{
    Amount, Block, Network, OutPoint, ScriptBuf, Sequence, TxIn, TxOut, Witness,
    blockdata::constants::genesis_block, consensus::serialize,
};
use bitview_default::DefaultPlugins;
use bitview_plugin::{ImportContext, UpdateContext};
use bitview_plugin_indexer::{HasIndexer, Indexer};
use bitview_query::AsyncQuery;
use bitview_runtime::update;
use brk_error::Result;
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::HOUR4_INTERVAL;
use serde_json::from_slice;
use tempfile::{TempDir, tempdir};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    runtime::Builder,
    spawn,
    task::JoinHandle,
};

use super::chain_rpc::reply;
use crate::{AppState, Server, ServerConfig};

/// Setup and lifetime only; route and reorg assertions live in named scenarios.
pub(crate) struct ChainFixture {
    pub(crate) chain: Arc<[Block; 7]>,
    pub(crate) tip: Arc<AtomicU32>,
    pub(crate) active: Arc<AtomicUsize>,
    pub(crate) plugins: DefaultPlugins,
    pub(crate) query: AsyncQuery,
    pub(crate) state: AppState,
    pub(crate) address: SocketAddr,
    serving: JoinHandle<Result<()>>,
    mock: JoinHandle<()>,
    pub(crate) directory: TempDir,
}

impl Drop for ChainFixture {
    fn drop(&mut self) {
        self.serving.abort();
        self.mock.abort();
    }
}

impl ChainFixture {
    async fn new(first: Block) -> Self {
        let directory = tempdir().unwrap();
        let blocks = directory.path().join("blocks");
        fs::create_dir(&blocks).unwrap();
        let block = genesis_block(Network::Bitcoin);
        // Synthetic forks: the reader trusts the node's canonical-chain
        // selection; this fixture does not claim consensus-valid PoW.
        let second = fork(&block, 2);
        let fourth = fork(&first, 2);
        let mut forward = fork(&fourth, 3);
        forward.header.time = block.header.time + 86_400;
        let mut backward = fork(&forward, 4);
        backward.header.time = block.header.time + 1200;
        let chain = Arc::new([
            block.clone(),
            first.clone(),
            second.clone(),
            fork(&second, 1),
            fourth,
            forward,
            backward,
        ]);
        let mut record = Vec::new();
        for block in chain.iter() {
            let bytes = serialize(block);
            record.extend_from_slice(&[0xf9, 0xbe, 0xb4, 0xd9]);
            record.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            record.extend_from_slice(&bytes);
        }
        fs::write(blocks.join("blk00000.dat"), record).unwrap();

        let node = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let client = Client::new_with(
            &format!("http://{}", node.local_addr().unwrap()),
            Auth::None,
            0,
            Duration::ZERO,
        )
        .unwrap();
        let tip = Arc::new(AtomicU32::new(0));
        let observed_tip = tip.clone();
        let active = Arc::new(AtomicUsize::new(1));
        let observed_active = active.clone();
        let observed_chain = chain.clone();
        let mock = spawn(async move {
            loop {
                let mut socket = BufReader::new(node.accept().await.unwrap().0);
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    assert_ne!(socket.read_line(&mut line).await.unwrap(), 0);
                    if line == "\r\n" {
                        break;
                    }
                    if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length: ")
                    {
                        length = value.trim().parse::<usize>().unwrap();
                    }
                }
                assert!(length < 4096);
                let mut body = vec![0; length];
                socket.read_exact(&mut body).await.unwrap();
                let request = from_slice(&body).unwrap();
                let body = reply(
                    &request,
                    observed_tip.load(Ordering::SeqCst),
                    &observed_chain[..],
                    observed_active.load(Ordering::SeqCst),
                )
                .to_string();
                socket.get_mut().write_all(format!("HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
            }
        });
        let reader = Reader::new_without_rlimit(blocks.clone(), &client);
        let mut indexer = Indexer::import(ImportContext::new(directory.path()), &reader).unwrap();
        indexer.checked_index(&Exit::default()).unwrap();
        indexer.finish_update().unwrap();
        assert!(indexer.safe_lengths().last_height().is_some());
        drop(indexer);
        let plugins =
            DefaultPlugins::import(ImportContext::new(directory.path()), &reader).unwrap();
        assert!(plugins.indexer().safe_lengths().last_height().is_some());
        let query = AsyncQuery::build(&plugins, None);
        let server = Server::bind(
            &query,
            ServerConfig {
                bind: Ipv4Addr::LOCALHOST.into(),
                port: 0.into(),
                data_path: directory.path().to_owned(),
                ..ServerConfig::default()
            },
        )
        .await
        .unwrap();
        let address = server.listener.local_addr().unwrap();
        let inspection_state = server.state.clone();
        let serving = spawn(server.serve());

        Self {
            chain,
            tip,
            active,
            plugins,
            query,
            state: inspection_state,
            address,
            serving,
            mock,
            directory,
        }
    }

    pub(crate) fn publish(&mut self, branch: usize, height: u32) {
        self.active.store(branch, Ordering::SeqCst);
        self.tip.store(height, Ordering::SeqCst);
        update(&mut self.plugins, UpdateContext::new(&Exit::default())).unwrap();
    }
}

fn fork(genesis: &Block, branch: u8) -> Block {
    let mut block = genesis.clone();
    block.txdata.truncate(1);
    block.header.prev_blockhash = genesis.block_hash();
    block.header.time += 600 * u32::from(branch);
    block.txdata[0].input[0].script_sig = ScriptBuf::from_bytes(vec![1, branch]);
    block.header.merkle_root = block.compute_merkle_root().unwrap();
    block
}

pub(crate) fn default_first() -> Block {
    let mut first = fork(&genesis_block(Network::Bitcoin), 1);
    // Close the genesis four-hour bucket; a replacement fork can reopen it.
    first.header.time += HOUR4_INTERVAL;
    first
}

pub(crate) fn raw_fixture_block() -> Block {
    let mut first = fork(&genesis_block(Network::Bitcoin), 1);
    first.txdata[0].output.push(TxOut {
        value: Amount::ZERO,
        script_pubkey: ScriptBuf::from_bytes(vec![0x6a; 2048]),
    });
    let mut multisig = vec![0x51, 0x41];
    multisig.extend_from_slice(&first.txdata[0].output[0].script_pubkey.as_bytes()[1..66]);
    multisig.extend_from_slice(&[0x51, 0xae]);
    for script in [Vec::new(), vec![0x51], multisig] {
        first.txdata[0].output.push(TxOut {
            value: Amount::ZERO,
            script_pubkey: ScriptBuf::from_bytes(script),
        });
    }
    let mut spend = first.txdata[0].clone();
    spend.input = (2..5)
        .map(|vout| TxIn {
            previous_output: OutPoint::new(first.txdata[0].compute_txid(), vout),
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
        })
        .collect();
    spend.output = vec![TxOut {
        value: Amount::ZERO,
        script_pubkey: ScriptBuf::new(),
    }];
    first.txdata.push(spend);
    let mut descendant = first.txdata[1].clone();
    descendant.input.truncate(1);
    descendant.input[0].previous_output = OutPoint::new(first.txdata[1].compute_txid(), 0);
    first.txdata.push(descendant);
    first.header.merkle_root = first.compute_merkle_root().unwrap();
    first
}

pub(crate) fn run<F: Future<Output = ()>>(
    inspect: impl FnOnce(AppState, SocketAddr) -> F + Send + 'static,
) {
    run_populated(default_first(), inspect);
}

pub(crate) fn run_populated<F: Future<Output = ()>>(
    first: Block,
    inspect: impl FnOnce(AppState, SocketAddr) -> F + Send + 'static,
) {
    run_genesis(first, move |mut fixture| async move {
        fixture.publish(1, 1);
        inspect(fixture.state.clone(), fixture.address).await;
    });
}

pub(crate) fn run_genesis<F: Future<Output = ()>>(
    first: Block,
    inspect: impl FnOnce(ChainFixture) -> F + Send + 'static,
) {
    // Full pipeline fixtures share a cache and retain process-lifetime catalogs.
    // Running them together can push updates beyond the HTTP deadline even
    // without eviction. Each scenario still runs its own readers and writers.
    static FIXTURE: Mutex<()> = Mutex::new(());
    let _fixture = FIXTURE.lock().unwrap_or_else(|error| error.into_inner());
    init_cache();
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .unwrap()
                .block_on(async { inspect(ChainFixture::new(first).await).await });
        })
        .unwrap()
        .join()
        .unwrap();
}
