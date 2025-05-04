# Enhanced WebSocket Transactions

This project provides a tool to subscribe to Solana transactions using the Helius API and WebSockets.

## Prerequisites

- Rust and Cargo installed on your system.
- A valid API key for the Helius API.

## Usage

To run the `enhanced_websocket_transactions` binary with a specific `program_id` and `api_key`, use the following
command:

```sh
cargo run --bin enhanced_websocket_transactions -- -p <PROGRAM_ID> -k <API_KEY> -d <CSV_DIR>
```

cargo run --bin enhanced_websocket_kafka -- --program_id <PROGRAM_ID> --api_key <API_KEY> --topic_name <TOPIC_NAME>

```

### Example

```sh
cargo run --bin enhanced_websocket_transactions -- -p 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P -k <api_key> -d /tmp/csv 
cargo run --bin enhanced_websocket_kafka -- --program_id 6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P --api_key <api_key> --topic_name pumpfun-create
cargo run --bin enhanced_websocket_redis_producer 
```

### Command Line Arguments

- `--program_id`: The program ID to subscribe to. If not provided, the default value is
  `6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P`.
- `--api_key`: The API key for the Helius API. If not provided, the default value is empty.