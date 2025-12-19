use libp2p::{
    core::upgrade,
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, Transport,
};
use libp2p::futures::StreamExt;
use libp2p::futures::future::Either; 
use serde::{Deserialize, Serialize};
use sha3::Sha3_512;
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use std::env;

// =============================================================
// ⚙️ CONFIGURAZIONE CLIENTE (MODIFICARE QUI PER NUOVI CLIENTI)
// =============================================================

// 1. Indirizzo del Faro (Relay).
// NB: Cambiare "127.0.0.1" con l'IP del VPS quando sarà pronto.
const BOOTSTRAP_RELAY: &str = "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWMfHfkz3ivqb4wLXBizUvtAXjgVidDF8UZyqY8XGWPapw";

// 2. Porta di Default per la Dashboard (se non specificata)
const DEFAULT_HTTP_PORT: &str = "3000";

// 3. Nome della Rete (usato per isolare le comunicazioni)
const NETWORK_TOPIC: &str = "adamas-enterprise-net";

// =============================================================

// --- STRUTTURE BLOCKCHAIN ---
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    index: u32,
    timestamp: u128,
    data: String,
    previous_hash: String,
    hash: String,
    node_id: String,
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
            data: "GENESIS BLOCK".to_string(),
            previous_hash: "0".to_string(),
            hash: "00000000000000000000".to_string(),
            node_id: "SYSTEM".to_string(),
        };
        self.chain.push(genesis_block);
    }

    fn add_block(&mut self, data: String, node_origin: String) -> Block {
        let previous_block = self.chain.last().unwrap();
        let new_index = previous_block.index + 1;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
        let previous_hash = previous_block.hash.clone();
        
        use sha3::Digest; 
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
        println!("✅ BLOCK #{} MINED: {}", new_index, new_block.data);
        new_block
    }

    fn receive_block(&mut self, remote_block: Block) {
        let last_index = self.chain.last().unwrap().index;
        if remote_block.index > last_index {
            self.chain.push(remote_block.clone());
            println!("📥 SYNC: Block #{} received from {}", remote_block.index, remote_block.node_id);
        }
    }
}

// --- NETWORK BEHAVIOUR ---
#[derive(NetworkBehaviour)]
struct AdamasBehaviour {
    gossipsub: gossipsub::Behaviour,
    mdns: mdns::tokio::Behaviour,
    relay_client: libp2p::relay::client::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let http_port = args.get(1).unwrap_or(&DEFAULT_HTTP_PORT.to_string()).clone();
    let relay_addr_str = args.get(2).map(|s| s.as_str()).unwrap_or(BOOTSTRAP_RELAY);

    let local_key = libp2p::identity::Keypair::generate_ed25519();
    let local_peer_id = libp2p::PeerId::from(local_key.public());
    println!("🔑 NODE ID: {}", local_peer_id);

    let (relay_transport, relay_behaviour) = libp2p::relay::client::new(local_peer_id);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_other_transport(|key| {
            let noise_config = noise::Config::new(key).unwrap();
            let yamux_config = yamux::Config::default();

            let tcp_transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
                .upgrade(upgrade::Version::V1)
                .authenticate(noise_config.clone())
                .multiplex(yamux_config.clone());

            let relay_transport_upgraded = relay_transport
                .upgrade(upgrade::Version::V1)
                .authenticate(noise_config)
                .multiplex(yamux_config);

            tcp_transport.or_transport(relay_transport_upgraded)
                .map(|either, _| match either {
                    Either::Left((peer, stream)) => (peer, libp2p::core::muxing::StreamMuxerBox::new(stream)),
                    Either::Right((peer, stream)) => (peer, libp2p::core::muxing::StreamMuxerBox::new(stream)),
                })
                .boxed()
        })?
        .with_behaviour(|key: &libp2p::identity::Keypair| {
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

            let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), key.public().to_peer_id())?;

            Ok(AdamasBehaviour {
                gossipsub,
                mdns,
                relay_client: relay_behaviour,
            })
        })?
        .build();

    // Usa il TOPIC configurato in alto
    let topic = gossipsub::IdentTopic::new(NETWORK_TOPIC);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    println!("🗼 CONNECTING TO RELAY: {}", relay_addr_str);
    if let Ok(relay_addr) = relay_addr_str.parse::<Multiaddr>() {
        if let Err(_) = swarm.dial(relay_addr.clone()) {
            println!("❌ Relay Unreachable (Localhost mismatch is normal if Relay is on Cloud)");
        } else {
            println!("✅ Dialing Relay...");
        }
        let circuit_addr = relay_addr.with(libp2p::multiaddr::Protocol::P2pCircuit);
        let _ = swarm.listen_on(circuit_addr);
    }

    println!("🚀 ADAMAS CLIENT v3.4 STARTED");
    println!("🌍 Dashboard: http://localhost:{}", http_port);

    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let blockchain_web = blockchain.clone();
    let (tx_p2p, mut rx_p2p) = tokio::sync::mpsc::unbounded_channel::<String>();

    // WEB SERVER
    tokio::spawn(async move {
        let addr = format!("0.0.0.0:{}", http_port);
        let listener = TcpListener::bind(&addr).await.unwrap();

        loop {
            if let Ok((mut socket, _)) = listener.accept().await {
                let blockchain_ref = blockchain_web.clone();
                let tx_p2p_ref = tx_p2p.clone();
                tokio::spawn(async move {
                    let mut buffer = [0; 1024];
                    if socket.read(&mut buffer).await.is_ok() {
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
                                let data_clean = data_raw.replace("%20", " ").replace("%7C", " | ").replace("%22", "");
                                let new_block = {
                                    let mut chain = blockchain_ref.lock().unwrap();
                                    chain.add_block(data_clean, "WEB_USER".to_string())
                                };
                                let block_json = serde_json::to_string(&new_block).unwrap();
                                let _ = tx_p2p_ref.send(block_json);
                                response_body = "{\"status\": \"ok\"}".to_string();
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
                            response_body = "".to_string();
                            response_header = "HTTP/1.1 404 NOT FOUND";
                        }
                        let response = format!("{}\r\nContent-Length: {}\r\n\r\n{}", response_header, response_body.len(), response_body);
                        let _ = socket.write_all(response.as_bytes()).await;
                    }
                });
            }
        }
    });

    // P2P LOOP
    loop {
        tokio::select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("📡 LISTENING ON: {:?}", address);
                },
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("🤝 CONNECTED TO: {:?}", peer_id);
                },
                SwarmEvent::Behaviour(AdamasBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _addr) in list {
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    }
                },
                SwarmEvent::Behaviour(AdamasBehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                    let msg_str = String::from_utf8_lossy(&message.data);
                    if let Ok(remote_block) = serde_json::from_str::<Block>(&msg_str) {
                        let mut chain = blockchain.lock().unwrap();
                        chain.receive_block(remote_block);
                    }
                },
                _ => {}
            },
            Some(block_json) = rx_p2p.recv() => {
                let topic = gossipsub::IdentTopic::new(NETWORK_TOPIC);
                if let Err(e) = swarm.behaviour_mut().gossipsub.publish(topic, block_json.as_bytes()) {
                    println!("❌ Broadcast Error: {:?}", e);
                }
            }
        }
    }
}