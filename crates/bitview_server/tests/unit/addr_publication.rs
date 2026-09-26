use std::{
    fs::{OpenOptions, read, write},
    mem::take,
    net::{Ipv4Addr, SocketAddr},
    panic::{AssertUnwindSafe, catch_unwind},
    path::Path,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use bitcoin::{Amount, Block, OutPoint, ScriptBuf, Transaction, consensus::encode::serialize_hex};
use bitview_default::DefaultPlugins;
use bitview_query::{AsyncQuery, ResolvedAddrTxs, ResolvedRbf};
use brk_error::Error as BrkError;
use brk_mempool::{Mempool, ReadOnlyMempool};
use brk_rpc::{Auth, Client};
use brk_types::{Addr, Vout};
use serde_json::{Value, from_slice, from_str, json, to_value};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    spawn,
    sync::oneshot,
    task::{AbortHandle, JoinHandle, JoinSet, spawn_blocking},
};

use super::{
    mempool_publication::MempoolPublication, server_routes::exchange_with_etag,
    transaction_publication::TransactionPublication,
};
use crate::{Server, ServerConfig};

struct Node {
    tip: String,
    transactions: Vec<Transaction>,
    listed: bool,
    final_tip: Option<String>,
    best_reads: usize,
}

impl Node {
    fn reply(&mut self, request: &Value) -> Value {
        if let Some(batch) = request.as_array() {
            return batch.iter().map(|request| self.reply(request)).collect();
        }
        let result = match request["method"].as_str().unwrap() {
            "getbestblockhash" => {
                self.best_reads += 1;
                json!(if self.best_reads.is_multiple_of(2) {
                    self.final_tip.as_ref().unwrap_or(&self.tip)
                } else {
                    &self.tip
                })
            }
            "getblocktemplate" => {
                let transactions: Vec<_> = self.transactions.iter().map(|tx| json!({
                    "data": serialize_hex(tx),
                    "txid": tx.compute_txid().to_string(), "hash": tx.compute_wtxid().to_string(),
                    "fee": 1000, "sigops": 0, "weight": tx.weight().to_wu(), "depends": []
                })).collect();
                json!({
                    "version": 1, "rules": [], "vbavailable": {}, "capabilities": [], "vbrequired": 0,
                    "previousblockhash": self.tip, "transactions": transactions, "coinbaseaux": {},
                    "coinbasevalue": 0, "target": "0".repeat(64), "mintime": 0, "mutable": [],
                    "noncerange": "00000000ffffffff", "sigoplimit": 80000, "sizelimit": 4000000,
                    "weightlimit": 4000000, "curtime": 0, "bits": "1d00ffff", "height": 2
                })
            }
            "getrawmempool" => json!(if self.listed {
                self.transactions
                    .iter()
                    .map(|tx| tx.compute_txid().to_string())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            }),
            "getmempoolinfo" => json!({
                "loaded": true, "size": self.transactions.len(), "bytes": 100, "usage": 1000, "total_fee": 0.00001,
                "maxmempool": 300000000, "mempoolminfee": 0.00001, "minrelaytxfee": 0.00001,
                "incrementalrelayfee": 0.00001, "unbroadcastcount": 0, "fullrbf": true
            }),
            method => panic!("unexpected address fixture RPC: {method}"),
        };
        json!({"id": request["id"], "result": result, "error": null})
    }
}

/// A real RPC-driven mempool beside the populated indexer fixture.
pub struct AddrPublication {
    query: AsyncQuery,
    writer: Arc<Mutex<Mempool>>,
    mempool: ReadOnlyMempool,
    node: Arc<Mutex<Node>>,
    address: SocketAddr,
    addr: Addr,
    paths: Vec<String>,
    tags: Vec<String>,
    transaction: TransactionPublication,
    aggregates: MempoolPublication,
    snapshot: Option<ResolvedAddrTxs>,
    rbf_snapshot: Option<ResolvedRbf>,
    snapshot_body: Value,
    tasks: Vec<AbortHandle>,
}

impl AddrPublication {
    pub async fn start(plugins: &DefaultPlugins, directory: &Path, first: &Block) -> Self {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await.unwrap();
        let client = Client::new_with(
            &format!("http://{}", listener.local_addr().unwrap()),
            Auth::None,
            0,
            Duration::ZERO,
        )
        .unwrap();
        // Synthetic, like the containing chain: exercise indexing and queries,
        // not proof of work or coinbase maturity validation.
        let mut pending = first.txdata[0].clone();
        pending.input[0].previous_output = OutPoint::new(first.txdata[0].compute_txid(), 0);
        pending.input[0].script_sig = ScriptBuf::new();
        pending.output.truncate(1);
        pending.output[0].value = Amount::from_sat(pending.output[0].value.to_sat() - 1000);
        let transaction = TransactionPublication::new(pending.compute_txid().into());
        let node = Arc::new(Mutex::new(Node {
            tip: first.block_hash().to_string(),
            transactions: vec![pending],
            listed: true,
            final_tip: None,
            best_reads: 0,
        }));
        let observed = node.clone();
        let mock = spawn(async move {
            loop {
                let mut socket = BufReader::new(listener.accept().await.unwrap().0);
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
                let mut bytes = vec![0; length];
                socket.read_exact(&mut bytes).await.unwrap();
                let body = observed
                    .lock()
                    .unwrap()
                    .reply(&from_slice(&bytes).unwrap())
                    .to_string();
                socket.get_mut().write_all(format!("HTTP/1.1 200 OK\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
            }
        });
        let mempool = Mempool::new(&client);
        let query = AsyncQuery::build(plugins, Some(mempool.read_only_clone()));
        let writer = Arc::new(Mutex::new(mempool));
        let mempool = writer.lock().unwrap().read_only_clone();
        let server = Server::bind(
            &query,
            ServerConfig {
                bind: Ipv4Addr::LOCALHOST.into(),
                port: 0.into(),
                data_path: directory.to_owned(),
                ..ServerConfig::default()
            },
        )
        .await
        .unwrap();
        let address = server.listener.local_addr().unwrap();
        let aggregates = MempoolPublication::new(server.state.clone());
        let serving = spawn(server.serve());
        let addr = Addr::try_from(&first.txdata[0].output[0].script_pubkey).unwrap();
        let paths = vec![
            format!("/api/address/{addr}"),
            format!("/api/address/{addr}/txs"),
            format!("/api/address/{addr}/txs/mempool"),
        ];
        let mut fixture = Self {
            query,
            writer,
            mempool,
            node,
            address,
            addr,
            paths,
            tags: Vec::new(),
            transaction,
            aggregates,
            snapshot: None,
            rbf_snapshot: None,
            snapshot_body: Value::Null,
            tasks: vec![mock.abort_handle(), serving.abort_handle()],
        };
        fixture.check_unavailable(true).await;
        fixture.tick(false).await;
        fixture.check_unavailable(false).await;
        fixture.aggregates.check_available(fixture.address).await;

        // Address-dependent reads reject unresolved inputs; statistics
        // continue serving the last complete membership observation.
        let (release, filling) = fixture.pause_resolution().await;
        fixture.aggregates.check_available(fixture.address).await;
        #[cfg(feature = "price")]
        fixture.aggregates.check_live_outputs(fixture.address).await;
        let mut requests = JoinSet::new();
        for path in &fixture.paths {
            for method in ["GET", "HEAD"] {
                let path = path.clone();
                let address = fixture.address;
                requests.spawn(async move {
                    let response = exchange_with_etag(address, method, &path, "*").await;
                    assert!(response.starts_with("HTTP/1.1 503"), "{path}: {response}");
                    assert!(!response.contains("\r\netag:"));
                    assert!(response.contains("\r\ncache-control: no-store\r\n"));
                });
            }
        }
        while let Some(result) = requests.join_next().await {
            result.unwrap();
        }
        release.send(()).unwrap();
        filling.await.unwrap();
        fixture.check_available().await;
        fixture
            .check_revalidation_without_chain_body(&directory.join("blocks/blk00000.dat"))
            .await;

        let pending = fixture.node.lock().unwrap().transactions.clone();
        let mut incoming = pending[0].clone();
        incoming.output[0].value = Amount::from_sat(incoming.output[0].value.to_sat() - 1);
        fixture.node.lock().unwrap().transactions.push(incoming);
        let retained = fixture.mempool.load();
        let (release, filling) = fixture.pause_resolution().await;
        fixture.check_available().await;
        #[cfg(feature = "price")]
        fixture.aggregates.check_live_outputs(fixture.address).await;
        assert!(Arc::ptr_eq(&retained, &fixture.mempool.load()));
        release.send(()).unwrap();
        filling.await.unwrap();
        assert!(!Arc::ptr_eq(&retained, &fixture.mempool.load()));
        fixture.node.lock().unwrap().transactions = pending;
        fixture.tick(true).await;
        fixture.check_available().await;

        // A resolver panic occurs after private membership application. The next
        // tick must recover its indexes before it can publish another version.
        let mut incoming = fixture.node.lock().unwrap().transactions[0].clone();
        incoming.output[0].value = Amount::from_sat(incoming.output[0].value.to_sat() - 2);
        fixture.node.lock().unwrap().transactions.push(incoming);
        let before_panic = fixture.mempool.load();
        let writer = fixture.writer.clone();
        spawn_blocking(move || {
            let mut writer = writer.lock().unwrap();
            assert!(
                catch_unwind(AssertUnwindSafe(
                    || writer.tick_with(|_| panic!("fixture resolver panic"))
                ))
                .is_err()
            );
        })
        .await
        .unwrap();
        assert!(Arc::ptr_eq(&before_panic, &fixture.mempool.load()));
        fixture.check_available().await;
        fixture.tick(true).await;
        assert_eq!(fixture.mempool.load().info().unwrap().count, 2);
        fixture.node.lock().unwrap().transactions.pop();
        fixture.tick(true).await;
        fixture.check_available().await;

        let published_info = to_value(fixture.mempool.load().info().unwrap()).unwrap();
        fixture.node.lock().unwrap().listed = false;
        fixture.tick(true).await;
        assert_eq!(
            to_value(fixture.mempool.load().info().unwrap()).unwrap(),
            published_info
        );
        fixture.check_available().await;
        fixture.node.lock().unwrap().listed = true;
        {
            let mut node = fixture.node.lock().unwrap();
            // The injected panic skipped a final read; begin this pair explicitly.
            node.best_reads = 0;
            node.final_tip = Some("11".repeat(32));
        }
        fixture.tick(true).await;
        assert_eq!(
            to_value(fixture.mempool.load().info().unwrap()).unwrap(),
            published_info
        );
        fixture.check_available().await;
        fixture.node.lock().unwrap().final_tip = None;
        fixture.tick(true).await;
        fixture.check_available().await;

        // Changing membership after resolution cannot mutate the captured body.
        let captured = fixture
            .query
            .sync(|q| q.resolve_addr_txs(&fixture.addr, 50, 25, 50))
            .unwrap();
        let pending = take(&mut fixture.node.lock().unwrap().transactions);
        fixture.tick(true).await;
        let body = fixture
            .query
            .run(move |q| q.addr_txs_resolved(captured))
            .await
            .unwrap();
        assert_eq!(to_value(body).unwrap(), fixture.snapshot_body);
        let response =
            exchange_with_etag(address, "GET", &fixture.paths[1], &fixture.tags[1]).await;
        assert!(
            response.starts_with("HTTP/1.1 200"),
            "membership change must invalidate: {response}"
        );
        // Model a lagging mempool cycle retaining an already published tx.
        // Confirmed native results and HTTP identities must win over that copy.
        if let Some(confirmed) = first.txdata.get(1) {
            let txid = confirmed.compute_txid().into();
            let expected = fixture
                .query
                .sync(|q| q.transaction_json_resolved(q.resolve_transaction(&txid)?))
                .unwrap();
            let expected_cpfp = fixture
                .query
                .sync(|q| q.confirmed_cpfp_resolved(q.resolve_confirmed_tx(&txid)?))
                .unwrap();
            let expected_rate = expected_cpfp.effective_fee_per_vsize;
            let expected_spends = fixture.query.sync(|q| q.outspends(&txid)).unwrap();
            assert!(expected_spends[0].status.as_ref().unwrap().confirmed);
            assert_eq!(
                from_slice::<Value>(&expected).unwrap()["status"]["confirmed"],
                true
            );
            fixture.node.lock().unwrap().transactions = vec![confirmed.clone()];
            fixture.tick(true).await;
            let tip = first.block_hash().into();
            assert!(fixture.mempool.load().contains_txid(&txid, &tip).unwrap());
            let stale_rbf = fixture
                .mempool
                .load()
                .rbf_for_tx(&txid, &tip)
                .unwrap()
                .root
                .unwrap();
            assert!(stale_rbf.in_mempool);
            assert_ne!(
                stale_rbf.rate, expected_rate,
                "fixture must distinguish live and chain rates"
            );
            let expected_rbf = fixture
                .query
                .sync(|q| {
                    q.resolve_rbf(&txid)
                        .and_then(|resolved| q.tx_rbf_resolved(resolved))
                })
                .unwrap();
            let replacement = expected_rbf.replacements.as_ref().unwrap();
            assert_eq!(replacement.mined, Some(true));
            assert_eq!(replacement.tx.rate, expected_rate);
            // Its confirmed descendant is absent from this stale mempool copy.
            assert!(
                !fixture
                    .mempool
                    .load()
                    .outspends_if_present(&txid, &tip)
                    .unwrap()
                    .unwrap()[0]
                    .spent
            );
            assert_eq!(
                to_value(fixture.query.sync(|q| q.outspends(&txid)).unwrap()).unwrap(),
                to_value(&expected_spends).unwrap()
            );
            assert_eq!(
                to_value(
                    fixture
                        .query
                        .sync(|q| q.outspend(&txid, Vout::ZERO))
                        .unwrap()
                )
                .unwrap(),
                to_value(&expected_spends[0]).unwrap()
            );
            assert_eq!(
                to_value(
                    fixture
                        .query
                        .sync(|q| q.confirmed_cpfp_resolved(q.resolve_confirmed_tx(&txid)?))
                        .unwrap()
                )
                .unwrap(),
                to_value(&expected_cpfp).unwrap()
            );
            assert_eq!(
                fixture
                    .query
                    .sync(|q| q.confirmed_cpfp_resolved(q.resolve_confirmed_tx(&txid)?))
                    .unwrap()
                    .effective_fee_per_vsize,
                expected_rate
            );
            assert!(
                fixture
                    .query
                    .sync(|q| q.transaction_status(&txid))
                    .unwrap()
                    .confirmed
            );
            assert_eq!(
                fixture
                    .query
                    .sync(|q| q.transaction_json_resolved(q.resolve_transaction(&txid)?))
                    .unwrap(),
                expected,
            );
            for suffix in ["", "/status", "/outspends", "/outspend/0", "/cpfp", "/rbf"] {
                let path = if suffix == "/cpfp" {
                    format!("/api/v1/cpfp/{txid}")
                } else if suffix == "/rbf" {
                    format!("/api/v1/tx/{txid}/rbf")
                } else {
                    format!("/api/tx/{txid}{suffix}")
                };
                let response = exchange_with_etag(address, "GET", &path, "\"old\"").await;
                assert!(response.starts_with("HTTP/1.1 200"), "{response}");
                let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
                let expected = match suffix {
                    "" => from_slice::<Value>(&expected).unwrap(),
                    "/status" => from_slice::<Value>(&expected).unwrap()["status"].clone(),
                    "/outspends" => to_value(&expected_spends).unwrap(),
                    "/outspend/0" => to_value(&expected_spends[0]).unwrap(),
                    "/cpfp" => to_value(&expected_cpfp).unwrap(),
                    "/rbf" => to_value(&expected_rbf).unwrap(),
                    _ => unreachable!(),
                };
                assert_eq!(body, expected);
            }
        }
        fixture.node.lock().unwrap().transactions = pending;
        fixture.tick(true).await;
        fixture.check_available().await;
        let pending_txid = fixture.node.lock().unwrap().transactions[0]
            .compute_txid()
            .into();
        fixture.rbf_snapshot = Some(
            fixture
                .query
                .sync(|q| q.resolve_rbf(&pending_txid))
                .unwrap(),
        );
        fixture.snapshot = Some(
            fixture
                .query
                .sync(|q| q.resolve_addr_txs(&fixture.addr, 50, 25, 50))
                .unwrap(),
        );
        fixture
    }

    async fn pause_resolution(&self) -> (mpsc::Sender<()>, JoinHandle<()>) {
        let mempool = self.writer.clone();
        let resolver = self.query.sync(|q| q.indexer_prevout_resolver());
        let (started, ready) = oneshot::channel();
        let (release, resume) = mpsc::channel();
        let mut started = Some(started);
        let filling = spawn_blocking(move || {
            // tick_with accepts Fn, so synchronize the one resolver invocation.
            let signal = Mutex::new(started.take());
            let resume = Mutex::new(resume);
            mempool
                .lock()
                .unwrap()
                .tick_with(|holes| {
                    if let Some(started) = signal.lock().unwrap().take() {
                        started.send(()).unwrap();
                    }
                    resume.lock().unwrap().recv().unwrap();
                    resolver(holes)
                })
                .unwrap();
        });
        ready.await.unwrap();
        (release, filling)
    }

    async fn tick(&self, fill: bool) {
        let mempool = self.writer.clone();
        let resolver = self.query.sync(|q| q.indexer_prevout_resolver());
        spawn_blocking(move || {
            mempool.lock().unwrap().tick_with(|holes| {
                if fill {
                    resolver(holes)
                } else {
                    Default::default()
                }
            })
        })
        .await
        .unwrap()
        .unwrap();
    }

    async fn check_unavailable(&self, aggregates: bool) {
        if aggregates {
            self.aggregates.check_unavailable(self.address).await;
        }
        self.transaction.check_unavailable(self.address).await;
        let mut requests = JoinSet::new();
        for (index, path) in self.paths.iter().enumerate() {
            for method in ["GET", "HEAD"] {
                for tag in [
                    "*",
                    self.tags
                        .get(index)
                        .map(String::as_str)
                        .unwrap_or("\"old\""),
                ] {
                    let address = self.address;
                    let path = path.clone();
                    let tag = tag.to_owned();
                    requests.spawn(async move {
                        let response = exchange_with_etag(address, method, &path, &tag).await;
                        assert!(response.starts_with("HTTP/1.1 503"), "{path}: {response}");
                        assert!(!response.contains("\r\netag:"));
                        assert!(response.contains("\r\ncache-control: no-store\r\n"));
                    });
                }
            }
        }
        while let Some(result) = requests.join_next().await {
            result.unwrap();
        }
    }

    async fn check_available(&mut self) {
        self.aggregates.check_available(self.address).await;
        self.transaction
            .check_available(&self.query, self.address)
            .await;
        self.tags.clear();
        for (index, path) in self.paths.iter().enumerate() {
            let response = exchange_with_etag(self.address, "GET", path, "\"old\"").await;
            assert!(response.starts_with("HTTP/1.1 200"), "{path}: {response}");
            let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
            let expected = self.query.sync(|q| match index {
                0 => to_value(q.addr(self.addr.clone()).unwrap()).unwrap(),
                1 => to_value(
                    q.addr_txs_resolved(q.resolve_addr_txs(&self.addr, 50, 25, 50).unwrap())
                        .unwrap(),
                )
                .unwrap(),
                _ => to_value(q.addr_mempool_txs(&self.addr, 50).unwrap()).unwrap(),
            });
            assert_eq!(body, expected);
            if index == 1 {
                assert!(!body[0]["status"]["confirmed"].as_bool().unwrap());
                assert!(body[1]["status"]["confirmed"].as_bool().unwrap());
                self.snapshot_body = body;
            }
            let tag = response
                .lines()
                .find_map(|line| line.strip_prefix("etag: "))
                .unwrap()
                .to_owned();
            for method in ["GET", "HEAD"] {
                for validator in [tag.as_str(), "*"] {
                    let response = exchange_with_etag(self.address, method, path, validator).await;
                    assert!(response.starts_with("HTTP/1.1 304"), "{path}: {response}");
                    assert!(response.ends_with("\r\n\r\n"));
                }
            }
            self.tags.push(tag);
        }
    }

    async fn check_revalidation_without_chain_body(&self, block_file: &Path) {
        let original = read(block_file).unwrap();
        OpenOptions::new()
            .write(true)
            .open(block_file)
            .unwrap()
            .set_len(0)
            .unwrap();
        let mut responses = Vec::new();
        for method in ["GET", "HEAD"] {
            responses.push(
                exchange_with_etag(self.address, method, &self.paths[1], &self.tags[1]).await,
            );
        }
        let uncached = exchange_with_etag(self.address, "GET", &self.paths[1], "\"old\"").await;
        write(block_file, original).unwrap();
        for response in responses {
            assert!(
                response.starts_with("HTTP/1.1 304"),
                "revalidation must not load confirmed bodies: {response}"
            );
        }
        assert!(
            uncached.starts_with("HTTP/1.1 500"),
            "a 200 must really load the captured confirmed body: {uncached}"
        );
    }

    pub async fn consume_snapshot(&mut self) {
        let snapshot = self.snapshot.take().unwrap();
        let transactions = self
            .query
            .run(move |q| q.addr_txs_resolved(snapshot))
            .await
            .unwrap();
        assert_eq!(to_value(transactions).unwrap(), self.snapshot_body);
    }

    pub async fn after_reorg(mut self, second: &Block) {
        let rbf = self.rbf_snapshot.take().unwrap();
        assert!(matches!(
            self.query.run(move |q| q.tx_rbf_resolved(rbf)).await,
            Err(BrkError::StateUpdating)
        ));
        self.check_unavailable(false).await;
        #[cfg(feature = "price")]
        self.aggregates
            .check_live_outputs_unavailable(self.address)
            .await;
        {
            let mut node = self.node.lock().unwrap();
            node.tip = second.block_hash().to_string();
            node.transactions.clear();
        }
        self.tick(true).await;
        for (index, path) in self.paths.iter().enumerate() {
            let response = exchange_with_etag(self.address, "GET", path, &self.tags[index]).await;
            assert!(
                response.starts_with("HTTP/1.1 200"),
                "reconciled source must replace old response: {response}"
            );
            let body: Value = from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
            if index == 1 {
                assert!(
                    body.as_array()
                        .unwrap()
                        .iter()
                        .all(|tx| tx["status"]["confirmed"] == true)
                );
            } else if index == 2 {
                assert_eq!(body, json!([]));
            }
        }
    }
}

impl Drop for AddrPublication {
    fn drop(&mut self) {
        for task in &self.tasks {
            task.abort();
        }
    }
}
