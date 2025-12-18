use warp::Filter;
use crate::config::NodeConfig;
use serde::Serialize;
use std::sync::{Arc, Mutex};

// Questa è la struttura "leggera" del blocco che mandiamo al sito web
#[derive(Serialize, Clone)]
pub struct BlockView {
    pub index: u64,
    pub hash: String,
    pub tx_count: usize,
    pub timestamp: u64, // Simulato per ora
}

// Lo stato condiviso: Configurazione + Lista ultimi blocchi
pub struct AppState {
    pub config: NodeConfig,
    pub latest_blocks: Arc<Mutex<Vec<BlockView>>>,
}

pub async fn start_web_server(
    config: NodeConfig, 
    latest_blocks: Arc<Mutex<Vec<BlockView>>>, 
    web_port: u16
) {
    // Creiamo lo stato da passare alle rotte
    let state = Arc::new(AppState {
        config,
        latest_blocks,
    });

    // Filtro per passare lo stato alle funzioni
    let state_filter = warp::any().map(move || state.clone());

    // 1. API: Info Nodo
    let stats_route = warp::path!("api" / "stats")
        .and(state_filter.clone())
        .map(|state: Arc<AppState>| {
            let response = serde_json::json!({
                "chain_name": state.config.chain_name,
                "node_role": state.config.node_role,
                "version": state.config.version,
                "port": state.config.server_port,
            });
            warp::reply::json(&response)
        });

    // 2. API: Lista Blocchi (La novità!)
    let blocks_route = warp::path!("api" / "blocks")
        .and(state_filter.clone())
        .map(|state: Arc<AppState>| {
            // Blocchiamo il mutex per leggere i dati in sicurezza
            let blocks = state.latest_blocks.lock().unwrap();
            warp::reply::json(&*blocks)
        });

    // 3. Frontend: Dashboard HTML
    let dashboard_route = warp::path::end()
        .and(warp::fs::file("dashboard.html"));

    let routes = stats_route.or(blocks_route).or(dashboard_route);

    println!("   [WEB] 🌍 Dashboard available at http://localhost:{}", web_port);
    
    warp::serve(routes).run(([0, 0, 0, 0], web_port)).await;
}