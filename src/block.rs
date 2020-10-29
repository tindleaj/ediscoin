use super::generate_hash;
use chrono::offset::Utc;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub timestamp: DateTime<Utc>,
    pub data: String,
    pub hash: String,
    pub prev_hash: String,
    pub nonce: u32,
    pub difficulty: usize,
}

impl Block {
    pub fn new(
        index: u32,
        timestamp: DateTime<Utc>,
        data: &str,
        prev_hash: &str,
        nonce: u32,
        difficulty: usize,
    ) -> Block {
        let block = Block {
            index,
            timestamp,
            data: data.to_string(),
            nonce,
            difficulty,
            hash: generate_hash(index, prev_hash, timestamp, data, nonce),
            prev_hash: prev_hash.to_string(),
        };

        block
    }
}

impl Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Block {{
            index: {},
            timestamp: {},
            data: {},
            hash: {},
            prev_hash: {}
        }}",
            self.index,
            self.timestamp,
            self.data,
            hex::encode(&self.hash),
            hex::encode(&self.prev_hash)
        )
    }
}
