use super::*;

pub async fn broadcast_latest_chain(blockchain: &Blockchain, peers: &Vec<String>) {
    // This isn't async, and would struggle with a larger network
    let client = reqwest::Client::new();

    for peer in peers {
        let addr = format!("http://{}/update-chain", peer);

        println!("Broadcasting blockchain to '{}'", peer);
        client
            .post(&addr)
            .body(
                serde_json::to_string(&blockchain.chain).unwrap_or("{\"error\": \"true\"}".into()),
            )
            .send()
            .await
            .unwrap();
    }
}

pub async fn replace_and_broadcast_chain(
    new_blocks: Vec<Block>,
    blockchain: &mut Blockchain,
    peers: &Vec<String>,
) {
    if is_valid_chain(&Blockchain {
        chain: new_blocks.clone(),
    }) && cumulative_difficulty(&new_blocks) > cumulative_difficulty(&blockchain.chain)
    {
        println!(
            "Received blockchain is valid. Replacing current blockchain with recieved blockchain"
        );

        blockchain.replace_chain(new_blocks);
        broadcast_latest_chain(blockchain, peers).await;
    } else {
        println!("Received blockchain is invalid");
    }
}

pub async fn query_peer_latest_block(peer_addr: &str) -> Block {
    let client = reqwest::Client::new();
    let addr = format!("http://{}/latest-block", peer_addr);

    println!("Querying latest block at '{}'", addr);

    let res = client.get(&addr).send().await;

    match res {
        Err(e) => {
            panic!("Error querying latest block: {}", e);
        }
        Ok(res) => {
            let raw = res.text().await.unwrap();

            return serde_json::from_str(&raw).unwrap();
        }
    }
}

pub async fn query_peer_blockchain(peer_addr: &str) -> Vec<Block> {
    let client = reqwest::Client::new();
    let addr = format!("http://{}/blocks", peer_addr);

    let res = client
        .get(&addr)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    serde_json::from_str(&res).unwrap()
}
