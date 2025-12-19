use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use sha3::{Digest, Sha3_512}; 

// --- STRUTTURE DATI ---

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    index: u32,
    timestamp: u128,
    data: String,
    previous_hash: String,
    hash: String,
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
            data: "GENESIS BLOCK - ADAMAS ENTERPRISE SYSTEM".to_string(),
            previous_hash: "0".to_string(),
            hash: "00000000000000000000".to_string(),
        };
        self.chain.push(genesis_block);
    }

    fn add_block(&mut self, data: String) -> Block {
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
        };

        self.chain.push(new_block.clone());
        println!("📦 BLOCK #{} MINED: {}", new_index, new_block.data);
        new_block
    }
}

// --- SERVER WEB ---

fn handle_client(mut stream: TcpStream, blockchain: Arc<Mutex<Blockchain>>) {
    let mut buffer = [0; 1024];
    // Leggi la richiesta, ma gestisci il caso in cui sia vuota o corrotta
    match stream.read(&mut buffer) {
        Ok(_) => {},
        Err(_) => return,
    }
    
    let request = String::from_utf8_lossy(&buffer[..]);

    let response_header;
    let response_body;

    // --- ROUTING ---
    
    // 1. Richiesta Dati Blockchain (JSON)
    if request.starts_with("GET /blocks") {
        let chain_data = blockchain.lock().unwrap();
        response_body = serde_json::to_string(&chain_data.chain).unwrap();
        // Diciamo al browser che questo è JSON
        response_header = "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8";
    
    // 2. Richiesta Mining (Azione)
    } else if request.starts_with("GET /mine/") {
        let parts: Vec<&str> = request.split_whitespace().collect();
        if parts.len() > 1 {
            let path = parts[1];
            // Gestione sicura della stringa per evitare crash se l'URL è corto
            if path.len() > 6 {
                let data_raw = &path[6..];
                let data_clean = data_raw.replace("%20", " ").replace("%7C", " | ").replace("%22", ""); // Pulisci caratteri URL

                let mut chain_guard = blockchain.lock().unwrap();
                chain_guard.add_block(data_clean);
                
                response_body = "{\"status\": \"success\"}".to_string();
                response_header = "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=UTF-8";
            } else {
                response_body = "{}".to_string();
                response_header = "HTTP/1.1 400 BAD REQUEST\r\nContent-Type: application/json";
            }
        } else {
             response_body = "{}".to_string();
             response_header = "HTTP/1.1 400 BAD REQUEST\r\nContent-Type: application/json";
        }

    // 3. Dashboard HTML (Home Page)
    // Nota: Controlliamo sia la root "/" sia "/dashboard.html"
    } else if request.contains("GET / ") || request.contains("GET /dashboard.html") {
        let contents = std::fs::read_to_string("dashboard.html").unwrap_or("<h1>Dashboard File Missing</h1>".to_string());
        response_body = contents;
        // Diciamo al browser che questo è HTML
        response_header = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8";

    // 4. Favicon (Evita errori 404 fastidiosi nel log)
    } else if request.contains("GET /favicon.ico") {
        response_body = "".to_string();
        response_header = "HTTP/1.1 404 NOT FOUND";

    } else {
        response_body = "404 Not Found".to_string();
        response_header = "HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/plain";
    }

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        response_header,
        response_body.len(),
        response_body
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn main() {
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));

    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    println!("🚀 ADAMAS ENTERPRISE NODE STARTED");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let blockchain_ref = Arc::clone(&blockchain);
                thread::spawn(move || {
                    handle_client(stream, blockchain_ref);
                });
            }
            Err(e) => { println!("Connection failed: {}", e); }
        }
    }
}