pub mod worker;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use rand::Rng;
use smol::unblock;
use std::time;

use std::thread;

use crate::blockchain;
use crate::blockchain::Blockchain;
use crate::types::block::Block;
use crate::types::block::BlockContent;
use crate::types::block::BlockHeader;
use crate::types::hash::Hashable;
use crate::types::hash::H256;
use crate::types::merkle::MerkleTree;
use crate::types::transaction::SignedTransaction;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update,     // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    finished_block_chan: Sender<Block>,
    blockchain: Arc<Mutex<Blockchain>>, // midterm2, according to document, implement this type
    tip: H256, // midterm2, the reason why add this part is from the discusssion on piazza
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(blockchain: &Arc<Mutex<Blockchain>>) -> (Context, Handle, Receiver<Block>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_block_sender, finished_block_receiver) = unbounded();

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
        blockchain: Arc::clone(blockchain),    // midterm2 added
        tip: blockchain.lock().unwrap().tip(), // midterm2 added
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_block_receiver)
}

// According to midterm project2
#[cfg(any(test, test_utilities))]
fn test_new() -> (Context, Handle, Receiver<Block>) {
    let fake_blockchain = Blockchain::new();
    let blockchain = Arc::new(Mutex::new(fake_blockchain));
    new(&blockchain)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn miner_loop(&mut self) {
        // main mining loop

        // Midterm2, uncomment this, although it would pass the test case
        // Maybe will use it in the future
        // let blockchain = self.blockchain.lock().unwrap();
        // let mut block_parent = blockchain.tip();
        // let block_difficulty = [255u8; 32].into();   // Source: GitLab instructions

        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Miner shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Miner starting in continuous mode with lambda {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Update => {
                            // in paused state, don't need to update
                        }
                    };
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        match signal {
                            ControlSignal::Exit => {
                                info!("Miner shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Miner starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                unimplemented!()
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // TODO for student: actual mining, create a block
            // TODO for student: if block mining finished, you can have something like:
            // self.finished_block_chan.send(block.clone()).expect("Send finished block error");

            let blockchain = self.blockchain.lock().unwrap();
            // let mut block_parent = blockchain2.tip();    // Uncomment this, due to test case error
            let block_parent = self.tip;

            let mut rng = rand::thread_rng();
            let block_nonce: u32 = rng.gen();
            // let block_difficulty = blockchain.blocks[&block_parent].header.difficulty;
            let block_difficulty = [255u8; 32].into();

            let block_timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let fake_transcation: Vec<SignedTransaction> = Vec::new();
            let merklt_tree = MerkleTree::new(&fake_transcation);

            let block_header = BlockHeader {
                parent: block_parent,
                nonce: block_nonce,
                difficulty: block_difficulty,
                timestamp: block_timestamp,
                merkle_root: merklt_tree.root(),
            };
            let block_content = BlockContent {
                content: fake_transcation,
            };
            let new_block = Block {
                header: block_header,
                content: block_content,
            };

            if new_block.hash() <= block_difficulty {
                self.finished_block_chan
                    .send(new_block.clone())
                    .expect("Send finished block error");

                // block_parent = new_block.hash();     // this will not work, failed to pass miner_three_block() case
                self.tip = new_block.hash();
            }

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use crate::types::hash::Hashable;
    use ntest::timeout;

    #[test]
    #[timeout(60000)]
    fn miner_three_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..2 {
            let block_next = finished_block_chan.recv().unwrap();
            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
