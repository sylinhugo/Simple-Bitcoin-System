extern crate ring;

use std::collections::VecDeque;

use crate::types::hash::{Hashable, H256, H160};
// use crate::crypto::hash::{H256, Hashable, H160};
use rand::Rng;
use ring::digest::{self, Context, Digest, SHA256};
use ring::signature::{
    self, Ed25519KeyPair, EdDSAParameters, KeyPair, Signature, VerificationAlgorithm,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::address::Address;
// use super::hash::{Hashable, H256};






#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    sender: Address,
    receiver: Address,
    value: u32,
    input: Vec<UTXO_input>,
    output: Vec<UTXO_output>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    // ollow the definition in Midterm1 handout
    public_key: Vec<u8>,
    signature: Vec<u8>,
    transcation: Transaction,
}


// According to Midterm1, impl Hashable for Transcation
// impl Hashable for Transaction {
//     fn hash(&self) -> H256 {
//         let serial_res = bincode::serialize(&self).unwrap();
//         let res: H256 = digest::digest(&digest::SHA256, serial_res.as_ref()).into();
//         res
//     }
// }

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

/// Verify digital signature of a transaction, using public key instead of secret key
pub fn verify(t: &Transaction, public_key: &[u8], signature: &[u8]) -> bool {
    // Because in UnparsedPublicKey, the verify need to accept &[u8] as parameter
    // so we need to convert it
    let serial_res = bincode::serialize(t).unwrap();
    let u8_serial_res_2: &[u8] = &serial_res;

    // Verify the signature of the message using the public key. Normally the
    // verifier of the message would parse the inputs to this code out of the
    // protocol message(s) sent by the signer.
    let public_key_ = signature::UnparsedPublicKey::new(&signature::ED25519, public_key.as_ref());
    let res = public_key_
        .verify(u8_serial_res_2, signature.as_ref())
        .is_ok();
    res
}


#[cfg(any(test, test_utilities))]
pub fn generate_random_transaction() -> Transaction {
    use std::{convert::TryInto};
    use crate::types::key_pair;
    let mut rng = rand::thread_rng();
    let mut sender = Vec::<u8>::with_capacity(20);
    let mut receiver = Vec::<u8>::with_capacity(20);

    for _ in 0..20 {
        sender.push(rng.gen());
        receiver.push(rng.gen());
    }
    // assemble utx0_input
    let key = key_pair::random();
    let public_key = key.public_key();
    let pb_hash: H256 = digest::digest(&digest::SHA256, public_key.as_ref()).into();
    let recipient: H160 = pb_hash.to_addr().into();
    let value: u64 = rng.gen();
    let fake_utxo_out = UTXO_output{receipient_address: recipient, value: value};

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
    transc
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::key_pair;
    use ring::signature::KeyPair;

    #[test]
    fn sign_verify() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
    }
    #[test]
    fn sign_verify_two() {
        let t = generate_random_transaction();
        let key = key_pair::random();
        let signature = sign(&t, &key);
        let key_2 = key_pair::random();
        let t_2 = generate_random_transaction();
        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST


#[derive(Serialize, Deserialize, Debug, Default,Clone, Eq, PartialEq, Hash)]
pub struct UTXO_input{
  pub prev_tx_hash: H256,
  pub index: u8,  
}
// impl UTXO_input {
//     pub fn new() -> Self {
//         let pb_hash: H256 = digest::digest(&digest::SHA256, public_key.as_ref()).into();
//         return UTXO_input{prev_tx_hash: pb_hash, index: 0u8};
//     }
// }

#[derive(Serialize, Deserialize, Debug, Default,Clone)]
pub struct UTXO_output{
  pub receipient_address: H160,
  pub value: u64,
}

#[derive(Debug, Default, Clone)]
pub struct Mempool{
    // pub queue: VecDeque<H256>,
    pub deque: VecDeque<H256>,
    // pub nonce_map: HashMap<(H160, u32), H256>,
    pub tx_map: HashMap<H256, SignedTransaction>,
}

impl Mempool {
    pub fn new() -> Self {
        return Mempool{deque: VecDeque::new(), tx_map: HashMap::new()}  
    }

    pub fn insert(&mut self, t: &SignedTransaction) {
        let t_hash = t.hash();
        // already exists
        if (self.tx_map.contains_key(&t_hash)){
            return;
        }
        self.deque.push_back(t_hash);
        self.tx_map.insert(t_hash, t.clone());
    //     let t_hash = t.hash();
    //     let acc_nonce = (t.transaction.from, t.transaction.nonce);
    //     // reject double spend in mempool
    //     if self.tx_map.contains_key(&t_hash) || self.nonce_map.contains_key(&acc_nonce) {
    //         return;
    //     }
    //     self.nonce_map.insert(acc_nonce, t_hash);
    //     self.tx_map.insert(t_hash, t.clone());
    }

    pub fn remove(&mut self, t: &SignedTransaction) {
        let t_hash = t.hash();
        // let nonce = (t.transaction.from, t.transaction.nonce);
        if (self.tx_map.contains_key(&t_hash)){
            self.tx_map.remove(&t_hash);
        }
        // if (self.tx_map.contains_key(&t_hash)) {
        //     self.tx_map.remove(&t_hash);
        // }
    }
}