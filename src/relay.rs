use libp2p::{
    noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp,
    yamux,
};
use libp2p::futures::StreamExt;
use std::error::Error;

// IMPORT FONDAMENTALE: Usiamo la libreria specifica che abbiamo aggiunto
use libp2p_relay as relay;

#[derive(NetworkBehaviour)]
struct RelayBehaviour {
    // Ora "relay::Behaviour" si riferisce correttamente alla libreria Server
    relay: relay::Behaviour,
    ping: libp2p::ping::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Configurazione del Relay Server
    // Usiamo la configurazione di default che è perfetta per iniziare
    let relay_behaviour = relay::Behaviour::new(
        libp2p::PeerId::random(),
        relay::Config::default(),
    );

    let ping_behaviour = libp2p::ping::Behaviour::new(libp2p::ping::Config::new());

    // 2. Costruzione dello Swarm
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_key| {
            Ok(RelayBehaviour {
                relay: relay_behaviour,
                ping: ping_behaviour,
            })
        })?
        .build();

    // 3. Ascolto sulla porta pubblica 4001
    swarm.listen_on("/ip4/0.0.0.0/tcp/4001".parse()?)?;

    let my_peer_id = *swarm.local_peer_id();
    println!("🗼 ADAMAS RELAY TOWER ACTIVE");
    println!("-------------------------------------------------");
    println!("🔑 SERVER ID: {}", my_peer_id);
    println!("📡 Listening on port 4001...");
    println!("-------------------------------------------------");

    // 4. Loop Infinito
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("👂 Listening Interface: {:?}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("🔗 New Client Connected: {:?}", peer_id);
            }
            _ => {}
        }
    }
}