// --- ADAMAS NODE v0.1.0 (TRANSACTION TEST) ---

mod block;
mod p2p;
mod wallet;
mod transaction;
mod database;
mod avm;

use crate::block::Block;
use crate::wallet::Wallet;
use crate::transaction::Transaction; 
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
    
    // 1. Wallet Identity (DILITHIUM-5)
    let dev_wallet = Wallet::new();
    println!("[*] Wallet Identity (Truncated): {}...", &dev_wallet.public_key[..50]);

    // 2. Transazione Test Alice -> Bob
    let bob_address = "0xBob_Receiver_Address_Fake_Quantum_Key".to_string();

    println!("\n   [TX] Creating Quantum Transaction: Alice -> Bob (50 ADM)...");
    
    let tx = Transaction::new(&dev_wallet, bob_address.clone(), 50);
    
    println!("   [TX] Transaction Signed.");
    println!("   [TX] Signature Length: {} chars", tx.signature.len());

    println!("   [TX] Verifying integrity...");
    if tx.verify() {
        println!("   [SUCCESS] Transaction is VALID and SECURE ✅");
    } else {
        println!("   [ERROR] Transaction INVALID ❌");
    }

    let db = BlockchainDB::new(&db_path)?;
    println!("\n[*] Database Mounted:  ./{}", db_path);

    let genesis_block = Block::new(
        0, 
        String::from("0000000000000000000000000000000000000000000000000000000000000000"), 
        format!("Genesis Allocation: {} ADM", MAX_SUPPLY), 
        String::from("Adamas-Foundation")
    );

    if let Ok(None) = db.load_block(&genesis_block.hash) {
        db.save_block(&genesis_block)?;
        println!("[+] Genesis Block:     CREATED & SAVED.");
    } else {
        println!("[*] Genesis Block:     LOADED from Disk.");
    }

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