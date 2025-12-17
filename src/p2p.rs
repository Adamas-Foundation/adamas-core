use libp2p::{
    gossipsub, mdns, noise, swarm::NetworkBehaviour, tcp, yamux, PeerId, Swarm, 
    Transport // <--- FIX 1: Abbiamo importato il "Manuale" per far funzionare .upgrade()
};
use libp2p::swarm::SwarmBuilder; // <--- FIX 2: SwarmBuilder abita qui, non nella radice
use std::error::Error;
use std::time::Duration;

#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

pub async fn setup_p2p() -> Result<(Swarm<AppBehaviour>, PeerId), Box<dyn Error>> {
    let id_keys = libp2p::identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    println!("[*] P2P Identity Generated: {}", peer_id);

    // Costruiamo il "Transport" (il cavo di rete virtuale)
    let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
        .upgrade(libp2p::core::upgrade::Version::V1) // Ora questo funzionerà grazie a "use libp2p::Transport"
        .authenticate(noise::Config::new(&id_keys).unwrap())
        .multiplex(yamux::Config::default())
        .boxed();

    // Configurazione GossipSub (Chat di gruppo dei nodi)
    let gossipsub_config = gossipsub::ConfigBuilder::default()
        .heartbeat_interval(Duration::from_secs(10))
        .validation_mode(gossipsub::ValidationMode::Strict)
        .build()
        .expect("Config valida");

    let gossipsub = gossipsub::Behaviour::new(
        gossipsub::MessageAuthenticity::Signed(id_keys.clone()),
        gossipsub_config,
    )?;

    let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), peer_id)?;

    let behaviour = AppBehaviour { gossipsub, mdns };

    // Costruiamo lo Swarm (Il gestore dei peer)
    let swarm = SwarmBuilder::with_tokio_executor(transport, behaviour, peer_id).build();

    Ok((swarm, peer_id))
}