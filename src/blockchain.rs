use super::Block;
use chrono::Utc;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new(genesis_block: Block) -> Self {
        Blockchain {
            chain: vec![genesis_block],
        }
    }

    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn generate_next_block(&mut self, data: &str) {
        let prev_block = self.get_latest_block().clone();

        self.chain.push(Block::new(
            prev_block.index + 1,
            Utc::now(),
            data,
            &prev_block.hash,
        ));
    }

    pub fn replace_chain(&mut self, new_blocks: Vec<Block>) {
        self.chain = new_blocks
    }
}
