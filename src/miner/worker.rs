use crate::blockchain::Blockchain;
use crate::network::message::Message;
use crate::network::server::Handle as ServerHandle;
use crate::types::block::Block;
use crate::types::hash::Hashable;
use crossbeam::channel::Receiver;
use log::info;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        blockchain: &Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain: Arc::clone(blockchain),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _block = self
                .finished_block_chan
                .recv()
                .expect("Receive finished block error");

            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            let mut new_blockchain = self.blockchain.lock().unwrap();
            new_blockchain.insert(&_block);

            // Midterm2, according to the GitLab, we need to broadcast a hash of block
            // Although, worker will not run in "miner_three_block()" case
            let mut blk_hashes = Vec::new();
            blk_hashes.push(_block.hash());
            self.server.broadcast(Message::NewBlockHashes(blk_hashes));
            // drop(new_blockchain);
        }
    }
}
