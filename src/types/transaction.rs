extern crate ring;

use super::address::Address;
use crate::types::block::Block;
use crate::types::hash::{Hashable, H256};
// use crate::types::{address, key_pair};
// use rand::seq::index;
use rand::Rng;
// use ring::agreement::PublicKey;
use ring::digest::{self};
// use ring::pkcs8::Document;
// use ring::rand::SystemRandom;
use ring::signature::{self, Ed25519KeyPair, KeyPair, Signature};
use serde::{Deserialize, Serialize};
use std::cmp;
use std::collections::VecDeque;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::{convert::TryInto, ops::Add};

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

#[derive(Serialize, Deserialize, Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct UTXO_output {
    pub receipient_address: Address,
    pub value: u64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct State {
    pub state_map: HashMap<UTXO_input, UTXO_output>,
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
            self.state_map.remove(&transcation_input);
        }

        let mut idx = 0;
        for transcation_output in transcation.output {
            let transcation_hash = signed_transaction.hash();
            let utxo_tmp = UTXO_input {
                prev_tx_hash: transcation_hash,
                index: idx,
            };
            self.state_map.insert(utxo_tmp, transcation_output);
            idx += 1;
        }
    }

    // NOT SURE YET!!!!!!!!!!!!!!!!!!!!
    pub fn double_spending_check(&self, t: SignedTransaction) -> bool {
        let mut res: bool = true;
        for utxo in t.transcation.input {
            if self.state_map.contains_key(&utxo) {
                res = false;
                break;
            }
        }
        res
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
    pub fn initial_coin_offering(&mut self, h: H256, local_addr: String) {
        let mut ico_state = State::new();

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

        let utxo_out_1 = UTXO_output {
            receipient_address: addr1,
            value: 10000,
        };

        // let mut rng = rand::thread_rng();
        // let index: u8 = rng.gen();
        // let value: u64 = 10000;
        // let address_array = [0u8; 20];

        // for i in 0..20 {
        //     address_array[i] = rng.gen();
        // }
        // let fake_address = Address::new(address_array);

        let index = 0;
        let rand_num = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        let previous_output: H256 = rand_num.into();

        // let UTXO_intmp = UTXO_input {
        //     prev_tx_hash: previous_output,
        //     index: index,
        // };

        let utxo_intmp = UTXO_input {
            prev_tx_hash: previous_output,
            index: index,
        };

        // let UTXO_outtmp = UTXO_output{value: value, receipient_address: fake_address};
        ico_state.state_map.insert(utxo_intmp, utxo_out_1);

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
    pub used_tx: HashSet<UTXO_output>,
}

impl Mempool {
    pub fn new() -> Self {
        return Mempool {
            deque: VecDeque::new(),
            tx_map: HashMap::new(),
            used_tx: HashSet::new(),
        };
    }

    pub fn insert(&mut self, t: &SignedTransaction) {
        let t_hash = t.hash();

        // already exists
        if self.tx_map.contains_key(&t_hash) {
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
        if self.tx_map.contains_key(&t_hash) {
            self.tx_map.remove(&t_hash);
        }
    }

    pub fn restore_transaction(&mut self, t: &SignedTransaction) {
        self.insert(t);
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
    // use serde_json::to_vec;

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
