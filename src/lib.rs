use chrono::prelude::*;
use sha2::{Digest, Sha256};

pub mod block;
pub mod blockchain;
pub mod p2p;

pub use block::*;
pub use blockchain::*;
pub use p2p::*;

pub fn generate_hash(index: u32, prev_hash: &str, timestamp: DateTime<Utc>, data: &str) -> String {
    let hash = Sha256::new()
        .chain(index.to_string())
        .chain(prev_hash)
        .chain(timestamp.to_string())
        .chain(data)
        .finalize();

    hex::encode(hash)
}

pub fn is_valid_block(new_block: &Block, prev_block: &Block) -> bool {
    if prev_block.index + 1 != new_block.index {
        println!("Invalid index");
        return false;
    } else if prev_block.hash != new_block.prev_hash {
        println!("Invalid previous hash");
        return false;
    } else if generate_hash(
        new_block.index,
        &new_block.prev_hash,
        new_block.timestamp,
        &new_block.data,
    ) != new_block.hash
    {
        println!("Invalid hash");
        return false;
    }

    true
}

pub fn is_valid_chain(blockchain: &Blockchain) -> bool {
    for (pos, block) in blockchain.chain.iter().enumerate() {
        // skip the genesis block
        if pos == 0 {
            continue;
        }

        if !is_valid_block(block, &blockchain.chain[pos - 1]) {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_single_block_chain() {
        let genesis = Block::new(0, Utc.timestamp(0, 0), "", &hex::encode([0; 64]));
        let chain = Blockchain::new(genesis);

        assert!(is_valid_chain(&chain));
    }

    #[test]
    fn validates_chain() {
        let genesis = Block::new(0, Utc.timestamp(0, 0), "", &hex::encode([0; 64]));
        let mut chain = Blockchain::new(genesis);

        chain.generate_next_block("new data 0");
        chain.generate_next_block("new data 1");
        chain.generate_next_block("new data 2");
        chain.generate_next_block("new data 3");

        assert!(is_valid_chain(&chain));
    }
}
