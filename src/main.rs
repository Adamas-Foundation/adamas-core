// --- ADAMAS NODE v0.1.0 (MINING ENABLED) ---

mod block;
mod p2p;
mod wallet;
mod transaction;
mod database;
mod avm;
mod mempool;

use crate::block::Block;
use crate::wallet::Wallet;
use crate::transaction::Transaction;
use crate::mempool::Mempool;
use crate::database::BlockchainDB;
use crate::p2p::setup_p2p;
use libp2p::futures::StreamExt;
use std::env;

const MAX_SUPPLY: u64 = 20_633_239; 
const VERSION: &str = "0.1.0-alpha";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let node_id = if args.len() > 1 { &args[1] } else { "0" };
    let db_path = format!("adamas_db_{}", node_id);

    println!("--------------------------------------------------");
    println!("   ADAMAS NODE v{} - Post-Quantum Chain", VERSION);
    println!("--------------------------------------------------");
    
    // 1. Inizializziamo i componenti
    let dev_wallet = Wallet::new();
    let mut node_mempool = Mempool::new();
    let db = BlockchainDB::new(&db_path)?; // Carichiamo il DB subito
    println!("[*] Database Mounted:  ./{}", db_path);

    // --- GENESIS BLOCK (Blocco 0) ---
    // Lo creiamo solo se non esiste già
    let genesis_block = Block::new(
        0, 
        String::from("0000000000000000000000000000000000000000000000000000000000000000"), 
        format!("Genesis Allocation: {} ADM", MAX_SUPPLY), 
        String::from("Adamas-Foundation")
    );

    let mut previous_hash = genesis_block.hash.clone(); // Ci serve per collegare i blocchi

    if let Ok(None) = db.load_block(&genesis_block.hash) {
        db.save_block(&genesis_block)?;
        println!("[+] Genesis Block:     CREATED & SAVED.");
    } else {
        println!("[*] Genesis Block:     ALREADY EXISTS (Skipping).");
    }

    // --- SIMULAZIONE CICLO VITA: Transazione -> Mempool -> Blocco ---
    println!("\n   [FLOW] 1. Alice creates transaction (50 ADM -> Bob)...");
    let bob_address = "0xBob_Fake_Address".to_string();
    let tx = Transaction::new(&dev_wallet, bob_address, 50);

    println!("   [FLOW] 2. Submitting to Node Mempool...");
    if node_mempool.add_transaction(tx) {
        println!("   [MEMPOOL] Transaction Accepted.");
        
        // --- FASE DI MINING (Nuova!) ---
        println!("\n   [MINER] ⛏️  Miner activated! Fetching transactions...");
        
        // 1. Prendiamo tutte le transazioni dalla mempool
        // (Per ora le trasformiamo in JSON grezzo per metterle nel blocco)
        let tx_list: Vec<Transaction> = node_mempool.pending_txs.values().cloned().collect();
        let block_data = serde_json::to_string(&tx_list)?;

        // 2. Creiamo il Blocco #1
        println!("   [MINER] 🔨 Forging Block #1...");
        let block_1 = Block::new(
            1,              // Indice
            previous_hash,  // Hash del blocco Genesi
            block_data,     // I dati (la transazione di Alice)
            dev_wallet.public_key.clone() // Il validatore siamo noi
        );

        // 3. Salviamo il blocco nel Database
        db.save_block(&block_1)?;
        println!("   [CHAIN] 🔗 Block #1 added to chain!");
        println!("   [CHAIN] Hash: {}", block_1.hash);
        
        // 4. Puliamo la Mempool (le transazioni sono ormai nel blocco)
        node_mempool.clear();
        println!("   [MEMPOOL] Cleared. Ready for next block.");
    }
    // -------------------------------------------------------------

    println!("\n[*] Initializing GossipSub Network...");
    let (mut swarm, peer_id) = setup_p2p().await?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("--------------------------------------------------");
    println!("   NODE ONLINE | ID: {}", peer_id);
    println!("   STATUS: WAITING FOR PEERS...");
    println!("--------------------------------------------------");

    loop {
        match swarm.select_next_some().await {
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                println!("[NET] Listening on: {}", address);
            },
            _ => {} 
        }
    }
}