// use super::block::{self, Block};
use super::transaction::StatePerBlock;
use crate::blockchain::Blockchain;
use crate::network::message::Message;
// use crate::network::peer;
use crate::network::server::Handle as ServerHandle;
use crate::types::address::Address;
use crate::types::hash::{Hashable, H256};
use crate::types::transaction::{sign, SignedTransaction, Transaction, UTXO_input, UTXO_output};
use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
// use futures::AsyncWriteExt;
use log::{debug, info, warn};
// use rand::seq::{SliceRandom, index};
use rand::Rng;
use ring::signature::{
    self, Ed25519KeyPair, EdDSAParameters, KeyPair, Signature, VerificationAlgorithm,
};
use ring::{digest, rand::SystemRandom};
// use std::ptr::null;
use crate::types::{address, key_pair};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use std::{convert::TryInto, ops::Add};

// enum class that supports message in channel
enum ControlSignal {
    Start(u64, u16), // the number controls the theta of interval between block generation
    Update,          // change name Gen to Update
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64, u16),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    state_per_block: Arc<Mutex<StatePerBlock>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the generator thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(
    server: &ServerHandle,
    blockchain: &Arc<Mutex<Blockchain>>,
    state_per_block: &Arc<Mutex<StatePerBlock>>,
) -> (Context, Handle) {
    // bound receiver and sender to comunication in channels
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        server: server.clone(),
        blockchain: blockchain.clone(),
        state_per_block: state_per_block.clone(),
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

    pub fn start(&self, theta: u64, port_number: u16) {
        self.control_chan
            .send(ControlSignal::Start(theta, port_number))
            .unwrap();
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
        let mut addr_index: u16 = 0;

        // let rngg = SystemRandom::new();
        // let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rngg).unwrap();
        // let key1 = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref().into()).unwrap();

        // let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rngg).unwrap();
        // let key2 = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref().into()).unwrap();

        // let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rngg).unwrap();
        // let key3 = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref().into()).unwrap();

        // // get addr1
        // let public_key_hash1 = digest::digest(&digest::SHA256, key1.public_key().as_ref());
        // let mut tmp_address1 = [0u8; 20];
        // tmp_address1.copy_from_slice(&(public_key_hash1.as_ref()[0..20]));
        // let addr1: Address = (tmp_address1).into();
        // // get addr2
        // let public_key_hash2 = digest::digest(&digest::SHA256, key2.public_key().as_ref());
        // let mut tmp_address2 = [0u8; 20];
        // tmp_address2.copy_from_slice(&(public_key_hash2.as_ref()[0..20]));
        // let addr2: Address = (tmp_address2).into();
        // // get addr3
        // let public_key_hash3 = digest::digest(&digest::SHA256, key3.public_key().as_ref());
        // let mut tmp2_address3 = [0u8; 20];
        // tmp2_address3.copy_from_slice(&(public_key_hash3.as_ref()[0..20]));
        // let addr3: Address = (tmp2_address3).into();

        let fake_pkcs8_1 = [
            48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 233, 77, 246, 201, 167, 107,
            216, 106, 66, 251, 234, 64, 21, 147, 230, 47, 60, 94, 121, 184, 191, 244, 54, 34, 105,
            240, 129, 214, 231, 89, 151, 251, 161, 35, 3, 33, 0, 142, 102, 135, 38, 117, 18, 39,
            110, 156, 134, 54, 214, 125, 16, 18, 15, 0, 138, 121, 106, 130, 231, 57, 135, 201, 252,
            122, 104, 160, 135, 37, 22,
        ];
        let fake_pkcs8_2 = [
            48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 221, 160, 0, 69, 142, 238,
            92, 231, 167, 252, 227, 123, 243, 51, 8, 112, 159, 115, 176, 53, 121, 76, 64, 36, 99,
            158, 201, 83, 40, 193, 170, 90, 161, 35, 3, 33, 0, 46, 208, 26, 27, 143, 84, 111, 162,
            182, 237, 87, 114, 174, 244, 111, 85, 43, 207, 177, 86, 152, 163, 72, 149, 245, 214,
            81, 95, 128, 149, 87, 177,
        ];
        let fake_pkcs8_3 = [
            48, 83, 2, 1, 1, 48, 5, 6, 3, 43, 101, 112, 4, 34, 4, 32, 75, 123, 64, 228, 183, 56,
            33, 50, 44, 73, 244, 34, 178, 89, 108, 50, 191, 37, 170, 254, 186, 221, 226, 111, 158,
            93, 134, 75, 127, 91, 244, 71, 161, 35, 3, 33, 0, 46, 5, 254, 114, 71, 33, 136, 70,
            218, 28, 186, 52, 144, 117, 214, 72, 15, 23, 187, 67, 169, 127, 42, 99, 16, 249, 29,
            107, 196, 71, 0, 132,
        ];
        let key1 = Ed25519KeyPair::from_pkcs8(fake_pkcs8_1.as_ref().into()).unwrap();
        let public_key_hash1 = digest::digest(&digest::SHA256, key1.public_key().as_ref());
        let mut tmp_address1 = [0u8; 20];
        tmp_address1.copy_from_slice(&(public_key_hash1.as_ref()[0..20]));
        let addr1: Address = (tmp_address1).into();

        let key2 = Ed25519KeyPair::from_pkcs8(fake_pkcs8_2.as_ref().into()).unwrap();
        let public_key_hash1 = digest::digest(&digest::SHA256, key2.public_key().as_ref());
        let mut tmp_address1 = [0u8; 20];
        tmp_address1.copy_from_slice(&(public_key_hash1.as_ref()[0..20]));
        let addr2: Address = (tmp_address1).into();

        let key3 = Ed25519KeyPair::from_pkcs8(fake_pkcs8_3.as_ref().into()).unwrap();
        let public_key_hash1 = digest::digest(&digest::SHA256, key3.public_key().as_ref());
        let mut tmp_address1 = [0u8; 20];
        tmp_address1.copy_from_slice(&(public_key_hash1.as_ref()[0..20]));
        let addr3: Address = (tmp_address1).into();

        println!("get 3 address");
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
                        ControlSignal::Start(i, j) => {
                            // info!("Generator starting in continuous mode with theta {}", i);
                            addr_index = j;
                            self.operating_state = OperatingState::Run(i, j);
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
                            ControlSignal::Start(i, j) => {
                                info!("Generator starting in continuous mode with theta {}", i);
                                self.operating_state = OperatingState::Run(i, j);
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
            let locked_state_per_block = self.state_per_block.lock().unwrap();

            let mut used_tx = mempool_locked.used_tx.clone();
            // println!("Generate a transaction, size of mempool 0 {}", mempool_locked.deque.len());
            // assemble a fake transaction

            let mut rng = rand::thread_rng(); // thread to generate random integers
            let mut sender = Vec::<u8>::with_capacity(20); // no use
            let mut receiver = Vec::<u8>::with_capacity(20); // no use

            // randomly generate a recipient address
            let rand_address_idx: u8 = rng.gen();

            let rand_recip_addr: Address;
            if rand_address_idx % 3 == 0 {
                rand_recip_addr = addr1;
            } else if rand_address_idx % 3 == 1 {
                rand_recip_addr = addr2;
            } else {
                rand_recip_addr = addr3;
            }
            println!("rand_recip_addr is: {:?}", rand_recip_addr);
            println!(
                "Generate a transaction, size of mempool {}",
                mempool_locked.deque.len()
            );
            // println!("add   s{:?}", rand_recip_addr);

            // local_addr of this port
            let mut local_addr = addr1;
            if addr_index == 7001 {
                local_addr = addr2
            } else if addr_index == 7002 {
                local_addr = addr3;
            }
            println!("local_addr   {:?}", local_addr);

            for i in 0..20 {
                sender.push(rng.gen());
                receiver.push(rng.gen());
            }

            // get to the newest state to avoid state
            let mut aval_amount = 0;
            let newest_state = &locked_state_per_block.state_block_map[&blockchain_mtx.tip()];

            for key in newest_state.state_map.keys() {
                // get the transfer receiver addr

                let val = &newest_state.state_map[key].clone();
                if used_tx.contains(val) {
                    continue;
                }
                println!("avaible utxo input here");
                let prev_tx_receiver = val.receipient_address;

                // available previous txs, add total_num
                if prev_tx_receiver == local_addr {
                    let mut sender = Vec::<u8>::with_capacity(20); // no use
                    let mut receiver = Vec::<u8>::with_capacity(20); // no use
                    for i in 0..20 {
                        sender.push(rng.gen());
                        receiver.push(rng.gen());
                    }

                    aval_amount += val.value;
                    println!("total available amount this utxo  {}", aval_amount);
                    // NOT SURE YET!!!!!!!!!!!!!!!!!!!

                    let one_input_UTXO = UTXO_input {
                        prev_tx_hash: key.prev_tx_hash,
                        index: key.index,
                    }
                    .clone();
                    let mut utxo_in_vec: Vec<UTXO_input> = Vec::new();
                    utxo_in_vec.push(one_input_UTXO);

                    // transfer all the amount to another
                    let utxo_out = UTXO_output {
                        receipient_address: rand_recip_addr,
                        value: aval_amount,
                    };

                    // assemble utxo_output
                    let mut utxo_out_vec = Vec::new();
                    utxo_out_vec.push(utxo_out);
                    // utxo_out_vec.push(utxo_out_2);

                    // sender and receiver, two paras in transaction, fake here
                    let sender_addr: [u8; 20] = sender.try_into().unwrap();
                    let receiver_addr: [u8; 20] = receiver.try_into().unwrap();

                    let transc = Transaction {
                        sender: Address::new(sender_addr),
                        receiver: Address::new(receiver_addr),
                        value: 2,
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

                    println!("get a txs");
                    mempool_locked.insert(&signed_tx);
                    println!("add txs into mempool");

                    println!(
                        "Generate a transaction, size of mempool {}",
                        mempool_locked.deque.len()
                    );
                    let signed_tx_hash: H256 = signed_tx.hash();
                    // broadcast new signedtx inserted
                    // println!("new_transaction hash");
                    self.server
                        .broadcast(Message::NewTransactionHashes(vec![signed_tx_hash]));
                    println!("broadcast txs");
                    // used_tx.insert(&val);
                    used_tx.insert(val.clone());
                }
            }
            // assemble utxo_out, random
            // transfer 2 to rand_addr

            drop(mempool_locked);
            drop(blockchain_mtx);

            // add assembled random signedtransaction into mempool
            // println!("mempool size {}", mempool_locked.deque.len());

            if let OperatingState::Run(i, j) = self.operating_state {
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
