use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::blockchain::Blockchain;
use crate::types::block::Block;
use crate::types::hash::{Hashable, H256};
use crate::types::transaction::{Mempool, Transaction, SignedTransaction, verify};
use std::collections::HashMap;
use std::convert::TryInto;

// use futures::executor::block_on;
// use futures::lock;
use log::{debug, error, warn};
use ring::signature;
use std::sync::{Arc, Mutex};
use std::{thread, mem, clone};

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
    orphan_buffer: Arc<Mutex<HashMap<H256, Block>>>,
    mempool: Arc<Mutex<Mempool>>,
}

impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
        buffer: &Arc<Mutex<HashMap<H256, Block>>>,
        orphan_buffer: &Arc<Mutex<HashMap<H256, Block>>>, 
        mempool: &Arc<Mutex<Mempool>>,
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(blockchain),
            buffer: Arc::clone(buffer),
            orphan_buffer: Arc::clone(orphan_buffer),
            mempool: Arc::clone(mempool)
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
            let mut locked_orphan_buffer = self.orphan_buffer.lock().unwrap();

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
                    let mut new_block_hashes: Vec<H256> = Vec::new();
                    let mut unseen: Vec<H256> = Vec::new();

                    for block in blocks.iter() {
                        // judge if the block already exists in the block chain
                        if locked_blockchian.blocks.contains_key(&block.hash()){
                            continue;
                        }
                        // judge if not parent already exists, then add current block into buffer
                        if !locked_blockchian.blocks.contains_key(&block.header.parent){
                            unseen.push(block.header.parent);
                            // structure in orphan_buffer:
                            // key : H256 of parent, value : block of child
                            locked_orphan_buffer.insert(block.header.parent, block.clone());
                        }
                        // parent exists, check from the buffer
                        else {
                            let mut tmp_difficulty = [255u8; 32];
                            tmp_difficulty[0] = 0u8;
                            tmp_difficulty[1] = 0u8;
                            tmp_difficulty[2] = 63u8;
                            let root_diff:H256 = tmp_difficulty.into();
                            // let root_diff  = locked_blockchian.blocks[&block.header.parent].header.difficulty;
                            if block.hash() < block.header.difficulty && block.header.difficulty == root_diff{
                                // add current block into chain
                                locked_blockchian.insert(block);
                                new_block_hashes.push(block.hash());
                                // search childs in the buffer and add them into chain iterately.
                                let mut parent_hash = block.hash();

                                while locked_orphan_buffer.contains_key(&parent_hash){
                                    // child block of parent_hash
                                    let child_block = locked_orphan_buffer.remove(&parent_hash).unwrap();
                                    // POV validation
                                    if child_block.header.difficulty == root_diff && child_block.header.difficulty > child_block.hash(){
                                        // add child block
                                        locked_blockchian.insert(&child_block);
                                        new_block_hashes.push(child_block.hash());
                                    }
                                    // update parent hash for next iteration
                                    parent_hash = child_block.hash();
                                }
                            }
                        }
                        
                    }
                    if new_block_hashes.len() > 0{
                        self.server.broadcast(Message::NewBlockHashes(new_block_hashes.clone()));
                    }
                    if unseen.len() > 0 {
                        self.server.broadcast(Message::GetBlocks(unseen.clone()));
                    }


                    // let mut new_blocks: Vec<H256> = Vec::new();
                    // let mut buffer_parents: Vec<H256> = Vec::new();

                    // for block in blocks.iter() {
                    //     // If the block isn't inside blockchain, then we can try to insert it
                    //     // Or, we just pass it
                    //     if !(locked_blockchian.blocks.contains_key(&block.hash())) {
                    //         // If block's parent isn't inside the blockchain,
                    //         // then we need to wait until block's parent be added in.
                    //         if locked_blockchian.blocks.contains_key(&block.header.parent) {
                    //             locked_blockchian.insert(block);
                    //             new_blocks.push(block.hash());
                    //         } else {
                    //             locked_bffer.insert(block.header.parent, block.clone());
                    //             buffer_parents.push(block.header.parent.clone());
                    //             peer.write(Message::GetBlocks(buffer_parents.clone()));
                    //         }
                    //     }
                    // }
                    // if new_blocks.len() > 0 {
                    //     self.server
                    //         .broadcast(Message::NewBlockHashes(new_blocks.clone()));
                    // }
                }
                Message::NewTransactionHashes(hashes) => {
                    println!("receive req new txs");
                    let mempool_mutex = self.mempool.lock().unwrap();
                    // vector to store transaction not included in mempool
                    let mut transactions_new = Vec::new();
                    for hash in hashes.iter() {
                        if !mempool_mutex.tx_map.contains_key(hash){
                            transactions_new.push(hash.clone());
                        }
                    }
                    if transactions_new.len() > 0 {
                        peer.write(Message::GetTransactions(transactions_new));
                        println!("request new txs");
                    }
                }
                Message::GetTransactions(hashes) => {
                    println!("receive req get txs");
                    let mempool_mutex = self.mempool.lock().unwrap();
                    // vector to store requested blocks
                    let mut transactions = Vec::new();

                    // let mut map = mempool_mutex.tx_map;
                    for hash in hashes.iter() {
                        if mempool_mutex.tx_map.contains_key(hash){
                            transactions.push(mempool_mutex.tx_map[hash].clone());
                        }
                    }
                    if transactions.len() > 0 {
                        peer.write(Message::Transactions(transactions));
                        println!("return all txs that are requested");
                    }

                }
                Message::Transactions(signedtransactions) => {
                    println!("receive req txs");
                    let mut mempool_mutex = self.mempool.lock().unwrap();

                    let mut transactions_new = Vec::new();

                    for tx in signedtransactions {
                        
                        let t_hash = tx.hash();
                        let public_key_tx = tx.public_key;
                        let signature_tx = tx.signature;
                        let transaction = tx.transcation;
                        // verify with the publickey
                        // if !verify(&transaction, &public_key_tx, &signature_tx){
                        //     continue;
                        // }
                        // add check here
                        if !mempool_mutex.tx_map.contains_key(&t_hash){
                            let clone_tx = transaction.clone();
                            let clone_signed_tx = SignedTransaction{public_key: public_key_tx.clone(), signature: signature_tx.clone(), transcation: clone_tx};
                            mempool_mutex.insert(&clone_signed_tx);
                            println!("adding new txs in mempool");
                            transactions_new.push(t_hash);
                        }
                    }
                    if transactions_new.len() > 0 {
                        peer.write(Message::NewTransactionHashes(transactions_new));
                        println!("inserting some in mempool, tell others adding some new txs");
                    }
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
    let fake_orphan_buffer = HashMap::new();
    let orphan_buffer: Arc<Mutex<HashMap<H256, Block>>> = Arc::new(Mutex::new(fake_orphan_buffer));
    let fake_mempool = Mempool::new();
    let mempool: Arc<Mutex<Mempool>> = Arc::new(Mutex::new(fake_mempool));
    let worker = Worker::new(1, msg_chan, &server, &blockchain, &buffer, &orphan_buffer, &mempool);
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
