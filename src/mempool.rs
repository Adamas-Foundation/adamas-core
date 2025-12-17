use std::collections::HashMap;
use crate::transaction::Transaction;

// La Mempool è la "Sala d'Attesa" delle transazioni
pub struct Mempool {
    // Usiamo una HashMap per trovare le transazioni velocemente tramite la loro firma
    pub pending_txs: HashMap<String, Transaction>,
}

impl Mempool {
    // Crea una Mempool vuota
    pub fn new() -> Self {
        Mempool {
            pending_txs: HashMap::new(),
        }
    }

    // Aggiunge una transazione SOLO se è valida
    pub fn add_transaction(&mut self, tx: Transaction) -> bool {
        // 1. Verifica crittografica (Dilithium)
        if !tx.verify() {
            println!("   [MEMPOOL] ❌ Rifiutata transazione invalida (Firma errata)");
            return false;
        }

        // 2. Se è valida, aggiungila alla lista
        // Usiamo la firma come "ID" unico della transazione
        self.pending_txs.insert(tx.signature.clone(), tx);
        
        println!("   [MEMPOOL] ✅ Transazione aggiunta. Totale in attesa: {}", self.pending_txs.len());
        true
    }

    // Pulisce la Mempool (si usa dopo aver creato un blocco)
    pub fn clear(&mut self) {
        self.pending_txs.clear();
    }
}