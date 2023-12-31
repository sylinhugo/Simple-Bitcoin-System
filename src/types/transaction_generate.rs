use super::block::{self, Block};
use crate::blockchain::Blockchain;
use crate::network::message::Message;
use crate::network::peer;
use crate::network::server::Handle as ServerHandle;
use crate::types::address::Address;
use crate::types::hash::{self, Hashable, H256};
use crate::types::transaction::{
    sign, Mempool, SignedTransaction, Transaction, UTXO_input, UTXO_output,
};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use futures::AsyncWriteExt;
use log::{debug, info, warn};
use rand::distributions::Open01;
use rand::seq::SliceRandom;
use rand::Rng;
use ring::digest;
use ring::signature::{
    self, Ed25519KeyPair, EdDSAParameters, KeyPair, Signature, VerificationAlgorithm,
};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

// enum class that supports message in channel
enum ControlSignal {
    Start(u64), // the number controls the theta of interval between block generation
    Update,     // change name Gen to Update
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
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the generator thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(server: &ServerHandle, blockchain: &Arc<Mutex<Blockchain>>) -> (Context, Handle) {
    // bound receiver and sender to comunication in channels
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: blockchain.clone(),
    };
    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, theta: u64) {
        self.control_chan.send(ControlSignal::Start(theta)).unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("transaction_gene".to_string())
            .spawn(move || {
                self.generator_loop();
            })
            .unwrap();
        info!("generator initialized into paused mode");
    }

    fn generator_loop(&mut self) {
        use crate::types::{address, key_pair};
        use std::{convert::TryInto, ops::Add};

        // On startup, insert and ICO for yourself into the mempool
        info!("Executing ICO - Generating Sourceless TX");

        // Share ICO TX with others - they likely won't mine it, but should have it on hand
        // let signed_tx_hash: H256 = signed_tx.hash();
        // self.server.broadcast(Message::NewTransactionHashes(vec![signed_tx_hash]));
        loop {
            // print!("matching state");
            match self.operating_state {
                OperatingState::Paused => {
                    // receive control signal from channels
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Generator shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Generator starting in continuous mode with theta {}", i);
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
                                info!("Generator shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Generator starting in continuous mode with theta {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                unimplemented!()
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Generator control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }
            let blockchain_mtx = self.blockchain.lock().unwrap();
            let mut mempool_locked = blockchain_mtx.mempool.lock().unwrap();
            // println!("Generate a transaction, size of mempool 0 {}", mempool_locked.deque.len());
            use crate::types::{address, key_pair};
            use std::{convert::TryInto, ops::Add};
            // assemble a fake transaction
            let mut rng = rand::thread_rng();
            let mut sender = Vec::<u8>::with_capacity(20);
            let mut receiver = Vec::<u8>::with_capacity(20);
            let mut address_array = [0u8; 20];
            for i in 0..20 {
                sender.push(rng.gen());
                receiver.push(rng.gen());
                address_array[i] = rng.gen();
            }
            // assemble utxo_out, random
            let fake_address = Address::new(address_array);
            let value: u64 = rng.gen();
            let fake_utxo_out = UTXO_output {
                receipient_address: fake_address,
                value: value,
            };
            // assemble utxo_input random
            let rand_num: u8 = rng.gen();
            let previous_output: H256 = [rand_num; 32].into();
            let index: u8 = rng.gen();
            let fake_utxo_in = UTXO_input {
                prev_tx_hash: previous_output,
                index: index,
            };

            let utxo_in_vec = vec![fake_utxo_in];
            let utxo_out_vec = vec![fake_utxo_out];
            // sender and receiver, two paras in transaction
            let sender_addr: [u8; 20] = sender.try_into().unwrap();
            let receiver_addr: [u8; 20] = receiver.try_into().unwrap();

            let transc = Transaction {
                sender: Address::new(sender_addr),
                receiver: Address::new(receiver_addr),
                value: rng.gen(),
                input: utxo_in_vec,
                output: utxo_out_vec,
            };
            // use public key to sign a transaction
            let key = key_pair::random();
            let signature = sign(&transc, &key);
            // assemble to a signedtransaction
            let signed_tx = SignedTransaction {
                public_key: key.public_key().as_ref().to_vec(),
                signature: signature.as_ref().to_vec(),
                transcation: transc,
            };
            // add assembled random signedtransaction into mempool
            // println!("mempool size {}", mempool_locked.deque.len());
            println!(
                "Generate a transaction, size of mempool 1 {}",
                mempool_locked.deque.len()
            );
            mempool_locked.insert(&signed_tx);
            let signed_tx_hash: H256 = signed_tx.hash();
            // broadcast new signedtx inserted
            // println!("new_transaction hash");
            self.server
                .broadcast(Message::Transactions(vec![signed_tx]));
            drop(mempool_locked);
            drop(blockchain_mtx);
            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_millis(i / 10 as u64);
                    thread::sleep(interval);
                }
            }

            // println!("Generate a transaction, size of mempool 2 {}", mempool_locked.deque.len());
            // println!("Generate a transaction, size of map {}", mempool_locked.tx_map.len());
            // thread::sleep(time::Duration::from_millis(1000));
        }
    }
}
