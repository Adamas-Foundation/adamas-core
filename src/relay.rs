use libp2p::{
    core::upgrade,
    identity,
    noise,
    relay,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp, yamux, Transport,
};
use libp2p::futures::StreamExt;
use std::error::Error;
use std::time::Duration;

#[derive(NetworkBehaviour)]
struct RelayBehaviour {
    relay: relay::Behaviour,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Chiavi
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = libp2p::PeerId::from(local_key.public());
    println!("🗼 ADAMAS RELAY TOWER ACTIVE");
    println!("🆔 SERVER ID: {}", local_peer_id);

    // 2. Configurazione
    let relay_config = relay::Config::default();
    let relay_behaviour = relay::Behaviour::new(local_peer_id, relay_config);

    // 3. Swarm
    let mut swarm = SwarmBuilder::with_existing_identity(local_key.clone())
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| RelayBehaviour { relay: relay_behaviour })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // 4. Ascolto
    swarm.listen_on("/ip4/0.0.0.0/tcp/4001".parse()?)?;
    println!("📡 LISTENING ON PORT 4001");

    // 5. Loop
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("🔗 ADDRESS: {:?}", address);
            },
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("🔗 New Client Connected: {:?}", peer_id);
            },
            _ => {}
        }
    }
}