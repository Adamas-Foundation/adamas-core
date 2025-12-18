// --- ADAMAS ENTERPRISE v1.1 (FIXED) ---

mod block;
mod p2p;
mod wallet;
mod transaction;
mod database;
mod avm;
mod mempool;
mod network_messages;
mod config;
mod http_server;

use crate::block::Block;
use crate::wallet::Wallet;
use crate::transaction::Transaction;
use crate::mempool::Mempool;
use crate::database::BlockchainDB;
use crate::p2p::setup_p2p;
use crate::network_messages::NetworkMessage;
use crate::config::NodeConfig;
use crate::http_server::start_web_server;
use libp2p::futures::StreamExt;
use libp2p::gossipsub;
use tokio::io::{self, AsyncBufReadExt}; 
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 { &args[1] } else { "node_config.json" };
    
    let config = match NodeConfig::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ Config Error: {}", e);
            return Ok(());
        }
    };

    println!("==================================================");
    println!("   🚀 SYSTEM: {}", config.chain_name.to_uppercase());
    println!("   📦 VERSION: {}", config.version);
    println!("==================================================");
    
    // --- AVVIO DASHBOARD WEB ---
    let web_config = config.clone();
    tokio::spawn(async move {
        start_web_server(web_config, 3000).await;
    });

    let db_path = &config.db_path;
    let dev_wallet = Wallet::new();
    let mut node_mempool = Mempool::new();
    let db = BlockchainDB::new(db_path)?;
    println!("[*] Secure DB: ./{}", db_path);

    // *** CORREZIONE QUI SOTTO: .repeat(64) invece di *64 ***
    let genesis_block = Block::new(
        0, 
        "0".repeat(64), // Corretto!
        String::from("Genesis Block"), 
        String::from("Root")
    );
    let mut last_hash = genesis_block.hash.clone(); 

    if let Ok(None) = db.load_block(&genesis_block.hash) {
        db.save_block(&genesis_block)?;
    } else {
        last_hash = genesis_block.hash.clone(); 
    }

    let (mut swarm, peer_id) = setup_p2p().await?;
    let topic = gossipsub::IdentTopic::new("enterprise-net-channel");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    if args.len() > 2 {
        let remote_addr: libp2p::Multiaddr = args[2].parse()?;
        swarm.dial(remote_addr)?;
    }

    println!("--------------------------------------------------");
    println!("   NODE ONLINE | ID: {}", peer_id);
    println!("--------------------------------------------------");

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    print!("{} > ", config.chain_name);
    use std::io::Write; std::io::stdout().flush().unwrap();

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                match event {
                    libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                        print!("{} > ", config.chain_name); 
                        use std::io::Write; std::io::stdout().flush().unwrap();
                    },
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("\n[NET] 🤝 Connected: {}", peer_id);
                        print!("{} > ", config.chain_name);
                        use std::io::Write; std::io::stdout().flush().unwrap();
                    },
                    libp2p::swarm::SwarmEvent::Behaviour(crate::p2p::AppBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { propagation_source: peer_id, message_id: _, message }
                    )) => {
                        if let Ok(net_msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                            match net_msg {
                                NetworkMessage::Block(incoming_block) => {
                                    println!("\n\n   [SYNC] 📦 BLOCK RECEIVED from {}!", peer_id);
                                    db.save_block(&incoming_block).ok();
                                    last_hash = incoming_block.hash.clone(); 
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
                if parts.is_empty() { continue; }
                match parts[0] {
                    "help" => println!("Commands: info, create_entry, seal, exit"),
                    "info" => println!("Key: {}", dev_wallet.public_key),
                    "create_entry" => {
                        let tx = Transaction::new(&dev_wallet, "0xInternal".to_string(), 1);
                        node_mempool.add_transaction(tx);
                        println!("✅ Entry buffered.");
                    },
                    "seal" => {
                        if !node_mempool.pending_txs.is_empty() {
                            let tx_list: Vec<Transaction> = node_mempool.pending_txs.values().cloned().collect();
                            let block_data = serde_json::to_string(&tx_list).unwrap_or_default();
                            let new_block = Block::new(1, last_hash.clone(), block_data, dev_wallet.public_key.clone());
                            db.save_block(&new_block).unwrap();
                            last_hash = new_block.hash.clone();
                            node_mempool.clear();
                            println!("🔗 Sealed: {}", last_hash);
                            let msg = NetworkMessage::Block(new_block);
                            let json_msg = serde_json::to_vec(&msg).unwrap();
                            swarm.behaviour_mut().gossipsub.publish(topic.clone(), json_msg).ok();
                        } else {
                            println!("⚠️ Empty buffer.");
                        }
                    }
                    "exit" => break,
                    _ => {}
                }
                print!("{} > ", config.chain_name);
                use std::io::Write; std::io::stdout().flush().unwrap();
            }
        }
    }
    Ok(())
}