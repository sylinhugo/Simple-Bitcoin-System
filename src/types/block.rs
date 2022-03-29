use super::transaction::SignedTransaction;
use crate::types::hash::{Hashable, H256};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::str;
use crate::types::merkle::MerkleTree;
use rand::Rng;

// According to midterm1, add Block, BlockHeader and BlockContent
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub content: BlockContent,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    pub parent: H256,
    pub nonce: u32,
    pub difficulty: H256,
    pub timestamp: u128,
    pub merkle_root: H256,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockContent {
    // Fuck, we didn't implement SignedTranscation in warmup1...
    pub content: Vec<SignedTransaction>,
}

// According to midterm1, imple Hashable for Block
impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.header.hash()
    }
}

// According to midterm1, imple Hashable for BlockHeader
impl Hashable for BlockHeader {
    fn hash(&self) -> H256 {
        let serial_res = bincode::serialize(&self).unwrap();
        let res = ring::digest::digest(&ring::digest::SHA256, &serial_res).into();
        res
    }
}

impl Block {
    // Add this in midterm1, never use it though
    pub fn get_parent(&self) -> H256 {
        let res = self.header.parent;
        res
    }

    // Add this in midterm1, never use it though
    pub fn get_difficulty(&self) -> H256 {
        let res = self.header.difficulty;
        res
    }
}

#[cfg(any(test, test_utilities))]
pub fn generate_random_block(parent: &H256) -> Block {
    

    let mut rng = rand::thread_rng();
    // Generate random nonce
    let block_nonce: u32 = rng.gen();
    // let head_nonce: u8 = 0;

    // //Generate random difficulty, and remember H256 is [u8;32]
    // let mut difficulty = Vec::<u8>::with_capacity(32);
    // for _ in 0..32 {
    //     difficulty.push(head_nonce);
    //     //difficulty.push(rng.gen());
    // }
    let tmp_difficulty: [u8; 32] =[63u8; 32];
    let block_difficulty: H256 = tmp_difficulty.into();

    // Assign current system timestamp to block
    let block_timestamp: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Generate fake transcation for testing
    let fake_content: Vec<SignedTransaction> = Vec::new();

    // Generate merkle tree with fake content
    let merkle_tree: MerkleTree = MerkleTree::new(&fake_content);

    let block_header = BlockHeader {
        parent: *parent,
        nonce: block_nonce,
        difficulty: block_difficulty,
        timestamp: block_timestamp,
        merkle_root: merkle_tree.root(),
    };
    let block_content = BlockContent {
        content: fake_content,
    };
    let block = Block {
        header: block_header,
        content: block_content,
    };
    block
}
