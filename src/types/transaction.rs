extern crate ring;

use rand::Rng;
use ring::digest::{self, Context, Digest, SHA256};
use ring::signature::{
    self, Ed25519KeyPair, EdDSAParameters, KeyPair, Signature, VerificationAlgorithm,
};
use serde::{Deserialize, Serialize};

use super::address::Address;
use super::hash::{Hashable, H256};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    sender: Address,
    receiver: Address,
    value: u8,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SignedTransaction {
    // ollow the definition in Midterm1 handout
    public_key: Vec<u8>,
    signature: Vec<u8>,
    transcation: Transaction,
}

// According to Midterm1, impl Hashable for Transcation
impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        let serial_res = bincode::serialize(&self).unwrap();
        let res: H256 = digest::digest(&digest::SHA256, serial_res.as_ref()).into();
        res
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
    use std::convert::TryInto;

    let mut rng = rand::thread_rng();
    let mut sender = Vec::<u8>::with_capacity(20);
    let mut receiver = Vec::<u8>::with_capacity(20);

    for _ in 0..20 {
        sender.push(rng.gen());
        receiver.push(rng.gen());
    }

    let sender_addr: [u8; 20] = sender.try_into().unwrap();
    let receiver_addr: [u8; 20] = receiver.try_into().unwrap();

    let transc = Transaction {
        sender: Address::new(sender_addr),
        receiver: Address::new(receiver_addr),
        value: rng.gen(),
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
