use sled::Db;
use crate::block::Block;
use std::error::Error;

pub struct BlockchainDB {
    db: Db,
}

impl BlockchainDB {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let db = sled::open(path)?;
        Ok(BlockchainDB { db })
    }

    pub fn save_block(&self, block: &Block) -> Result<(), Box<dyn Error>> {
        let serialized = bincode::serialize(block)?;
        self.db.insert(&block.hash, serialized)?;
        self.db.flush()?;
        Ok(())
    }

    pub fn load_block(&self, hash: &str) -> Result<Option<Block>, Box<dyn Error>> {
        match self.db.get(hash)? {
            Some(data) => {
                let block: Block = bincode::deserialize(&data)?;
                Ok(Some(block))
            },
            None => Ok(None),
        }
    }
}