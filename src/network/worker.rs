use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::blockchain::Blockchain;
use crate::types::block::Block;
use crate::types::hash::{Hashable, H256};
use std::collections::HashMap;

use futures::executor::block_on;
use log::{debug, error, warn};
use std::sync::{Arc, Mutex};
use std::thread;

#[cfg(any(test, test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test, test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,       // proj3 added
    buffer: Arc<Mutex<HashMap<H256, Block>>>, // proj3 added
}

impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
        buffer: &Arc<Mutex<HashMap<H256, Block>>>,
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(blockchain),
            buffer: Arc::clone(buffer),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();

            // I think it world be better to initialize lock type variables in advanced
            let mut locked_blockchian = self.blockchain.lock().unwrap();
            let mut locked_bffer = self.buffer.lock().unwrap();

            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(hashes) => {
                    let mut new_blocks: Vec<H256> = Vec::new();

                    for hash in hashes {
                        if !locked_blockchian.blocks.contains_key(&hash) {
                            new_blocks.push(hash);
                        }
                    }
                    if new_blocks.len() > 0 {
                        peer.write(Message::GetBlocks(new_blocks.clone()));
                    }
                }
                Message::GetBlocks(hashes) => {
                    let mut new_blocks = Vec::new();

                    for hash in hashes {
                        if locked_blockchian.blocks.contains_key(&hash) {
                            new_blocks.push(locked_blockchian.blocks[&hash].clone());
                        }
                    }
                    if new_blocks.len() > 0 {
                        peer.write(Message::Blocks(new_blocks));
                    }
                }
                Message::Blocks(blocks) => {
                    let mut new_blocks: Vec<H256> = Vec::new();
                    let mut buffer_parents: Vec<H256> = Vec::new();

                    for block in blocks.iter() {
                        // If the block isn't inside blockchain, then we can try to insert it
                        // Or, we just pass it
                        if !(locked_blockchian.blocks.contains_key(&block.hash())) {
                            // If block's parent isn't inside the blockchain,
                            // then we need to wait until block's parent be added in.
                            if locked_blockchian.blocks.contains_key(&block.header.parent) {
                                locked_blockchian.insert(block);
                                new_blocks.push(block.hash());
                            } else {
                                locked_bffer.insert(block.header.parent, block.clone());
                                buffer_parents.push(block.header.parent.clone());
                                peer.write(Message::GetBlocks(buffer_parents.clone()));
                            }
                        }
                    }
                    if new_blocks.len() > 0 {
                        self.server
                            .broadcast(Message::NewBlockHashes(new_blocks.clone()));
                    }
                }
                Message::NewTransactionHashes(hashes) => {
                    unimplemented!();
                }
                Message::GetTransactions(hashes) => {
                    unimplemented!();
                }
                Message::Transactions(transactions) => {
                    unimplemented!();
                }
            }
        }
    }
}

#[cfg(any(test, test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>,
}
#[cfg(any(test, test_utilities))]
impl TestMsgSender {
    fn new() -> (
        TestMsgSender,
        smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    ) {
        let (s, r) = smol::channel::unbounded();
        (TestMsgSender { s }, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}
#[cfg(any(test, test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();

    let fake_blockchain = Blockchain::new();
    let blockchain = Arc::new(Mutex::new(fake_blockchain));
    let fake_buffer = HashMap::new();
    let buffer: Arc<Mutex<HashMap<H256, Block>>> = Arc::new(Mutex::new(fake_buffer));

    let worker = Worker::new(1, msg_chan, &server, &blockchain, &buffer);
    worker.start();

    let mut res: Vec<H256> = Vec::new();
    let locked_blockchain = blockchain.lock().unwrap();
    for (_hash, _block) in locked_blockchain.blocks.iter() {
        res.push(*_hash);
    }

    (test_msg_sender, server_receiver, res)
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;
    use ntest::timeout;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut peer_receiver =
            test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
