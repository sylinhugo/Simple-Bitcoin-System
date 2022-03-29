use crate::blockchain::{self, Blockchain};
use crate::miner::Handle as MinerHandle;
use crate::network::message::Message;
use crate::network::server::Handle as NetworkServerHandle;
use crate::types::block;
use crate::types::hash::Hashable;
use crate::types::transaction::StatePerBlock;
use crate::types::transaction_generate::Handle as TXGenerateHandle;
use serde::Serialize;

use log::{debug, info};
use std::collections::HashMap;
use std::ops::RangeBounds;
use std::sync::{Arc, Mutex};
use std::thread;
use tiny_http::Header;
use tiny_http::Response;
use tiny_http::Server as HTTPServer;
use url::Url;

pub struct Server {
    handle: HTTPServer,
    miner: MinerHandle,
    network: NetworkServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    tx_generator: TXGenerateHandle,
    state_per_block: Arc<Mutex<StatePerBlock>>,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

macro_rules! respond_result {
    ( $req:expr, $success:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let payload = ApiResponse {
            success: $success,
            message: $message.to_string(),
        };
        let resp = Response::from_string(serde_json::to_string_pretty(&payload).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}
macro_rules! respond_json {
    ( $req:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let resp = Response::from_string(serde_json::to_string(&$message).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}

impl Server {
    pub fn start(
        addr: std::net::SocketAddr,
        miner: &MinerHandle,
        network: &NetworkServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
        tx_generator: &TXGenerateHandle,
        state_per_block: &Arc<Mutex<StatePerBlock>>,
    ) {
        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            handle,
            miner: miner.clone(),
            network: network.clone(),
            blockchain: Arc::clone(blockchain),
            tx_generator: tx_generator.clone(),
            state_per_block: Arc::clone(state_per_block),
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                let miner = server.miner.clone();
                let network = server.network.clone();
                let blockchain = Arc::clone(&server.blockchain);
                let tx_generator = server.tx_generator.clone();
                let state_per_block = Arc::clone(&server.state_per_block);
                thread::spawn(move || {
                    // a valid url requires a base
                    let base_url = Url::parse(&format!("http://{}/", &addr)).unwrap();
                    let url = match base_url.join(req.url()) {
                        Ok(u) => u,
                        Err(e) => {
                            respond_result!(req, false, format!("error parsing url: {}", e));
                            return;
                        }
                    };
                    match url.path() {
                        "/miner/start" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let lambda = match params.get("lambda") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing lambda");
                                    return;
                                }
                            };
                            let lambda = match lambda.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing lambda: {}", e)
                                    );
                                    return;
                                }
                            };
                            miner.start(lambda);
                            respond_result!(req, true, "ok");
                        }
                        "/tx-generator/start" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            // without theta return null
                            let theta = match params.get("theta") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing theta");
                                    return;
                                }
                            };
                            // cannot parse theta
                            let theta = match theta.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing theta: {}", e)
                                    );
                                    return;
                                }
                            };
                            tx_generator.start(theta);
                            respond_result!(req, true, "ok!");
                        }
                        "/network/ping" => {
                            network.broadcast(Message::Ping(String::from("Test ping")));
                            respond_result!(req, true, "ok");
                        }
                        "/blockchain/longest-chain" => {
                            println!("test1");
                            let blockchain = blockchain.lock().unwrap();
                            println!("test2");
                            let v = blockchain.all_blocks_in_longest_chain();
                            println!("test3");
                            let v_string: Vec<String> =
                                v.into_iter().map(|h| h.to_string()).collect();
                            println!("test4");
                            // drop(blockchain);
                            respond_json!(req, v_string);
                        }
                        "/blockchain/longest-chain-tx" => {
                            let blockchain_mtx = blockchain.lock().unwrap();
                            let mut res = Vec::new();
                            // get all txs of a single block
                            let mut i = 0;
                            println!("visiting long chain tx");

                            for block_hash in blockchain_mtx.all_blocks_in_longest_chain() {
                                let block = blockchain_mtx.get(block_hash);
                                let content = block.content.content;
                                let len = content.len();

                                // tmp Vec to record hashes of txs in a block
                                let mut tmp = Vec::new();
                                for i in 0..len {
                                    tmp.push(content[i].hash().to_string());
                                }
                                i += 1;
                                res.push(tmp);
                            }
                            // drop(blockchain_mtx);
                            println!("how many blocks in chain {}", i);
                            respond_json!(req, res);
                            // respond_result!(req, true, "ok");
                        }
                        "/blockchain/longest-chain-tx-count" => {
                            // unimplemented!()
                            respond_result!(req, false, "unimplemented!");
                        }
                        "/blockchain/state" => {
                            let blockchain_mtx = blockchain.lock().unwrap();
                            let locked_state_per_block = state_per_block.lock().unwrap();
                            // let res = Vec::new();

                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let block = match params.get("block") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing block");
                                    return;
                                }
                            };
                            let block = match block.parse::<u32>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing block: {}", e)
                                    );
                                    return;
                                }
                            };
                            // here we get the number of block
                            let mut cur_block_hash = blockchain_mtx.tip();
                            let mut cur_block = &blockchain_mtx.blocks[&cur_block_hash];
                            loop {
                                if blockchain_mtx.lengths[&cur_block_hash] == block {
                                    break;
                                }
                                cur_block = &blockchain_mtx.blocks[&cur_block_hash];
                                cur_block_hash = cur_block.header.parent;
                            }
                            // get the state according to the block seq num
                            let block_state =
                                locked_state_per_block.state_block_map[&cur_block_hash].clone();
                            let mut res = Vec::new();
                            for key in block_state.state_map.keys() {
                                let mut tmp = Vec::new();
                                let value = &block_state.state_map[key];
                                // convert to string
                                let prev_tx_hash_s = key.prev_tx_hash.to_string();
                                let index_s = key.index.to_string();
                                let value_s = value.value.to_string();
                                let recipient_s = value.receipient_address.to_string();
                                let tuple_res = prev_tx_hash_s
                                    + " "
                                    + &index_s
                                    + " "
                                    + &value_s
                                    + " "
                                    + &recipient_s;
                                tmp.push(tuple_res);
                                res.push(tmp);
                            }

                            respond_json!(req, res);
                        }
                        _ => {
                            let content_type =
                                "Content-Type: application/json".parse::<Header>().unwrap();
                            let payload = ApiResponse {
                                success: false,
                                message: "endpoint not found".to_string(),
                            };
                            let resp = Response::from_string(
                                serde_json::to_string_pretty(&payload).unwrap(),
                            )
                            .with_header(content_type)
                            .with_status_code(404);
                            req.respond(resp).unwrap();
                        }
                    }
                });
            }
        });
        info!("API server listening at {}", &addr);
    }
}
