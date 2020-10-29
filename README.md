# ediscoin

A proof-of-work cryptocurrency based on [lhartikk's naivecoin](https://github.com/lhartikk/naivecoin), written in Rust.

![Demo](demo.gif)

## Overview

`ediscoin` is a simple SHA256, proof-of-work cryptocurrency. It does basic network syncing between nodes (based on Nakamoto consensus) over HTTP, dynamically adjusts difficulty, and exposes an HTTP interface to the client.

## Getting started

`cargo run` starts a node on `localhost:8080` by default, or you can pass a port as an argument: `cargo run -- 8000`. Each node exposes an HTTP interface with the following routes available:

**Control routes**
- `GET /blocks`: Returns a JSON payload with the current blockchain
- `GET /latest-block`: Returns a JSON payload with the latest block
- `POST /mine`: Attempts to mine the next block, returning it if successful and returning the latest block if not

**P2P routes**
- `GET /peers`: Returns a list of peers
- `POST /add-peer`: Takes a single address string as a body and adds it to the peerlist. Currently there is no automatic peer discovery, so peers must be added manually.
- `POST /update-chain`: Takes a JSON array of blocks and updates the node with those blocks if they're valid and longer than the node's current chain