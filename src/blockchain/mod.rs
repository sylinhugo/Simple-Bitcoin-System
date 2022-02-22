use crate::types::block::{Block, BlockContent, BlockHeader};
use crate::types::hash::{Hashable, H256};
use crate::types::merkle::MerkleTree;
use std::collections::HashMap;

pub struct Blockchain {
    pub tip: H256,                    // tip is the last block's hash in the longest chain
    pub blocks: HashMap<H256, Block>, // mapping hashing of block and the block
    pub lengths: HashMap<H256, u32>,  // mapping hashing of block and its length index
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new() -> Self {
        // generate elements for a new block
        let parent: H256 = [0u8; 32].into();

        let nonce = 0u32;

        let mut _difficulty = [0u8; 32];
        _difficulty[0] = 16u8;
        let difficulty: H256 = _difficulty.into();

        let timestamp = 0u128;

        let transactions = Vec::new();
        let merkle_tree = MerkleTree::new(&transactions);
        let merkle_root = merkle_tree.root();

        // assemble block
        let header = BlockHeader {
            parent,
            nonce,
            difficulty,
            timestamp,
            merkle_root,
        };
        let content = BlockContent {
            content: transactions,
        };
        let new_block = Block { header, content };
        let _hash = new_block.hash();

        // two hashmap store blocks and lengths
        let mut _blocks = HashMap::new();
        let mut _lengths = HashMap::new();

        _blocks.insert(_hash, new_block);
        _lengths.insert(_hash, 0);

        Self {
            tip: _hash,
            blocks: _blocks,
            lengths: _lengths,
        }
    }

    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) {
        let block_hash = block.hash();
        let new_block = block.clone();

        // Hash collision, return in advanced
        if self.blocks.contains_key(&block_hash) {
            return;
        }

        // Parent of current block
        let cur_parent = block.header.parent;

        // Add the cloned block into blocks map
        self.blocks.insert(block_hash, new_block);

        // Update the length index of new block according to its parent
        self.lengths
            .insert(block_hash, self.lengths[&cur_parent] + 1);

        // Update the tip accroding to longest sub-chain
        if self.lengths[&block_hash] > self.lengths[&self.tip] {
            self.tip = block_hash;
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        return self.tip;
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let mut res = Vec::new();

        // Current block points to the tip
        let mut cur = self.tip();

        let len = self.lengths[&cur];
        let mut i = 0;
        while i < len {
            res.push(cur);

            let block = &self.blocks[&cur];
            cur = block.header.parent;
            i += 1;
        }
        res.reverse();
        res
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    #[test]
    fn insert_one() {
        let mut blockchain = Blockchain::new();
        let genesis_hash = blockchain.tip();
        let block = generate_random_block(&genesis_hash);
        blockchain.insert(&block);
        assert_eq!(blockchain.tip(), block.hash());
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
