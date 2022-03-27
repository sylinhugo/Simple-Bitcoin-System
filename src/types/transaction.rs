extern crate ring;

use super::address::Address;
use crate::types::block::Block;
use crate::types::hash::{Hashable, H256};
use rand::Rng;
use ring::digest::{self, Context, Digest, SHA256};
use ring::signature::{
    self, Ed25519KeyPair, EdDSAParameters, KeyPair, Signature, VerificationAlgorithm,
};
use serde::{Deserialize, Serialize};
use std::cmp;
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    pub sender: Address,
    pub receiver: Address,
    pub value: u32,
    pub input: Vec<UTXO_input>,
    pub output: Vec<UTXO_output>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    // follow the definition in Midterm1 handout
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub transcation: Transaction,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Eq, PartialEq, Hash)]
// According to midproj5, add UTXO format into transcation!
pub struct UTXO_input {
    pub prev_tx_hash: H256,
    pub index: u8,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UTXO_output {
    pub receipient_address: Address,
    pub value: u64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct State {
    pub state_map: HashMap<(H256, u8), (u64, Address)>,
}

impl State {
    pub fn new() -> Self {
        let state_map = HashMap::new();

        State {
            state_map: state_map,
        }
    }

    // According to the definition of state update in README
    pub fn update(&mut self, signed_transaction: &SignedTransaction) {
        let transcation = signed_transaction.transcation.clone();
        for transcation_input in transcation.input {
            self.state_map
                .remove(&(transcation_input.prev_tx_hash, transcation_input.index));
        }

        let mut idx = 0;
        for transcation_output in transcation.output {
            let transcation_hash = signed_transaction.hash();
            self.state_map.insert(
                (transcation_hash, idx),
                (
                    transcation_output.value,
                    transcation_output.receipient_address,
                ),
            );
            idx += 1;
        }
    }
}

pub struct StatePerBlock {
    pub state_block_map: HashMap<H256, State>,
}

impl StatePerBlock {
    pub fn new() -> Self {
        return StatePerBlock {
            state_block_map: HashMap::new(),
        };
    }

    // NOT SURE!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    pub fn initial_coin_offering(&mut self, h: H256) {
        let mut ico_state = State::new();

        let mut rng = rand::thread_rng();
        let index: u8 = rng.gen();
        let value: u64 = rng.gen();
        let mut address_array = [0u8; 20];
        for i in 0..20 {
            address_array[i] = rng.gen();
        }
        let fake_address = Address::new(address_array);
        let rand_num: u8 = rng.gen();
        let previous_output: H256 = [rand_num; 32].into();

        ico_state
            .state_map
            .insert((previous_output, index), (value, fake_address));

        self.state_block_map.insert(h, ico_state);
    }

    pub fn update(&mut self, tip: H256, block: &Block) {
        let mut newest_state = self.state_block_map[&tip].clone();
        let transactions = block.content.content.clone();

        for transaction in &transactions {
            newest_state.update(&transaction);
        }

        self.state_block_map.insert(block.hash(), newest_state);
    }
}

#[derive(Debug, Default, Clone)]
pub struct Mempool {
    pub deque: VecDeque<H256>,
    pub tx_map: HashMap<H256, SignedTransaction>,
}

impl Mempool {
    pub fn new() -> Self {
        return Mempool {
            deque: VecDeque::new(),
            tx_map: HashMap::new(),
        };
    }

    pub fn insert(&mut self, t: &SignedTransaction) {
        let t_hash = t.hash();

        // already exists
        if (self.tx_map.contains_key(&t_hash)) {
            return;
        }
        self.deque.push_back(t_hash);
        self.tx_map.insert(t_hash, t.clone());
    }

    pub fn get_headtransactions(&self) -> Vec<SignedTransaction> {
        let count = cmp::min(20, self.deque.len());
        self.deque
            .iter()
            .take(count as usize)
            .map(|h| self.tx_map.get(h).unwrap().clone())
            .collect()
    }

    pub fn remove(&mut self, t: &SignedTransaction) {
        let t_hash = t.hash();
        if (self.tx_map.contains_key(&t_hash)) {
            self.tx_map.remove(&t_hash);
        }
    }
}

// According to Midterm1, impl Hashable for SignedTranscation
impl Hashable for SignedTransaction {
    fn hash(&self) -> H256 {
        let serial_res = bincode::serialize(&self).unwrap();
        let res: H256 = digest::digest(&digest::SHA256, serial_res.as_ref()).into();
        res
    }
}

/// Create digital signature of a transaction
pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
    // Because key.sign() only accept &[u8], so we need to figure out how to convert t into &[u8] type
    // Convert to Vec<u8> first
    let serial_res = bincode::serialize(t).unwrap();

    // Convert to &Vec<u8>
    // let u8_serial_res = &serial_res;
    let u8_serial_res_2: &[u8] = &serial_res;
    // let hash_txt = digest::digest(&digest::SHA256, serial_res.as_ref()).as_ref();
    let signature: Signature = key.sign(u8_serial_res_2);
    signature
}

// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &Vec<u8>, signature: &Vec<u8>) -> bool {
    // Because in UnparsedPublicKey, the verify need to accept &[u8] as parameter
    // so we need to convert it
    let serial_res = bincode::serialize(t).unwrap();
    let u8_serial_res_2: &[u8] = &serial_res;

    // Verify the signature of the message using the public key. Normally the
    // verifier of the message would parse the inputs to this code out of the
    // protocol message(s) sent by the signer.
    let public_key_ = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
    let res = public_key_
        .verify(u8_serial_res_2, signature.as_ref())
        .is_ok();
    res
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    use crate::types::{address, key_pair};
    use std::{convert::TryInto, ops::Add};

    let mut rng = rand::thread_rng();
    let mut sender = Vec::<u8>::with_capacity(20);
    let mut receiver = Vec::<u8>::with_capacity(20);
    let mut address_array = [0u8; 20];
    for i in 0..20 {
        sender.push(rng.gen());
        receiver.push(rng.gen());
        address_array[i] = rng.gen();
    }

    // assemble utx0_input
    // let key = key_pair::random();
    // let public_key = key.public_key();
    // let pb_hash: H256 = digest::digest(&digest::SHA256, public_key.as_ref()).into();
    let fake_address = Address::new(address_array);
    let value: u64 = rng.gen();
    let fake_utxo_out = UTXO_output {
        receipient_address: fake_address,
        value: value,
    };

    let rand_num: u8 = rng.gen();
    let previous_output: H256 = [rand_num; 32].into();
    let index: u8 = rng.gen();
    let fake_utxo_in = UTXO_input {
        prev_tx_hash: previous_output,
        index: index,
    };

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
    transc
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;
    use serde_json::to_vec;

    #[test]
    fn sign_verify() {
        // let t = generate_random_transaction();
        // let key = key_pair::random();
        // let signature = sign(&t, &key);
        // assert!(verify(&t, &(key.public_key()), &signature));

        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(
            &t,
            &(key.public_key().as_ref().to_vec()),
            &signature.as_ref().to_vec()
        ));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(
            &t_2,
            &key.public_key().as_ref().to_vec(),
            &signature.as_ref().to_vec()
        ));
        assert!(!verify(
            &t,
            &key_2.public_key().as_ref().to_vec(),
            &signature.as_ref().to_vec()
        ));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
