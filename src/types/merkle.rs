

// Add more crates
use ring::digest::{self, Context, Digest, SHA256};

use super::hash::{Hashable, H256};

/// A Merkle tree.
#[derive(Debug, Default)]
pub struct MerkleTree {
    /*
    every node is H256 structure, cause every node is hash
    nodes = total node
    leaf_nums = numbers of leaf
    lavels[0] = number of leaf at level 0, levels[1] = number of leaf at level 1
    levels.len() should be the height of merkletree
     */
    nodes: Vec<H256>,
    leaf_nums: usize,
    levels: Vec<usize>,
}

impl MerkleTree {
    pub fn new<T>(data: &[T]) -> Self
    where
        T: Hashable,
    {
        // unimplemented!()
        let mut data_len = data.len();
        let mut mkltree: Vec<H256> = Vec::new();
        let mut level_num: Vec<usize> = Vec::new();

        // for debugging
        // println!("\n--");
        // println!("Inside new");
        // println!("data_len is: {} ", data_len);

        // Finish this function in advanced, if data_len is zero
        // 2022/02/13 update, I didn't consider corner case before,
        // if input is 0, we should pad one element into it,
        // so the only node in merkle tree is root
        if data_len == 0 {
            mkltree.push([0u8; 32].into());
            level_num.push(1);
            return MerkleTree {
                nodes: mkltree,
                leaf_nums: 1,
                levels: level_num,
            };
        }

        for i in 0..data_len {
            let digest_val = data[i].hash();
            mkltree.push(digest_val);
            // println!("print digest_value: {}", digest_val);
        }

        // if leaf number is odd, we need to add one more leaf
        // to make it become even
        if data_len != 0 && data_len % 2 == 1 {
            let leaf_last: H256 = mkltree[mkltree.len() - 1];
            mkltree.push(leaf_last);
            data_len += 1;
        }
        level_num.push(data_len);

        let mut start = 0;
        let mut tree_len = data_len;
        let mut half = data_len / 2;
        while half > 0 {
            // flag imply whether next level will have odd node or not
            let flag = half % 2;

            for i in 0..half {
                let mut context = Context::new(&SHA256);
                context.update(mkltree[start + 2 * i].as_ref());
                context.update(mkltree[start + 2 * i + 1].as_ref());
                let res = context.finish();
                mkltree.push(res.into());
            }

            // if amount of new generated nodes in next level is odd, we need to add one more to make it even
            // After adding one more node, half should add one more
            // if half=1, means we are at the top of mkltree
            if flag == 1 && half != 1 {
                let last_elem = mkltree[mkltree.len() - 1];
                mkltree.push(last_elem);
                half += 1;
            }

            level_num.push(half);
            start += tree_len;
            tree_len = half;
            half /= 2;
        }

        // for debugging
        // println!(
        //     "mlktree.len() is {} and data_len is {} and level_num is {} and {} {}",
        //     mkltree.len(),
        //     data_len,
        //     level_num.len(),
        //     level_num[0],
        //     level_num[1],
        // );
        // println!("mlktree is {} {} {}", mkltree[0], mkltree[1], mkltree[2]);

        MerkleTree {
            nodes: mkltree,
            leaf_nums: data_len,
            levels: level_num,
        }
    }

    pub fn root(&self) -> H256 {
        // for debugging
        // println!("\n--");
        // println!("Inside root");
        // println!(
        //     "The digest of root is: {}",
        //     self.nodes[self.nodes.len() - 1]
        // );

        return self.nodes[self.nodes.len() - 1];
    }

    /// Returns the Merkle Proof of data at index i
    /// The index start from zero
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let mut proof_res: Vec<H256> = Vec::new();
        let mut height = self.levels.len();
        let mut start = 0;
        let mut level = 0;
        let mut idx = index;

        // for debugging
        // println!("\n--");
        // println!("Inside proof");
        // println!("The idx number is:");
        // println!("{}", idx);

        while height > 1 {
            if idx % 2 == 0 {
                // for debugging
                // println!("Inside idx % 2 == 0");
                // println!("push: {}", self.nodes[start + idx + 1]);

                proof_res.push(self.nodes[start + idx + 1]);
            } else {
                // for debugging
                // println!("Inside idx % 2 == 1");
                // println!("push: {}", self.nodes[start + idx - 1]);

                proof_res.push(self.nodes[start + idx - 1]);
            }
            start += self.levels[level];
            level += 1;
            height -= 1;
            idx /= 2;
        }

        proof_res
    }
}

/// Verify that the datum hash with a vector of proofs will produce the Merkle root. Also need the
/// index of datum and `leaf_size`, the total number of leaves.
pub fn verify(root: &H256, datum: &H256, proof: &[H256], index: usize, leaf_size: usize) -> bool {
    let mut digests = *datum;

    // for debugging
    // println!("\n--");
    // println!("Inside verify");
    // println!(
    //     "root is {} and datum is {} and proof is {:?} and index is {} and leaf_size is {}",
    //     root, datum, proof, index, leaf_size
    // );

    for i in 0..proof.len() {
        let mut context = Context::new(&SHA256);
        context.update(digests.as_ref());
        context.update(proof[i].as_ref());
        let res = context.finish();
        digests = res.into();
    }

    // for debugging
    // println!("digests in verify is {}", digests);

    digests == *root
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
