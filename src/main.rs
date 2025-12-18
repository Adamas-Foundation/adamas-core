// --- ADAMAS WHITE-LABEL ENGINE v1.0 ---

mod block;
mod p2p;
mod wallet;
mod transaction;
mod database;
mod avm;
mod mempool;
mod network_messages;
mod config; // Nuovo modulo

use crate::block::Block;
use crate::wallet::Wallet;
use crate::transaction::Transaction;
use crate::mempool::Mempool;
use crate::database::BlockchainDB;
use crate::p2p::setup_p2p;
use crate::network_messages::NetworkMessage;
use crate::config::NodeConfig;
use libp2p::futures::StreamExt;
use libp2p::gossipsub;
use tokio::io::{self, AsyncBufReadExt}; 
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. GESTIONE ARGOMENTI AVANZATA
    // Uso: cargo run -- <path_to_config_file> <peer_address_optional>
    let args: Vec<String> = env::args().collect();
    
    // Se non specifichiamo nulla, cerca "node_config.json"
    let config_path = if args.len() > 1 { &args[1] } else { "node_config.json" };
    
    // CARICAMENTO CONFIGURAZIONE WHITE-LABEL
    let config = match NodeConfig::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Errore caricamento config '{}': {}", config_path, e);
            eprintln!("   Usa: cargo run -- <config_file>");
            return Ok(());
        }
    };

    let db_path = &config.db_path;

    // --- INTERFACCIA PERSONALIZZATA PER IL CLIENTE ---
    println!("==================================================");
    println!("   🚀 SYSTEM: {}", config.chain_name.to_uppercase());
    println!("   📦 VERSION: {}", config.version);
    println!("   👤 ROLE:    {}", config.node_role);
    println!("==================================================");
    
    let dev_wallet = Wallet::new();
    let mut node_mempool = Mempool::new();
    let db = BlockchainDB::new(db_path)?;
    println!("[*] Secure Storage Mounted: ./{}", db_path);

    // Genesi Block (Standard per tutti i clienti, ma potremmo configurarlo)
    let genesis_block = Block::new(
        0, 
        String::from("0000000000000000000000000000000000000000000000000000000000000000"), 
        String::from("Genesis Block - Secure Ledger Start"), 
        String::from("System-Root")
    );
    let mut last_hash = genesis_block.hash.clone(); 

    if let Ok(None) = db.load_block(&genesis_block.hash) {
        db.save_block(&genesis_block)?;
        println!("[+] Ledger Initialized.");
    } else {
        println!("[*] Ledger Loaded.");
        last_hash = genesis_block.hash.clone(); 
    }

    println!("\n[*] Starting Encrypted Mesh Network...");
    let (mut swarm, peer_id) = setup_p2p().await?;
    
    let topic = gossipsub::IdentTopic::new("enterprise-net-channel");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    // Se c'è un terzo argomento, è l'indirizzo a cui connettersi
    if args.len() > 2 {
        let remote_addr: libp2p::Multiaddr = args[2].parse()?;
        println!("   [NET] 📞 Connecting to Branch Office: {}", remote_addr);
        swarm.dial(remote_addr)?;
    }

    println!("--------------------------------------------------");
    println!("   NODE ID: {}", peer_id);
    println!("   STATUS:  ONLINE & SECURE");
    println!("--------------------------------------------------");

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    print!("{} > ", config.chain_name); // Prompt personalizzato!
    use std::io::Write; std::io::stdout().flush().unwrap();

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                        println!("\n[NET] Listening on: {}", address);
                        print!("{} > ", config.chain_name);
                        use std::io::Write; std::io::stdout().flush().unwrap();
                    },
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("\n[NET] 🤝 Connected to: {}", peer_id);
                        print!("{} > ", config.chain_name);
                        use std::io::Write; std::io::stdout().flush().unwrap();
                    },
                    libp2p::swarm::SwarmEvent::Behaviour(crate::p2p::AppBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { propagation_source: peer_id, message_id: _, message }
                    )) => {
                        if let Ok(net_msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                            match net_msg {
                                NetworkMessage::Block(incoming_block) => {
                                    println!("\n\n   [SYNC] 📦 NEW ENTRY RECEIVED from {}!", peer_id);
                                    
                                    match db.save_block(&incoming_block) {
                                        Ok(_) => {
                                            println!("   [DB] ✅ Entry validated & saved.");
                                            last_hash = incoming_block.hash.clone(); 
                                        },
                                        Err(_) => println!("   [DB] ⚠️ Entry already exists."),
                                    }
                                    print!("{} > ", config.chain_name);
                                    use std::io::Write; std::io::stdout().flush().unwrap();
                                },
                                _ => {} 
                            }
                        }
                    },
                    _ => {} 
                }
            }

            Ok(Some(line)) = stdin.next_line() => {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.is_empty() { 
                    print!("{} > ", config.chain_name);
                    use std::io::Write; std::io::stdout().flush().unwrap();
                    continue; 
                }

                match parts[0] {
                    "help" => {
                        println!("\n--- {} COMMANDS ---", config.chain_name);
                        println!("info         -> System Status");
                        println!("create_entry -> Create Log Entry");
                        println!("seal         -> Seal Block (Mine)");
                        println!("exit         -> Shutdown");
                    }
                    "info" => {
                        println!("Role: {}", config.node_role);
                        println!("Key: {}", dev_wallet.public_key);
                    }
                    "create_entry" => { // Ho rinominato per renderlo più "Business"
                        println!("\n📝 Creating secure log entry...");
                        let tx = Transaction::new(&dev_wallet, "0xInternal_Department".to_string(), 1);
                        if node_mempool.add_transaction(tx) {
                            println!("✅ Entry buffered.");
                        }
                    }
                    "seal" => { // Ho rinominato "Mine" in "Seal" (Sigillare) per le aziende
                        if node_mempool.pending_txs.is_empty() {
                            println!("\n⚠️  No entries to seal.");
                        } else {
                            println!("\n🔒 Sealing Block...");
                            let tx_list: Vec<Transaction> = node_mempool.pending_txs.values().cloned().collect();
                            let block_data = serde_json::to_string(&tx_list).unwrap_or_default();
                            
                            let new_block = Block::new(1, last_hash.clone(), block_data, dev_wallet.public_key.clone());
                            
                            db.save_block(&new_block).unwrap();
                            last_hash = new_block.hash.clone();
                            node_mempool.clear();
                            println!("🔗 Block Sealed Locally: {}", last_hash);

                            println!("📡 Syncing with branches...");
                            let msg = NetworkMessage::Block(new_block);
                            let json_msg = serde_json::to_vec(&msg).unwrap();
                            
                            if let Err(_) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), json_msg) {
                                println!("❌ Sync Error.");
                            } else {
                                println!("✅ Network Synced.");
                            }
                        }
                    }
                    "exit" => {
                        println!("Shutting down...");
                        break;
                    }
                    _ => println!("Unknown command."),
                }
                print!("{} > ", config.chain_name);
                use std::io::Write; std::io::stdout().flush().unwrap();
            }
        }
    }
    Ok(())
}