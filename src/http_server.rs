use warp::Filter;
use crate::config::NodeConfig;
use serde::Serialize;
// use std::sync::Arc; <--- QUESTA RIGA L'ABBIAMO TOLTA

#[derive(Serialize)]
struct StatusResponse {
    chain_name: String,
    node_role: String,
    version: String,
    port: u16,
}

pub async fn start_web_server(config: NodeConfig, web_port: u16) {
    let config_clone = config.clone();
    let stats_route = warp::path!("api" / "stats")
        .map(move || {
            let response = StatusResponse {
                chain_name: config_clone.chain_name.clone(),
                node_role: config_clone.node_role.clone(),
                version: config_clone.version.clone(),
                port: config_clone.server_port,
            };
            warp::reply::json(&response)
        });

    let dashboard_route = warp::path::end()
        .and(warp::fs::file("dashboard.html"));

    let routes = stats_route.or(dashboard_route);

    println!("   [WEB] 🌍 Dashboard available at http://localhost:{}", web_port);
    
    warp::serve(routes).run(([0, 0, 0, 0], web_port)).await;
}