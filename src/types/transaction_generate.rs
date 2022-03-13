
use log::{debug, warn, info};
use crate::types::hash::{H256, Hashable, };

use crate::types::transaction::{SignedTransaction, UTXO_output, UTXO_input, Transaction, Mempool, sign};
use crate::types::address::{Address};
use std::thread;
use std::sync::{Arc, Mutex};
use crate::network::server::Handle as ServerHandle;
use crate::network::message::{Message};
use std::time;

use ring::digest;
use ring::signature::{self, Ed25519KeyPair, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};

use rand::Rng;
use rand::seq::SliceRandom;

#[derive(Clone)]
pub struct Context {
    server: ServerHandle,
    mempool: Arc<Mutex<Mempool>>, 
}

pub fn new(
    server: &ServerHandle, // broadcast random transactions
    mempool: &Arc<Mutex<Mempool>>, // add random transactions into mempool
) -> Context {

    let ctx = Context {
        
        server: server.clone(),
        mempool: Arc::clone(mempool),
        
    };  
    ctx
}

impl Context {
    pub fn start(self) {
        thread::spawn(move || {
            self.generator_loop();
        });
    }

    fn generator_loop(&self) {

        // On startup, insert and ICO for yourself into the mempool
        info!("Executing ICO - Generating Sourceless TX");


        let mut mempool_locked = self.mempool.lock().unwrap();


        // Share ICO TX with others - they likely won't mine it, but should have it on hand
        // let signed_tx_hash: H256 = signed_tx.hash();
        // self.server.broadcast(Message::NewTransactionHashes(vec![signed_tx_hash]));

        loop {
            use std::{convert::TryInto, ops::Add};
            use crate::types::{key_pair, address};
            let mut rng = rand::thread_rng();
            let mut sender = Vec::<u8>::with_capacity(20);
            let mut receiver = Vec::<u8>::with_capacity(20);
            let mut address_array = [0u8; 20];
            for i in 0..20 {
                sender.push(rng.gen());
                receiver.push(rng.gen());
                address_array[i] = rng.gen();
            }
           
            let fake_address = Address::new(address_array);
            let value: u64 = rng.gen();
            let fake_utxo_out = UTXO_output{receipient_address: fake_address, value: value};

            let rand_num: u8 = rng.gen();
            let previous_output: H256 = [rand_num; 32].into();
            let index: u8 = rng.gen();
            let fake_utxo_in = UTXO_input { prev_tx_hash: previous_output, index: index };
            
            let utxo_in_vec = vec![fake_utxo_in];
            let utxo_out_vec = vec![fake_utxo_out];

            let sender_addr: [u8; 20] = sender.try_into().unwrap();
            let receiver_addr: [u8; 20] = receiver.try_into().unwrap();

            let transc = Transaction {
                sender: Address::new(sender_addr),
                receiver: Address::new(receiver_addr),
                value: rng.gen(),
                input: utxo_in_vec,
                output: utxo_out_vec,
            };

            let key = key_pair::random();
            let signature = sign(&transc, &key);

            let signed_tx = SignedTransaction{public_key: key.public_key().as_ref().to_vec(), signature: signature.as_ref().to_vec(), transcation: transc};

            mempool_locked.insert(&signed_tx);
            let signed_tx_hash: H256 = signed_tx.hash();
            self.server.broadcast(Message::NewTransactionHashes(vec![signed_tx_hash]));
            thread::sleep(time::Duration::from_millis(1000));
        }
    }
}
