use super::hash::{Hashable, H256};
use ring::digest::{self, Context, Digest, SHA256};
/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    length: usize,
    height: usize,
    nodes: Vec<H256>,
    root: H256,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self
    where
        T: Hashable,
    {
        if data.is_empty() {
            let merkel = MerkleTree {
                length: 0,
                height: 0,
                nodes: Vec::new(),
                root: H256::from([0; 32]),
            };
            return merkel;
        }

        // calculate the leaf, height of the merkle tree
        let mut new_Vec = Vec::new();
        let h = (data.len() as f64).log2().ceil() as u32;
        let leaf_n = i32::pow(2, h) as usize;
        let length = leaf_n;

        // construct the leaf layer of the merkel tree
        // last layer contains 2 ** h
        let mut i: usize = 0;
        while i < leaf_n {
            if i < data.len() {
                new_Vec.push(data[i].hash());
            } else {
                new_Vec.push(data[data.len() - 1].hash());
            }
            i = i + 1;
        }

        let height = h + 1;
        let total_num = (i32::pow(2, height) - 1) as usize;
        let mut nodes = vec![new_Vec[0]; total_num];

        let mut node_idx = total_num - new_Vec.len();
        let mut data_idx = 0;

        // load the leaf layer if the merkle in nodes
        while node_idx < total_num && data_idx < new_Vec.len() {
            nodes[node_idx] = new_Vec[data_idx];
            node_idx += 1;
            data_idx += 1;
        }

        let mut cur = (total_num - new_Vec.len() - 1) as i32;
        while cur >= 0 {
            let left_child = nodes[(cur as usize) * 2 + 1].as_ref();
            let right_child = nodes[(cur as usize) * 2 + 2].as_ref();

            let mut context = Context::new(&SHA256);
            context.update(&left_child[..]);
            context.update(&right_child[..]);
            let buffer_hash = context.finish();
            let parent = H256::from(buffer_hash);
            nodes[cur as usize] = parent;
            cur -= 1;
        }

        let root = nodes[0];
        MerkleTree {
            length: length,
            height: height as usize,
            nodes: nodes,
            root: root,
        }
    }

    pub fn root(&self) -> H256 {
        return self.root;
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut real_idx = index + self.nodes.len() - self.length;
        let mut res_p: Vec<H256> = Vec::new();
        let mut level = 1;
        while level < self.height {
            if real_idx % 2 == 0 {
                res_p.push(self.nodes[real_idx - 1]);
            } else {
                res_p.push(self.nodes[real_idx + 1]);
            }
            real_idx = (real_idx - 1) / 2;
            level += 1;
        }
        res_p
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    let mut my_root = *datum;
    let h = (leaf_size as f64).log2().ceil() as u32;
    let node_n = (i32::pow(2, h + 1) - 1) as usize;
    let mut converted_index = index + node_n - leaf_size;
    let mut cur = 0;
    while cur < proof.len() {
        let mut context = Context::new(&SHA256);
        if converted_index % 2 == 0 {
            // context.update(&<[u8; 32]>::from(proof[count]));
            // context.update(&<[u8; 32]>::from(val));
            context.update(proof[cur].as_ref());
            context.update(my_root.as_ref());
        } else {
            // context.update(&<[u8; 32]>::from(val));
            // context.update(&<[u8; 32]>::from(proof[count]));
            context.update(my_root.as_ref());
            context.update(proof[cur].as_ref());
        }
        my_root = H256::from(context.finish());
        cur += 1;
        converted_index = (converted_index - 1) / 2;
    }
    return my_root == *root;
}
// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::hash::H256;

    macro_rules! gen_merkle_tree_data {
        () => {{
            vec![
                (hex!("0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d")).into(),
                (hex!("0101010101010101010101010101010101010101010101010101010101010202")).into(),
            ]
        }};
    }

    #[test]
    fn merkle_root() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let root = merkle_tree.root();
        assert_eq!(
            root,
            (hex!("6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920")).into()
        );
        // "b69566be6e1720872f73651d1851a0eae0060a132cf0f64a0ffaea248de6cba0" is the hash of
        // "0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d0a0b0c0d0e0f0e0d"
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
        // "6b787718210e0b3b608814e04e61fde06d0df794319a12162f287412df3ec920" is the hash of
        // the concatenation of these two hashes "b69..." and "965..."
        // notice that the order of these two matters
    }

    #[test]
    fn merkle_proof() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert_eq!(
            proof,
            vec![hex!("965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f").into()]
        );
        // "965b093a75a75895a351786dd7a188515173f6928a8af8c9baa4dcff268a4f0f" is the hash of
        // "0101010101010101010101010101010101010101010101010101010101010202"
    }

    #[test]
    fn merkle_verifying() {
        let input_data: Vec<H256> = gen_merkle_tree_data!();
        let merkle_tree = MerkleTree::new(&input_data);
        let proof = merkle_tree.proof(0);
        assert!(verify(
            &merkle_tree.root(),
            &input_data[0].hash(),
            &proof,
            0,
            input_data.len()
        ));
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
