use ediscoin::*;

use actix_web::{get, middleware::Logger, post, web, App, HttpResponse, HttpServer, Responder};
use chrono::prelude::*;
use std::sync::Mutex;

struct State {
    blockchain: Blockchain,
    // TODO: Use an address type instead of String
    peers: Vec<String>,
    node_addr: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "my_errors=debug,actix_web=debug");
    std::env::set_var("RUST_BACKTRACE", "1");

    let port = std::env::args().nth(1).unwrap_or("8080".into());

    let state = web::Data::new(Mutex::new(State {
        blockchain: Blockchain::new(Block::new(0, Utc::now(), "", &hex::encode([0; 64]), 0, 2)),
        peers: vec![],
        node_addr: format!("127.0.0.1:{}'", port),
    }));

    println!("Starting node at address '127.0.0.1:{}'", port);

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(get_blockchain)
            .service(mine_block)
            .service(get_latest_block)
            .service(get_peers)
            .service(add_peer)
            .service(update_chain)
    })
    .bind(&format!("127.0.0.1:{}", port))?
    .run()
    .await
}

// Control Routes

#[get("/blocks")]
async fn get_blockchain<'app>(data: web::Data<Mutex<State>>) -> impl Responder {
    let state = data.lock().expect("failed to aquire lock on state");

    HttpResponse::Ok().body(
        serde_json::to_string(&state.blockchain.chain).unwrap_or("{\"error\": \"true\"}".into()),
    )
}

#[get("/latest-block")]
async fn get_latest_block<'app>(data: web::Data<Mutex<State>>) -> impl Responder {
    let state = data.lock().expect("failed to aquire lock on state");

    HttpResponse::Ok().body(
        serde_json::to_string(&state.blockchain.get_latest_block())
            .unwrap_or("{\"error\": \"true\"}".into()),
    )
}

#[post("/mine")]
async fn mine_block(req_body: String, data: web::Data<Mutex<State>>) -> impl Responder {
    let mut state = data.lock().expect("failed to aquire lock on state");

    let current_block = state.blockchain.get_latest_block();
    let new_block = find_block(
        current_block.index + 1,
        &current_block.hash,
        Utc::now(),
        &req_body,
        get_difficulty(&state.blockchain),
    );

    state.blockchain.add_block(new_block);

    broadcast_latest_chain(&state.blockchain, &state.peers, &state.node_addr).await;

    HttpResponse::Ok().body(
        serde_json::to_string(state.blockchain.get_latest_block())
            .unwrap_or("{\"error\": \"true\"}".into()),
    )
}

// P2P Routes

#[get("/peers")]
async fn get_peers(data: web::Data<Mutex<State>>) -> impl Responder {
    let state = data.lock().expect("failed to aquire lock on state");

    HttpResponse::Ok()
        .body(serde_json::to_string(&state.peers).unwrap_or("{\"error\": \"true\"}".into()))
}

#[post("/add-peer")]
async fn add_peer(req_body: String, data: web::Data<Mutex<State>>) -> impl Responder {
    let mut state = data.lock().expect("failed to aquire lock on state");
    let peer_addr = req_body;
    let host_addr = state.node_addr.clone();

    println!("Adding peer '{}'", peer_addr);

    let peer_latest_block = query_peer_latest_block(&peer_addr).await;
    let current_latest_block = state.blockchain.get_latest_block();

    if peer_latest_block.index > current_latest_block.index {
        let peer_blockchain = query_peer_blockchain(&peer_addr).await;
        // Don't broadcast to newly added peer
        let peers_list = state
            .peers
            .clone()
            .into_iter()
            .filter(|peer| *peer != peer_addr)
            .collect();

        replace_and_broadcast_chain(
            peer_blockchain,
            &mut state.blockchain,
            &peers_list,
            &host_addr,
        )
        .await;
    }

    state.peers.push(peer_addr);

    HttpResponse::Ok()
}

#[post("/update-chain")]
async fn update_chain(
    json: web::Json<UpdateChainPayload>,
    data: web::Data<Mutex<State>>,
) -> impl Responder {
    let mut state = data.lock().expect("failed to aquire lock on state");
    let new_blocks = json.blocks.clone();
    let req_addr = json.addr.clone();
    let host_addr = state.node_addr.clone();

    let peers_list = state
        .peers
        .clone()
        .into_iter()
        .filter(|peer| *peer != req_addr)
        .collect(); // filter out peer that broadcasted this update

    println!("Received broadcasted chain from '{}'", req_addr);

    replace_and_broadcast_chain(new_blocks, &mut state.blockchain, &peers_list, &host_addr).await;

    // Currently just sends back the current head even if the replace didn't go through
    HttpResponse::Ok().body(
        serde_json::to_string(state.blockchain.get_latest_block())
            .unwrap_or("{\"error\": \"true\"}".into()),
    )
}
