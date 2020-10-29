use chrono::prelude::*;
use sha2::{Digest, Sha256};

pub mod block;
pub mod blockchain;
pub mod p2p;

pub use block::*;
pub use blockchain::*;
pub use p2p::*;

pub const BLOCK_GENERATION_INTERVAL: usize = 10;
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: usize = 10;

pub fn generate_hash(
    index: u32,
    prev_hash: &str,
    timestamp: DateTime<Utc>,
    data: &str,
    nonce: u32,
) -> String {
    let hash = Sha256::new()
        .chain(index.to_string())
        .chain(prev_hash)
        .chain(timestamp.to_string())
        .chain(data)
        .chain(nonce.to_string())
        .finalize();

    hex::encode(hash)
}

pub fn find_block(
    index: u32,
    prev_hash: &str,
    timestamp: DateTime<Utc>,
    data: &str,
    difficulty: usize,
) -> Block {
    let mut nonce = 0u32;

    println!("Mining block:");
    std::thread::sleep(std::time::Duration::from_secs(1));

    let start = std::time::Instant::now();
    loop {
        let hash = generate_hash(index, prev_hash, timestamp, data, nonce);

        println!("{}", hash);

        if hash_matches_difficulty(&hash, difficulty) {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(start);
            println!(
                "\nMined block '{}'\n{} iterations\n{} seconds",
                hash,
                nonce,
                elapsed.as_secs()
            );
            return Block::new(index, timestamp, data, prev_hash, nonce, difficulty);
        }

        nonce += 1;
    }
}

/// Determines if a hash matches a given difficulty
/// The prefixing zeros are checked from the binary format of the hash.
pub fn hash_matches_difficulty(hash: &str, difficulty: usize) -> bool {
    let raw_hash = hex::decode(hash).unwrap();
    let required_prefix = vec![0; difficulty];

    return raw_hash.starts_with(&required_prefix);
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
        new_block.nonce,
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

/// Check difficulty every DIFFICULTY_ADJUSTMENT_INTERVAL
pub fn get_difficulty(blockchain: &Blockchain) -> usize {
    let latest_block = blockchain.get_latest_block();

    if latest_block.index % DIFFICULTY_ADJUSTMENT_INTERVAL as u32 == 0 && latest_block.index != 0 {
        get_adjusted_difficulty(&blockchain)
    } else {
        latest_block.difficulty
    }
}

pub fn get_adjusted_difficulty(blockchain: &Blockchain) -> usize {
    let previous_adjustment_block = blockchain
        .chain
        .get(blockchain.chain.len() - DIFFICULTY_ADJUSTMENT_INTERVAL)
        .unwrap();
    let time_expected = chrono::Duration::seconds(
        BLOCK_GENERATION_INTERVAL as i64 * DIFFICULTY_ADJUSTMENT_INTERVAL as i64,
    );
    let time_actual = blockchain
        .get_latest_block()
        .timestamp
        .signed_duration_since(previous_adjustment_block.timestamp);

    if time_actual < time_expected / 2 {
        previous_adjustment_block.difficulty + 1
    } else if time_actual > time_expected * 2 {
        previous_adjustment_block.difficulty - 1
    } else {
        previous_adjustment_block.difficulty
    }
}

/// A block is valid, if the timestamp is at most 1 min in the future from the time we perceive.
/// A block in the chain is valid, if the timestamp is at most 1 min in the past of the previous block.
pub fn is_valid_timestamp(new_block: &Block, prev_block: &Block) -> bool {
    let is_new_enough = prev_block
        .timestamp
        .checked_sub_signed(chrono::Duration::seconds(60))
        .unwrap()
        < new_block.timestamp;

    let is_old_enough =
        new_block.timestamp.signed_duration_since(Utc::now()) < chrono::Duration::seconds(60);

    is_new_enough && is_old_enough
}

/// Nakamoto consensus
pub fn cumulative_difficulty(blocks: &Vec<Block>) -> f64 {
    blocks
        .iter()
        .fold(0f64, |acc, elem| acc + 2f64.powi(elem.difficulty as i32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_cumulative_difficulty() {
        let genesis_block = Block::new(0, Utc::now(), "", "", 0, 0);
        let easy_block = Block::new(0, Utc::now(), "", "", 0, 2);
        let hard_block = Block::new(0, Utc::now(), "", "", 0, 8);
        let mut blockchain = Blockchain::new(genesis_block);

        blockchain.add_block(easy_block);
        blockchain.add_block(hard_block);

        assert_eq!(cumulative_difficulty(&blockchain.chain), 261.0);
    }

    #[test]
    fn validates_block_timestamps() {
        let root_time = Utc::now();
        let genesis_block = Block::new(0, root_time, "", "", 0, 0);
        let valid_block = Block::new(
            0,
            root_time
                .checked_add_signed(chrono::Duration::seconds(50))
                .unwrap(),
            "",
            "",
            0,
            0,
        );
        let another_valid_block = Block::new(
            0,
            root_time
                .checked_sub_signed(chrono::Duration::seconds(50))
                .unwrap(),
            "",
            "",
            0,
            0,
        );

        assert!(is_valid_timestamp(&valid_block, &genesis_block));
        assert!(is_valid_timestamp(&another_valid_block, &genesis_block));
    }

    #[test]
    fn invalidates_invalid_block_timestamps() {
        let root_time = Utc::now();
        let genesis_block = Block::new(0, root_time, "", "", 0, 0);
        let invalid_block = Block::new(
            0,
            root_time
                .checked_add_signed(chrono::Duration::seconds(70))
                .unwrap(),
            "",
            "",
            0,
            0,
        );
        let another_invalid_block = Block::new(
            0,
            root_time
                .checked_sub_signed(chrono::Duration::seconds(70))
                .unwrap(),
            "",
            "",
            0,
            0,
        );

        assert!(!is_valid_timestamp(&invalid_block, &genesis_block));
        assert!(!is_valid_timestamp(&another_invalid_block, &genesis_block));
    }

    #[test]
    fn validates_single_block_chain() {
        let genesis = Block::new(0, Utc.timestamp(0, 0), "", &hex::encode([0; 64]), 0, 0);
        let chain = Blockchain::new(genesis);

        assert!(is_valid_chain(&chain));
    }

    #[test]
    fn validates_hash_matches_difficulty() {
        let hash = hex::encode([0; 64]);

        assert!(hash_matches_difficulty(&hash, 5));
    }
}
