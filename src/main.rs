// --- ADAMAS NODE v0.1.0 (MANUAL CONNECT) ---

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
    // Leggiamo gli argomenti: ID nodo e (opzionale) Indirizzo a cui collegarsi
    let args: Vec<String> = env::args().collect();
    let node_id = if args.len() > 1 { &args[1] } else { "0" };
    let db_path = format!("adamas_db_{}", node_id);

    println!("--------------------------------------------------");
    println!("   ADAMAS NODE v{} - Post-Quantum Chain", VERSION);
    println!("--------------------------------------------------");
    
    // Inizializzazioni varie...
    let dev_wallet = Wallet::new();
    let mut node_mempool = Mempool::new();
    let db = BlockchainDB::new(&db_path)?;
    println!("[*] Database Mounted:  ./{}", db_path);

    // Genesi Block (Standard)
    let genesis_block = Block::new(
        0, 
        String::from("0000000000000000000000000000000000000000000000000000000000000000"), 
        format!("Genesis Allocation: {} ADM", MAX_SUPPLY), 
        String::from("Adamas-Foundation")
    );
    let previous_hash = genesis_block.hash.clone(); 
    if let Ok(None) = db.load_block(&genesis_block.hash) {
        db.save_block(&genesis_block)?;
        println!("[+] Genesis Block:     CREATED & SAVED.");
    }

    // Simulazione Transazione (Solo per il nodo principale 0)
    if node_id == "0" {
        println!("\n   [FLOW] 1. Alice creates transaction...");
        let tx = Transaction::new(&dev_wallet, "0xBob".to_string(), 50);
        node_mempool.add_transaction(tx);
        println!("   [MINER] ⛏️  Miner created Block #1 (Simulation)");
        // Qui saltiamo il salvataggio reale per brevità nel test di rete
    }

    println!("\n[*] Initializing GossipSub Network...");
    let (mut swarm, peer_id) = setup_p2p().await?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // --- CONNESSIONE MANUALE (NUOVA PARTE) ---
    // Se c'è un terzo argomento, proviamo a connetterci a quell'indirizzo
    if args.len() > 2 {
        let remote_addr: libp2p::Multiaddr = args[2].parse()?;
        println!("   [NET] 📞 Dialing node: {}", remote_addr);
        swarm.dial(remote_addr)?;
    }
    // -----------------------------------------

    println!("--------------------------------------------------");
    println!("   NODE ONLINE | ID: {}", peer_id);
    println!("   STATUS: WAITING FOR PEERS...");
    println!("--------------------------------------------------");

    loop {
        match swarm.select_next_some().await {
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                println!("[NET] Listening on: {}", address);
            },
            libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("\n   [NET] 🤝 ZACK! Connection Established with Peer: {}", peer_id);
                println!("   [NET] Network size: Growing...\n");
            },
            _ => {} 
        }
    }
}