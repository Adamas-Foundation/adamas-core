// --- ADAMAS ENTERPRISE NODE v0.1.0 (AUTO-SYNC FIXED) ---

mod block;
mod p2p;
mod wallet;
mod transaction;
mod database;
mod avm;
mod mempool;
mod network_messages; 

use crate::block::Block;
use crate::wallet::Wallet;
use crate::transaction::Transaction;
use crate::mempool::Mempool;
use crate::database::BlockchainDB;
use crate::p2p::setup_p2p;
use crate::network_messages::NetworkMessage;
use libp2p::futures::StreamExt;
use libp2p::gossipsub; 
use tokio::io::{self, AsyncBufReadExt}; 
use std::env;

const MAX_SUPPLY: u64 = 20_633_239; 
const VERSION: &str = "0.1.0-Enterprise";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let node_id = if args.len() > 1 { &args[1] } else { "0" };
    let db_path = format!("adamas_db_{}", node_id);

    // --- SETUP VISIVO ---
    println!("--------------------------------------------------");
    println!("   ADAMAS ENTERPRISE CORE v{}", VERSION);
    println!("   Post-Quantum Engine | White-Label Ready");
    println!("--------------------------------------------------");
    
    let dev_wallet = Wallet::new();
    let mut node_mempool = Mempool::new();
    let db = BlockchainDB::new(&db_path)?;
    println!("[*] Database Mounted:  ./{}", db_path);

    // Gestione Genesi
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
        println!("[*] Genesis Block:     LOADED.");
        last_hash = genesis_block.hash.clone(); 
    }

    println!("\n[*] Initializing Enterprise Mesh Network...");
    let (mut swarm, peer_id) = setup_p2p().await?;
    
    let topic = gossipsub::IdentTopic::new("adamas-global");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    if args.len() > 2 {
        let remote_addr: libp2p::Multiaddr = args[2].parse()?;
        println!("   [NET] 📞 Connecting to Partner Node: {}", remote_addr);
        swarm.dial(remote_addr)?;
    }

    println!("--------------------------------------------------");
    println!("   NODE ONLINE | ID: {}", peer_id);
    println!("   SYSTEM READY. Waiting for events...");
    println!("--------------------------------------------------");

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    print!("COMMAND > ");
    use std::io::Write; std::io::stdout().flush().unwrap();

    loop {
        tokio::select! {
            // 1. GESTIONE AUTOMATICA RETE
            event = swarm.select_next_some() => {
                match event {
                    libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                        println!("\n[NET] Listening on: {}", address);
                        print!("COMMAND > "); 
                        use std::io::Write; std::io::stdout().flush().unwrap();
                    },
                    libp2p::swarm::SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("\n[NET] 🤝 Partner Connected: {}", peer_id);
                        print!("COMMAND > ");
                        use std::io::Write; std::io::stdout().flush().unwrap();
                    },
                    // *** CORREZIONE QUI SOTTO: AppBehaviourEvent invece di MyBehaviourEvent ***
                    libp2p::swarm::SwarmEvent::Behaviour(crate::p2p::AppBehaviourEvent::Gossipsub(
                        gossipsub::Event::Message { propagation_source: peer_id, message_id: _, message }
                    )) => {
                        if let Ok(net_msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                            match net_msg {
                                NetworkMessage::Block(incoming_block) => {
                                    println!("\n\n   [SYNC] 📦 BLOCK RECEIVED from {}!", peer_id);
                                    println!("   [SYNC] Index: #{} | Hash: {}", incoming_block.index, incoming_block.hash);
                                    
                                    match db.save_block(&incoming_block) {
                                        Ok(_) => {
                                            println!("   [DB] ✅ Data Verified & Saved securely.");
                                            last_hash = incoming_block.hash.clone(); 
                                        },
                                        Err(_) => println!("   [DB] ⚠️ Block already exists."),
                                    }
                                    print!("COMMAND > ");
                                    use std::io::Write; std::io::stdout().flush().unwrap();
                                },
                                _ => {} 
                            }
                        }
                    },
                    _ => {} 
                }
            }

            // 2. GESTIONE MANUALE
            Ok(Some(line)) = stdin.next_line() => {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.is_empty() { 
                    print!("COMMAND > ");
                    use std::io::Write; std::io::stdout().flush().unwrap();
                    continue; 
                }

                match parts[0] {
                    "help" => {
                        println!("\n--- ADMIN COMMANDS ---");
                        println!("wallet       -> Show Node ID");
                        println!("create_tx    -> Simulate Business Data");
                        println!("mine         -> Seal Block & Broadcast");
                        println!("exit         -> Shutdown System");
                    }
                    "wallet" => {
                        println!("\n🔑 Node Public Key (Dilithium-5):");
                        println!("{}", dev_wallet.public_key);
                    }
                    "create_tx" => {
                        println!("\n💸 Generating encrypted data payload...");
                        let tx = Transaction::new(&dev_wallet, "0xRecipient_Partner".to_string(), 1000);
                        if node_mempool.add_transaction(tx) {
                            println!("✅ Data buffered in Mempool.");
                        }
                    }
                    "mine" => {
                        if node_mempool.pending_txs.is_empty() {
                            println!("\n⚠️  Buffer empty. No data to secure.");
                        } else {
                            println!("\n⛏️  Sealing Block with Post-Quantum Proof...");
                            
                            let tx_list: Vec<Transaction> = node_mempool.pending_txs.values().cloned().collect();
                            let block_data = serde_json::to_string(&tx_list).unwrap_or_default();
                            
                            let new_block = Block::new(1, last_hash.clone(), block_data, dev_wallet.public_key.clone());
                            
                            // 1. Salvataggio Locale
                            db.save_block(&new_block).unwrap();
                            last_hash = new_block.hash.clone();
                            node_mempool.clear();
                            println!("🔗 Block Sealed Locally: {}", last_hash);

                            // 2. BROADCAST
                            println!("📡 Broadcasting to Network Partners...");
                            let msg = NetworkMessage::Block(new_block);
                            let json_msg = serde_json::to_vec(&msg).unwrap();
                            
                            if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), json_msg) {
                                println!("❌ Broadcast Error: {:?}", e);
                            } else {
                                println!("✅ Data propagated to the network.");
                            }
                        }
                    }
                    "exit" => {
                        println!("Shutting down secure node...");
                        break;
                    }
                    _ => println!("Unknown command."),
                }
                print!("COMMAND > ");
                use std::io::Write; std::io::stdout().flush().unwrap();
            }
        }
    }
    Ok(())
}