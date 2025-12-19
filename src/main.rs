use libp2p::{
    gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux, PeerId, Swarm,
};
use libp2p::futures::StreamExt;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_512};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use std::env;

// --- STRUTTURE DATI (BLOCKCHAIN) ---

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    index: u32,
    timestamp: u128,
    data: String,
    previous_hash: String,
    hash: String,
    node_id: String, // Chi ha minato il blocco?
}

struct Blockchain {
    chain: Vec<Block>,
}

impl Blockchain {
    fn new() -> Self {
        let mut chain = Blockchain { chain: Vec::new() };
        chain.create_genesis_block();
        chain
    }

    fn create_genesis_block(&mut self) {
        let genesis_block = Block {
            index: 0,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
            data: "GENESIS BLOCK - ADAMAS NETWORK ONLINE".to_string(),
            previous_hash: "0".to_string(),
            hash: "00000000000000000000".to_string(),
            node_id: "GENESIS".to_string(),
        };
        self.chain.push(genesis_block);
    }

    fn add_block(&mut self, data: String, node_origin: String) -> Block {
        let previous_block = self.chain.last().unwrap();
        let new_index = previous_block.index + 1;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let previous_hash = previous_block.hash.clone();
        
        let content_to_hash = format!("{}{}{}{}", new_index, timestamp, data, previous_hash);
        let mut hasher = Sha3_512::new();
        hasher.update(content_to_hash);
        let result = hasher.finalize();
        let hash = format!("{:x}", result);

        let new_block = Block {
            index: new_index,
            timestamp,
            data,
            previous_hash,
            hash,
            node_id: node_origin,
        };

        self.chain.push(new_block.clone());
        println!("✅ BLOCK #{} ADDED TO CHAIN (Data: {})", new_index, new_block.data);
        new_block
    }

    // Funzione per aggiungere un blocco ricevuto dal P2P (validazione semplificata)
    fn receive_block(&mut self, remote_block: Block) {
        let last_index = self.chain.last().unwrap().index;
        if remote_block.index > last_index {
            self.chain.push(remote_block.clone());
            println!("📥 P2P SYNC: Received Block #{} from Node {}", remote_block.index, remote_block.node_id);
        }
    }
}

// --- NETWORK BEHAVIOUR (Comportamento del Nodo) ---
// Definiamo cosa sa fare il nodo: mDNS (trovare vicini) e GossipSub (parlare)
#[derive(NetworkBehaviour)]
struct AdamasBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Configurazione Iniziale (Porte e ID)
    let args: Vec<String> = env::args().collect();
    // Se lanci "cargo run -- 3001", usa la porta 3001, altrimenti 3000
    let http_port = args.get(1).unwrap_or(&"3000".to_string()).clone(); 
    
    // Generiamo una chiave crittografica per il nodo
    let swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            // Setup GossipSub (Canale di News)
            let message_id_fn = |message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                .validation_mode(gossipsub::ValidationMode::Strict)
                .message_id_fn(message_id_fn)
                .build()
                .map_err(|msg| std::io::Error::new(std::io::ErrorKind::Other, msg))?;

            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            // Setup mDNS (Discovery Locale)
            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;
            Ok(AdamasBehaviour { gossipsub, mdns })
        })?
        .build();

    let mut swarm = swarm;
    let my_peer_id = *swarm.local_peer_id();
    
    // Sottoscrizione al canale "adamas-blocks"
    let topic = gossipsub::IdentTopic::new("adamas-blocks");
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Ascolto su una porta P2P casuale (OS assigned) o fissa
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("🚀 ADAMAS NODE STARTED | PeerID: {}", my_peer_id);
    println!("🌍 Dashboard available at: http://localhost:{}", http_port);

    // Condivisione Blockchain tra Thread
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let blockchain_web = blockchain.clone();
    
    // Canale per mandare messaggi dal Web al P2P
    let (tx_p2p, mut rx_p2p) = tokio::sync::mpsc::unbounded_channel::<String>();

    // --- TASK 1: SERVER WEB (Dashboard) ---
    tokio::spawn(async move {
        let addr = format!("0.0.0.0:{}", http_port);
        let listener = TcpListener::bind(&addr).await.unwrap();

        loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            let blockchain_ref = blockchain_web.clone();
            let tx_p2p_ref = tx_p2p.clone();

            tokio::spawn(async move {
                let mut buffer = [0; 1024];
                match socket.read(&mut buffer).await {
                    Ok(_) => {
                        let request = String::from_utf8_lossy(&buffer[..]);
                        let response_body;
                        let response_header;

                        if request.starts_with("GET /blocks") {
                            let chain = blockchain_ref.lock().unwrap();
                            response_body = serde_json::to_string(&chain.chain).unwrap();
                            response_header = "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8";
                        
                        } else if request.contains("GET /mine/") {
                            let parts: Vec<&str> = request.split_whitespace().collect();
                            if parts.len() > 1 && parts[1].len() > 6 {
                                let data_raw = &parts[1][6..];
                                let data_clean = data_raw.replace("%20", " ").replace("%7C", " | ");
                                
                                // 1. Mina il blocco localmente
                                let new_block = {
                                    let mut chain = blockchain_ref.lock().unwrap();
                                    chain.add_block(data_clean, "LOCAL_WEB".to_string())
                                };

                                // 2. Invia al P2P
                                let block_json = serde_json::to_string(&new_block).unwrap();
                                let _ = tx_p2p_ref.send(block_json);

                                response_body = "{\"status\": \"mined_and_broadcasted\"}".to_string();
                                response_header = "HTTP/1.1 200 OK\r\nContent-Type: application/json";
                            } else {
                                response_body = "{}".to_string();
                                response_header = "HTTP/1.1 400 ERROR";
                            }

                        } else if request.contains("GET / ") || request.contains("dashboard.html") {
                            let content = std::fs::read_to_string("dashboard.html").unwrap_or("<h1>Missing Dashboard</h1>".to_string());
                            response_body = content;
                            response_header = "HTTP/1.1 200 OK\r\nContent-Type: text/html";
                        } else {
                            response_body = "404".to_string();
                            response_header = "HTTP/1.1 404 NOT FOUND";
                        }

                        let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", response_header, response_body.len(), response_body);
                        let _ = socket.write_all(response.as_bytes()).await;
                    }
                    Err(_) => {}
                }
            });
        }
    });

    // --- TASK 2: GESTORE EVENTI P2P (Il Cuore della Rete) ---
    loop {
        tokio::select! {
            // A. Ascolta eventi dalla rete (altri nodi)
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("📡 P2P Listening on {:?}", address);
                },
                SwarmEvent::Behaviour(AdamasBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        println!("👀 FOUND PEER: {:?}", peer_id);
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(AdamasBehaviourEvent::Gossipsub(gossipsub::Event::Message { propagation_source: peer_id, message_id: _, message })) => {
                    // Abbiamo ricevuto un messaggio!
                    let msg_str = String::from_utf8_lossy(&message.data);
                    if let Ok(remote_block) = serde_json::from_str::<Block>(&msg_str) {
                        println!("⚡ NETWORK: Block received from {:?}", peer_id);
                        let mut chain = blockchain.lock().unwrap();
                        chain.receive_block(remote_block);
                    }
                },
                _ => {}
            },

            // B. Ascolta comandi dal Web (Blocchi minati da noi da inviare agli altri)
            Some(block_json) = rx_p2p.recv() => {
                let topic = gossipsub::IdentTopic::new("adamas-blocks");
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, block_json.as_bytes()) {
                    println!("❌ Publish Error: {:?}", e);
                } else {
                    println!("📡 BROADCAST: Block sent to network.");
                }
            }
        }
    }
}