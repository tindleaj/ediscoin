use super::*;
use actix_web::client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateChainPayload {
    pub blocks: Vec<Block>,
    pub addr: String,
}

pub async fn broadcast_latest_chain(blockchain: &Blockchain, peers: &Vec<String>, host: &str) {
    let client = client::Client::default();

    println!("Broadcasting latest chain to: {:?}", peers);

    for peer in peers {
        let addr = format!("http://{}/update-chain", peer);
        let payload = serde_json::to_string(&UpdateChainPayload {
            blocks: blockchain.chain.clone(),
            addr: host.to_string(),
        })
        .unwrap_or("{\"error\": \"true\"}".into());

        let req = client
            .post(&addr)
            .set_header_if_none("Content-Type", "application/json");

        let res = req.send_body(&payload).await.unwrap();

        println!("'{}' responded to broadcast with: {}", peer, res.status());
    }
}

pub async fn replace_and_broadcast_chain(
    new_blocks: Vec<Block>,
    blockchain: &mut Blockchain,
    peers: &Vec<String>,
    host: &str,
) {
    if is_valid_chain(&Blockchain {
        chain: new_blocks.clone(),
    }) && cumulative_difficulty(&new_blocks) > cumulative_difficulty(&blockchain.chain)
    {
        println!(
            "Received blockchain is valid. Replacing current blockchain with recieved blockchain"
        );

        blockchain.replace_chain(new_blocks);
        broadcast_latest_chain(blockchain, peers, host).await;
    } else {
        println!("Received blockchain is invalid");
    }
}

pub async fn query_peer_latest_block(peer_addr: &str) -> Block {
    let client = client::Client::default();
    let addr = format!("http://{}/latest-block", peer_addr);

    println!("Querying latest block at '{}'", addr);

    let res = client
        .get(&addr)
        .send()
        .await
        .unwrap()
        .body()
        .await
        .unwrap();

    serde_json::from_slice(&res).unwrap()
}

pub async fn query_peer_blockchain(peer_addr: &str) -> Vec<Block> {
    let client = client::Client::new();
    let addr = format!("http://{}/blocks", peer_addr);

    let res = client
        .get(&addr)
        .send()
        .await
        .unwrap()
        .body()
        .await
        .unwrap();

    serde_json::from_slice(&res).unwrap()
}
