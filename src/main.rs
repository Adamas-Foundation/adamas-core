// --- ADAMAS NODE v0.1.0 (INTERACTIVE CLI) ---

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
use tokio::io::{self, AsyncBufReadExt}; 
use std::env;

const MAX_SUPPLY: u64 = 20_633_239; 
const VERSION: &str = "0.1.0-alpha";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let node_id = if args.len() > 1 { &args[1] } else { "0" };
    let db_path = format!("adamas_db_{}", node_id);

    // --- SETUP INIZIALE ---
    println!("--------------------------------------------------");
    println!("   ADAMAS NODE v{} - Post-Quantum Chain", VERSION);
    println!("--------------------------------------------------");
    
    let dev_wallet = Wallet::new();
    let mut node_mempool = Mempool::new();
    let db = BlockchainDB::new(&db_path)?;
    println!("[*] Database Mounted:  ./{}", db_path);

    // Genesi (Solo se non esiste)
    let genesis_block = Block::new(
        0, 
        String::from("0000000000000000000000000000000000000000000000000000000000000000"), 
        format!("Genesis Allocation: {} ADM", MAX_SUPPLY), 
        String::from("Adamas-Foundation")
    );
    let mut last_hash = genesis_block.hash.clone(); 

    if let Ok(None) = db.load_block(&genesis_block.hash) {
        db.save_block(&genesis_block)?;
        println!("[+] Genesis Block:     CREATED & SAVED.");
    } else {
        println!("[*] Genesis Block:     ALREADY EXISTS.");
        // Se esiste già, recuperiamo l'hash (in un sistema vero caricheremmo l'ultimo blocco)
        last_hash = genesis_block.hash.clone(); 
    }

    println!("\n[*] Initializing Network...");
    let (mut swarm, peer_id) = setup_p2p().await?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Connessione manuale (Opzionale)
    if args.len() > 2 {
        let remote_addr: libp2p::Multiaddr = args[2].parse()?;
        println!("   [NET] 📞 Dialing node: {}", remote_addr);
        swarm.dial(remote_addr)?;
    }

    println!("--------------------------------------------------");
    println!("   NODE ONLINE | ID: {}", peer_id);
    println!("   Type 'help' for a list of commands.");
    println!("--------------------------------------------------");

    // Preparazione Lettura Input
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    
    // Stampiamo il primo prompt
    print!("ADAMAS > ");
    use std::io::Write; std::io::stdout().flush().unwrap();

    // --- LOOP PRINCIPALE (MULTITASKING) ---
    loop {
        tokio::select! {
            // 1. GESTIONE RETE (Ascolta eventi P2P)
            event = swarm.select_next_some() => {
                match event {
                    libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                        println!("\n[NET] Listening on: {}", address);
                        print!("ADAMAS > "); 
                        use std::io::Write; std::io::stdout().flush().unwrap();
                    },
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("\n[NET] 🤝 Peer Connected: {}", peer_id);
                        print!("ADAMAS > ");
                        use std::io::Write; std::io::stdout().flush().unwrap();
                    },
                    _ => {} 
                }
            }

            // 2. GESTIONE COMANDI (Ascolta te)
            Ok(Some(line)) = stdin.next_line() => {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.is_empty() { 
                    print!("ADAMAS > ");
                    use std::io::Write; std::io::stdout().flush().unwrap();
                    continue; 
                }

                match parts[0] {
                    "help" => {
                        println!("\n--- COMMANDS ---");
                        println!("wallet       -> Show your Dilithium Address");
                        println!("create_tx    -> Create a dummy transaction to Mempool");
                        println!("mempool      -> Show pending transactions");
                        println!("mine         -> Mine a block with pending txs");
                        println!("exit         -> Stop the node");
                        println!("----------------");
                    }
                    "wallet" => {
                        println!("\n🔑 Your Address (Dilithium-5):");
                        println!("{}", dev_wallet.public_key);
                    }
                    "create_tx" => {
                        println!("\n💸 Creating Test Transaction...");
                        let tx = Transaction::new(&dev_wallet, "0xBob_Receiver".to_string(), 10);
                        if node_mempool.add_transaction(tx) {
                            println!("✅ Transaction sent to Mempool.");
                        }
                    }
                    "mempool" => {
                        println!("\n🌊 Mempool Status: {} transactions pending.", node_mempool.pending_txs.len());
                    }
                    "mine" => {
                        if node_mempool.pending_txs.is_empty() {
                            println!("\n⚠️  Mempool is empty. Nothing to mine.");
                        } else {
                            println!("\n⛏️  Mining Block...");
                            
                            let tx_list: Vec<Transaction> = node_mempool.pending_txs.values().cloned().collect();
                            // In un sistema reale useremmo bincode, qui usiamo JSON per leggibilità
                            let block_data = serde_json::to_string(&tx_list).unwrap_or_default();
                            
                            let new_block = Block::new(1, last_hash.clone(), block_data, dev_wallet.public_key.clone());
                            
                            match db.save_block(&new_block) {
                                Ok(_) => {
                                    last_hash = new_block.hash.clone();
                                    node_mempool.clear(); // Pulisce la mempool
                                    println!("🔗 Block Mined & Saved! Hash: {}", last_hash);
                                }
                                Err(e) => println!("❌ Error saving block: {}", e),
                            }
                        }
                    }
                    "exit" => {
                        println!("Shutting down...");
                        break;
                    }
                    _ => println!("Unknown command. Type 'help'."),
                }
                // Ristampa il prompt dopo ogni comando
                print!("ADAMAS > ");
                use std::io::Write; std::io::stdout().flush().unwrap();
            }
        }
    }
    Ok(())
}