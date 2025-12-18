use serde::{Deserialize, Serialize};
use crate::block::Block;
use crate::transaction::Transaction;

#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkMessage {
    Block(Block),             // Pacchetto contenente un Blocco
    Transaction(Transaction), // Pacchetto contenente una Transazione (futuro)
}